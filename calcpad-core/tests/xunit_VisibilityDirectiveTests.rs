//! Transcribed from Calcpad.Tests/ExpressionParser/VisibilityDirectiveTests.cs by tools/transcribe_xunit.py.
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
    noop_method_call
)]
mod common;
use common::*;

const PrePostSource: &str = "#pre\naaa1 = 1\n#end pre\n#post\nbbb2 = 2\n#end post";

// named-options
// private static string Render(string source, bool forPrint = false, bool enableUi = false, bool showHiddenOutput = false)
fn Render(source: &str, named: &[(&str, bool)]) -> String {
    let get = |k: &str| named.iter().find(|(n, _)| *n == k).map(|(_, v)| *v).unwrap_or(false);
    let mut parser = new_expression_parser(&[("ForPrint", get("forPrint")), ("EnableUi", get("enableUi"))]);
    parser.math_settings.borrow_mut().show_hidden_output = get("showHiddenOutput");
    parser.parse(source, true);
    parser.html_result
}

#[test]
fn EndHide_RestoresPriorState() {
    let mut html = Render("aaa1 = 1\n#hide\nbbb2 = 2\n#end hide\nccc3 = 3", &[]);
    assert_contains("aaa1", &html);
    assert_does_not_contain("bbb2", &html);
    assert_contains("ccc3", &html);
}
#[test]
fn EndPre_Nested_RestoresPreHiddenState_NotDefault() {
    let mut html = Render(
        "aaa1 = 1\n#pre\nbbb2 = 2\n#hide\nccc3 = 3\n#end hide\nddd4 = 4\n#end pre\neee5 = 5",
        &[("forPrint", true)],
    );
    assert_contains("aaa1", &html);
    assert_does_not_contain("bbb2", &html);
    assert_does_not_contain("ccc3", &html);
    assert_does_not_contain("ddd4", &html);
    assert_contains("eee5", &html);
}
#[test]
fn EndHide_WithEmptyStack_FallsBackToVisible() {
    let mut html = Render("#hide\naaa1 = 1\n#end hide\n#end hide\nbbb2 = 2", &[]);
    assert_does_not_contain("aaa1", &html);
    assert_contains("bbb2", &html);
}
#[test]
fn HideCondition_True_Hides() {
    let mut html = Render("x = 5\n#hide x ≡ 5\naaa1 = 1\n#end hide\nbbb2 = 2", &[]);
    assert_does_not_contain("aaa1", &html);
    assert_contains("bbb2", &html);
}
#[test]
fn HideCondition_False_LeavesStateUntouched() {
    let mut html = Render(
        "x = 5\n#hide\n#show x ≡ 3\naaa1 = 1\n#end show\n#end hide\nbbb2 = 2",
        &[],
    );
    assert_does_not_contain("aaa1", &html);
    assert_contains("bbb2", &html);
}
#[test]
fn ShowCondition_True_MakesVisible_WhenNestedInsideHide() {
    let mut html = Render(
        "x = 5\n#hide\n#show x ≡ 5\naaa1 = 1\n#end show\n#end hide\nbbb2 = 2",
        &[],
    );
    assert_contains("aaa1", &html);
    assert_contains("bbb2", &html);
}
#[test]
fn Pre_HidesOnPrint_ShowsOnScreen() {
    let mut onScreen = Render(&PrePostSource, &[("forPrint", false)]);
    assert_contains("aaa1", &onScreen);
    assert_contains("bbb2", &onScreen);
    let mut forPrint = Render(&PrePostSource, &[("forPrint", true)]);
    assert_does_not_contain("aaa1", &forPrint);
    assert_contains("bbb2", &forPrint);
}
#[test]
fn Post_HidesInUiMode() {
    let mut uiMode = Render(&PrePostSource, &[("enableUi", true)]);
    assert_contains("aaa1", &uiMode);
    assert_does_not_contain("bbb2", &uiMode);
}
#[test]
fn Hide_InsideFalseIfBranch_DoesNotTakeEffect() {
    let mut html = Render("#if 0\n#hide\n#end if\naaa1 = 1", &[]);
    assert_contains("aaa1", &html);
}
#[test]
fn ShowHiddenOutput_IgnoresHide() {
    let mut html = Render(
        "aaa1 = 1\n#hide\nbbb2 = 2\n#end hide\nccc3 = 3",
        &[("showHiddenOutput", true)],
    );
    assert_contains("aaa1", &html);
    assert_contains("bbb2", &html);
    assert_contains("ccc3", &html);
}
#[test]
fn ShowHiddenOutput_DoesNotOverrideEnclosingPre() {
    let mut html = Render(
        "#pre\naaa1 = 1\n#hide\nbbb2 = 2\n#end hide\n#end pre\nccc3 = 3",
        &[("forPrint", true), ("showHiddenOutput", true)],
    );
    assert_does_not_contain("aaa1", &html);
    assert_does_not_contain("bbb2", &html);
    assert_contains("ccc3", &html);
}
#[test]
fn ShowHiddenOutput_SetByDirective() {
    let mut html = Render(
        "#settings {\"showHiddenOutput\": true}\n#hide\naaa1 = 1\n#end hide\nbbb2 = 2",
        &[],
    );
    assert_contains("aaa1", &html);
    assert_contains("bbb2", &html);
}
#[test]
fn HideCondition_BadExpression_RecordsErrorAndLeavesStateUnchanged() {
    let mut parser = new_expression_parser(&[]);
    parser.parse("#hide 1/0\naaa1 = 1", true);
    assert_contains("err", &parser.html_result);
    assert_contains("aaa1", &parser.html_result);
}
#[test]
fn Hide_ExpressionError_DoesNotLeakToHtml_ButIsRecorded() {
    let mut parser = new_expression_parser(&[("Debug", true)]);
    parser.parse("#hide\nbad\n#end hide\naaa1 = 1", true);
    assert_does_not_contain("err", &parser.html_result);
    assert_contains("aaa1", &parser.html_result);
    assert_eq!(parser.errors_list.len(), 1);
    assert_contains("bad", &parser.errors_list[0].message);
}
