//! LegacyInputHighlighterTests.cs
mod common;
use calcpad_highlighter::linter::models::{LinterDiagnostic, LinterSeverity};

fn diagnostics(source: &str, code: &str) -> Vec<LinterDiagnostic> {
    common::lint_source(source, &common::include_files())
        .diagnostics
        .into_iter()
        .filter(|d| d.code == code)
        .collect()
}

#[test]
fn stored_input_value_is_warned() {
    for source in ["L = ?{6}m", "L = ? {6}m", "L = ?{}m", "f(x) = x\nA = f(?{2})"] {
        let d = diagnostics(source, "CPD-3419");
        assert_eq!(1, d.len(), "{source}");
        assert_eq!(LinterSeverity::Warning, d[0].severity, "{source}");
    }
}

#[test]
fn stored_input_value_highlights_the_field_only() {
    let d = diagnostics("L = ?{6}m", "CPD-3419");
    assert_eq!(1, d.len());
    assert_eq!(0, d[0].line);
    assert_eq!(4, d[0].column);
    assert_eq!(8, d[0].end_column);
}

#[test]
fn stored_input_value_after_a_comment_keeps_its_columns() {
    let d = diagnostics("'note' L = ?{6}m", "CPD-3419");
    assert_eq!(1, d.len());
    assert_eq!(11, d[0].column);
    assert_eq!(15, d[0].end_column);
}

#[test]
fn every_stored_input_value_on_a_line_is_warned() {
    assert_eq!(2, diagnostics("f(x; y) = x + y\nA = f(?{2}; ?{3})", "CPD-3419").len());
}

#[test]
fn plain_input_and_comments_are_not_warned() {
    for source in ["L = ?m", "'What is ?{6}?'", "L = 6m 'sets ?{6}'"] {
        assert!(diagnostics(source, "CPD-3419").is_empty(), "{source}");
    }
}

#[test]
fn include_input_values_are_warned() {
    let d = diagnostics("#include import.cpd #{2; 3}", "CPD-1103");
    assert_eq!(1, d.len());
    assert_eq!(LinterSeverity::Warning, d[0].severity);
    assert_eq!(20, d[0].column);
    assert_eq!(27, d[0].end_column);
}

#[test]
fn include_without_input_values_is_not_warned() {
    assert!(diagnostics("#include import.cpd", "CPD-1103").is_empty());
}

#[test]
fn stored_input_values_inside_an_included_file_are_warned_on_the_include_line() {
    let d = diagnostics("d = 1\n#include legacy_input.cpd", "CPD-3419");
    assert_eq!(1, d.len());
    assert_eq!(1, d[0].line);
}
