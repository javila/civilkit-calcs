//! StageContext.cs, Stage1Context.cs, Stage2Context.cs, Stage3Context.cs.
//!
//! The abstract C# `StageContext` base (`Lines`, `SourceMap`) is flattened into each context.
use crate::content_resolution::result::{
    CommandBlockInfo, FunctionInfo, LineContinuationSegment, MacroExpansionInfo, MacroInfo, SourceInfo,
};
use crate::linter::helpers::type_tracker::TypeTracker;
use crate::text::{StrMap, StrSet};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Stage1Context {
    pub lines: Vec<String>,
    /// Stage line -> original line.
    pub source_map: BTreeMap<i32, i32>,
    /// Stage1 line index -> original line indices merged via line continuation.
    pub line_continuation_map: BTreeMap<i32, Vec<i32>>,
    pub line_continuation_segments: BTreeMap<i32, Vec<LineContinuationSegment>>,
}

#[derive(Clone, Debug, Default)]
pub struct Stage2Context {
    pub lines: Vec<String>,
    pub source_map: BTreeMap<i32, i32>,
    pub stage2_to_stage1_map: BTreeMap<i32, i32>,
    pub macro_definitions: Vec<MacroDefinition>,
    pub duplicate_macros: Vec<DuplicateMacroInfo>,
}

#[derive(Clone, Debug, Default)]
pub struct MacroDefinition {
    pub name: String,
    pub parameter_count: i32,
    pub line_number: i32,
}

#[derive(Clone, Debug, Default)]
pub struct DuplicateMacroInfo {
    pub name: String,
    pub original_line_number: i32,
    pub duplicate_line_number: i32,
}

#[derive(Clone, Debug)]
pub struct Stage3Context {
    pub lines: Vec<String>,
    pub source_map: BTreeMap<i32, i32>,
    pub stage3_to_stage2_map: BTreeMap<i32, i32>,
    pub include_map: BTreeMap<i32, SourceInfo>,
    pub defined_variables: StrSet,
    /// Ordinal.
    pub defined_functions: StrMap<FunctionInfo>,
    /// OrdinalIgnoreCase.
    pub defined_macros: StrMap<MacroInfo>,
    pub custom_units: StrSet,
    pub macro_expansions: BTreeMap<i32, MacroExpansionInfo>,
    pub type_tracker: TypeTracker,
    pub command_block_functions: StrMap<CommandBlockInfo>,
    pub variable_reassignments: Vec<(String, i32, i32)>,
    pub outer_scope_assignments: Vec<(String, i32, i32)>,
    /// (Name, Line, Column, Length)
    pub variable_assignments: Vec<(String, i32, i32, i32)>,
    pub variable_usages: Vec<(String, i32, i32)>,
}

impl Default for Stage3Context {
    fn default() -> Self {
        Stage3Context {
            lines: Vec::new(),
            source_map: BTreeMap::new(),
            stage3_to_stage2_map: BTreeMap::new(),
            include_map: BTreeMap::new(),
            defined_variables: StrSet::new(),
            defined_functions: StrMap::new(),
            defined_macros: StrMap::new_ci(),
            custom_units: StrSet::new(),
            macro_expansions: BTreeMap::new(),
            type_tracker: TypeTracker::default(),
            command_block_functions: StrMap::new_ci(),
            variable_reassignments: Vec::new(),
            outer_scope_assignments: Vec::new(),
            variable_assignments: Vec::new(),
            variable_usages: Vec::new(),
        }
    }
}
