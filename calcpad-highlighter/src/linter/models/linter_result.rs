//! LinterResult.cs
use super::{DiagnosticRange, LinterDiagnostic, LinterSeverity, Stage1Context, Stage2Context, Stage3Context};
use crate::linter::helpers::diagnostic_extensions::LineStage;
use crate::linter::helpers::source_mapper;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct LinterResult {
    pub diagnostics: Vec<LinterDiagnostic>,
    error_count: i32,
    warning_count: i32,
    stage1_context: Option<Arc<Stage1Context>>,
    stage2_context: Option<Arc<Stage2Context>>,
    stage3_context: Option<Arc<Stage3Context>>,
}

impl LinterResult {
    pub fn new() -> Self {
        Self::default()
    }
    /// The `LintResponse` JSON of the server: `errorCount`, `warningCount`, `diagnostics`.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "errorCount": self.error_count,
            "warningCount": self.warning_count,
            "diagnostics": self.diagnostics.iter().map(|d| d.to_json()).collect::<Vec<_>>(),
        })
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }
    pub fn error_count(&self) -> i32 {
        self.error_count
    }
    pub fn warning_count(&self) -> i32 {
        self.warning_count
    }

    pub fn append_diagnostic(&mut self, d: LinterDiagnostic) {
        match d.severity {
            LinterSeverity::Error => self.error_count += 1,
            LinterSeverity::Warning => self.warning_count += 1,
            _ => {}
        }
        self.diagnostics.push(d);
    }

    pub fn remove_diagnostics(&mut self, mut matches: impl FnMut(&LinterDiagnostic) -> bool) {
        let mut errors = self.error_count;
        let mut warnings = self.warning_count;
        self.diagnostics.retain(|d| {
            if !matches(d) {
                return true;
            }
            match d.severity {
                LinterSeverity::Error => errors -= 1,
                LinterSeverity::Warning => warnings -= 1,
                _ => {}
            }
            false
        });
        self.error_count = errors;
        self.warning_count = warnings;
    }

    pub fn set_stage_contexts(&mut self, s1: Arc<Stage1Context>, s2: Arc<Stage2Context>, s3: Arc<Stage3Context>) {
        self.stage1_context = Some(s1);
        self.stage2_context = Some(s2);
        self.stage3_context = Some(s3);
    }

    pub fn stage1_context(&self) -> Option<&Stage1Context> {
        self.stage1_context.as_deref()
    }
    pub fn stage2_context(&self) -> Option<&Stage2Context> {
        self.stage2_context.as_deref()
    }
    pub fn stage3_context(&self) -> Option<&Stage3Context> {
        self.stage3_context.as_deref()
    }

    pub fn has_line_continuation_mapping(&self) -> bool {
        self.stage1_context
            .as_ref()
            .is_some_and(|s| !s.line_continuation_segments.is_empty())
    }

    /// Maps all diagnostics from their stage line numbers to original line numbers.
    pub fn map_diagnostics_to_original(&mut self) {
        let Some(s1) = self.stage1_context.clone() else {
            return;
        };
        let s2 = self.stage2_context.clone();
        let s3 = self.stage3_context.clone();
        for d in &mut self.diagnostics {
            match d.stage {
                LineStage::Stage1 => d.line = source_mapper::map_stage1_to_original(d.stage_line, &s1),
                LineStage::Stage2 => {
                    if let Some(s2) = &s2 {
                        d.line = source_mapper::map_stage2_to_original(d.stage_line, s2, &s1);
                    }
                }
                LineStage::Stage3 => {
                    if let (Some(s2), Some(s3)) = (&s2, &s3) {
                        let m =
                            source_mapper::map_position_to_original(d.stage_line, d.column, d.end_column, s3, s2, &s1);
                        d.line = m.line;
                        d.column = m.column;
                        d.end_column = m.end_column;
                        if let Some(r) = m.additional_ranges.filter(|r| !r.is_empty()) {
                            d.additional_ranges = Some(
                                r.into_iter()
                                    .map(|(line, column, end_column)| DiagnosticRange {
                                        line,
                                        column,
                                        end_column,
                                    })
                                    .collect(),
                            );
                        }
                    }
                }
            }
        }
    }
}
