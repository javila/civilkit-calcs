//! Tests for the deliberate deviations listed in DEVIATIONS.md.
use calcpad_core::parsers::expression_parser::{ExpressionParser, UNRESOLVED_CONDITION};

fn render(source: &str) -> String {
    let mut p = ExpressionParser::new();
    p.parse(source, true);
    p.html_result
}

/// D1: CalcpadCE runs the #else branch when the #if condition fails to evaluate
/// (for example a misspelled variable). calcpad-rs runs neither branch and reports it.
#[test]
fn d1_failed_if_condition_runs_neither_branch() {
    let html = render("v = 5\n#if v > v_typo\n'NOT OK\n#else\n'OK\n#end if\n'after");
    assert!(!html.contains(">NOT OK "), "the #if branch must not run: {html}");
    assert!(!html.contains(">OK "), "the #else branch must not run: {html}");
    assert!(
        html.contains(UNRESOLVED_CONDITION),
        "the block must be reported unresolved: {html}"
    );
    assert!(
        html.contains("v_typo"),
        "the error must name the failing expression: {html}"
    );
    assert!(html.contains(">after "), "lines after the block still run: {html}");
}

#[test]
fn d1_failed_else_if_condition_skips_the_rest_of_the_chain() {
    let html = render("v = 5\n#if v < 0\n'NEGATIVE\n#else if v > v_typo\n'BIG\n#else\n'SMALL\n#end if");
    assert!(!html.contains(">NEGATIVE "), "{html}");
    assert!(!html.contains(">BIG "), "{html}");
    assert!(!html.contains(">SMALL "), "{html}");
    assert!(html.contains(UNRESOLVED_CONDITION), "{html}");
}

/// The deviation must not change valid conditions.
#[test]
fn valid_conditions_are_unchanged() {
    let html = render("v = 5\n#if v > 3\n'YES\n#else\n'NO\n#end if");
    assert!(html.contains(">YES "), "{html}");
    assert!(!html.contains(">NO "), "{html}");
    let html = render("v = 1\n#if v > 3\n'YES\n#else\n'NO\n#end if");
    assert!(!html.contains(">YES "), "{html}");
    assert!(html.contains(">NO "), "{html}");
    assert!(!html.contains(UNRESOLVED_CONDITION), "{html}");
}
