//! ContentResolverResult.cs: the data classes of the three resolution stages.
use crate::linter::helpers::type_tracker::TypeTracker;
use crate::path_roots::PathRoots;
use crate::text::{StrMap, StrSet};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct MacroDefinition {
    pub name: String,
    pub params: Vec<String>,
    /// Default values parallel to `params`. `None` marks a required parameter.
    pub defaults: Vec<Option<String>>,
    pub content: Vec<String>,
    pub line_number: i32,
    /// "local" | "include"
    pub source: String,
    pub source_file: Option<String>,
    /// Parameters used in comments (directly or transitively through nested macro calls).
    pub comment_parameters: StrSet,
    pub description: Option<String>,
    pub param_types: Option<Vec<String>>,
    pub param_descriptions: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct FunctionDefinition {
    pub name: String,
    pub params: Vec<String>,
    /// Default values parallel to `params`. `None` marks a required parameter.
    pub defaults: Vec<Option<String>>,
    pub line_number: i32,
    pub source: String,
    pub source_file: Option<String>,
    /// Defined with `#const` (readonly).
    pub is_const: bool,
    pub expression: Option<String>,
    pub command_block: Option<CommandBlockInfo>,
    pub description: Option<String>,
    pub param_types: Option<Vec<String>>,
    pub param_descriptions: Option<Vec<String>>,
    pub return_type: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CommandBlockInfo {
    pub function_name: String,
    pub parameters: Vec<String>,
    /// Inline, Block or While.
    pub block_type: String,
    pub statements: Vec<String>,
    pub full_line: String,
    pub line_number: i32,
}

#[derive(Clone, Debug, Default)]
pub struct VariableDefinition {
    pub name: String,
    pub definition: String,
    pub line_number: i32,
    pub source: String,
    pub source_file: Option<String>,
    pub is_const: bool,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CustomUnitDefinition {
    pub name: String,
    pub definition: String,
    pub line_number: i32,
    pub source: String,
    pub source_file: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DuplicateMacro {
    pub name: String,
    pub duplicate_line_number: i32,
    pub original_line_number: i32,
}

#[derive(Clone, Debug, Default)]
pub struct MacroInfo {
    pub line_number: i32,
    pub param_count: i32,
    /// Number of parameters without a default value.
    pub required_param_count: i32,
    pub param_names: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct FunctionInfo {
    pub line_number: i32,
    pub param_count: i32,
    /// Number of parameters without a default value.
    pub required_param_count: i32,
    pub param_names: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MacroExpansionInfo {
    pub macro_names: Vec<String>,
    pub call_site_line: String,
    pub call_site_stage2_line: i32,
    pub content_line_index: i32,
    pub total_content_lines: i32,
}

#[derive(Clone, Debug)]
pub struct SourceInfo {
    /// "local" | "include"
    pub source: String,
    pub source_file: Option<String>,
    /// For included files: the original 0-based line inside the included file, -1 if not applicable.
    pub original_line: i32,
}

impl Default for SourceInfo {
    fn default() -> Self {
        SourceInfo {
            source: String::new(),
            source_file: None,
            original_line: -1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SymbolLocation {
    pub line: i32,
    pub column: i32,
    pub length: i32,
    pub source: String,
    pub source_file: Option<String>,
    pub is_assignment: bool,
}

#[derive(Clone, Debug, Default)]
pub struct LineContinuationSegment {
    pub original_line: i32,
    pub start_column: i32,
    pub length: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Stage1Result {
    pub lines: Vec<String>,
    pub source_map: BTreeMap<i32, i32>,
    pub line_continuation_map: BTreeMap<i32, Vec<i32>>,
    pub line_continuation_segments: BTreeMap<i32, Vec<LineContinuationSegment>>,
}

#[derive(Clone, Debug)]
pub struct Stage2Result {
    pub lines: Vec<String>,
    pub source_map: BTreeMap<i32, i32>,
    pub include_map: BTreeMap<i32, SourceInfo>,
    pub macro_definitions: Vec<MacroDefinition>,
    pub duplicate_macros: Vec<DuplicateMacro>,
    pub path_roots: PathRoots,
    /// Macro name (ignore case) to its comment parameters.
    pub macro_comment_parameters: StrMap<StrSet>,
    /// Macro name (ignore case) to the ordered parameter names.
    pub macro_parameter_order: StrMap<Vec<String>>,
    /// Macro name (ignore case) to its body text.
    pub macro_bodies: StrMap<String>,
    pub user_defined_macros: StrMap<MacroInfo>,
    /// SHA256 hex digest of each included file, keyed by resolved path (ignore case).
    pub included_file_hashes: StrMap<String>,
}

impl Default for Stage2Result {
    fn default() -> Self {
        Stage2Result {
            lines: Vec::new(),
            source_map: BTreeMap::new(),
            include_map: BTreeMap::new(),
            macro_definitions: Vec::new(),
            duplicate_macros: Vec::new(),
            path_roots: PathRoots::default(),
            macro_comment_parameters: StrMap::new_ci(),
            macro_parameter_order: StrMap::new_ci(),
            macro_bodies: StrMap::new_ci(),
            user_defined_macros: StrMap::new_ci(),
            included_file_hashes: StrMap::new_ci(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stage3Result {
    pub lines: Vec<String>,
    pub source_map: BTreeMap<i32, i32>,
    pub include_map: BTreeMap<i32, SourceInfo>,
    pub macro_expansions: BTreeMap<i32, MacroExpansionInfo>,
    pub user_defined_functions: StrMap<FunctionInfo>,
    pub functions_with_params: Vec<FunctionDefinition>,
    pub user_defined_macros: StrMap<MacroInfo>,
    pub defined_variables: StrSet,
    pub variables_with_definitions: Vec<VariableDefinition>,
    pub custom_units: Vec<CustomUnitDefinition>,
    pub type_tracker: TypeTracker,
    /// Functions that use command blocks, keyed by function name (ordinal).
    pub command_block_functions: StrMap<CommandBlockInfo>,
    pub variable_reassignments: Vec<(String, i32, i32)>,
    pub outer_scope_assignments: Vec<(String, i32, i32)>,
    /// (Name, Line, Column, Length)
    pub variable_assignments: Vec<(String, i32, i32, i32)>,
    pub variable_usages: Vec<(String, i32, i32)>,
    pub variable_index: StrMap<Vec<SymbolLocation>>,
    pub function_index: StrMap<Vec<SymbolLocation>>,
    /// Ignore-case keyed.
    pub macro_index: StrMap<Vec<SymbolLocation>>,
    pub macro_comment_parameters: StrMap<StrSet>,
    pub macro_parameter_order: StrMap<Vec<String>>,
}

impl Default for Stage3Result {
    fn default() -> Self {
        Stage3Result {
            lines: Vec::new(),
            source_map: BTreeMap::new(),
            include_map: BTreeMap::new(),
            macro_expansions: BTreeMap::new(),
            user_defined_functions: StrMap::new(),
            functions_with_params: Vec::new(),
            user_defined_macros: StrMap::new_ci(),
            defined_variables: StrSet::new(),
            variables_with_definitions: Vec::new(),
            custom_units: Vec::new(),
            type_tracker: TypeTracker::default(),
            command_block_functions: StrMap::new(),
            variable_reassignments: Vec::new(),
            outer_scope_assignments: Vec::new(),
            variable_assignments: Vec::new(),
            variable_usages: Vec::new(),
            variable_index: StrMap::new(),
            function_index: StrMap::new(),
            macro_index: StrMap::new_ci(),
            macro_comment_parameters: StrMap::new_ci(),
            macro_parameter_order: StrMap::new_ci(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StagedResolvedContent {
    pub stage1: Stage1Result,
    pub stage2: Stage2Result,
    pub stage3: Stage3Result,
}

/// `ResolvedContent` (legacy single-pass result).
#[derive(Clone, Debug, Default)]
pub struct ResolvedContent {
    pub expanded_lines: Vec<String>,
    pub source_map: BTreeMap<i32, i32>,
    pub macro_expansions: BTreeMap<i32, MacroExpansionInfo>,
    pub line_continuation_map: BTreeMap<i32, Vec<i32>>,
    pub user_defined_functions: StrMap<FunctionInfo>,
    pub functions_with_params: Vec<FunctionDefinition>,
    pub user_defined_macros: StrMap<MacroInfo>,
    pub defined_variables: StrSet,
    pub variables_with_definitions: Vec<VariableDefinition>,
    pub custom_units: Vec<CustomUnitDefinition>,
    pub all_macros: Vec<MacroDefinition>,
    pub duplicate_macros: Vec<DuplicateMacro>,
}
