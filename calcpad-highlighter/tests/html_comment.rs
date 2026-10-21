//! HTMLCommentTests.cs
mod common;
use calcpad_highlighter::html_comment::{HtmlCommentParseStatus, HtmlCommentParser};
use calcpad_highlighter::linter::models::LinterSeverity;
use calcpad_highlighter::linter::LintIgnoreRegionParser;
use calcpad_highlighter::tokenizer::models::TokenType;
use calcpad_highlighter::tokenizer::CalcpadTokenizer;

const TEST_FILE: &str = "HTML Comment Data.cpd";

#[test]
fn lint_ignore_regions_suppress_only_codes_not_ended_selectively() {
    let full_path = common::base_dir().join("valid").join(TEST_FILE);
    let result = common::lint_file(&full_path);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == LinterSeverity::Error)
        .collect();
    // Line 6 is inside a LintIgnore for CPD-3404 and CPD-3312; the file should be error-free.
    assert!(
        errors.is_empty(),
        "Expected no error diagnostics, got: {}",
        errors
            .iter()
            .map(|e| format!("[{}] line {}: {}", e.code, e.line + 1, e.message))
            .collect::<Vec<_>>()
            .join("; ")
    );
}

#[test]
#[ignore = "needs the Calcpad.Core MacroParser/ExpressionParser engine (rendered HTML), which is outside the highlighter port"]
fn settings_directive_applies_four_decimals_to_output() {}

#[test]
fn line_continuation_metadata_comment_parses_as_single_json_block() {
    let source = "'<!--{\"desc\":\"k\",\"paramTypes\":[\"value\"], _\n\"paramDesc\":[\"ndfdf\"]}-->";
    let tokens = CalcpadTokenizer::new().tokenize(source);
    let blocks = HtmlCommentParser::new().parse_result(&tokens);
    assert_eq!(1, blocks.len());
    let block = &blocks[0];
    assert_eq!(HtmlCommentParseStatus::Success, block.status);
    let data = block.data.as_ref().expect("data");
    assert_eq!("k", data["desc"].as_str().unwrap());
    assert_eq!("value", data["paramTypes"][0].as_str().unwrap());
    assert_eq!("ndfdf", data["paramDesc"][0].as_str().unwrap());
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
    assert_eq!(1, html.len());
    assert!(html[0].text.starts_with("<!--"));
    assert!(html[0].text.ends_with("-->"));

    let blocks = HtmlCommentParser::new().parse_result(&tokens);
    assert_eq!(1, blocks.len());
    assert_eq!(HtmlCommentParseStatus::Success, blocks[0].status);
    assert_eq!(
        "Shape of Column: 1->W Section, 2->Rect HSS",
        blocks[0].data.as_ref().unwrap()["paramDesc"][0].as_str().unwrap()
    );
}

#[test]
fn tag_in_json_string_does_not_end_the_html_comment() {
    let source = "'<!--{\"desc\":\"use <b>bold</b> here\"}-->";
    let tokens = CalcpadTokenizer::new().tokenize(source);
    let blocks = HtmlCommentParser::new().parse_result(&tokens);
    assert_eq!(1, blocks.len());
    assert_eq!(HtmlCommentParseStatus::Success, blocks[0].status);
    assert_eq!(
        "use <b>bold</b> here",
        blocks[0].data.as_ref().unwrap()["desc"].as_str().unwrap()
    );
}

#[test]
fn non_object_json_in_html_comment_does_not_throw() {
    let content = "'<!--[]-->'\nx = 1\n";
    let regions = LintIgnoreRegionParser::extract_regions(content);
    assert!(regions.is_empty());
}
