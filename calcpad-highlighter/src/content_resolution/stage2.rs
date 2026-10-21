//! ContentResolver.Stage2.cs: include resolution and macro collection.
use super::content_resolver::join_lines;
use super::result::{MacroDefinition, SourceInfo, Stage1Result, Stage2Result};
use super::sha256::sha256_hex;
use super::ContentResolver;
use crate::linter::helpers::character_helpers::is_macro_letter;
use crate::linter::helpers::cs;
use crate::linter::helpers::directive_state::{DirectiveState, ScopeMode};
use crate::linter::helpers::parameter_parser::parse_macro_parameters;
use crate::linter::helpers::parsing_helpers::{find_matching_close, skip_whitespace};
use crate::parsing::split_lines;
use crate::path_roots::{self, IncludeProvider, PathRoots};
use crate::text::{is_white_space, sub, to_string, trim, StrMap, StrSet};
use crate::tokenizer::models::TokenizerMode;
use crate::tokenizer::CalcpadTokenizer;
use std::collections::BTreeMap;

/// The state shared by the root pass and every (recursive) include expansion.
pub(crate) struct IncludeContext<'a> {
    include_files: &'a StrMap<String>,
    client_file_cache: &'a StrMap<Vec<u8>>,
    provider: &'a dyn IncludeProvider,
    expanded_lines: Vec<String>,
    source_map: BTreeMap<i32, i32>,
    include_map: BTreeMap<i32, SourceInfo>,
    visited: StrSet,
    included_file_hashes: StrMap<String>,
    /// Shared across the root and every included file, like MacroParser's own instance field: a
    /// `{project}`/`{library}` reference resolves against the first declaration in document order.
    path_roots: PathRoots,
}

impl IncludeContext<'_> {
    fn push_line(&mut self, line: String, stage1_line: usize, info: SourceInfo) {
        self.expanded_lines.push(line);
        let index = (self.expanded_lines.len() - 1) as i32;
        self.source_map.insert(index, stage1_line as i32);
        self.include_map.insert(index, info);
    }

    fn push_include_error(&mut self, raw_file_name: &str, stage1_line: usize) {
        self.push_line(
            format!("' Error: Include file not provided: {raw_file_name}"),
            stage1_line,
            SourceInfo {
                source: "include".to_string(),
                source_file: Some(raw_file_name.to_string()),
                original_line: -1,
            },
        );
    }
}

impl ContentResolver {
    /// STAGE 2: resolve includes, then collect macros.
    /// Pass 1 replaces `#include` lines with the file content; pass 2 scans the combined content
    /// for macro definitions. `#read` lines are kept as they are (they read data at run time).
    pub(crate) fn process_stage2(
        &self,
        stage1: &Stage1Result,
        include_files: &StrMap<String>,
        client_file_cache: &StrMap<Vec<u8>>,
        source_file_path: Option<&str>,
    ) -> Stage2Result {
        // Pass 1: expand includes recursively, like Core's MacroParser
        let ctx = self.expand_includes(stage1, include_files, client_file_cache, source_file_path);

        // Pass 2: tokenize in Macro mode to collect macro definitions
        let mut tokenizer = CalcpadTokenizer::new();
        tokenizer.set_include_map(&ctx.include_map);
        let source = join_lines(&ctx.expanded_lines);
        let tokenizer_result = tokenizer.tokenize_mode(&source, TokenizerMode::Macro);

        let mut macro_definitions = tokenizer_result.macro_definitions;
        let duplicate_macros = tokenizer_result.duplicate_macros;

        let (macro_comment_params, macro_param_order) = compute_macro_comment_parameters(&mut macro_definitions);

        // Macro bodies for argument type resolution
        let mut macro_bodies: StrMap<String> = StrMap::new_ci();
        for m in &macro_definitions {
            if !m.content.is_empty() && !macro_bodies.contains_key(&m.name) {
                macro_bodies.insert(&m.name, join_lines(&m.content));
            }
        }

        Stage2Result {
            lines: ctx.expanded_lines,
            source_map: ctx.source_map,
            include_map: ctx.include_map,
            macro_definitions,
            duplicate_macros,
            path_roots: ctx.path_roots,
            macro_comment_parameters: macro_comment_params,
            macro_parameter_order: macro_param_order,
            macro_bodies,
            user_defined_macros: tokenizer_result.user_defined_macros,
            included_file_hashes: ctx.included_file_hashes,
        }
    }

    /// Pass 1 of stage 2: replaces `#include` lines with the file content, recursively, and
    /// collects the `#ProjectPath` / `#LibraryPath` declarations.
    pub(crate) fn expand_includes<'a>(
        &'a self,
        stage1: &Stage1Result,
        include_files: &'a StrMap<String>,
        client_file_cache: &'a StrMap<Vec<u8>>,
        source_file_path: Option<&str>,
    ) -> IncludeContext<'a> {
        let provider: &dyn IncludeProvider = self.provider.as_ref();
        let root_dir: Option<String> = match source_file_path {
            Some(p) if !p.is_empty() => path_roots::get_directory_name(p),
            _ => None,
        };

        let mut ctx = IncludeContext {
            include_files,
            client_file_cache,
            provider,
            expanded_lines: Vec::new(),
            source_map: BTreeMap::new(),
            include_map: BTreeMap::new(),
            visited: StrSet::new_ci(),
            included_file_hashes: StrMap::new_ci(),
            path_roots: PathRoots::new(),
        };

        for (i, line) in stage1.lines.iter().enumerate() {
            let line_chars: Vec<char> = line.chars().collect();
            let trimmed = trim(&line_chars);

            if let Some((is_project, decl_start, decl_length)) = PathRoots::is_declaration(trimmed) {
                if decl_length > 0 {
                    // A malformed or duplicate declaration is reported when the document is
                    // parsed by Core; the editor only needs the roots it can resolve.
                    let _ = ctx.path_roots.try_declare(
                        is_project,
                        &sub(trimmed, decl_start, decl_length),
                        root_dir.as_deref(),
                        provider,
                    );
                }
            } else if is_include_directive(trimmed) {
                let raw_file_name = extract_include_filename(trimmed);
                let (raw_file_name, _) = ctx.path_roots.try_expand(&raw_file_name, provider);
                expand_include(&mut ctx, &raw_file_name, i, 0, root_dir.as_deref());
                continue;
            }

            ctx.push_line(
                line.clone(),
                i,
                SourceInfo {
                    source: "local".to_string(),
                    ..Default::default()
                },
            );
        }
        ctx
    }
}

/// Computes which parameters are "comment parameters" for each macro: those appearing directly in a
/// comment section of the macro content, or passed to another macro's comment parameter position.
/// Fixed-point iteration handles the transitive dependencies. Also sets
/// `MacroDefinition::comment_parameters` on every definition.
pub(crate) fn compute_macro_comment_parameters(
    macro_definitions: &mut [MacroDefinition],
) -> (StrMap<StrSet>, StrMap<Vec<String>>) {
    let mut comment_params: StrMap<StrSet> = StrMap::new_ci();
    let mut param_order: StrMap<Vec<String>> = StrMap::new_ci();

    // Macro lookup (first definition wins) as an index into `macro_definitions`
    let mut macros_by_name: StrMap<usize> = StrMap::new_ci();
    for (idx, m) in macro_definitions.iter().enumerate() {
        if !macros_by_name.contains_key(&m.name) {
            macros_by_name.insert(&m.name, idx);
            param_order.insert(&m.name, m.params.clone());
            comment_params.insert(&m.name, StrSet::new());
        }
    }

    // First pass: direct comment parameters (params that appear in comment text)
    for idx in 0..macro_definitions.len() {
        if macro_definitions[idx].params.is_empty() {
            continue;
        }
        let name = macro_definitions[idx].name.clone();
        for content_line in &macro_definitions[idx].content {
            let found = CalcpadTokenizer::find_comment_params_in_line(content_line, &macro_definitions[idx].params);
            if let Some(set) = comment_params.get_mut(&name) {
                for param in found.iter() {
                    set.insert(param);
                }
            }
        }
        if let Some(set) = comment_params.get(&name) {
            macro_definitions[idx].comment_parameters = set.clone();
        }
    }

    // Second pass: transitive closure. If a param is passed to another macro's comment param
    // position it is a comment parameter too. Repeat until nothing changes.
    let mut changed = true;
    let max_iterations = 100;
    let mut iteration = 0;
    while changed && iteration < max_iterations {
        changed = false;
        iteration += 1;

        for macro_idx in 0..macro_definitions.len() {
            let macro_def = &macro_definitions[macro_idx];
            if macro_def.params.is_empty() {
                continue;
            }
            for content_line in &macro_def.content {
                let calls = find_macro_calls_in_line(content_line, &macros_by_name);
                for (called_macro, args) in calls {
                    let Some(&called_idx) = macros_by_name.get(&called_macro) else {
                        continue;
                    };
                    let called_params = &macro_definitions[called_idx].params;
                    let mut arg_idx = 0;
                    while arg_idx < args.len() && arg_idx < called_params.len() {
                        let called_param_name = &called_params[arg_idx];
                        let is_comment_param = comment_params
                            .get(&called_macro)
                            .is_some_and(|set| set.contains(called_param_name));
                        if is_comment_param {
                            let arg_text = &args[arg_idx];
                            for our_param in &macro_def.params {
                                if arg_text.contains(our_param.as_str()) {
                                    if let Some(own) = comment_params.get_mut(&macro_def.name) {
                                        if !own.contains(our_param) {
                                            own.insert(our_param);
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                        arg_idx += 1;
                    }
                }
            }
        }
    }

    // Final results onto the definitions
    for m in macro_definitions.iter_mut() {
        if let Some(cp) = comment_params.get(&m.name) {
            m.comment_parameters = cp.clone();
        }
    }

    (comment_params, param_order)
}

/// Finds macro calls (`name$(args)` where `name$` is a known macro) in a line and returns the called
/// macro name and its arguments.
pub(crate) fn find_macro_calls_in_line(line: &str, known_macros: &StrMap<usize>) -> Vec<(String, Vec<String>)> {
    let mut results = Vec::new();
    if line.is_empty() {
        return results;
    }
    let span: Vec<char> = line.chars().collect();

    let mut i = 0;
    while i < span.len() {
        let Some(dollar_idx) = crate::text::index_of_char(&span, '$', i) else {
            break;
        };

        // Work backwards from the $ to the start of the identifier
        let mut name_start = dollar_idx;
        while name_start > 0 && is_macro_letter(span[name_start - 1], dollar_idx - name_start) {
            name_start -= 1;
        }

        if name_start < dollar_idx {
            let macro_name = sub(&span, name_start, dollar_idx - name_start + 1);
            if known_macros.contains_key(&macro_name) {
                let mut after_dollar = dollar_idx + 1;
                skip_whitespace(&span, &mut after_dollar);

                if after_dollar < span.len() && span[after_dollar] == '(' {
                    let paren_start = after_dollar;
                    let close_pos = find_matching_close(&span, paren_start, '(', ')');
                    if close_pos >= 0 {
                        let close_pos = close_pos as usize;
                        let args_str = sub(&span, paren_start + 1, close_pos - paren_start - 1);
                        let args = parse_macro_parameters(&args_str);
                        results.push((macro_name, args));
                        i = close_pos + 1;
                        continue;
                    }
                }
            }
        }

        i = dollar_idx + 1;
    }

    results
}

/// Whether a trimmed line is an `#include` directive: starts with `#include` followed by whitespace.
pub(crate) fn is_include_directive(trimmed: &[char]) -> bool {
    cs::starts_with_ic(trimmed, "#include") && trimmed.len() > 8 && is_white_space(trimmed[8])
}

/// Whether a trimmed line is a `#local` or `#global` directive.
fn is_local_or_global_directive(trimmed: &[char]) -> bool {
    cs::starts_with_ic(trimmed, "#local") || cs::starts_with_ic(trimmed, "#global")
}

/// The raw file name of an `#include` directive (Core's `MacroParser.ParseInclude`): the text from
/// position 8 up to the first comment marker (`'` or `"`) or field parameter (`#`), trimmed.
/// `line_content` must be longer than 8 characters.
pub(crate) fn extract_include_filename(line_content: &[char]) -> String {
    let mut n = line_content.len();
    let quote_idx = line_content[8..]
        .iter()
        .position(|c| *c == '\'' || *c == '"')
        .map(|p| p as isize + 8)
        .unwrap_or(-1);
    let hash_idx = line_content[1..]
        .iter()
        .rposition(|c| *c == '#')
        .map(|p| p as isize + 1)
        .unwrap_or(-1);

    if quote_idx >= 9 {
        n = quote_idx as usize;
    }
    if hash_idx > 0 && (n == line_content.len() || (hash_idx as usize) < n) {
        n = hash_idx as usize;
    }
    if n < 9 {
        n = line_content.len();
    }
    to_string(trim(&line_content[8..n]))
}

/// Recursively expands an `#include` directive: resolves the file content and processes any
/// nested `#include` inside it, like Core's recursive Parse().
fn expand_include(
    ctx: &mut IncludeContext<'_>,
    raw_file_name: &str,
    stage1_line: usize,
    depth: i32,
    source_dir: Option<&str>,
) {
    if depth > 20 || raw_file_name.is_empty() || !ctx.visited.insert(raw_file_name) {
        ctx.push_include_error(raw_file_name, stage1_line);
        return;
    }

    let (file_content, resolved_path, from_filesystem) = resolve_file_content(ctx, raw_file_name, source_dir);
    let Some(file_content) = file_content else {
        ctx.push_include_error(raw_file_name, stage1_line);
        return;
    };

    if from_filesystem {
        if let Some(path) = &resolved_path {
            ctx.included_file_hashes
                .insert(path, sha256_hex(file_content.as_bytes()));
        }
    }

    // Process the included content through Stage 1 (line continuations)
    let content_chars: Vec<char> = file_content.chars().collect();
    let included_lines: Vec<String> = split_lines(&content_chars).into_iter().map(to_string).collect();
    let included_stage1 = ContentResolver::process_stage1(&included_lines);

    // #local sections are excluded from includes (Core's CalcpadReader.Include)
    let mut directives = DirectiveState::new();

    for (j, line) in included_stage1.lines.iter().enumerate() {
        let line_chars: Vec<char> = line.chars().collect();
        let trimmed = trim(&line_chars);

        if is_local_or_global_directive(trimmed) {
            directives.apply(trimmed);
            continue;
        }

        if directives.scope == ScopeMode::Local {
            continue;
        }

        let nested_source_dir: Option<String> = match &resolved_path {
            Some(p) => path_roots::get_directory_name(p),
            None => source_dir.map(str::to_string),
        };
        if let Some((is_project, decl_start, decl_length)) = PathRoots::is_declaration(trimmed) {
            // A #local-scoped declaration never gets here. One that does is part of the flattened
            // document and is added below like any other line.
            if decl_length > 0 {
                let _ = ctx.path_roots.try_declare(
                    is_project,
                    &sub(trimmed, decl_start, decl_length),
                    nested_source_dir.as_deref(),
                    ctx.provider,
                );
            }
        } else if is_include_directive(trimmed) {
            let nested_file_name = extract_include_filename(trimmed);
            let (nested_file_name, _) = ctx.path_roots.try_expand(&nested_file_name, ctx.provider);
            expand_include(
                ctx,
                &nested_file_name,
                stage1_line,
                depth + 1,
                nested_source_dir.as_deref(),
            );
            continue;
        }

        let original_line_in_include = included_stage1.source_map.get(&(j as i32)).copied().unwrap_or(j as i32);
        ctx.push_line(
            line.clone(),
            stage1_line,
            SourceInfo {
                source: "include".to_string(),
                source_file: Some(raw_file_name.to_string()),
                original_line: original_line_in_include,
            },
        );
    }
}

/// Resolves file content in Core's MacroParser lookup order: the file system (through the provider),
/// then `include_files`, then `client_file_cache`. Returns the content, the resolved full path and
/// whether the content came from the file system (only those have on-disk state worth hashing).
fn resolve_file_content(
    ctx: &IncludeContext<'_>,
    raw_file_name: &str,
    source_dir: Option<&str>,
) -> (Option<String>, Option<String>, bool) {
    let mut resolved_path: Option<String> = None;

    // 1. File system: expand env vars, get the full path, check it exists
    let expanded_path = ctx.provider.expand_environment_variables(raw_file_name);
    let cwd = ctx.provider.current_directory();
    // An invalid path (URLs, API syntax, ...) leaves `resolved_path` unset
    if let Ok(full) = path_roots::get_full_path(&expanded_path, source_dir, cwd.as_deref()) {
        if let Some(text) = ctx.provider.read_file(&full) {
            return (Some(text), Some(full), true);
        }
        resolved_path = Some(full);
    }

    // 2. includeFiles: resolved path first, then the raw file name
    if let Some(rp) = &resolved_path {
        if let Some(content) = ctx.include_files.get(rp) {
            return (Some(content.clone()), resolved_path, false);
        }
    }
    if let Some(content) = ctx.include_files.get(raw_file_name) {
        return (Some(content.clone()), None, false);
    }

    // 3. clientFileCache: resolved path first, then the raw file name
    if let Some(rp) = &resolved_path {
        if let Some(bytes) = ctx.client_file_cache.get(rp) {
            return (Some(String::from_utf8_lossy(bytes).into_owned()), resolved_path, false);
        }
    }
    if let Some(bytes) = ctx.client_file_cache.get(raw_file_name) {
        return (Some(String::from_utf8_lossy(bytes).into_owned()), None, false);
    }

    (None, None, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::split_lines;
    use crate::text::chars;

    #[test]
    fn include_directive_detection() {
        assert!(is_include_directive(&chars("#include a.cpd")));
        assert!(is_include_directive(&chars("#INCLUDE\ta.cpd")));
        assert!(!is_include_directive(&chars("#include")));
        assert!(!is_include_directive(&chars("#includes a")));
    }

    #[test]
    fn include_filename_extraction() {
        let f = |s: &str| extract_include_filename(&chars(s));
        assert_eq!(f("#include lib/a.cpd"), "lib/a.cpd");
        assert_eq!(f("#include lib/a.cpd 'comment"), "lib/a.cpd");
        assert_eq!(f("#include lib/a.cpd #x=1"), "lib/a.cpd");
        assert_eq!(f("#include   a b.cpd  "), "a b.cpd");
        assert_eq!(f("#include  'x"), "");
    }

    #[test]
    fn macro_calls_in_a_line() {
        let mut known: StrMap<usize> = StrMap::new_ci();
        known.insert("m$", 0);
        let calls = find_macro_calls_in_line("a = M$(x; (y)) + m$ (z) + other$(1)", &known);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, "M$");
        assert_eq!(calls[0].1, vec!["x".to_string(), "(y)".to_string()]);
        assert_eq!(calls[1].0, "m$");
    }

    use crate::path_roots::{path_combine, NativeDisk};

    struct Tmp(std::path::PathBuf);
    impl Tmp {
        fn new(tag: &str) -> Tmp {
            let n = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let p = std::env::temp_dir().join(format!("cpd_s2_{tag}_{}_{n}", std::process::id()));
            std::fs::create_dir_all(&p).unwrap();
            Tmp(p.canonicalize().unwrap())
        }
        fn path(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn stage1(content: &str) -> Stage1Result {
        let c = chars(content);
        let lines: Vec<String> = split_lines(&c).into_iter().map(to_string).collect();
        ContentResolver::process_stage1(&lines)
    }

    fn pass1(
        r: &ContentResolver,
        content: &str,
        files: &StrMap<String>,
        source: Option<&str>,
    ) -> (Vec<String>, PathRoots) {
        let cache = StrMap::new_ci();
        let s1 = stage1(content);
        let ctx = r.expand_includes(&s1, files, &cache, source);
        (ctx.expanded_lines, ctx.path_roots)
    }

    #[test]
    fn include_with_library_token_resolves_against_the_declared_root() {
        let t = Tmp::new("lib");
        std::fs::create_dir_all(format!("{}/lib", t.path())).unwrap();
        let lib_dir = format!("{}/lib", t.path());
        let resolved = path_combine(&lib_dir, "steel.cpd");
        let content = format!("#LibraryPath {lib_dir}\n#include {{library}}/steel.cpd\nx = 1");
        let mut files = StrMap::new();
        files.insert(&resolved, "y = 2".to_string());

        let r = ContentResolver::with_provider(Box::new(NativeDisk));
        let (lines, _) = pass1(&r, &content, &files, Some(&format!("{}/main.cpd", t.path())));
        let joined = lines.join("\n");
        assert!(!joined.contains("Error: Include file not provided"), "{joined}");
        assert!(joined.contains("y = 2"));
    }

    #[test]
    fn include_with_an_undeclared_token_falls_back_to_not_found() {
        let r = ContentResolver::new();
        let (lines, _) = pass1(
            &r,
            "#include {library}/steel.cpd\nx = 1",
            &StrMap::new_ci(),
            Some("/tmp/project/main.cpd"),
        );
        assert!(lines.join("\n").contains("Error: Include file not provided"));
    }

    #[test]
    fn root_declared_inside_an_include_is_usable_by_the_parent() {
        let t = Tmp::new("decl");
        std::fs::create_dir_all(format!("{}/lib", t.path())).unwrap();
        let lib_dir = format!("{}/lib", t.path());
        let mut files = StrMap::new();
        files.insert(&path_combine(&t.path(), "decl.cpd"), format!("#LibraryPath {lib_dir}"));
        files.insert(&path_combine(&lib_dir, "steel.cpd"), "y = 2".to_string());

        let r = ContentResolver::with_provider(Box::new(NativeDisk));
        let (lines, roots) = pass1(
            &r,
            "#include decl.cpd\n#include {library}/steel.cpd\nx = 1",
            &files,
            Some(&format!("{}/main.cpd", t.path())),
        );
        let joined = lines.join("\n");
        assert!(!joined.contains("Error: Include file not provided"), "{joined}");
        assert!(joined.contains("y = 2"));
        assert_eq!(roots.library(), Some(lib_dir.as_str()));
    }

    #[test]
    fn include_with_a_user_token_resolves_with_no_declaration() {
        struct Home;
        impl IncludeProvider for Home {
            fn user_profile(&self) -> String {
                "/home/u".to_string()
            }
        }
        let mut files = StrMap::new();
        files.insert("/home/u/lib/steel.cpd", "y = 2".to_string());
        let r = ContentResolver::with_provider(Box::new(Home));
        let (lines, _) = pass1(
            &r,
            "#include {user}/lib/steel.cpd\nx = 1",
            &files,
            Some("/tmp/project/main.cpd"),
        );
        let joined = lines.join("\n");
        assert!(!joined.contains("Error: Include file not provided"), "{joined}");
        assert!(joined.contains("y = 2"));
    }

    #[test]
    fn includes_from_the_maps_map_lines_and_filter_local_sections() {
        let mut files = StrMap::new_ci();
        files.insert(
            "a.cpd",
            "p = 1\n#local\nsecret = 2\n#global\nq = (1;\n2)\n#include b.cpd".to_string(),
        );
        files.insert("B.CPD", "r = 3".to_string());
        let mut cache: StrMap<Vec<u8>> = StrMap::new_ci();
        cache.insert("c.cpd", b"z = 9".to_vec());
        let r = ContentResolver::new();
        let s1 = stage1("#include a.cpd\nx = 1\n#include c.cpd\n#include a.cpd\n#include nope.cpd");
        let ctx = r.expand_includes(&s1, &files, &cache, None);
        assert_eq!(
            ctx.expanded_lines,
            vec![
                "p = 1",
                "q = (1;2)",
                "r = 3",
                "x = 1",
                "z = 9",
                // The same file again: the visited set is shared, as in C#
                "' Error: Include file not provided: a.cpd",
                "' Error: Include file not provided: nope.cpd",
            ]
        );
        // Original line inside the include, after its own stage 1 merging
        assert_eq!(ctx.include_map[&0].original_line, 0);
        assert_eq!(ctx.include_map[&1].original_line, 4);
        assert_eq!(ctx.include_map[&1].source_file.as_deref(), Some("a.cpd"));
        assert_eq!(ctx.include_map[&3].source, "local");
        assert_eq!(ctx.source_map[&1], 0);
        assert_eq!(ctx.source_map[&3], 1);
        assert!(ctx.included_file_hashes.is_empty());
    }

    #[test]
    fn filesystem_includes_are_hashed() {
        let t = Tmp::new("hash");
        let file = path_combine(&t.path(), "inc.cpd");
        std::fs::write(&file, "a = 1").unwrap();
        let r = ContentResolver::with_provider(Box::new(NativeDisk));
        let s1 = stage1("#include inc.cpd");
        let files = StrMap::new_ci();
        let cache = StrMap::new_ci();
        let ctx = r.expand_includes(&s1, &files, &cache, Some(&path_combine(&t.path(), "main.cpd")));
        assert_eq!(ctx.expanded_lines, vec!["a = 1"]);
        assert_eq!(
            ctx.included_file_hashes.get(&file).map(String::as_str),
            Some(super::super::sha256::sha256_hex(b"a = 1").as_str())
        );
    }
}
