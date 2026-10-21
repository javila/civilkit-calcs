//! Linter/Models.
mod calcpad_type;
mod definition_metadata;
mod function_signature;
mod lint_ignore_region;
mod linter_diagnostic;
mod linter_result;
mod linter_severity;
mod stage_context;
mod variable_info;

pub use calcpad_type::CalcpadType;
pub use definition_metadata::DefinitionMetadata;
pub use function_signature::FunctionSignature;
pub use lint_ignore_region::LintIgnoreRegion;
pub use linter_diagnostic::{DiagnosticRange, LinterDiagnostic};
pub use linter_result::LinterResult;
pub use linter_severity::LinterSeverity;
pub use stage_context::{DuplicateMacroInfo, MacroDefinition, Stage1Context, Stage2Context, Stage3Context};
pub use variable_info::VariableInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParseMode {
    Cpd,
    Html,
    Markdown,
}
