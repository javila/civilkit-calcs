//! Transcribed from Calcpad.Tests/TrailingNewlineRenderTests.cs by tools/transcribe_xunit.py.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::identity_op,
    clippy::needless_borrow,
    clippy::redundant_clone,
    clippy::bool_assert_comparison,
    clippy::useless_format,
    clippy::clone_on_copy,
    unused_imports,
    dead_code,
    suspicious_double_ref_op,
    noop_method_call,
    clippy::needless_borrows_for_generic_args
)]
mod common;
use common::*;

fn Render(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (has_macro_errors, expanded_source) = macro_parser.Parse(source);
    assert!(!has_macro_errors);
    let mut parser = new_expression_parser(&[]);
    parser.parse(&expanded_source, true);
    parser.html_result
}
// private static int CountOccurrences(string text, string value)
fn CountOccurrences(text: impl AsRef<str>, value: &str) -> f64 {
    text.as_ref().matches(value).count() as f64
}

#[test]
fn TrailingLineBreak_DoesNotEmitBlankParagraph() {
    // [MemberData(nameof(SingleTrailingLineBreakCases))]
    for source in ["5 = 5\n", "5 = 5\r\n", "5 = 5\r"] {
        let html = Render(source);
        assert_equal(0.0, CountOccurrences(&html, "<p>&nbsp;</p>"));
        assert_equal(0.0, CountOccurrences(&html, "<p></p>"));
    }
}
#[test]
fn MidFileBlankLine_EmitsExactlyOneBlankParagraph() {
    let mut html = Render("5 = 5\n\n7 = 7\n");
    assert_equal(1.0, CountOccurrences(&html, "<p>&nbsp;</p>"));
    assert_equal(0.0, CountOccurrences(&html, "<p></p>"));
}
#[test]
fn AllLineEndings_ProduceEqualHtml() {
    let mut lf = Render("'note\n5 = 5\n");
    let mut crlf = Render("'note\r\n5 = 5\r\n");
    let mut cr = Render("'note\r5 = 5\r");
    assert_equal(lf.clone(), crlf.clone());
    assert_equal(lf.clone(), cr.clone());
}
#[test]
fn MultipleTrailingLineBreaks_AreSymmetric() {
    let mut lf = Render("5 = 5\n\n\n");
    let mut crlf = Render("5 = 5\r\n\r\n\r\n");
    let mut cr = Render("5 = 5\r\r\r");
    assert_equal(lf.clone(), crlf.clone());
    assert_equal(lf.clone(), cr.clone());
    assert_equal(2.0, CountOccurrences(&lf, "<p>&nbsp;</p>"));
    assert_equal(0.0, CountOccurrences(&lf, "<p></p>"));
}
