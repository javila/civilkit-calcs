//! MacroDiagnosticRangeTests.cs
mod common;
use calcpad_highlighter::linter::models::LinterDiagnostic;

fn lint_undefined(source: &str, name: &str) -> LinterDiagnostic {
    common::lint_source(source, &common::include_files())
        .diagnostics
        .into_iter()
        .find(|d| d.code == "CPD-3301" && d.message.contains(name))
        .expect("expected a CPD-3301 diagnostic")
}

#[test]
fn inline_macro_call_highlights_entire_call_site_line() {
    let lines = [
        "#def area$(w$; h$) = w$*h$*zzzMissing",
        "a = 1",
        "b = 2",
        "result = area$(a; b)",
    ];
    let d = lint_undefined(&lines.join("\n"), "zzzMissing");
    assert_eq!(3, d.line);
    assert_eq!(0, d.column);
    assert_eq!(lines[3].len() as i32, d.end_column);
    assert!(d.additional_ranges.is_none());
}

#[test]
fn multiline_macro_call_highlights_entire_call_site_line() {
    let lines = [
        "#def block$(x$)",
        "\ty = x$ + zzzMissing",
        "#end def",
        "q = 3",
        "block$(q)",
    ];
    let d = lint_undefined(&lines.join("\n"), "zzzMissing");
    assert_eq!(4, d.line);
    assert_eq!(0, d.column);
    assert_eq!(lines[4].len() as i32, d.end_column);
}

#[test]
fn macro_call_across_line_continuation_highlights_every_call_site_line() {
    let lines = [
        "#def sum3$(a$; b$; c$) = a$ + b$ + c$ + zzzMissing",
        "p = 1",
        "r = sum3$(p; _",
        "    p; p)",
    ];
    let d = lint_undefined(&lines.join("\n"), "zzzMissing");
    assert_eq!(2, d.line);
    assert_eq!(0, d.column);
    // The continuation marker " _" is not part of the merged content
    assert_eq!(lines[2].len() as i32 - 2, d.end_column);

    let additional = d.additional_ranges.as_ref().expect("additional ranges");
    assert_eq!(1, additional.len());
    assert_eq!(3, additional[0].line);
    assert_eq!(0, additional[0].column);
    assert_eq!(lines[3].len() as i32, additional[0].end_column);
}

#[test]
fn line_without_macro_expansion_keeps_token_range() {
    let d = lint_undefined("a = 1\nb = a + zzzMissing", "zzzMissing");
    assert_eq!(1, d.line);
    assert_eq!(8, d.column);
    assert_eq!(18, d.end_column);
}
