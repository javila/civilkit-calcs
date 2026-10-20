//! Transcribed from Calcpad.Tests/ExpressionParser/SubstitutionDirectiveTests.cs by tools/transcribe_xunit.py.
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

const Substituted: &str = "2\u{2006}·\u{2006}5";
const Preamble: &str = "xxx = 5\n";

fn Render(source: &str) -> String {
    render(source)
}

#[test]
fn EndNoSub_RestoresPriorState() {
    assert_does_not_contain(Substituted, &Render(&format!("{}{}", Preamble, "#nosub\naaa1 = 2*xxx")));
    assert_contains(
        Substituted,
        &Render(&format!("{}{}", Preamble, "#nosub\n#end nosub\naaa1 = 2*xxx")),
    );
}
#[test]
fn EndNoVar_Nested_RestoresOuterMode_NotDefault() {
    let mut html = Render(&format!("{}{}", Preamble, "#nosub\n#novar\n#end novar\naaa1 = 2*xxx"));
    assert_contains("aaa1", &html);
    assert_does_not_contain(Substituted, &html);
}
#[test]
fn EndSub_WithEmptyStack_FallsBackToVarSub() {
    let mut html = Render(&format!(
        "{}{}",
        Preamble, "#nosub\n#end nosub\n#end nosub\naaa1 = 2*xxx"
    ));
    assert_contains(Substituted, &html);
}
#[test]
fn NoVarCondition_False_LeavesStateUntouched() {
    let mut html = Render(&format!("{}{}", Preamble, "#nosub\n#novar 1 ≡ 0\naaa1 = 2*xxx"));
    assert_contains("aaa1", &html);
    assert_does_not_contain(Substituted, &html);
}
#[test]
fn NoVarCondition_True_Applies_WhenNestedInsideNoSub() {
    let mut html = Render(&format!("{}{}", Preamble, "#nosub\n#novar 1 ≡ 1\naaa1 = 2*xxx"));
    assert_does_not_contain("aaa1", &html);
    assert_contains(Substituted, &html);
}
#[test]
fn NoSub_InsideFalseIfBranch_DoesNotTakeEffect() {
    let mut html = Render(&format!("{}{}", Preamble, "#if 0\n#nosub\n#end if\naaa1 = 2*xxx"));
    assert_contains(Substituted, &html);
}
#[test]
fn NoSubCondition_BadExpression_RecordsErrorAndLeavesStateUnchanged() {
    let mut parser = new_expression_parser(&[]);
    parser.parse(&format!("{}{}", Preamble, "#nosub 1/0\naaa1 = 2*xxx"), true);
    assert_contains("err", &parser.html_result);
    assert_contains(Substituted, &parser.html_result);
}
