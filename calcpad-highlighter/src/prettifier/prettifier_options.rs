//! PrettifierOptions.cs
#[derive(Clone, Debug)]
pub struct PrettifierOptions {
    /// String emitted per indent level. The default is a single tab; `None` also means a tab.
    pub indent_unit: Option<String>,
    /// Trim trailing whitespace from each line before re-emitting.
    pub trim_trailing_whitespace: bool,
}

impl Default for PrettifierOptions {
    fn default() -> Self {
        PrettifierOptions {
            indent_unit: Some("\t".to_string()),
            trim_trailing_whitespace: true,
        }
    }
}
