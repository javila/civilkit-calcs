//! PdfMetadataHighlighterTests.cs
//!
//! The `pdf` object of a metadata comment, and the metadata-comment checks that share its code
//! path: recognized keys, region markers, and multi-line payloads.
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};
use calcpad_highlighter::text::StrMap;

fn lint(content: &str) -> LinterResult {
    let empty: StrMap<String> = StrMap::new();
    let staged = ContentResolver::new().get_staged_content(content, Some(&empty), None, None);
    let ignore = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &ignore)
}

fn has(r: &LinterResult, code: &str) -> bool {
    r.diagnostics.iter().any(|d| d.code == code)
}

fn has_msg(r: &LinterResult, code: &str, part: &str) -> bool {
    r.diagnostics.iter().any(|d| d.code == code && d.message.contains(part))
}

#[test]
fn pdf_settings_valid_reports_nothing() {
    let result = lint(
        "'<!--{\"pdf\":{\"format\":\"A4\",\"orientation\":\"landscape\",\"marginTop\":\"2cm\",\"showDate\":false}}-->\nx = 1",
    );
    assert!(!has(&result, "CPD-3414"));
    assert!(!has(&result, "CPD-3412"));
}

#[test]
fn pdf_settings_unknown_key_warns_cpd3414() {
    // 'author' is a real PdfOptions field but is deliberately not offered yet,
    // so a document naming it should be told rather than silently ignored.
    let result = lint("'<!--{\"pdf\":{\"author\":\"Me\"}}-->\nx = 1");
    assert!(has(&result, "CPD-3414"));
}

#[test]
fn pdf_settings_unknown_paper_format_warns_cpd3414() {
    let result = lint("'<!--{\"pdf\":{\"format\":\"A9\"}}-->\nx = 1");
    assert!(has_msg(&result, "CPD-3414", "format"));
}

#[test]
fn pdf_settings_unitless_margin_warns_cpd3414() {
    let result = lint("'<!--{\"pdf\":{\"marginTop\":\"2 centimetres\"}}-->\nx = 1");
    assert!(has_msg(&result, "CPD-3414", "marginTop"));
}

#[test]
fn pdf_settings_bare_number_margin_warns_cpd3414() {
    // Puppeteer reads a unitless margin as pixels, which is never the intent.
    let result = lint("'<!--{\"pdf\":{\"marginLeft\":\"2\"}}-->\nx = 1");
    assert!(has_msg(&result, "CPD-3414", "marginLeft"));
}

#[test]
fn pdf_settings_wrong_value_type_warns_cpd3414() {
    let result = lint("'<!--{\"pdf\":{\"showDate\":\"yes\"}}-->\nx = 1");
    assert!(has(&result, "CPD-3414"));
}

#[test]
fn pdf_settings_not_an_object_warns_cpd3414() {
    let result = lint("'<!--{\"pdf\":5}-->\nx = 1");
    assert!(has(&result, "CPD-3414"));
}

#[test]
fn metadata_comment_malformed_json_warns_cpd3412() {
    let result = lint("'<!--{\"pdf\":{bad}}-->\nx = 1");
    assert!(has(&result, "CPD-3412"));
}

#[test]
fn metadata_comment_unknown_top_level_key_warns_cpd3412() {
    let result = lint("'<!--{\"nonsense\":1}-->\nx = 1");
    assert!(has_msg(&result, "CPD-3412", "nonsense"));
}

#[test]
fn metadata_comment_plain_prose_comment_reports_nothing() {
    // A non-JSON HTML comment is not metadata; flagging it would light up
    // every ordinary commented-out block of markup.
    let result = lint("'<!-- a note to self -->\nx = 1");
    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3412" || d.code == "CPD-3414" || d.code == "CPD-3411"));
}

#[test]
fn lint_ignore_unknown_code_warns_cpd3412() {
    let result = lint("'<!--{\"LintIgnore\":[\"CPD-9999\"]}-->\nx = 1\n'<!--{\"EndLintIgnore\":[]}-->");
    assert!(has_msg(&result, "CPD-3412", "CPD-9999"));
}

#[test]
fn lint_ignore_known_code_reports_nothing() {
    let result = lint("'<!--{\"LintIgnore\":[\"CPD-3301\"]}-->\nx = 1\n'<!--{\"EndLintIgnore\":[]}-->");
    assert!(!has(&result, "CPD-3412"));
}

#[test]
fn lint_ignore_not_an_array_warns_cpd3412() {
    let result = lint("'<!--{\"LintIgnore\":\"CPD-3301\"}-->\nx = 1");
    assert!(has_msg(&result, "CPD-3412", "LintIgnore"));
}

#[test]
fn ui_overrides_saved_values_report_nothing() {
    let result = lint("'<!--{\"uiOverrides\":{\"L:1\":\"8\",\"v:1\":\"[4; 5; 6]\",\"y:1:2\":\"5\"}}-->\n#UI L = 4");
    assert!(!has(&result, "CPD-3412"));
}

#[test]
fn ui_overrides_not_an_object_warns_cpd3412() {
    let result = lint("'<!--{\"uiOverrides\":[\"L:1\"]}-->\nx = 1");
    assert!(has_msg(&result, "CPD-3412", "uiOverrides"));
}

#[test]
fn ui_overrides_non_string_value_warns_cpd3412() {
    // The host writes every entered value as a string; a bare number here would
    // come back as a value the form can't round-trip.
    let result = lint("'<!--{\"uiOverrides\":{\"L:1\":8}}-->\nx = 1");
    assert!(has_msg(&result, "CPD-3412", "L:1"));
}

#[test]
fn ui_overrides_not_on_first_line_warns_cpd3416() {
    // The host only ever reads a 'uiOverrides' comment off the file's first line,
    // so one lower down - e.g. above a #UI line it means to target - is inert.
    let result = lint("#UI L = 4\n'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI q = 1");
    assert!(has(&result, "CPD-3416"));
}

#[test]
fn ui_overrides_on_first_line_does_not_warn_cpd3416() {
    let result = lint("'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4");
    assert!(!has(&result, "CPD-3416"));
}

#[test]
fn ui_overrides_duplicate_warns_cpd3417_for_the_second_one() {
    // Only the first 'uiOverrides' comment in a file is ever read back, so a second
    // one - e.g. from a merge or a stray copy-paste - would otherwise fail silently.
    let result = lint(
        "'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4\n'<!--{\"uiOverrides\":{\"q:1\":\"2\"}}-->\n#UI q = 1",
    );
    let duplicate_warnings: Vec<_> = result.diagnostics.iter().filter(|d| d.code == "CPD-3417").collect();
    assert_eq!(duplicate_warnings.len(), 1);
    assert_eq!(duplicate_warnings[0].line, 2);
}

#[test]
fn ui_overrides_sharing_comment_with_another_key_warns_cpd3418() {
    // The Properties tab and the host's save/restore both rewrite this comment by
    // position, so mixing 'uiOverrides' with another key makes the result unpredictable.
    let result = lint("'<!--{\"desc\":\"Beam span\",\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4");
    assert!(has(&result, "CPD-3418"));
}

#[test]
fn ui_overrides_alone_does_not_warn_cpd3418() {
    let result = lint("'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4");
    assert!(!has(&result, "CPD-3418"));
}

#[test]
fn ui_overrides_single_does_not_warn_cpd3417() {
    let result = lint("'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI L = 4");
    assert!(!has(&result, "CPD-3417"));
}

#[test]
fn ui_overrides_from_include_reports_nothing() {
    // CalcpadService strips 'uiOverrides' out of included content unconditionally, so a comment
    // only reached through #include is dead weight regardless of where it sits.
    let dir = std::env::temp_dir().join("uioverrides-include");
    let main_path = dir.join("main.cpd").to_string_lossy().into_owned();
    let sub_path = dir.join("sub.cpd").to_string_lossy().into_owned();
    let content = "#UI L = 4\n#include sub.cpd\n";
    let mut include_files: StrMap<String> = StrMap::new();
    include_files.insert(
        &sub_path,
        "'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n#UI q = 1".to_string(),
    );

    let staged = ContentResolver::new().get_staged_content(content, Some(&include_files), None, Some(&main_path));
    let ignore = LintIgnoreRegionParser::extract_regions(content);
    let result = CalcpadLinter::new().lint(&staged, &ignore);

    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3416" || d.code == "CPD-3417" || d.code == "CPD-3418" || d.code == "CPD-3412"));
}

#[test]
fn pdf_settings_spanning_lines_is_validated_not_skipped() {
    // A `_`-continued comment used to be invisible to this validator, which read
    // one token at a time; the payload below has to be judged as a whole.
    let result = lint("'<!--{\"pdf\":{\"format\":\"A9\"}, _\n\"documentTitle\":\"Beam\"}-->\nx = 1");
    assert!(has(&result, "CPD-3414"));
}

#[test]
fn pdf_settings_spanning_lines_valid_payload_reports_nothing() {
    // The companion to the case above: the halves must be rejoined into valid
    // JSON, not reported as malformed for ending mid-object.
    let result = lint("'<!--{\"pdf\":{\"format\":\"A4\"}, _\n\"desc\":\"Beam check\"}-->\nx = 1");
    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3412" || d.code == "CPD-3414"));
}

#[test]
fn metadata_comment_above_definition_keeps_desc_and_pdf_together() {
    let result =
        lint("'<!--{\"desc\":\"Area\",\"returnType\":\"value\",\"pdf\":{\"format\":\"A4\"}}-->\nA(b; h) = b*h");
    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3411" || d.code == "CPD-3412" || d.code == "CPD-3414"));
}

#[test]
fn metadata_comment_arrow_in_param_desc_reports_nothing() {
    // "1->W" is ordinary prose, not a comment terminator; only "-->" closes.
    let result = lint(
        "'<!--{\"desc\":\"Base plate\",\"paramTypes\":[\"value\"], _\n\
\"paramDesc\":[\"Shape: 1->W Section, 2->Rect HSS, 3->Round HSS\"]}-->\nA(b) = b",
    );
    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3411" || d.code == "CPD-3412"));
}
