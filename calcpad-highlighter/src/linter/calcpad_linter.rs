//! CalcpadLinter.cs
use crate::content_resolution::result::{Stage1Result, Stage2Result, Stage3Result, StagedResolvedContent};
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{
    DuplicateMacroInfo, LintIgnoreRegion, LinterResult, MacroDefinition, Stage1Context, Stage2Context, Stage3Context,
};
use crate::linter::validators::stage1::IncludeValidator;
use crate::linter::validators::stage2::MacroValidator;
use crate::linter::validators::stage3::{
    BalanceValidator, CommandBlockValidator, FormatValidator, FunctionTypeValidator, HtmlCommentValidator,
    LegacyInputValidator, NamingValidator, SemanticValidator, SettingsValidator, UiValidator, UsageValidator,
};
use crate::text::StrSet;
use std::sync::Arc;

#[derive(Default)]
pub struct CalcpadLinter {
    include_validator: IncludeValidator,
    macro_validator: MacroValidator,
    balance_validator: BalanceValidator,
    naming_validator: NamingValidator,
    usage_validator: UsageValidator,
    semantic_validator: SemanticValidator,
    function_type_validator: FunctionTypeValidator,
    command_block_validator: CommandBlockValidator,
    format_validator: FormatValidator,
    html_comment_validator: HtmlCommentValidator,
    settings_validator: SettingsValidator,
    ui_validator: UiValidator,
    legacy_input_validator: LegacyInputValidator,
}

impl CalcpadLinter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lints pre-processed staged content from the ContentResolver. `ignore_regions` are
    /// source-level regions (0-based original lines) in which codes are suppressed; they are
    /// applied after all diagnostics are mapped back. (C# `Lint` returns an empty result for a
    /// null `staged`; a Rust reference cannot be null.) `LintAsync` is not ported: it only wraps
    /// `Lint` in `Task.Run`.
    pub fn lint(&self, staged: &StagedResolvedContent, ignore_regions: &[LintIgnoreRegion]) -> LinterResult {
        let mut result = LinterResult::new();

        let stage1 = Arc::new(convert_to_stage1_context(&staged.stage1));
        let stage2 = Arc::new(convert_to_stage2_context(&staged.stage2, &staged.stage1));
        let stage3 = Arc::new(convert_to_stage3_context(&staged.stage3, &staged.stage2));

        result.set_stage_contexts(stage1.clone(), stage2.clone(), stage3.clone());

        let mut token_provider = TokenizedLineProvider::new();
        token_provider.set_macro_comment_parameters(
            Some(&staged.stage2.macro_comment_parameters),
            Some(&staged.stage2.macro_parameter_order),
            Some(&staged.stage2.macro_bodies),
        );
        token_provider.tokenize_lines(&stage3.lines);

        self.include_validator.validate(&stage1, &mut result);
        self.macro_validator.validate(&stage2, &mut result);
        self.validate_stage3(&stage3, &mut result, &token_provider);

        result.map_diagnostics_to_original();

        if !ignore_regions.is_empty() {
            apply_ignore_regions(&mut result, ignore_regions);
        }
        result
    }

    fn validate_stage3(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        self.balance_validator.validate(stage3, result, tp);
        self.naming_validator.validate(stage3, result, tp);
        self.usage_validator.validate(stage3, result, tp);
        self.semantic_validator.validate(stage3, result, tp);
        self.function_type_validator.validate(stage3, result, tp);
        self.command_block_validator.validate(stage3, result, tp);
        self.format_validator.validate(stage3, result, tp);
        self.html_comment_validator.validate(stage3, result, tp);
        self.settings_validator.validate(stage3, result, tp);
        self.ui_validator.validate(stage3, result, tp);
        self.legacy_input_validator.validate(stage3, result, tp);
    }
}

fn apply_ignore_regions(result: &mut LinterResult, regions: &[LintIgnoreRegion]) {
    result.remove_diagnostics(|d| {
        regions.iter().any(|r| {
            d.line >= r.start_line && d.line <= r.end_line && (r.codes.is_empty() || r.codes.contains(&d.code))
        })
    });
}

fn convert_to_stage1_context(s: &Stage1Result) -> Stage1Context {
    Stage1Context {
        lines: s.lines.clone(),
        source_map: s.source_map.clone(),
        line_continuation_map: s.line_continuation_map.clone(),
        line_continuation_segments: s.line_continuation_segments.clone(),
    }
}

fn convert_to_stage2_context(s2: &Stage2Result, s1: &Stage1Result) -> Stage2Context {
    let mut c = Stage2Context {
        lines: s2.lines.clone(),
        ..Default::default()
    };
    for (k, v) in &s2.source_map {
        c.source_map.insert(*k, s1.source_map.get(v).copied().unwrap_or(*v));
        c.stage2_to_stage1_map.insert(*k, *v);
    }
    for m in &s2.macro_definitions {
        c.macro_definitions.push(MacroDefinition {
            name: m.name.clone(),
            parameter_count: m.params.len() as i32,
            line_number: m.line_number,
        });
    }
    for d in &s2.duplicate_macros {
        c.duplicate_macros.push(DuplicateMacroInfo {
            name: d.name.clone(),
            original_line_number: d.original_line_number,
            duplicate_line_number: d.duplicate_line_number,
        });
    }
    c
}

fn convert_to_stage3_context(s3: &Stage3Result, s2: &Stage2Result) -> Stage3Context {
    let mut c = Stage3Context {
        lines: s3.lines.clone(),
        ..Default::default()
    };
    for (k, stage2_line) in &s3.source_map {
        let stage1_line = s2.source_map.get(stage2_line).copied().unwrap_or(*stage2_line);
        c.source_map.insert(*k, stage1_line);
        c.stage3_to_stage2_map.insert(*k, *stage2_line);
    }
    c.include_map = s3.include_map.clone();
    c.defined_variables = s3.defined_variables.clone();
    c.defined_functions = s3.user_defined_functions.clone();
    for (k, v) in s3.user_defined_macros.iter() {
        c.defined_macros.insert(k, v.clone());
    }
    c.macro_expansions = s3.macro_expansions.clone();
    let mut units = StrSet::new();
    for u in &s3.custom_units {
        units.insert(&u.name);
    }
    c.custom_units = units;
    c.type_tracker = s3.type_tracker.clone();
    c.command_block_functions = s3.command_block_functions.clone();
    c.variable_reassignments = s3.variable_reassignments.clone();
    c.outer_scope_assignments = s3.outer_scope_assignments.clone();
    c.variable_assignments = s3.variable_assignments.clone();
    c.variable_usages = s3.variable_usages.clone();
    c
}
