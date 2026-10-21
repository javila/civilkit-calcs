//! LinterSeverity.cs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum LinterSeverity {
    Error = 0,
    Warning = 1,
    Information = 2,
}

impl LinterSeverity {
    /// The severity name the server sends: "error", "warning" or "information".
    pub fn name(self) -> &'static str {
        match self {
            LinterSeverity::Error => "error",
            LinterSeverity::Warning => "warning",
            LinterSeverity::Information => "information",
        }
    }
}
