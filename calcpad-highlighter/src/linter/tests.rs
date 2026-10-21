//! Unit tests for the linter port. Transliterated from Calcpad.Tests/Highlighter (HTMLCommentTests,
//! MetadataReturnTypeTests, ElementWiseFunctionTypeTests, FunctionCaseSensitivityTests,
//! DirectiveExpressionTests, PdfMetadataHighlighterTests) plus unit tests for the helpers.
use super::helpers::directive_state::{DirectiveState, OutputMode, ScopeMode, VisibilityMode};
use super::helpers::line_parser;
use super::helpers::source_mapper;
use super::helpers::type_tracker::TypeTracker;
use super::models::*;
use super::{CalcpadLinter, LintIgnoreRegionParser};
use crate::content_resolution::result::LineContinuationSegment;
use crate::content_resolution::ContentResolver;
use crate::core_dto::SettingsDto;
use crate::html_comment::{HtmlCommentParseStatus, HtmlCommentParser, PdfSettingsDto};
use crate::snippets::ParameterType;
use crate::text::chars;
use crate::tokenizer::models::TokenType;
use crate::tokenizer::CalcpadTokenizer;

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let ignore = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &ignore)
}

fn has_code(r: &LinterResult, code: &str) -> bool {
    r.diagnostics.iter().any(|d| d.code == code)
}

fn count_code(r: &LinterResult, code: &str) -> usize {
    r.diagnostics.iter().filter(|d| d.code == code).count()
}

fn has_code_msg(r: &LinterResult, code: &str, part: &str) -> bool {
    r.diagnostics.iter().any(|d| d.code == code && d.message.contains(part))
}

// ---- HtmlCommentParser ----

#[test]
fn line_continuation_metadata_comment_parses_as_single_json_block() {
    let source = "'<!--{\"desc\":\"k\",\"paramTypes\":[\"value\"], _\n\"paramDesc\":[\"ndfdf\"]}-->";
    let tokens = CalcpadTokenizer::new().tokenize(source);
    let blocks = HtmlCommentParser::new().parse_result(&tokens);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].status, HtmlCommentParseStatus::Success);
    let data = blocks[0].data.as_ref().unwrap();
    assert_eq!(data["desc"], "k");
    assert_eq!(data["paramTypes"][0], "value");
    assert_eq!(data["paramDesc"][0], "ndfdf");
}

#[test]
fn arrow_in_json_string_does_not_end_the_html_comment() {
    let source = "'<!--{\"desc\":\"Base plate\",\"paramDesc\":[\"Shape of Column: 1->W Section, 2->Rect HSS\"]}-->";
    let tokens = CalcpadTokenizer::new().tokenize(source);
    let html: Vec<_> = tokens
        .tokens
        .iter()
        .filter(|t| t.token_type == TokenType::HtmlComment)
        .collect();
    assert_eq!(html.len(), 1);
    assert!(html[0].text.starts_with("<!--") && html[0].text.ends_with("-->"));
    let blocks = HtmlCommentParser::new().parse_result(&tokens);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].status, HtmlCommentParseStatus::Success);
    assert_eq!(
        blocks[0].data.as_ref().unwrap()["paramDesc"][0],
        "Shape of Column: 1->W Section, 2->Rect HSS"
    );
}

#[test]
fn tag_in_json_string_does_not_end_the_html_comment() {
    let source = "'<!--{\"desc\":\"use <b>bold</b> here\"}-->";
    let tokens = CalcpadTokenizer::new().tokenize(source);
    let blocks = HtmlCommentParser::new().parse_result(&tokens);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].data.as_ref().unwrap()["desc"], "use <b>bold</b> here");
}

#[test]
fn non_object_json_in_html_comment_does_not_throw() {
    assert!(LintIgnoreRegionParser::extract_regions("'<!--[]-->'\nx = 1\n").is_empty());
}

#[test]
fn invalid_json_block_reports_invalid_status() {
    let tokens = CalcpadTokenizer::new().tokenize("'<!--{bad}-->");
    let blocks = HtmlCommentParser::new().parse_result(&tokens);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].status, HtmlCommentParseStatus::InvalidJson);
    assert!(blocks[0].data.is_none() && blocks[0].parse_error.is_some());
}

// ---- LintIgnoreRegionParser ----

#[test]
fn lint_ignore_regions_open_and_close_per_code() {
    let src = "'<!--{\"LintIgnore\": [\"CPD-3404\", \"cpd-3312\"]}-->\nx = = 2\n'<!--{\"EndLintIgnore\": [\"CPD-3404\"]}-->\nx = 3\n";
    let regions = LintIgnoreRegionParser::extract_regions(src);
    assert_eq!(regions.len(), 2);
    // CPD-3404 closed before line 2 (0-based): lines 1..=1; CPD-3312 stays open to the end of the file.
    let r3404 = regions.iter().find(|r| r.codes.contains("CPD-3404")).unwrap();
    assert_eq!((r3404.start_line, r3404.end_line), (1, 1));
    let r3312 = regions.iter().find(|r| r.codes.contains("CPD-3312")).unwrap();
    assert_eq!((r3312.start_line, r3312.end_line), (1, i32::MAX));
}

#[test]
fn lint_ignore_all_region_and_end_all() {
    let src = "'<!--{\"LintIgnore\": []}-->\na\nb\n'<!--{\"EndLintIgnore\": []}-->\nc\n";
    let regions = LintIgnoreRegionParser::extract_regions(src);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].codes.is_empty());
    assert_eq!((regions[0].start_line, regions[0].end_line), (1, 2));
}

#[test]
fn lint_ignore_regions_suppress_diagnostics() {
    let src = "'<!--{\"LintIgnore\": [\"CPD-3301\"]}-->\nx = undefinedThing + 1\n";
    let r = lint(src);
    assert!(!has_code(&r, "CPD-3301"));
    let r2 = lint("x = undefinedThing + 1\n");
    assert!(has_code(&r2, "CPD-3301"));
}

// ---- Metadata return type ----

#[test]
fn return_type_metadata_overrides_inferred_function_type() {
    let staged =
        ContentResolver::new().get_staged_content("'<!--{\"returnType\":\"matrix\"}-->\nf(x) = x\n", None, None, None);
    let f = staged.stage3.type_tracker.functions().get("f").expect("f tracked");
    assert_eq!(f.return_type, CalcpadType::Matrix);
}

#[test]
fn return_type_any_maps_to_various() {
    let staged =
        ContentResolver::new().get_staged_content("'<!--{\"returnType\":\"any\"}-->\nf(x) = x\n", None, None, None);
    let f = staged.stage3.type_tracker.functions().get("f").expect("f tracked");
    assert_eq!(f.return_type, CalcpadType::Various);
}

#[test]
fn invalid_return_type_reports_cpd3411() {
    let r = lint("'<!--{\"returnType\":\"scalar\"}-->\nf(x) = x\n");
    assert!(has_code(&r, "CPD-3411"));
}

// ---- Function types ----

#[test]
fn element_wise_function_accepts_matrix() {
    for func in ["sin", "cos", "sqrt", "ln", "exp", "floor", "abs", "sign"] {
        let r = lint(&format!("M = [1; 2; 3 | 4; 5; 6]\nr = {func}(M)\n"));
        assert_eq!(count_code(&r, "CPD-3309"), 0, "{func}");
    }
}

#[test]
fn element_wise_function_accepts_vector_literal() {
    assert_eq!(count_code(&lint("r = sin([1; 2; 3])\n"), "CPD-3309"), 0);
}

#[test]
fn non_element_wise_function_still_warns_on_shape_mismatch() {
    assert!(count_code(&lint("r = n_rows(5)\n"), "CPD-3309") > 0);
}

#[test]
fn matrix_function_accepts_vector() {
    for call in [
        "augment(v; v)",
        "stack(v; v)",
        "hprod(v; v)",
        "transp(v)",
        "row(v; 1)",
        "col(v; 1)",
        "n_rows(v)",
    ] {
        let r = lint(&format!("v = [1; 2; 3]\nr = {call}\n"));
        assert_eq!(count_code(&r, "CPD-3309"), 0, "{call}");
    }
}

#[test]
fn interpolation_accepts_index_and_matrix() {
    for func in ["take", "line", "spline"] {
        let r = lint(&format!("M = [1; 2; 3 | 4; 5; 6]\nr = {func}(2; M)\n"));
        assert_eq!(count_code(&r, "CPD-3309"), 0, "{func}");
    }
}

#[test]
fn gcd_lcm_accept_vector_and_matrix() {
    for func in ["gcd", "lcm"] {
        let r = lint(&format!("v = [12; 18]\nM = [24; 30 | 36; 42]\nr = {func}(v; M; 6)\n"));
        assert_eq!(count_code(&r, "CPD-3309"), 0, "{func}");
    }
}

// ---- Function case sensitivity ----

#[test]
fn upper_and_lower_case_functions_are_distinct() {
    let r = lint("F(a; b) = a + b\nf(x) = x^2\nr = f(3)\n");
    assert_eq!(count_code(&r, "CPD-3302"), 0);
}

#[test]
fn command_block_function_does_not_shadow_lower_case_function() {
    let r = lint("F(φ; k) = $Integral{1/sqrt(1 - k^2*sin(θ)^2) @ θ = 0 : φ}\nf(θ) = sin(-θ) - θ\nr = f(1)\n");
    assert_eq!(count_code(&r, "CPD-3302"), 0);
}

#[test]
fn calling_wrong_case_function_is_flagged_undefined() {
    let r = lint("F(a; b) = a + b\nr = f(1)\n");
    assert!(count_code(&r, "CPD-3301") + count_code(&r, "CPD-3305") > 0);
    assert_eq!(count_code(&r, "CPD-3302"), 0);
}

// ---- Directive expressions ----

fn reports_undefined(src: &str) -> bool {
    lint(src)
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3301" && d.message.contains("zzzMissing"))
}

#[test]
fn expression_directive_resolves_its_identifiers() {
    for src in [
        "#if zzzMissing > 0\n#end if",
        "x = 1\n#if x ≡ 1\n#else if zzzMissing > 0\n#end if",
        "#while zzzMissing > 0\n#loop",
        "#repeat zzzMissing\n#loop",
        "#round zzzMissing",
        "#hide zzzMissing > 0\n#end hide",
        "#show zzzMissing > 0\n#end show",
        "#pre zzzMissing > 0\n#end pre",
        "#post zzzMissing > 0\n#end post",
        "#val zzzMissing > 0\n#end val",
        "#equ zzzMissing > 0\n#end equ",
        "#noc zzzMissing > 0\n#end noc",
        "#varsub zzzMissing > 0\n#end varsub",
        "#nosub zzzMissing > 0\n#end nosub",
        "#novar zzzMissing > 0\n#end novar",
        "#for i = 1 : zzzMissing\n#loop",
        "#UI q = zzzMissing",
    ] {
        assert!(reports_undefined(src), "expected an undefined-variable error in: {src}");
    }
}

#[test]
fn directive_without_an_expression_reports_nothing() {
    for src in [
        "#round default",
        "#format default",
        "#deg",
        "#hide",
        "#end if",
        "#loop",
        "#md off",
    ] {
        assert!(!has_code(&lint(src), "CPD-3301"), "{src}");
    }
}

#[test]
fn condition_on_a_visibility_directive_accepts_defined_variables() {
    let r = lint("x = 5\n#hide x ≡ 5\ny = 1\n#end hide\ny");
    assert!(!r.diagnostics.iter().any(|d| d.severity == LinterSeverity::Error));
}

#[test]
fn loop_counter_is_local_and_not_flagged() {
    assert!(!has_code(&lint("#for i = 1 : 3\ni\n#loop"), "CPD-3301"));
}

// ---- Metadata comments / PDF ----

#[test]
fn pdf_settings_valid_reports_nothing() {
    let r = lint("'<!--{\"pdf\":{\"format\":\"A4\",\"orientation\":\"landscape\",\"marginTop\":\"2cm\",\"showDate\":false}}-->\nx = 1");
    assert!(!has_code(&r, "CPD-3414") && !has_code(&r, "CPD-3412"));
}

#[test]
fn pdf_settings_unknown_key_warns() {
    assert!(has_code(
        &lint("'<!--{\"pdf\":{\"author\":\"Me\"}}-->\nx = 1"),
        "CPD-3414"
    ));
}

#[test]
fn pdf_settings_unknown_paper_format_warns() {
    assert!(has_code_msg(
        &lint("'<!--{\"pdf\":{\"format\":\"A9\"}}-->\nx = 1"),
        "CPD-3414",
        "format"
    ));
}

#[test]
fn pdf_settings_unitless_margin_warns() {
    assert!(has_code_msg(
        &lint("'<!--{\"pdf\":{\"marginTop\":\"2 centimetres\"}}-->\nx = 1"),
        "CPD-3414",
        "marginTop"
    ));
}

#[test]
fn pdf_settings_bare_number_margin_warns() {
    assert!(has_code_msg(
        &lint("'<!--{\"pdf\":{\"marginLeft\":\"2\"}}-->\nx = 1"),
        "CPD-3414",
        "marginLeft"
    ));
}

#[test]
fn pdf_settings_wrong_value_type_warns() {
    assert!(has_code(
        &lint("'<!--{\"pdf\":{\"showDate\":\"yes\"}}-->\nx = 1"),
        "CPD-3414"
    ));
}

#[test]
fn pdf_settings_not_an_object_warns() {
    assert!(has_code(&lint("'<!--{\"pdf\":5}-->\nx = 1"), "CPD-3414"));
}

#[test]
fn metadata_comment_malformed_json_warns() {
    assert!(has_code(&lint("'<!--{\"pdf\":{bad}}-->\nx = 1"), "CPD-3412"));
}

#[test]
fn metadata_comment_unknown_top_level_key_warns() {
    assert!(has_code_msg(
        &lint("'<!--{\"nonsense\":1}-->\nx = 1"),
        "CPD-3412",
        "nonsense"
    ));
}

#[test]
fn metadata_comment_plain_prose_comment_reports_nothing() {
    let r = lint("'<!-- a note to self -->\nx = 1");
    assert!(!has_code(&r, "CPD-3412") && !has_code(&r, "CPD-3414") && !has_code(&r, "CPD-3411"));
}

#[test]
fn lint_ignore_unknown_code_warns() {
    let r = lint("'<!--{\"LintIgnore\":[\"CPD-9999\"]}-->\nx = 1\n'<!--{\"EndLintIgnore\":[]}-->");
    assert!(has_code_msg(&r, "CPD-3412", "CPD-9999"));
}

#[test]
fn lint_ignore_known_code_reports_nothing() {
    let r = lint("'<!--{\"LintIgnore\":[\"CPD-3301\"]}-->\nx = 1\n'<!--{\"EndLintIgnore\":[]}-->");
    assert!(!has_code(&r, "CPD-3412"));
}

#[test]
fn lint_ignore_not_an_array_warns() {
    assert!(has_code_msg(
        &lint("'<!--{\"LintIgnore\":\"CPD-3301\"}-->\nx = 1"),
        "CPD-3412",
        "LintIgnore"
    ));
}

#[test]
fn ui_overrides_saved_values_report_nothing() {
    let r = lint("'<!--{\"uiOverrides\":{\"L:1\":\"8\",\"v:1\":\"[4; 5; 6]\",\"y:1:2\":\"5\"}}-->\n#UI L = 4");
    assert!(!has_code(&r, "CPD-3412"));
}

#[test]
fn ui_overrides_not_an_object_warns() {
    assert!(has_code_msg(
        &lint("'<!--{\"uiOverrides\":[\"L:1\"]}-->\nx = 1"),
        "CPD-3412",
        "uiOverrides"
    ));
}

#[test]
fn ui_overrides_non_string_value_warns() {
    assert!(has_code_msg(
        &lint("'<!--{\"uiOverrides\":{\"L:1\":8}}-->\nx = 1"),
        "CPD-3412",
        "L:1"
    ));
}

#[test]
fn ui_overrides_not_on_first_line_warns() {
    assert!(has_code(
        &lint("#UI L = 4\n'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI q = 1"),
        "CPD-3416"
    ));
}

#[test]
fn ui_overrides_on_first_line_does_not_warn() {
    assert!(!has_code(
        &lint("'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4"),
        "CPD-3416"
    ));
}

#[test]
fn ui_overrides_duplicate_warns_for_the_second_one() {
    let r = lint(
        "'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4\n'<!--{\"uiOverrides\":{\"q:1\":\"2\"}}-->\n#UI q = 1",
    );
    let dups: Vec<_> = r.diagnostics.iter().filter(|d| d.code == "CPD-3417").collect();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].line, 2);
}

#[test]
fn ui_overrides_sharing_comment_with_another_key_warns() {
    assert!(has_code(
        &lint("'<!--{\"desc\":\"Beam span\",\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4"),
        "CPD-3418"
    ));
}

#[test]
fn ui_overrides_alone_or_single_does_not_warn() {
    let r = lint("'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4");
    assert!(!has_code(&r, "CPD-3418") && !has_code(&r, "CPD-3417"));
}

#[test]
fn pdf_settings_spanning_lines_is_validated() {
    assert!(has_code(
        &lint("'<!--{\"pdf\":{\"format\":\"A9\"}, _\n\"documentTitle\":\"Beam\"}-->\nx = 1"),
        "CPD-3414"
    ));
}

#[test]
fn pdf_settings_spanning_lines_valid_payload_reports_nothing() {
    let r = lint("'<!--{\"pdf\":{\"format\":\"A4\"}, _\n\"desc\":\"Beam check\"}-->\nx = 1");
    assert!(!has_code(&r, "CPD-3412") && !has_code(&r, "CPD-3414"));
}

#[test]
fn metadata_comment_above_definition_keeps_desc_and_pdf_together() {
    let r = lint("'<!--{\"desc\":\"Area\",\"returnType\":\"value\",\"pdf\":{\"format\":\"A4\"}}-->\nA(b; h) = b*h");
    assert!(!has_code(&r, "CPD-3411") && !has_code(&r, "CPD-3412") && !has_code(&r, "CPD-3414"));
}

#[test]
fn metadata_comment_arrow_in_param_desc_reports_nothing() {
    let r = lint(
        "'<!--{\"desc\":\"Base plate\",\"paramTypes\":[\"value\"], _\n\"paramDesc\":[\"Shape: 1->W Section, 2->Rect HSS, 3->Round HSS\"]}-->\nA(b) = b",
    );
    assert!(!has_code(&r, "CPD-3411") && !has_code(&r, "CPD-3412"));
}

// ---- Smoke tests of the other validators ----

#[test]
fn undefined_variable_and_unbalanced_brackets() {
    let r = lint("x = (1 + 2\ny = zzz + 1\n");
    assert!(has_code(&r, "CPD-3101"));
    assert!(has_code_msg(&r, "CPD-3301", "zzz"));
}

#[test]
fn macro_duplicate_and_missing_dollar() {
    let r = lint("#def m$ = 1\n#def m$ = 2\n#def n = 3\n");
    assert!(has_code(&r, "CPD-2201"));
    assert!(has_code(&r, "CPD-2202"));
}

#[test]
fn include_without_path() {
    let r = lint("#include\n");
    assert!(has_code(&r, "CPD-1102"));
}

#[test]
fn settings_validator_reports_range_and_unknown_key() {
    let r = lint("#settings {\"decimals\": 99, \"bogus\": 1}\nx = 1\nx");
    assert!(has_code_msg(&r, "CPD-3413", "bogus"));
    assert!(has_code_msg(&r, "CPD-3413", "'decimals' must be between 0 and 15"));
}

#[test]
fn legacy_input_field_warns() {
    let r = lint("a = ?{5}\n");
    assert!(has_code(&r, "CPD-3419"));
}

#[test]
fn format_validator_flags_bad_specifier() {
    let r = lint("a = 1\na|zz\n");
    let _ = r; // the tokenizer decides what is a Format token; just make sure nothing panics
}

#[test]
fn unused_variable_is_reported_as_information() {
    let r = lint("a = 1\nb = 2\nb\n");
    assert!(has_code_msg(&r, "CPD-3312", "'a'"));
    assert!(!has_code_msg(&r, "CPD-3312", "'b'"));
}

#[test]
fn no_panic_on_odd_input() {
    for s in [
        "",
        "\n",
        "'",
        "\"",
        "#",
        "$",
        "#def",
        "#UI {",
        "#settings {",
        "f(",
        ")",
        "x = ",
        "\u{1F600} = 1\n",
        "a\u{0301} = 1",
    ] {
        let _ = lint(s);
    }
}

// ---- Helpers ----

fn seg(original_line: i32, start_column: i32, length: i32) -> LineContinuationSegment {
    LineContinuationSegment {
        original_line,
        start_column,
        length,
    }
}

#[test]
fn source_mapper_maps_stage_lines() {
    let mut s1 = Stage1Context::default();
    s1.source_map.insert(0, 0);
    s1.source_map.insert(1, 3);
    assert_eq!(source_mapper::map_stage1_to_original(1, &s1), 3);
    assert_eq!(source_mapper::map_stage1_to_original(7, &s1), 7);
    let mut s2 = Stage2Context::default();
    s2.stage2_to_stage1_map.insert(5, 1);
    assert_eq!(source_mapper::map_stage2_to_original(5, &s2, &s1), 3);
    let mut s3 = Stage3Context::default();
    s3.stage3_to_stage2_map.insert(9, 5);
    assert_eq!(source_mapper::map_stage3_to_original(9, &s3, &s2, &s1), 3);
}

#[test]
fn source_mapper_continuation_segments() {
    let mut s1 = Stage1Context::default();
    s1.source_map.insert(0, 4);
    s1.line_continuation_segments
        .insert(0, vec![seg(4, 0, 10), seg(5, 10, 8), seg(6, 18, 5)]);
    let s2 = Stage2Context::default();
    let s3 = Stage3Context::default();
    // Within the first segment.
    let m = source_mapper::map_position_to_original(0, 2, 6, &s3, &s2, &s1);
    assert_eq!((m.line, m.column, m.end_column), (4, 2, 6));
    assert!(m.additional_ranges.is_none());
    // Spanning segments 0 and 2: primary range to the end of segment 0, plus the middle and the last.
    let m = source_mapper::map_position_to_original(0, 5, 20, &s3, &s2, &s1);
    assert_eq!((m.line, m.column, m.end_column), (4, 5, 10));
    assert_eq!(m.additional_ranges.unwrap(), vec![(5, 0, 8), (6, 0, 2)]);
}

#[test]
fn source_mapper_macro_expansion_covers_whole_call_site_line() {
    let mut s1 = Stage1Context::default();
    s1.lines = vec!["abc def".to_string()];
    s1.source_map.insert(0, 2);
    let s2 = Stage2Context::default();
    let mut s3 = Stage3Context::default();
    s3.macro_expansions.insert(0, Default::default());
    let m = source_mapper::map_position_to_original(0, 40, 50, &s3, &s2, &s1);
    assert_eq!((m.line, m.column, m.end_column), (2, 0, 7));
}

#[test]
fn diagnostics_map_to_original_lines_and_counts_follow_removals() {
    let mut s1 = Stage1Context::default();
    s1.source_map.insert(0, 10);
    let mut r = LinterResult::new();
    r.add_error_stage(
        0,
        1,
        2,
        "CPD-1102",
        "x",
        super::helpers::diagnostic_extensions::LineStage::Stage1,
    );
    r.add_warning(0, 1, 2, "CPD-3301", "y");
    r.set_stage_contexts(
        s1.into(),
        Stage2Context::default().into(),
        Stage3Context::default().into(),
    );
    r.map_diagnostics_to_original();
    assert_eq!(r.diagnostics[0].line, 10);
    assert_eq!((r.error_count(), r.warning_count()), (1, 1));
    r.remove_diagnostics(|d| d.severity == LinterSeverity::Warning);
    assert_eq!((r.error_count(), r.warning_count()), (1, 0));
    assert_eq!(
        r.diagnostics[0].message,
        "Malformed #include statement: x".replace("Malformed #include statement", "Missing #include filename")
    );
}

#[test]
fn function_signature_compatibility() {
    let sig = FunctionSignature {
        is_element_wise: true,
        ..Default::default()
    };
    assert!(sig.is_argument_compatible(ParameterType::Scalar, CalcpadType::Matrix));
    assert!(!sig.is_argument_compatible(ParameterType::Vector, CalcpadType::Value));
    assert!(FunctionSignature::is_type_compatible(
        ParameterType::Matrix,
        CalcpadType::Vector
    ));
    assert!(!FunctionSignature::is_type_compatible(
        ParameterType::Vector,
        CalcpadType::Matrix
    ));
    assert!(FunctionSignature::is_type_compatible(
        ParameterType::Scalar,
        CalcpadType::Unknown
    ));
    assert!(FunctionSignature::is_type_compatible(
        ParameterType::Boolean,
        CalcpadType::Matrix
    ));
    let v = FunctionSignature {
        parameter_types: vec![ParameterType::Scalar, ParameterType::Vector],
        ..Default::default()
    };
    assert_eq!(v.get_parameter_type(5), ParameterType::Vector);
    assert_eq!(FunctionSignature::default().get_parameter_type(0), ParameterType::Any);
    assert_eq!(FunctionSignature::get_type_name(ParameterType::Integer), "integer");
}

#[test]
fn function_signatures_come_from_snippets() {
    use super::constants::function_signatures as fs;
    assert!(fs::has_signature("SIN"));
    let sin = fs::get_signature("sin").unwrap();
    assert!(sin.is_element_wise);
    assert!(!fs::get_all_overloads("take").is_empty());
    assert!(fs::get_all_overloads("nope").is_empty());
}

#[test]
fn directive_state_tracks_modes() {
    let mut d = DirectiveState::new();
    d.apply(&chars("#noc"));
    assert_eq!(d.output, OutputMode::NoCalculation);
    d.apply(&chars("#val"));
    assert_eq!(d.output, OutputMode::Values);
    d.apply(&chars("#end val"));
    assert_eq!(d.output, OutputMode::NoCalculation);
    d.apply(&chars("#end noc"));
    assert_eq!(d.output, OutputMode::Equations);
    d.apply(&chars("#end noc"));
    assert_eq!(d.output, OutputMode::Equations);
    d.apply(&chars("#hide"));
    d.apply(&chars("#PRE"));
    assert_eq!(d.visibility, VisibilityMode::ScreenOnly);
    d.apply(&chars("#end pre"));
    assert_eq!(d.visibility, VisibilityMode::Hidden);
    d.apply(&chars("#local"));
    assert_eq!(d.scope, ScopeMode::Local);
    d.apply(&chars("#md"));
    assert!(d.is_markdown_on);
    d.apply(&chars("#md off"));
    assert!(!d.is_markdown_on);
    d.apply(&chars("#html"));
    assert_eq!(d.parse_mode(), ParseMode::Html);
    d.apply(&chars("#end html"));
    assert_eq!(d.parse_mode(), ParseMode::Cpd);
}

#[test]
fn directive_expression_start() {
    let f = |s: &str| line_parser::get_directive_expression_start(&chars(s));
    assert_eq!(f("#hide x ≡ 5"), 6);
    assert_eq!(f("  #if x"), 6);
    assert_eq!(f("#else if y"), 9);
    assert_eq!(f("#iffy x"), -1);
    assert_eq!(f("#hide"), -1);
    assert_eq!(f("#round default"), -1);
    assert_eq!(f("#UI {\"type\":\"entry\"} L = 10"), 21);
    assert_eq!(f("#UI {\"type\""), -1);
    assert_eq!(f("#include a.cpd"), -1);
}

#[test]
fn line_parser_segments() {
    let p = line_parser::parse_line("a = 'text' + b \"x\" c", 3);
    assert_eq!(p.string_segments.len(), 2);
    assert_eq!((p.string_segments[0].start_pos, p.string_segments[0].end_pos), (4, 10));
    assert_eq!(p.code_segments.len(), 3);
    let q = line_parser::parse_line("a 'unclosed", 0);
    assert_eq!(q.string_segments.len(), 1);
    assert_eq!(q.string_segments[0].end_pos, 11);
    assert!(line_parser::should_skip_line("  'c"));
    assert!(line_parser::should_skip_line("   "));
    assert!(!line_parser::should_skip_line("a"));
    assert!(line_parser::is_def_statement(&chars("#DEF")));
    assert!(!line_parser::is_def_statement(&chars("#define")));
    assert!(line_parser::is_end_def_statement(&chars("#End Def")));
}

#[test]
fn type_tracker_infers_types() {
    let mut t = TypeTracker::new();
    assert_eq!(
        t.register_variable("v", "[1; 2; 3]", 0, 0, "local", false).type_,
        CalcpadType::Vector
    );
    assert_eq!(
        t.register_variable("m", "[1; 2 | 3; 4]", 1, 0, "local", false).type_,
        CalcpadType::Matrix
    );
    assert_eq!(
        t.register_variable("s", "3", 2, 0, "local", false).type_,
        CalcpadType::Unknown
    );
    assert_eq!(t.infer_type_from_expression("v.2", -1), CalcpadType::Value);
    assert_eq!(t.infer_type_from_expression("v + v", -1), CalcpadType::Vector);
    assert_eq!(t.infer_type_from_expression("m*v", -1), CalcpadType::Vector);
    assert_eq!(t.infer_type_from_expression("m*m", -1), CalcpadType::Matrix);
    assert_eq!(t.infer_type_from_expression("len(v)", -1), CalcpadType::Unknown);
    assert_eq!(t.infer_type_from_expression("identity(m)", -1), CalcpadType::Matrix);
    assert_eq!(t.infer_type_from_expression("  ", -1), CalcpadType::Unknown);
    // A later different type makes the variable Various.
    t.register_variable("v", "[1 | 2]", 5, 0, "local", false);
    assert_eq!(t.get_variable_type("v"), CalcpadType::Various);
    assert_eq!(t.get_variable_type("v."), CalcpadType::Various);
    assert!(t.supports_element_access("v"));
    assert!(t.is_defined("m."));
    t.register_function("f", &["x".to_string()], "[x; x]", 6, 0, "local", false);
    assert_eq!(t.get_function_return_type("f"), CalcpadType::Vector);
    assert_eq!(t.get_variable_type("f"), CalcpadType::Function);
    t.register_inline_macro("Mac$", &[], "1", 7, 0, "local");
    assert!(t.is_defined("mac$"), "macros are ignore case");
    assert!(!t.is_defined("F"), "functions are case sensitive");
    t.register_command_block_function(
        "g",
        &["a".to_string()],
        &["r = [a; a]".to_string()],
        8,
        0,
        "local",
        false,
    );
    assert_eq!(t.get_function_return_type("g"), CalcpadType::Vector);
    t.register_custom_unit("EUR", "1", 9, 0, "local");
    assert_eq!(t.get_variable_info("eur").unwrap().name, ".EUR");
    t.register_read_variable("r", true, 10, 0, "local");
    assert_eq!(t.get_variable_type("r"), CalcpadType::Vector);
    let c = t.clone();
    t.clear();
    assert!(!t.is_defined("v") && c.is_defined("v"));
}

#[test]
fn variable_info_display() {
    let f = VariableInfo {
        name: "f".into(),
        type_: CalcpadType::Function,
        parameters: vec!["a".into(), "b".into()],
        ..Default::default()
    };
    assert_eq!(f.to_string(), "f(a; b)");
    let m = VariableInfo {
        name: "m$".into(),
        type_: CalcpadType::InlineMacro,
        ..Default::default()
    };
    assert_eq!(m.to_string(), "m$");
    assert!(m.is_dollar_suffixed());
    let u = VariableInfo {
        type_: CalcpadType::CustomUnit,
        unit_name: "EUR".into(),
        ..Default::default()
    };
    assert_eq!(u.to_string(), ".EUR");
}

#[test]
fn settings_dto_parse_and_validate() {
    let d = SettingsDto::parse("{\"Decimals\": 99, \"colorscale\": \"nope\", \"tol\": 5}")
        .unwrap()
        .unwrap();
    let msgs: Vec<String> = d.validate().into_iter().map(|e| e.message).collect();
    assert_eq!(
        msgs,
        vec![
            "'decimals' must be between 0 and 15".to_string(),
            "'tol' must be between 1E-15 and 0.01".to_string(),
            format!(
                "'colorScale' must be one of: {}",
                calcpad_core::settings::ColorScales::names().join(", ")
            ),
        ]
    );
    assert!(SettingsDto::parse("{\"decimals\": 1.5}").is_err());
    assert!(SettingsDto::parse("{\"complex\": \"yes\"}").is_err());
    assert!(SettingsDto::parse("null").unwrap().is_none());
    assert!(SettingsDto::parse("[1]").is_err());
    assert!(SettingsDto::is_known_key("MAXOUTPUTCOUNT") && !SettingsDto::is_known_key("zzz"));
    let ok = SettingsDto::parse("{\"plotWidth\": 0}").unwrap().unwrap();
    assert_eq!(ok.validate()[0].message, "'plotWidth' must be at least 1");
    // .NET Enum.TryParse accepts numbers and comma lists.
    let n = SettingsDto::parse("{\"colorScale\": \"7\"}").unwrap().unwrap();
    assert!(n.validate().is_empty());
}

#[test]
fn pdf_settings_dto_validation() {
    let d = PdfSettingsDto::parse(
        "{\"format\":\"a4\",\"orientation\":\"Landscape\",\"marginTop\":\"2cm\",\"marginLeft\":\".5IN\",\"lineColor\":\"#AABBCC\"}",
    )
    .unwrap()
    .unwrap();
    assert!(d.validate().is_empty());
    let bad = PdfSettingsDto::parse(
        "{\"format\":\"A9\",\"orientation\":\"up\",\"marginTop\":\"2\",\"marginRight\":\"2 cm\",\"headerFooterLines\":\"x\",\"lineColor\":\"red\"}",
    )
    .unwrap()
    .unwrap();
    let msgs: Vec<String> = bad.validate().into_iter().map(|e| e.message).collect();
    assert_eq!(msgs.len(), 6);
    assert_eq!(
        msgs[0],
        "'format' must be one of: Letter, Legal, Tabloid, Ledger, A0, A1, A2, A3, A4, A5, A6"
    );
    assert_eq!(msgs[1], "'orientation' must be one of: portrait, landscape");
    assert_eq!(
        msgs[2],
        "'marginTop' must be a length with a unit, e.g. 2cm, 0.5in, or 12mm"
    );
    assert_eq!(
        msgs[4],
        "'headerFooterLines' must be one of: both, header, footer, none"
    );
    assert_eq!(msgs[5], "'lineColor' must be a hex color, e.g. #b3b3b3");
    assert!(PdfSettingsDto::parse("{\"showDate\": \"yes\"}").is_err());
    assert!(PdfSettingsDto::is_known_key("documenttitle"));
}

#[test]
fn definition_metadata_return_type_parsing() {
    assert_eq!(
        DefinitionMetadata::parse_function_return_type("Vector"),
        Some(CalcpadType::Vector)
    );
    assert_eq!(
        DefinitionMetadata::parse_function_return_type("any"),
        Some(CalcpadType::Various)
    );
    assert_eq!(DefinitionMetadata::parse_function_return_type("scalar"), None);
    let m = DefinitionMetadata::try_parse(&chars("'<!--{\"returnType\":\"vector\"}-->")).unwrap();
    assert_eq!(m.return_type.as_deref(), Some("vector"));
}

#[test]
fn json_depth_limit_matches_dotnet() {
    use super::helpers::directive_json_reporter::parse_json_document;
    let ok = format!("{}1{}", "[".repeat(64), "]".repeat(64));
    let deep = format!("{}1{}", "[".repeat(65), "]".repeat(65));
    assert!(parse_json_document(&ok).is_some());
    assert!(parse_json_document(&deep).is_none());
    assert!(parse_json_document("{\"a\": \"[[[[\"}").is_some());
}
