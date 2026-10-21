//! CalcpadTokenizer.Definitions.cs: lint-mode definition extraction and the include map.
use super::models::TokenType;
use super::CalcpadTokenizer;
use crate::content_resolution::result::{
    CommandBlockInfo, CustomUnitDefinition, FunctionDefinition, SourceInfo, VariableDefinition,
};
use crate::linter::helpers::parsing_helpers::{find_matching_close, parse_param_list};
use crate::linter::models::DefinitionMetadata;
use crate::text::{eq_ignore_case, is_white_space_slice, to_string, trim};
use std::collections::BTreeMap;

fn trim_end_ws(s: &str) -> &str {
    s.trim_end_matches(crate::text::is_white_space)
}

impl CalcpadTokenizer {
    /// Sets the include map for source tracking in Lint and Macro modes.
    pub fn set_include_map(&mut self, include_map: &BTreeMap<i32, SourceInfo>) {
        self.include_map = Some(include_map.clone());
    }

    pub(super) fn init_lint_state(&mut self) {
        self.lint_capturing_expression = false;
        self.lint_expression_start_column = 0;
        self.lint_def_line_text = Vec::new();
        self.lint_def_line = -1;
        self.lint_def_name = None;
        self.lint_def_name_line = -1;
        self.lint_def_is_function = false;
        self.lint_def_is_custom_unit = false;
        self.lint_def_function_params = None;
        self.lint_def_is_const = false;
        self.lint_pending_function_params = Vec::new();
        self.lint_pending_is_const = false;
        self.lint_expecting_read_variable = false;
        self.lint_pending_read_variable_name = None;
        self.lint_pending_metadata = None;
    }

    /// Called from `track_definitions` for all tokens except Operator "=".
    pub(super) fn track_definitions_lint(&mut self, ty: TokenType, text: &str) {
        if self.lint_capturing_expression {
            // A comment delimiter ends the current code region: emit the pending definition so a
            // later definition after the comment can be captured on its own
            if ty >= TokenType::Comment && ty <= TokenType::Svg {
                self.emit_pending_definition_lint();
            }
            return;
        }

        match ty {
            TokenType::Keyword => {
                let trimmed = trim_end_ws(text);
                if eq_ignore_case(trimmed, "#const") {
                    self.lint_pending_is_const = true;
                } else if eq_ignore_case(trimmed, "#read") {
                    self.lint_expecting_read_variable = true;
                }
            }
            TokenType::Variable => {
                if self.lint_expecting_read_variable && self.lint_pending_read_variable_name.is_none() {
                    self.lint_pending_read_variable_name = Some(text.to_string());
                }
            }
            TokenType::LocalVariable => {
                // Only while defining params (not in default values)
                if self.pending_function_name.is_some()
                    && self.pending_function_paren_depth > 0
                    && self.state.is_in_function_params
                {
                    self.lint_pending_function_params.push(text.to_string());
                }
            }
            _ => {}
        }
    }

    /// Called when "=" confirms a variable or function definition: starts expression capture.
    pub(super) fn on_equals_seen_lint(&mut self, is_first_def: bool) {
        if self.lint_capturing_expression {
            return;
        }
        if !is_first_def {
            self.lint_pending_is_const = false;
            self.lint_pending_function_params.clear();
            return;
        }

        let mut is_custom_unit = false;
        if self.pending_variable_name.is_some() && self.pending_function_name.is_none() {
            is_custom_unit = self.is_custom_unit_line(self.state.line);
        }

        if self.pending_function_name.is_some() && self.pending_function_paren_depth == 0 {
            self.lint_def_name = self.pending_function_name.clone();
            self.lint_def_name_line = self.pending_function_line;
            self.lint_def_is_function = true;
            self.lint_def_is_custom_unit = false;
            self.lint_def_function_params = Some(self.lint_pending_function_params.clone());
            self.lint_def_is_const = self.lint_pending_is_const;
        } else if self.pending_variable_name.is_some() {
            if is_custom_unit {
                self.lint_def_name = self.pending_variable_name.clone();
                self.lint_def_name_line = self.pending_variable_line;
                self.lint_def_is_function = false;
                self.lint_def_is_custom_unit = true;
                self.lint_def_is_const = false;
            } else {
                self.lint_def_name = self.pending_variable_name.clone();
                self.lint_def_name_line = self.pending_variable_line;
                self.lint_def_is_function = false;
                self.lint_def_is_custom_unit = false;
                self.lint_def_function_params = None;
                self.lint_def_is_const = self.lint_pending_is_const;
            }
        } else {
            self.lint_pending_is_const = false;
            self.lint_pending_function_params.clear();
            return;
        }

        self.lint_capturing_expression = true;
        self.lint_expression_start_column = self.state.token_start_column + 1;
        self.lint_def_line_text = self.state.text.to_vec();
        self.lint_def_line = self.state.line;
        self.lint_pending_is_const = false;
        self.lint_pending_function_params.clear();
    }

    /// Called at the end of each line to finalize pending lint-mode definitions.
    pub(super) fn finalize_line_lint(&mut self) {
        let mut produced = self.emit_pending_definition_lint();

        // #read variable (no = on the line, so not captured by expression)
        if let Some(name) = self.lint_pending_read_variable_name.clone() {
            let line_text = to_string(&self.state.text).to_uppercase();
            let is_vector = line_text.contains("TYPE=V");
            let definition = if is_vector { "#read vector" } else { "#read matrix" };
            let (source, source_file) = self.get_source_info(self.state.line);

            if !self.result.defined_variables.contains_key(&name) {
                self.result.variable_definitions.push(VariableDefinition {
                    name,
                    definition: definition.to_string(),
                    line_number: self.state.line,
                    source,
                    source_file,
                    is_const: false,
                    description: self.lint_pending_metadata.as_ref().and_then(|m| m.description.clone()),
                });
                produced = true;
            }
        }

        if produced {
            self.lint_pending_metadata = None;
        }

        if !produced {
            let text = self.state.text.clone();
            if let Some(m) = DefinitionMetadata::try_parse(&text) {
                self.lint_pending_metadata = Some(m);
            } else if !is_white_space_slice(&text) {
                self.lint_pending_metadata = None;
            }
        }

        self.lint_pending_is_const = false;
        self.lint_expecting_read_variable = false;
        self.lint_pending_read_variable_name = None;
    }

    /// Emits the captured definition (if any) and clears the capture state; true if emitted.
    fn emit_pending_definition_lint(&mut self) -> bool {
        if !self.lint_capturing_expression || self.lint_def_name.is_none() {
            return false;
        }
        let name = self.lint_def_name.clone().unwrap();
        let line_text = self.lint_def_line_text.clone();
        let expression = extract_expression_from_line(&line_text, self.lint_expression_start_column);
        let (source, source_file) = self.get_source_info(self.lint_def_name_line);
        let meta = self.lint_pending_metadata.clone();
        let description = meta.as_ref().and_then(|m| m.description.clone());

        if self.lint_def_is_custom_unit {
            self.result.custom_unit_definitions.push(CustomUnitDefinition {
                name,
                definition: expression,
                line_number: self.lint_def_name_line,
                source,
                source_file,
                description,
            });
        } else if self.lint_def_is_function {
            let func_params = self.lint_def_function_params.clone().unwrap_or_default();
            let mut func_def = FunctionDefinition {
                name: name.clone(),
                params: func_params,
                defaults: extract_function_param_defaults(&line_text).unwrap_or_default(),
                line_number: self.lint_def_name_line,
                source,
                source_file,
                expression: Some(expression.clone()),
                is_const: self.lint_def_is_const,
                description,
                param_types: meta.as_ref().and_then(|m| m.param_types.clone()),
                param_descriptions: meta.as_ref().and_then(|m| m.param_descriptions.clone()),
                return_type: meta.as_ref().and_then(|m| m.return_type.clone()),
                command_block: None,
            };
            if let Some(cb) = detect_command_block(&expression, &func_def) {
                func_def.command_block = Some(cb.clone());
                self.result.command_block_functions.insert(&name, cb);
            }
            self.result.function_definitions.push(func_def);
        } else {
            self.result.variable_definitions.push(VariableDefinition {
                name,
                definition: expression,
                line_number: self.lint_def_name_line,
                source,
                source_file,
                is_const: self.lint_def_is_const,
                description,
            });
        }

        self.lint_capturing_expression = false;
        self.lint_def_name = None;
        self.lint_def_name_line = -1;
        self.lint_def_is_function = false;
        self.lint_def_is_custom_unit = false;
        self.lint_def_function_params = None;
        self.lint_def_is_const = false;
        true
    }

    /// (source, source file) of a line from the include map; "local" when absent.
    pub(super) fn get_source_info(&self, line: i32) -> (String, Option<String>) {
        if let Some(m) = &self.include_map {
            if let Some(si) = m.get(&line) {
                return (si.source.clone(), si.source_file.clone());
            }
        }
        ("local".to_string(), None)
    }

    /// A custom unit line starts with the Operator "." as its first meaningful token.
    fn is_custom_unit_line(&self, line: i32) -> bool {
        let Some(tokens) = self.result.tokens_by_line.get(&line) else {
            return false;
        };
        for t in tokens {
            if t.token_type == TokenType::None && t.text.trim_start_matches(crate::text::is_white_space).is_empty() {
                continue;
            }
            return t.text == ".";
        }
        false
    }
}

/// The expression text after the `=`, up to the first comment delimiter, trimmed.
fn extract_expression_from_line(line_text: &[char], start_column: i32) -> String {
    let start = start_column.max(0) as usize;
    if start >= line_text.len() {
        return String::new();
    }
    let after = &line_text[start..];
    let end = after
        .iter()
        .position(|c| *c == '\'' || *c == '"')
        .unwrap_or(after.len());
    to_string(trim(&after[..end]))
}

fn extract_function_param_defaults(line: &[char]) -> Option<Vec<Option<String>>> {
    let open = line.iter().position(|c| *c == '(')?;
    let close = find_matching_close(line, open, '(', ')');
    if close < 0 {
        return None;
    }
    parse_param_list(&to_string(&line[open + 1..close as usize])).1
}

fn starts_with_ic(s: &[char], p: &str) -> bool {
    let n = p.chars().count();
    s.len() >= n && eq_ignore_case(&to_string(&s[..n]), p)
}

/// Detects `$Inline{...}`, `$Block{...}` or `$While{...}` and extracts its statements.
fn detect_command_block(expression: &str, func_def: &FunctionDefinition) -> Option<CommandBlockInfo> {
    let expr: Vec<char> = expression.chars().collect();
    let block_type = if starts_with_ic(&expr, "$Inline") {
        "Inline"
    } else if starts_with_ic(&expr, "$Block") {
        "Block"
    } else if starts_with_ic(&expr, "$While") {
        "While"
    } else {
        return None;
    };
    let brace = expr.iter().position(|c| *c == '{')?;
    let content = extract_block_content(&expr, brace);
    let statements = split_block_statements(&content);
    Some(CommandBlockInfo {
        function_name: func_def.name.clone(),
        parameters: func_def.params.clone(),
        block_type: block_type.to_string(),
        statements,
        full_line: format!("{}({}) = {}", func_def.name, func_def.params.join(";"), expression),
        line_number: func_def.line_number,
    })
}

fn extract_block_content(text: &[char], brace_start: usize) -> String {
    let mut depth = 0;
    let start = brace_start + 1;
    for i in brace_start..text.len() {
        if text[i] == '{' {
            depth += 1;
        } else if text[i] == '}' {
            depth -= 1;
            if depth == 0 {
                return to_string(&text[start..i]);
            }
        }
    }
    to_string(&text[start.min(text.len())..])
}

/// Splits command block content on `;` outside (), {} and [].
fn split_block_statements(content: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut cur: Vec<char> = Vec::new();
    let (mut paren, mut brace, mut bracket) = (0, 0, 0);
    for c in content.chars() {
        match c {
            '(' => paren += 1,
            ')' => paren -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            _ => {}
        }
        if c == ';' && paren == 0 && brace == 0 && bracket == 0 {
            let stmt = to_string(trim(&cur));
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    let last = to_string(trim(&cur));
    if !last.is_empty() {
        statements.push(last);
    }
    statements
}
