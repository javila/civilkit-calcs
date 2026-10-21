//! DiagnosticExtensions.cs: `AddDiagnostic`/`AddError`/`AddWarning`/`AddInformation` on LinterResult.
use crate::linter::constants::error_codes;
use crate::linter::models::{LinterDiagnostic, LinterResult, LinterSeverity};

/// Which processing stage a line number refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineStage {
    /// After line continuation processing.
    Stage1,
    /// After include handling.
    Stage2,
    /// After full processing (default).
    Stage3,
}

impl LinterResult {
    /// `AddDiagnostic(line, col, endCol, code, details, severity, stage)`.
    pub fn add_diagnostic_stage(
        &mut self,
        line: i32,
        column: i32,
        end_column: i32,
        code: &str,
        details: &str,
        severity: LinterSeverity,
        stage: LineStage,
    ) {
        let message = format!("{}: {}", error_codes::get_description(code), details);
        self.append_diagnostic(LinterDiagnostic {
            stage_line: line,
            stage,
            line,
            column,
            end_column,
            code: code.to_string(),
            message,
            severity,
            ..Default::default()
        });
    }

    /// `AddDiagnostic(line, col, endCol, code, details, severity)` (Stage 3).
    pub fn add_diagnostic(
        &mut self,
        line: i32,
        column: i32,
        end_column: i32,
        code: &str,
        details: &str,
        severity: LinterSeverity,
    ) {
        self.add_diagnostic_stage(line, column, end_column, code, details, severity, LineStage::Stage3);
    }

    /// `AddDiagnostic(line, col, endCol, code, severity)`: message is the code description only.
    pub fn add_diagnostic_plain(
        &mut self,
        line: i32,
        column: i32,
        end_column: i32,
        code: &str,
        severity: LinterSeverity,
    ) {
        self.append_diagnostic(LinterDiagnostic {
            stage_line: line,
            stage: LineStage::Stage3,
            line,
            column,
            end_column,
            code: code.to_string(),
            message: error_codes::get_description(code).to_string(),
            severity,
            ..Default::default()
        });
    }

    pub fn add_error(&mut self, line: i32, column: i32, end_column: i32, code: &str, details: &str) {
        self.add_diagnostic(line, column, end_column, code, details, LinterSeverity::Error);
    }
    pub fn add_error_stage(
        &mut self,
        line: i32,
        column: i32,
        end_column: i32,
        code: &str,
        details: &str,
        stage: LineStage,
    ) {
        self.add_diagnostic_stage(line, column, end_column, code, details, LinterSeverity::Error, stage);
    }
    pub fn add_error_plain(&mut self, line: i32, column: i32, end_column: i32, code: &str) {
        self.add_diagnostic_plain(line, column, end_column, code, LinterSeverity::Error);
    }
    pub fn add_warning(&mut self, line: i32, column: i32, end_column: i32, code: &str, details: &str) {
        self.add_diagnostic(line, column, end_column, code, details, LinterSeverity::Warning);
    }
    pub fn add_warning_stage(
        &mut self,
        line: i32,
        column: i32,
        end_column: i32,
        code: &str,
        details: &str,
        stage: LineStage,
    ) {
        self.add_diagnostic_stage(line, column, end_column, code, details, LinterSeverity::Warning, stage);
    }
    pub fn add_warning_plain(&mut self, line: i32, column: i32, end_column: i32, code: &str) {
        self.add_diagnostic_plain(line, column, end_column, code, LinterSeverity::Warning);
    }
    pub fn add_information(&mut self, line: i32, column: i32, end_column: i32, code: &str, details: &str) {
        self.add_diagnostic(line, column, end_column, code, details, LinterSeverity::Information);
    }
    pub fn add_information_stage(
        &mut self,
        line: i32,
        column: i32,
        end_column: i32,
        code: &str,
        details: &str,
        stage: LineStage,
    ) {
        self.add_diagnostic_stage(
            line,
            column,
            end_column,
            code,
            details,
            LinterSeverity::Information,
            stage,
        );
    }
    pub fn add_information_plain(&mut self, line: i32, column: i32, end_column: i32, code: &str) {
        self.add_diagnostic_plain(line, column, end_column, code, LinterSeverity::Information);
    }
}
