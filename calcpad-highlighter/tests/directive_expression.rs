//! DirectiveExpressionTests.cs
mod common;
use calcpad_highlighter::linter::models::{LinterResult, LinterSeverity};

fn lint(source: &str) -> LinterResult {
    common::lint_source(source, &common::include_files())
}

fn reports_undefined(source: &str) -> bool {
    lint(source)
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3301" && d.message.contains("zzzMissing"))
}

#[test]
fn expression_directive_resolves_its_identifiers() {
    // (source, override source)
    let rows: [(&str, Option<&str>); 17] = [
        ("#if zzzMissing > 0\n#end if", None),
        ("#else if", Some("x = 1\n#if x ≡ 1\n#else if zzzMissing > 0\n#end if")),
        ("#while zzzMissing > 0\n#loop", None),
        ("#repeat zzzMissing\n#loop", None),
        ("#round zzzMissing", None),
        ("#hide zzzMissing > 0\n#end hide", None),
        ("#show zzzMissing > 0\n#end show", None),
        ("#pre zzzMissing > 0\n#end pre", None),
        ("#post zzzMissing > 0\n#end post", None),
        ("#val zzzMissing > 0\n#end val", None),
        ("#equ zzzMissing > 0\n#end equ", None),
        ("#noc zzzMissing > 0\n#end noc", None),
        ("#varsub zzzMissing > 0\n#end varsub", None),
        ("#nosub zzzMissing > 0\n#end nosub", None),
        ("#novar zzzMissing > 0\n#end novar", None),
        ("#for i = 1 : zzzMissing\n#loop", None),
        ("#UI q = zzzMissing", None),
    ];
    let mut failures = Vec::new();
    for (source, over) in rows {
        let s = over.unwrap_or(source);
        if !reports_undefined(s) {
            failures.push(format!("Expected an undefined-variable error for 'zzzMissing' in: {s}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn directive_without_an_expression_reports_nothing() {
    for source in [
        "#round default",
        "#format default",
        "#deg",
        "#hide",
        "#end if",
        "#loop",
        "#md off",
    ] {
        assert!(
            !lint(source).diagnostics.iter().any(|d| d.code == "CPD-3301"),
            "{source}"
        );
    }
}

#[test]
fn condition_on_a_visibility_directive_accepts_defined_variables() {
    let r = lint("x = 5\n#hide x ≡ 5\ny = 1\n#end hide\ny");
    assert!(!r.diagnostics.iter().any(|d| d.severity == LinterSeverity::Error));
}

#[test]
fn loop_counter_is_local_and_not_flagged() {
    let r = lint("#for i = 1 : 3\ni\n#loop");
    assert!(!r.diagnostics.iter().any(|d| d.code == "CPD-3301"));
}
