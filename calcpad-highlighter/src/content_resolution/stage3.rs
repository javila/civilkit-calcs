//! ContentResolver.Stage3.cs: macro expansion and definition collection.
use super::content_resolver::join_lines;
use super::result::{
    CustomUnitDefinition, FunctionDefinition, FunctionInfo, LineContinuationSegment, MacroDefinition,
    MacroExpansionInfo, MacroInfo, SourceInfo, Stage1Result, Stage2Result, Stage3Result, SymbolLocation,
    VariableDefinition,
};
use super::ContentResolver;
use crate::linter::helpers::character_helpers::is_macro_letter;
use crate::linter::helpers::cs;
use crate::linter::helpers::line_parser::{get_directive_expression_start, is_directive_line, should_skip_line};
use crate::linter::helpers::parameter_parser::parse_macro_parameters;
use crate::linter::helpers::parsing_helpers::{
    count_required_params, find_matching_close, skip_whitespace, try_parse_keyword_arg,
};
use crate::linter::helpers::type_tracker::TypeTracker;
use crate::linter::models::DefinitionMetadata;
use crate::parsing::split_lines;
use crate::text::{range, sub, to_string, trim, trim_start, StrMap, StrSet};
use crate::tokenizer::models::{Token, TokenType, TokenizerMode};
use crate::tokenizer::CalcpadTokenizer;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// `char.ToUpperInvariant`.
fn up(c: char) -> char {
    let mut u = c.to_uppercase();
    match (u.next(), u.next()) {
        (Some(x), None) => x,
        _ => c,
    }
}

/// A macro with its name pre-split into characters, for the longest-name-first scan.
struct SortedMacro<'a> {
    name: Vec<char>,
    def: &'a MacroDefinition,
}

impl ContentResolver {
    /// STAGE 3: expand macros, collect all definitions. Removes macro definitions and substitutes
    /// macro calls with their expanded content.
    pub(crate) fn process_stage3(stage2: &Stage2Result, stage1: &Stage1Result) -> Stage3Result {
        let mut lines: Vec<String> = Vec::new();
        let mut source_map: BTreeMap<i32, i32> = BTreeMap::new();
        let mut macro_expansions: BTreeMap<i32, MacroExpansionInfo> = BTreeMap::new();

        // Macro map from the stage 2 definitions (duplicates skipped, the first definition is used)
        let mut macros: StrMap<&MacroDefinition> = StrMap::new_ci();
        for macro_def in &stage2.macro_definitions {
            macros.try_add(&macro_def.name, macro_def);
        }

        // Sort macros by name length descending so longer names match first (e.g. "string$" before
        // "ng$" in "gstring$"). The sort is stable like LINQ's OrderByDescending.
        let mut sorted_macros: Vec<SortedMacro> = macros
            .values()
            .map(|m| SortedMacro {
                name: m.name.chars().collect(),
                def: m,
            })
            .collect();
        sorted_macros.sort_by_key(|a| std::cmp::Reverse(a.name.len()));

        // Process lines: skip macro definitions, expand macro calls
        let mut in_multiline_macro = false;

        for (i, line) in stage2.lines.iter().enumerate() {
            let line_chars: Vec<char> = line.chars().collect();
            let trimmed = trim(&line_chars);

            if in_multiline_macro {
                if cs::equals_ic(trimmed, "#end def") {
                    in_multiline_macro = false;
                }
                // Skip all lines inside the macro definition
                continue;
            }

            if cs::starts_with_ic(trimmed, "#def ") {
                if has_equals_outside_parens(trimmed, 5) {
                    // Inline macro (#def name$(params) = content): skip this single line
                    continue;
                }
                // Multiline macro start (#def name$(params)): skip until #end def
                in_multiline_macro = true;
                continue;
            }

            // Regular line: expand macro calls, tracking which macros were expanded
            let original_line = line.clone();
            let mut expanded_macro_names: Vec<String> = Vec::new();
            let mut expanding = StrSet::new();
            let expanded_line = expand_macros(line, &sorted_macros, &macros, &mut expanded_macro_names, &mut expanding);
            let is_from_macro_expansion = expanded_line != original_line;

            // Macro content can have several lines; LineEnumerator splits on \r\n, \r and \n
            let expanded_chars: Vec<char> = expanded_line.chars().collect();
            let total_content_lines = expanded_chars.iter().filter(|c| **c == '\n').count() as i32 + 1;

            for (sub_idx, line_span) in split_lines(&expanded_chars).into_iter().enumerate() {
                lines.push(to_string(line_span));
                let index = (lines.len() - 1) as i32;
                source_map.insert(index, i as i32);
                if is_from_macro_expansion {
                    macro_expansions.insert(
                        index,
                        MacroExpansionInfo {
                            macro_names: expanded_macro_names.clone(),
                            call_site_line: original_line.clone(),
                            call_site_stage2_line: i as i32,
                            content_line_index: sub_idx as i32,
                            total_content_lines,
                        },
                    );
                }
            }
        }

        // New include map for the filtered lines
        let mut filtered_include_map: BTreeMap<i32, SourceInfo> = BTreeMap::new();
        for (k, v) in &source_map {
            if let Some(info) = stage2.include_map.get(v) {
                filtered_include_map.insert(*k, info.clone());
            }
        }

        // Tokenize in Lint mode: extracts the full definition metadata
        let mut tokenizer = CalcpadTokenizer::new();
        tokenizer.set_include_map(&filtered_include_map);
        let source = join_lines(&lines);
        let mut tokenizer_result = tokenizer.tokenize_mode(&source, TokenizerMode::Lint);

        // All definitions come from the tokenizer's Lint mode
        let mut variables_with_defs: Vec<VariableDefinition> =
            std::mem::take(&mut tokenizer_result.variable_definitions);
        // Some built-ins implicitly define a variable as a side effect (the arg extremum of $Inf/$Sup,
        // the row-permutation vector 'ind' of lu(...)): emit them as real definitions.
        collect_implicit_solver_variables(&tokenizer_result.tokens, &mut variables_with_defs);
        collect_lu_index_variable(&tokenizer_result.tokens, &mut variables_with_defs);
        let mut variables = StrSet::new_ci();
        for v in &variables_with_defs {
            variables.insert(&v.name);
        }
        // Also variables from the basic tracking (e.g. inside command blocks)
        for name in tokenizer_result.defined_variables.keys() {
            variables.insert(name);
        }

        let mut functions_with_params: Vec<FunctionDefinition> =
            std::mem::take(&mut tokenizer_result.function_definitions);
        let mut functions: StrMap<FunctionInfo> = StrMap::new();
        for f in &functions_with_params {
            let defaults = if f.defaults.is_empty() { None } else { Some(&f.defaults) };
            functions.insert(
                &f.name,
                FunctionInfo {
                    line_number: f.line_number,
                    param_count: f.params.len() as i32,
                    required_param_count: count_required_params(f.params.len() as i32, defaults),
                    param_names: f.params.clone(),
                },
            );
        }

        let user_defined_macros: StrMap<MacroInfo> = stage2.user_defined_macros.clone();
        let mut custom_units: Vec<CustomUnitDefinition> = std::mem::take(&mut tokenizer_result.custom_unit_definitions);
        let command_block_functions = std::mem::take(&mut tokenizer_result.command_block_functions);

        // TypeTracker from the collected definitions
        let mut type_tracker = TypeTracker::new();
        type_tracker.set_tokens_by_line(&tokenizer_result.tokens_by_line);
        populate_type_tracker(
            &mut type_tracker,
            &variables_with_defs,
            &functions_with_params,
            &stage2.macro_definitions,
            &custom_units,
        );

        // Variable assignments and usages from the Lint-mode tokens
        let (variable_assignments, variable_usages) =
            collect_variable_assignments_and_usages(&tokenizer_result, &lines);

        let maps = PositionMaps {
            stage3_to_stage2: &source_map,
            include_map: &filtered_include_map,
            stage2_to_stage1: &stage2.source_map,
            stage1_to_original: &stage1.source_map,
            segments: &stage1.line_continuation_segments,
        };

        let variable_index = build_variable_index(&variable_assignments, &variable_usages, &maps);
        let function_index = build_function_index(&tokenizer_result.tokens, &functions_with_params, &functions, &maps);
        let macro_index = build_macro_index(
            &stage2.macro_definitions,
            &macro_expansions,
            &stage2.lines,
            &stage2.include_map,
            &maps,
        );

        // The definition lists carry Stage 3 line numbers (post macro expansion and include
        // filtering). Consumers that key by original editor lines need original lines, so remap
        // after the Stage 3 indexes (which rely on Stage 3 lines) were built.
        let map_def_line_to_original = |stage3_line: i32| -> i32 {
            if let Some(info) = filtered_include_map.get(&stage3_line) {
                if info.source == "include" && info.original_line >= 0 {
                    return info.original_line;
                }
            }
            let stage2_line = source_map.get(&stage3_line).copied().unwrap_or(stage3_line);
            let stage1_line = stage2.source_map.get(&stage2_line).copied().unwrap_or(stage2_line);
            map_column_through_continuation(stage1_line, 0, &stage1.line_continuation_segments, &stage1.source_map).0
        };
        for f in functions_with_params.iter_mut() {
            f.line_number = map_def_line_to_original(f.line_number);
        }
        for v in variables_with_defs.iter_mut() {
            v.line_number = map_def_line_to_original(v.line_number);
        }
        for u in custom_units.iter_mut() {
            u.line_number = map_def_line_to_original(u.line_number);
        }

        Stage3Result {
            lines,
            source_map,
            include_map: filtered_include_map,
            macro_expansions,
            user_defined_functions: functions,
            functions_with_params,
            user_defined_macros,
            defined_variables: variables,
            variables_with_definitions: variables_with_defs,
            custom_units,
            type_tracker,
            command_block_functions,
            variable_reassignments: std::mem::take(&mut tokenizer_result.variable_reassignments),
            outer_scope_assignments: std::mem::take(&mut tokenizer_result.outer_scope_assignments),
            variable_assignments,
            variable_usages,
            variable_index,
            function_index,
            macro_index,
            ..Default::default()
        }
    }
}

/// Registers all definitions in order of appearance.
fn populate_type_tracker(
    type_tracker: &mut TypeTracker,
    variables: &[VariableDefinition],
    functions: &[FunctionDefinition],
    macros: &[MacroDefinition],
    custom_units: &[CustomUnitDefinition],
) {
    // Custom units first (they can be used in expressions)
    for unit in custom_units {
        let info = type_tracker.register_custom_unit(&unit.name, &unit.definition, unit.line_number, 0, &unit.source);
        info.description = unit.description.clone();
    }

    for func in functions {
        let info = if let Some(cb) = &func.command_block {
            // Command block function: proper return type inference from the statements
            type_tracker.register_command_block_function(
                &func.name,
                &func.params,
                &cb.statements,
                func.line_number,
                0,
                &func.source,
                func.is_const,
            )
        } else {
            // Normal function: the expression gives the return type
            type_tracker.register_function(
                &func.name,
                &func.params,
                func.expression.as_deref().unwrap_or(""),
                func.line_number,
                0,
                &func.source,
                func.is_const,
            )
        };
        // Metadata from the definition comment
        info.description = func.description.clone();
        info.param_types = func.param_types.clone();
        info.param_descriptions = func.param_descriptions.clone();

        // A declared return type in the metadata comment overrides inference
        if let Some(declared) =
            DefinitionMetadata::parse_function_return_type(func.return_type.as_deref().unwrap_or(""))
        {
            info.return_type = declared;
        }
    }

    for m in macros {
        let is_inline = m.content.len() == 1 && !cs::is_blank(&m.content[0]);
        let info = if is_inline {
            type_tracker.register_inline_macro(&m.name, &m.params, &m.content[0], m.line_number, 0, &m.source)
        } else {
            type_tracker.register_multiline_macro(&m.name, &m.params, m.line_number, 0, &m.source)
        };
        info.description = m.description.clone();
        info.param_types = m.param_types.clone();
        info.param_descriptions = m.param_descriptions.clone();
    }

    // Variables in order: this handles type changes correctly
    for variable in variables {
        // #read variables have known types: matrix (default) or vector (TYPE=V)
        let info = if variable.definition == "#read matrix" {
            type_tracker.register_read_variable(&variable.name, false, variable.line_number, 0, &variable.source)
        } else if variable.definition == "#read vector" {
            type_tracker.register_read_variable(&variable.name, true, variable.line_number, 0, &variable.source)
        } else {
            type_tracker.register_variable(
                &variable.name,
                &variable.definition,
                variable.line_number,
                0,
                &variable.source,
                variable.is_const,
            )
        };
        info.description = variable.description.clone();
    }
}

/// Scans the Lint-mode tokens for variable assignment and usage positions. Assignments are Variable
/// tokens on the left side of `=` (not a comparison operator); usages are all other Variable tokens.
fn collect_variable_assignments_and_usages(
    tokenizer_result: &crate::tokenizer::models::TokenizerResult,
    lines: &[String],
) -> (Vec<(String, i32, i32, i32)>, Vec<(String, i32, i32)>) {
    let mut assignments: Vec<(String, i32, i32, i32)> = Vec::new();
    let mut usages: Vec<(String, i32, i32)> = Vec::new();
    let mut assignment_positions: HashSet<(i32, i32)> = HashSet::new();

    // First pass: assignment targets per line
    for (&line_number, tokens) in &tokenizer_result.tokens_by_line {
        // Skip empty and comment lines (same filtering as the linter). Directives are scanned from
        // their expression onwards, so '#UI L = 10m' still registers an assignment while the
        // keyword itself is ignored.
        if line_number < 0 || line_number as usize >= lines.len() {
            continue;
        }
        let line = &lines[line_number as usize];
        if should_skip_line(line) {
            continue;
        }

        let line_chars: Vec<char> = line.chars().collect();
        let mut expression_start = 0;
        if is_directive_line(crate::text::trim(&line_chars)) {
            expression_start = get_directive_expression_start(&line_chars);
            if expression_start < 0 {
                continue;
            }
        }

        // The first Variable token on the line
        let mut first_var: Option<&Token> = None;

        for t in 0..tokens.len() {
            let token = &tokens[t];
            if token.column < expression_start {
                continue;
            }

            if first_var.is_none() && token.token_type == TokenType::Variable {
                first_var = Some(token);
            }

            if token.token_type != TokenType::Operator || token.text != "=" {
                continue;
            }

            // Not a comparison operator (==, <=, >=, !=)
            let mut is_comparison = false;
            if t > 0 {
                let prev = &tokens[t - 1];
                if prev.token_type == TokenType::Operator
                    && prev.column + prev.length == token.column
                    && (prev.text == "<" || prev.text == ">" || prev.text == "!" || prev.text == "=")
                {
                    is_comparison = true;
                }
            }
            if t + 1 < tokens.len() {
                let next = &tokens[t + 1];
                if next.token_type == TokenType::Operator
                    && token.column + token.length == next.column
                    && next.text == "="
                {
                    is_comparison = true;
                }
            }

            if !is_comparison {
                if let Some(v) = first_var {
                    assignments.push((v.text.clone(), v.line, v.column, v.length));
                    assignment_positions.insert((v.line, v.column));
                }
            }
            break;
        }
    }

    // Second pass: all Variable tokens that are NOT assignment targets
    for token in &tokenizer_result.tokens {
        if token.token_type != TokenType::Variable {
            continue;
        }
        if assignment_positions.contains(&(token.line, token.column)) {
            continue;
        }
        usages.push((token.text.clone(), token.line, token.column));
    }

    (assignments, usages)
}

/// Registers the variables that `$Inf` and `$Sup` implicitly define: for a solver whose counter is
/// `k`, the argument extremum is exposed as `k_inf` / `k_sup` (MathParser.SolveBlock). A brace stack
/// tracks the enclosing command per '@', so nested solvers attribute each counter correctly.
fn collect_implicit_solver_variables(tokens: &[Token], variables_with_defs: &mut Vec<VariableDefinition>) {
    let mut existing = StrSet::new_ci();
    for v in variables_with_defs.iter() {
        existing.insert(&v.name);
    }

    let mut scope_suffixes: Vec<Option<&'static str>> = Vec::new();
    let mut pending_suffix: Option<&'static str> = None;
    let mut after_at = false;
    for token in tokens {
        if token.token_type == TokenType::Command {
            pending_suffix = if crate::text::eq_ignore_case(&token.text, "$Inf") {
                Some("_inf")
            } else if crate::text::eq_ignore_case(&token.text, "$Sup") {
                Some("_sup")
            } else {
                None
            };
        } else if token.token_type == TokenType::Bracket && token.text == "{" {
            scope_suffixes.push(pending_suffix);
            pending_suffix = None;
            after_at = false;
        } else if token.token_type == TokenType::Bracket && token.text == "}" {
            scope_suffixes.pop();
            after_at = false;
        } else if token.token_type == TokenType::Operator && token.text == "@" {
            after_at = scope_suffixes.last().is_some_and(|s| s.is_some());
        } else if after_at && (token.token_type == TokenType::Variable || token.token_type == TokenType::LocalVariable)
        {
            let suffix = scope_suffixes.last().copied().flatten().unwrap_or("");
            let name = format!("{}{}", token.text, suffix);
            after_at = false;
            if existing.insert(&name) {
                variables_with_defs.push(VariableDefinition {
                    name,
                    definition: if suffix == "_sup" {
                        "$Sup argument (maximizer)".to_string()
                    } else {
                        "$Inf argument (minimizer)".to_string()
                    },
                    line_number: token.line,
                    source: "local".to_string(),
                    ..Default::default()
                });
            }
        }
    }
}

/// `lu(...)` writes the row-permutation vector into a global variable named `ind` (MathParser).
/// Register it as defined at the first `lu` call so later references resolve.
fn collect_lu_index_variable(tokens: &[Token], variables_with_defs: &mut Vec<VariableDefinition>) {
    if variables_with_defs
        .iter()
        .any(|v| crate::text::eq_ignore_case(&v.name, "ind"))
    {
        return;
    }
    for token in tokens {
        if token.token_type == TokenType::Function && crate::text::eq_ignore_case(&token.text, "lu") {
            variables_with_defs.push(VariableDefinition {
                name: "ind".to_string(),
                definition: "lu(...) row-permutation index vector".to_string(),
                line_number: token.line,
                source: "local".to_string(),
                ..Default::default()
            });
            return;
        }
    }
}

/// The line maps used to take a Stage 3 position back to the original editor position.
struct PositionMaps<'a> {
    stage3_to_stage2: &'a BTreeMap<i32, i32>,
    include_map: &'a BTreeMap<i32, SourceInfo>,
    stage2_to_stage1: &'a BTreeMap<i32, i32>,
    stage1_to_original: &'a BTreeMap<i32, i32>,
    segments: &'a BTreeMap<i32, Vec<LineContinuationSegment>>,
}

impl PositionMaps<'_> {
    /// Stage 3 position to original line and adjusted column, through the continuation segments.
    fn map_position_to_original(&self, stage3_line: i32, column: i32) -> (i32, i32) {
        let stage2_line = self.stage3_to_stage2.get(&stage3_line).copied().unwrap_or(stage3_line);
        let stage1_line = self.stage2_to_stage1.get(&stage2_line).copied().unwrap_or(stage2_line);
        map_column_through_continuation(stage1_line, column, self.segments, self.stage1_to_original)
    }

    /// Stage 2 position to original line and adjusted column.
    fn map_stage2_position_to_original(&self, stage2_line: i32, column: i32) -> (i32, i32) {
        let stage1_line = self.stage2_to_stage1.get(&stage2_line).copied().unwrap_or(stage2_line);
        map_column_through_continuation(stage1_line, column, self.segments, self.stage1_to_original)
    }
}

/// (source, source file, original line inside the include) of a line, `("local", None, -1)` if absent.
fn get_source_info(include_map: &BTreeMap<i32, SourceInfo>, line: i32) -> (String, Option<String>, i32) {
    match include_map.get(&line) {
        Some(info) => (info.source.clone(), info.source_file.clone(), info.original_line),
        None => ("local".to_string(), None, -1),
    }
}

/// Maps a Stage 3 token position to (original line, column): positions inside an include use the
/// included file's own line, others go through the continuation segments.
fn resolve_location(maps: &PositionMaps, line: i32, column: i32) -> (String, Option<String>, i32, i32) {
    let (source, source_file, include_original_line) = get_source_info(maps.include_map, line);
    let (original_line, adjusted_column) = if source == "include" && include_original_line >= 0 {
        (include_original_line, column)
    } else {
        maps.map_position_to_original(line, column)
    };
    (source, source_file, original_line, adjusted_column)
}

/// Variable name to every occurrence (assignments and usages), mapped back to the original source
/// lines and annotated with include file info.
fn build_variable_index(
    assignments: &[(String, i32, i32, i32)],
    usages: &[(String, i32, i32)],
    maps: &PositionMaps,
) -> StrMap<Vec<SymbolLocation>> {
    let mut index: StrMap<Vec<SymbolLocation>> = StrMap::new();

    for (name, line, column, length) in assignments {
        let (source, source_file, original_line, adjusted_column) = resolve_location(maps, *line, *column);
        index.get_or_insert_with(name, Vec::new).push(SymbolLocation {
            line: original_line,
            column: adjusted_column,
            length: *length,
            source,
            source_file,
            is_assignment: true,
        });
    }

    for (name, line, column) in usages {
        let (source, source_file, original_line, adjusted_column) = resolve_location(maps, *line, *column);
        index.get_or_insert_with(name, Vec::new).push(SymbolLocation {
            line: original_line,
            column: adjusted_column,
            length: name.chars().count() as i32,
            source,
            source_file,
            is_assignment: false,
        });
    }

    index
}

/// Each user-defined function name to every occurrence (definition and call sites), mapped back to
/// the original source lines.
fn build_function_index(
    tokens: &[Token],
    function_definitions: &[FunctionDefinition],
    user_defined_functions: &StrMap<FunctionInfo>,
    maps: &PositionMaps,
) -> StrMap<Vec<SymbolLocation>> {
    let mut index: StrMap<Vec<SymbolLocation>> = StrMap::new();

    if user_defined_functions.is_empty() {
        return index;
    }

    // Definition lines per function, to mark the defining token as an assignment. The definition
    // token is not necessarily at column 0 (a leading comment shifts it right), so match on name and
    // line and take the first occurrence (recursive self-calls stay usages).
    let mut definition_lines: HashSet<(String, i32)> = HashSet::new();
    for func_def in function_definitions {
        if user_defined_functions.contains_key(&func_def.name) {
            definition_lines.insert((func_def.name.clone(), func_def.line_number));
        }
    }
    let mut marked_definitions: HashSet<(String, i32)> = HashSet::new();

    for token in tokens {
        if token.token_type != TokenType::Function {
            continue;
        }
        if !user_defined_functions.contains_key(&token.text) {
            continue;
        }

        let (source, source_file, original_line, adjusted_column) = resolve_location(maps, token.line, token.column);
        let key = (token.text.clone(), token.line);
        let is_definition = definition_lines.contains(&key) && marked_definitions.insert(key);

        index.get_or_insert_with(&token.text, Vec::new).push(SymbolLocation {
            line: original_line,
            column: adjusted_column,
            length: token.length,
            source,
            source_file,
            is_assignment: is_definition,
        });
    }

    index
}

/// Each macro name to every occurrence (definitions and call sites). Definitions come from Stage 2
/// macro collection; call sites from the macro expansion tracking.
fn build_macro_index(
    macro_definitions: &[MacroDefinition],
    macro_expansions: &BTreeMap<i32, MacroExpansionInfo>,
    stage2_lines: &[String],
    stage2_include_map: &BTreeMap<i32, SourceInfo>,
    maps: &PositionMaps,
) -> StrMap<Vec<SymbolLocation>> {
    let mut index: StrMap<Vec<SymbolLocation>> = StrMap::new_ci();

    if macro_definitions.is_empty() {
        return index;
    }

    // Stage 2 position to (source, file, original line, column)
    let resolve_stage2 = |stage2_line: i32, merged_column: i32| -> (String, Option<String>, i32, i32) {
        let (source, source_file, include_original_line) = get_source_info(stage2_include_map, stage2_line);
        let (original_line, adjusted_column) = if source == "include" && include_original_line >= 0 {
            (include_original_line, merged_column)
        } else {
            maps.map_stage2_position_to_original(stage2_line, merged_column)
        };
        (source, source_file, original_line, adjusted_column)
    };

    // Definitions
    for macro_def in macro_definitions {
        // Column of the macro name in the merged stage 2 line
        let mut merged_column = 0;
        if macro_def.line_number >= 0 && (macro_def.line_number as usize) < stage2_lines.len() {
            let name_idx = find_macro_name_column(&stage2_lines[macro_def.line_number as usize], &macro_def.name);
            if name_idx >= 0 {
                merged_column = name_idx;
            }
        }
        let (source, source_file, original_line, adjusted_column) =
            resolve_stage2(macro_def.line_number, merged_column);

        index
            .get_or_insert_with(&macro_def.name, Vec::new)
            .push(SymbolLocation {
                line: original_line,
                column: adjusted_column,
                length: macro_def.name.chars().count() as i32,
                source,
                source_file,
                is_assignment: true,
            });
    }

    // Call sites from the expansion tracking; each unique (macro name, call site stage 2 line) is
    // one call site (names compare ignoring case).
    let mut seen_call_sites: BTreeSet<(String, i32)> = BTreeSet::new();

    for expansion in macro_expansions.values() {
        let stage2_line = expansion.call_site_stage2_line;

        for macro_name in &expansion.macro_names {
            if !seen_call_sites.insert((crate::text::upper_invariant(macro_name), stage2_line)) {
                continue; // Already tracked this call site
            }

            // The name must be on the call line with a proper boundary. macro_names includes macros
            // pulled in by nested expansion (e.g. doubleCheck$'s body calls double$), which are not
            // literally on this line: skip those.
            if stage2_line < 0 || stage2_line as usize >= stage2_lines.len() {
                continue;
            }
            let merged_column = find_macro_name_column(&stage2_lines[stage2_line as usize], macro_name);
            if merged_column < 0 {
                continue;
            }

            let (source, source_file, original_line, adjusted_column) = resolve_stage2(stage2_line, merged_column);

            index.get_or_insert_with(macro_name, Vec::new).push(SymbolLocation {
                line: original_line,
                column: adjusted_column,
                length: macro_name.chars().count() as i32,
                source,
                source_file,
                is_assignment: false,
            });
        }
    }

    // Macro calls inside macro definition bodies: multiline bodies (between #def and #end def) are
    // skipped in Stage 3, so the expansion tracking never sees their references. Inline macro
    // content (after '=') is scanned too.
    let mut all_macro_names = StrSet::new_ci();
    for md in macro_definitions {
        all_macro_names.insert(&md.name);
    }

    for macro_def in macro_definitions {
        if macro_def.content.is_empty() {
            continue;
        }

        // Parameter names are not macro calls
        let mut param_names = StrSet::new();
        for p in &macro_def.params {
            param_names.insert(p);
        }

        // Inline macro (#def name$ = content) or multiline (#def name$ ... #end def)
        let mut is_inline_macro = false;
        let mut inline_content_start_col: isize = -1;
        if macro_def.line_number >= 0 && (macro_def.line_number as usize) < stage2_lines.len() {
            let def_line: Vec<char> = stage2_lines[macro_def.line_number as usize].chars().collect();
            let trimmed_def = trim_start(&def_line);
            if cs::starts_with_ic(trimmed_def, "#def ") {
                is_inline_macro = has_equals_outside_parens(trimmed_def, 5);
            }

            if is_inline_macro {
                // '=' position in the actual (untrimmed) line marks the content start
                let leading_spaces = def_line.len() - trimmed_def.len();
                let mut depth = 0;
                for ci in leading_spaces + 5..def_line.len() {
                    if def_line[ci] == '(' {
                        depth += 1;
                    } else if def_line[ci] == ')' {
                        depth -= 1;
                    } else if def_line[ci] == '=' && depth == 0 {
                        inline_content_start_col = ci as isize + 1;
                        break;
                    }
                }
            }
        }

        // Which stage 2 lines to scan
        let (body_start_line, body_end_line) = if is_inline_macro {
            (macro_def.line_number, macro_def.line_number)
        } else {
            (
                macro_def.line_number + 1,
                macro_def.line_number + macro_def.content.len() as i32,
            )
        };

        let mut s2_line = body_start_line;
        while s2_line <= body_end_line && s2_line >= 0 && (s2_line as usize) < stage2_lines.len() {
            let line_text: Vec<char> = stage2_lines[s2_line as usize].chars().collect();
            if line_text.is_empty() || !line_text.contains(&'$') {
                s2_line += 1;
                continue;
            }

            // For inline macros, only scan after the '=' to skip the definition name
            let scan_start = if is_inline_macro && s2_line == macro_def.line_number && inline_content_start_col >= 0 {
                inline_content_start_col as usize
            } else {
                0
            };

            for ci in scan_start..line_text.len() {
                if line_text[ci] != '$' {
                    continue;
                }

                // Work backwards from the $ to the start of the macro name
                let mut name_start = ci;
                while name_start > 0 && is_macro_letter(line_text[name_start - 1], ci - name_start) {
                    name_start -= 1;
                }

                if name_start >= ci {
                    continue; // No name characters before the $
                }

                let candidate_name = sub(&line_text, name_start, ci - name_start + 1);

                // Skip parameters and unknown names
                if param_names.contains(&candidate_name) {
                    continue;
                }
                if !all_macro_names.contains(&candidate_name) {
                    continue;
                }

                let (source, source_file, include_original_line) = get_source_info(stage2_include_map, s2_line);
                let (original_line, adjusted_column) = if source == "include" && include_original_line >= 0 {
                    (include_original_line, name_start as i32)
                } else {
                    maps.map_stage2_position_to_original(s2_line, name_start as i32)
                };

                let length = candidate_name.chars().count() as i32;
                index
                    .get_or_insert_with(&candidate_name, Vec::new)
                    .push(SymbolLocation {
                        line: original_line,
                        column: adjusted_column,
                        length,
                        source,
                        source_file,
                        is_assignment: false,
                    });
            }
            s2_line += 1;
        }
    }

    index
}

/// The column of a macro-name occurrence in a line, requiring that the previous character is not a
/// macro-name letter, so a shorter macro name cannot match inside a longer one ("check$" must not
/// match inside "doubleCheck$"). -1 when the name does not appear as a standalone macro reference.
fn find_macro_name_column(line: &str, macro_name: &str) -> i32 {
    let line: Vec<char> = line.chars().collect();
    let name_len = macro_name.chars().count();
    let mut from = 0usize;
    while from + name_len <= line.len() {
        let Some(idx) = cs::index_of_ic(&line, macro_name, from) else {
            return -1;
        };
        if idx == 0 || !is_macro_letter(line[idx - 1], 1) {
            return idx as i32;
        }
        from = idx + 1;
    }
    -1
}

/// Maps a column in a merged Stage 1 line back to the original line and adjusted column using the
/// continuation segments. Without segments it falls back to the Stage 1 to original map with the
/// raw column.
fn map_column_through_continuation(
    stage1_line: i32,
    column: i32,
    line_continuation_segments: &BTreeMap<i32, Vec<LineContinuationSegment>>,
    stage1_to_original_map: &BTreeMap<i32, i32>,
) -> (i32, i32) {
    if let Some(segments) = line_continuation_segments.get(&stage1_line) {
        for seg in segments {
            if column >= seg.start_column && column < seg.start_column + seg.length {
                return (seg.original_line, column - seg.start_column);
            }
        }
        // Column past all segments: use the last segment
        if let Some(last_seg) = segments.last() {
            if column >= last_seg.start_column {
                return (last_seg.original_line, column - last_seg.start_column);
            }
        }
    }

    let original_line = stage1_to_original_map.get(&stage1_line).copied().unwrap_or(stage1_line);
    (original_line, column)
}

/// Expands macros in a line, matching the longest name first as Core's MacroParser does: with macros
/// "string$" and "ng$", "gstring$" expands "string$". `expanded_macro_names` collects the macros
/// expanded, for source mapping; `currently_expanding` breaks self-referential and mutual cycles.
fn expand_macros(
    line: &str,
    sorted_macros: &[SortedMacro],
    macros: &StrMap<&MacroDefinition>,
    expanded_macro_names: &mut Vec<String>,
    currently_expanding: &mut StrSet,
) -> String {
    if sorted_macros.is_empty() || !line.contains('$') {
        return line.to_string();
    }

    let line: Vec<char> = line.chars().collect();
    let mut result = String::with_capacity(line.len() * 2);
    let mut text_buffer: Vec<char> = Vec::new();
    let mut i = 0usize;

    while i < line.len() {
        let c = line[i];

        // At a $, check if the accumulated text ends with a macro name
        if c == '$' {
            text_buffer.push(c);

            // The longest macro name that ends at this position (sorted order: the first match)
            let mut matched: Option<&MacroDefinition> = None;
            let mut macro_start_in_buffer = 0usize;

            for entry in sorted_macros {
                // Skip macros on the expansion stack (self-referential and mutual cycles)
                if currently_expanding.contains(&entry.def.name) {
                    continue;
                }
                if text_buffer.len() >= entry.name.len() && ends_with_ignore_case(&text_buffer, &entry.name) {
                    matched = Some(entry.def);
                    macro_start_in_buffer = text_buffer.len() - entry.name.len();
                    break;
                }
            }

            if let Some(macro_def) = matched {
                let macro_name = &macro_def.name;

                // Output the text before the macro name
                if macro_start_in_buffer > 0 {
                    result.extend(&text_buffer[..macro_start_in_buffer]);
                }
                text_buffer.clear();

                // Arguments after the macro name
                let mut pos = i + 1;
                skip_whitespace(&line, &mut pos);

                let arg_list: Vec<String>;
                let replacement_end: usize;

                if pos < line.len() && line[pos] == '(' {
                    let args_start = pos + 1;
                    let close_pos = find_matching_close(&line, pos, '(', ')');
                    if close_pos >= 0 {
                        let close_pos = close_pos as usize;
                        let args_str = range(&line, args_start, close_pos);
                        // Macro calls: only parentheses count for nesting
                        arg_list = parse_macro_parameters(&args_str);
                        replacement_end = close_pos + 1;
                    } else {
                        // Unbalanced parentheses: output as is and continue
                        result.push_str(macro_name);
                        i += 1;
                        continue;
                    }
                } else {
                    arg_list = Vec::new();
                    replacement_end = i + 1;
                }

                let resolved_args = resolve_macro_args(&macro_def.params, &macro_def.defaults, &arg_list, macros);
                if let Some(resolved_args) = resolved_args {
                    expanded_macro_names.push(macro_name.clone());

                    // Substitute the parameters, longest name first so nested names ("ab" before
                    // "a") work. C# throws on an empty parameter name; it is skipped here.
                    let mut macro_content = join_lines(&macro_def.content);
                    if !macro_def.params.is_empty() {
                        let mut sorted_params: Vec<(&String, &String)> =
                            macro_def.params.iter().zip(resolved_args.iter()).collect();
                        sorted_params.sort_by_key(|a| std::cmp::Reverse(a.0.chars().count()));
                        for (param, arg) in sorted_params {
                            if !param.is_empty() {
                                macro_content = macro_content.replace(param.as_str(), arg);
                            }
                        }
                    }

                    // Recursively expand macros in the result (nested expansions are tracked too),
                    // with this macro on the cycle guard.
                    currently_expanding.insert(macro_name);
                    macro_content = expand_macros(
                        &macro_content,
                        sorted_macros,
                        macros,
                        expanded_macro_names,
                        currently_expanding,
                    );
                    currently_expanding.remove(macro_name);

                    result.push_str(&macro_content);
                    i = replacement_end;
                } else {
                    // Parameter count mismatch: keep the full call text so the linter reports it
                    result.push_str(macro_name);
                    result.extend(&line[i + 1..replacement_end]);
                    i = replacement_end;
                }
            } else {
                // No macro matched: keep accumulating
                i += 1;
            }
        } else if is_macro_letter(c, text_buffer.len()) {
            // Potential macro name character
            text_buffer.push(c);
            i += 1;
        } else {
            // Non-macro character: flush the buffer and output the character
            if !text_buffer.is_empty() {
                result.extend(&text_buffer);
                text_buffer.clear();
            }
            result.push(c);
            i += 1;
        }
    }

    // Flush the remaining buffer
    if !text_buffer.is_empty() {
        result.extend(&text_buffer);
    }

    result
}

/// Resolves positional and keyword arguments against the macro parameters. Returns the resolved
/// arguments parallel to `param_names`, or `None` when the call is invalid.
///
/// `defaults` is parallel to `param_names` when any parameter has a default; the empty list stands
/// for C#'s null `Defaults`.
fn resolve_macro_args(
    param_names: &[String],
    defaults: &[Option<String>],
    arg_list: &[String],
    macros: &StrMap<&MacroDefinition>,
) -> Option<Vec<String>> {
    if param_names.is_empty() {
        return if arg_list.is_empty() { Some(Vec::new()) } else { None };
    }

    let has_defaults = !defaults.is_empty();

    // A single empty argument counts as "no arguments": macro$()
    let effective_args: &[String] =
        if arg_list.len() == 1 && trim(&arg_list[0].chars().collect::<Vec<_>>()).is_empty() && has_defaults {
            &[]
        } else {
            arg_list
        };

    let mut resolved: Vec<Option<String>> = vec![None; param_names.len()];
    let mut positional_count = 0usize;
    let mut seen_keyword = false;
    for arg in effective_args {
        let mut index: Option<usize> = None;
        let mut keyword_value: Option<String> = None;
        if let Some((name, value)) = try_parse_keyword_arg(arg) {
            index = param_names.iter().position(|p| *p == name);
            // Unknown names are left for the linter
            if index.is_none() && name.ends_with('$') && !macros.contains_key(&name) {
                return None;
            }
            keyword_value = Some(value);
        }
        if let (Some(index), Some(value)) = (index, keyword_value) {
            if resolved[index].is_some() {
                return None;
            }
            seen_keyword = true;
            resolved[index] = Some(value);
        } else {
            if seen_keyword || positional_count >= param_names.len() {
                return None;
            }
            resolved[positional_count] = Some(arg.clone());
            positional_count += 1;
        }
    }

    let mut out = Vec::with_capacity(resolved.len());
    for (i, r) in resolved.into_iter().enumerate() {
        out.push(r.or_else(|| defaults.get(i).cloned().flatten())?);
    }
    Some(out)
}

/// Whether a `#def` line has an '=' at parenthesis depth 0, which makes it an inline macro. Scans from
/// `start_pos` (after "#def ") so an '=' inside parentheses is not a false positive.
fn has_equals_outside_parens(line: &[char], start_pos: usize) -> bool {
    let mut depth = 0;
    for &c in line.iter().skip(start_pos) {
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
        } else if c == '=' && depth == 0 {
            return true;
        }
    }
    false
}

/// Whether the buffer ends with `value`, ignoring case (`char.ToUpperInvariant`).
fn ends_with_ignore_case(buffer: &[char], value: &[char]) -> bool {
    if buffer.len() < value.len() {
        return false;
    }
    let offset = buffer.len() - value.len();
    value.iter().enumerate().all(|(i, v)| up(buffer[offset + i]) == up(*v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mac(name: &str, params: &[&str], defaults: &[Option<&str>], content: &[&str]) -> MacroDefinition {
        MacroDefinition {
            name: name.to_string(),
            params: params.iter().map(|s| s.to_string()).collect(),
            defaults: defaults.iter().map(|d| d.map(str::to_string)).collect(),
            content: content.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    fn expand(line: &str, defs: &[MacroDefinition]) -> (String, Vec<String>) {
        let mut macros: StrMap<&MacroDefinition> = StrMap::new_ci();
        for d in defs {
            macros.try_add(&d.name, d);
        }
        let mut sorted: Vec<SortedMacro> = macros
            .values()
            .map(|m| SortedMacro {
                name: m.name.chars().collect(),
                def: m,
            })
            .collect();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.name.len()));
        let mut names = Vec::new();
        let mut expanding = StrSet::new();
        let out = expand_macros(line, &sorted, &macros, &mut names, &mut expanding);
        (out, names)
    }

    #[test]
    fn simple_and_parameterised_expansion() {
        let defs = [mac("pi2$", &[], &[], &["6.28"]), mac("sq$", &["x$"], &[], &["(x$*x$)"])];
        assert_eq!(
            expand("a = pi2$ + sq$(3)", &defs),
            ("a = 6.28 + (3*3)".to_string(), vec!["pi2$".into(), "sq$".into()])
        );
        assert_eq!(expand("no macros here", &defs).0, "no macros here");
    }

    #[test]
    fn longest_name_wins_and_names_match_ignoring_case() {
        let defs = [mac("ng$", &[], &[], &["1"]), mac("string$", &[], &[], &["2"])];
        assert_eq!(expand("gstring$ + NG$", &defs).0, "g2 + 1");
    }

    #[test]
    fn keyword_defaults_and_mismatched_calls() {
        let defs = [mac("f$", &["a", "b"], &[None, Some("10")], &["a+b"])];
        assert_eq!(expand("f$(1)", &defs).0, "1+10");
        assert_eq!(expand("f$(b=2; a=1)", &defs).0, "1+2");
        assert_eq!(expand("f$()", &defs).0, "f$()");
        assert_eq!(expand("f$(1; 2; 3)", &defs).0, "f$(1; 2; 3)");
        assert_eq!(expand("f$(1", &defs).0, "f$(1");
    }

    #[test]
    fn multiline_content_nested_and_cyclic_macros() {
        let defs = [
            mac("two$", &[], &[], &["a = 1", "b = 2"]),
            mac("outer$", &[], &[], &["x + inner$"]),
            mac("inner$", &[], &[], &["y"]),
            mac("loop$", &[], &[], &["loop$ + 1"]),
        ];
        assert_eq!(expand("two$", &defs).0, "a = 1\nb = 2");
        let (out, names) = expand("outer$", &defs);
        assert_eq!(out, "x + y");
        assert_eq!(names, vec!["outer$".to_string(), "inner$".to_string()]);
        assert_eq!(expand("loop$", &defs).0, "loop$ + 1");
    }

    #[test]
    fn longer_parameter_names_are_replaced_first() {
        let defs = [mac("m$", &["a", "ab"], &[], &["a ab"])];
        assert_eq!(expand("m$(1; 2)", &defs).0, "1 2");
        // An empty parameter name is skipped (C# would throw)
        let defs = [mac("e$", &[""], &[], &["x"])];
        assert_eq!(expand("e$(1)", &defs).0, "x");
    }

    #[test]
    fn macro_name_column_needs_a_boundary() {
        assert_eq!(find_macro_name_column("x = doubleCheck$ + check$", "check$"), 19);
        assert_eq!(find_macro_name_column("doubleCheck$", "check$"), -1);
        assert_eq!(find_macro_name_column("CHECK$", "check$"), 0);
    }

    #[test]
    fn column_mapping_through_continuation_segments() {
        let mut segs = BTreeMap::new();
        segs.insert(
            0,
            vec![
                LineContinuationSegment {
                    original_line: 4,
                    start_column: 0,
                    length: 5,
                },
                LineContinuationSegment {
                    original_line: 5,
                    start_column: 5,
                    length: 3,
                },
            ],
        );
        let mut s1 = BTreeMap::new();
        s1.insert(1, 9);
        assert_eq!(map_column_through_continuation(0, 2, &segs, &s1), (4, 2));
        assert_eq!(map_column_through_continuation(0, 6, &segs, &s1), (5, 1));
        assert_eq!(map_column_through_continuation(0, 20, &segs, &s1), (5, 15));
        assert_eq!(map_column_through_continuation(1, 3, &segs, &s1), (9, 3));
        assert_eq!(map_column_through_continuation(7, 3, &segs, &s1), (7, 3));
    }

    #[test]
    fn inline_def_detection() {
        let c = |s: &str| s.chars().collect::<Vec<_>>();
        assert!(has_equals_outside_parens(&c("#def a$ = 1"), 5));
        assert!(!has_equals_outside_parens(&c("#def a$(x=1)"), 5));
        assert!(has_equals_outside_parens(&c("#def a$(x=1) = x"), 5));
    }
}
