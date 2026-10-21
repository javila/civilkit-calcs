//! ContentResolver.cs: resolves Calcpad content through three stages.
//!
//! * Stage 1: line continuations
//! * Stage 2: include resolution and macro collection
//! * Stage 3: macro expansion and definition collection
//!
//! The caller pre-fetches the include files and passes them as maps. The C# resolver also reads
//! included files from disk; here that goes through an [`IncludeProvider`], which is [`NoDisk`]
//! unless [`ContentResolver::with_provider`] is used (the browser build never does).
use super::result::StagedResolvedContent;
use crate::parsing::split_lines;
use crate::path_roots::{IncludeProvider, NoDisk};
use crate::text::StrMap;

pub struct ContentResolver {
    pub(crate) provider: Box<dyn IncludeProvider>,
}

impl Default for ContentResolver {
    fn default() -> Self {
        ContentResolver::new()
    }
}

impl ContentResolver {
    /// A resolver with no file system access: includes come only from the maps.
    pub fn new() -> ContentResolver {
        ContentResolver {
            provider: Box::new(NoDisk),
        }
    }

    /// A resolver that also looks for included files, and for the folders named by
    /// `#ProjectPath` / `#LibraryPath`, through `provider` (for example `NativeDisk`).
    pub fn with_provider(provider: Box<dyn IncludeProvider>) -> ContentResolver {
        ContentResolver { provider }
    }

    /// Get staged content with all three stages.
    ///
    /// * `content`: raw Calcpad source
    /// * `include_files`: file name to file content for `#include`, case-insensitive in C# when the
    ///   caller passes none (use `StrMap::new_ci()`); `None` is an empty case-insensitive map
    /// * `client_file_cache`: file name to raw bytes from the client cache
    /// * `source_file_path`: path of the document, the base for relative includes
    ///
    /// C# throws `ArgumentException` for empty `content`; this returns the staged result of one
    /// empty line instead.
    pub fn get_staged_content(
        &self,
        content: &str,
        include_files: Option<&StrMap<String>>,
        client_file_cache: Option<&StrMap<Vec<u8>>>,
        source_file_path: Option<&str>,
    ) -> StagedResolvedContent {
        let default_files = StrMap::new_ci();
        let default_cache = StrMap::new_ci();
        let include_files = include_files.unwrap_or(&default_files);
        let client_file_cache = client_file_cache.unwrap_or(&default_cache);

        let chars: Vec<char> = content.chars().collect();
        let lines: Vec<String> = split_lines(&chars).into_iter().map(crate::text::to_string).collect();

        // Stage 1: process line continuations only
        let stage1 = Self::process_stage1(&lines);

        // Stage 2: resolve includes, collect macros (but do not expand)
        let stage2 = self.process_stage2(&stage1, include_files, client_file_cache, source_file_path);

        // Stage 3: expand macros, collect all definitions
        let stage3 = Self::process_stage3(&stage2, &stage1);

        StagedResolvedContent { stage1, stage2, stage3 }
    }
}

/// `JoinLines`: the lines joined by `\n`.
pub(crate) fn join_lines(lines: &[String]) -> String {
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_maps_continuations_macros_and_definitions() {
        let src = "#def add$(a; b) = a + b\nx = 1 + _\n2\ny = add$(x; 3)\nf(t) = t\n";
        let staged = ContentResolver::new().get_staged_content(src, None, None, None);

        // Stage 1 merged lines 1-2
        assert_eq!(staged.stage1.lines[1], "x = 1 +2");
        assert_eq!(staged.stage1.source_map[&2], 3);
        // Stage 2 keeps the macro definition and finds it
        assert_eq!(staged.stage2.macro_definitions.len(), 1);
        assert_eq!(staged.stage2.macro_definitions[0].name, "add$");
        // Stage 3 dropped the #def line and expanded the call
        assert_eq!(staged.stage3.lines, vec!["x = 1 +2", "y = x + 3", "f(t) = t", ""]);
        assert_eq!(staged.stage3.macro_expansions[&1].macro_names, vec!["add$".to_string()]);
        // Definitions carry original lines; the macro call site is indexed on its original line
        let x = &staged.stage3.variable_index.get("x").unwrap();
        assert!(x.iter().any(|l| l.is_assignment && l.line == 1));
        assert!(x.iter().any(|l| !l.is_assignment && l.line == 3));
        let m = staged.stage3.macro_index.get("ADD$").unwrap();
        assert!(m.iter().any(|l| l.is_assignment && l.line == 0 && l.column == 5));
        assert!(m.iter().any(|l| !l.is_assignment && l.line == 3 && l.column == 4));
        assert!(staged.stage3.user_defined_functions.contains_key("f"));
        assert!(staged.stage3.defined_variables.contains("X"));
        assert_eq!(staged.stage3.functions_with_params[0].line_number, 4);
    }

    #[test]
    fn empty_content_does_not_panic() {
        let staged = ContentResolver::new().get_staged_content("", None, None, None);
        assert_eq!(staged.stage1.lines, vec![String::new()]);
    }
}
