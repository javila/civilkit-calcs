//! LinterDiagnostic.cs (with DiagnosticRange)
use super::LinterSeverity;
use crate::linter::helpers::diagnostic_extensions::LineStage;

/// A range in the source code for highlighting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticRange {
    pub line: i32,
    pub column: i32,
    pub end_column: i32,
}

#[derive(Clone, Debug)]
pub struct LinterDiagnostic {
    /// 0-based original line number (after mapping).
    pub line: i32,
    pub column: i32,
    pub end_column: i32,
    pub code: String,
    pub message: String,
    pub severity: LinterSeverity,
    pub source: String,
    /// The line number in the processing stage (before mapping to original).
    pub stage_line: i32,
    /// Which processing stage `stage_line` refers to.
    pub stage: LineStage,
    /// Additional ranges when the diagnostic spans multiple lines (line continuations).
    pub additional_ranges: Option<Vec<DiagnosticRange>>,
}

impl Default for LinterDiagnostic {
    fn default() -> Self {
        LinterDiagnostic {
            line: 0,
            column: 0,
            end_column: 0,
            code: String::new(),
            message: String::new(),
            severity: LinterSeverity::Error,
            source: "Calcpad Linter".to_string(),
            stage_line: 0,
            stage: LineStage::Stage3,
            additional_ranges: None,
        }
    }
}

impl LinterDiagnostic {
    /// The camelCase JSON the server emits for a diagnostic (`LintDiagnostic` in the web
    /// controller). Columns are scalar-value counts; the caller converts them to UTF-16.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "line": self.line,
            "column": self.column,
            "endColumn": self.end_column,
            "code": self.code,
            "message": self.message,
            "severity": self.severity.name(),
            "severityId": self.severity as i32,
            "source": self.source,
        })
    }
}
