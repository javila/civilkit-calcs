//! Transcribed from Calcpad.Tests/Macros/MacroKeywordArgumentTests.cs by tools/transcribe_xunit.py.
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

// private static string Expand(string source)
fn Expand(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (has_errors, expanded) = macro_parser.Parse(source);
    assert!(!has_errors, "{expanded}");
    expanded
}
// private static string ExpandWithErrors(string source)
fn ExpandWithErrors(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (_, expanded) = macro_parser.Parse(source);
    expanded
}

#[test]
fn OmittedOptionalArguments_UseDefaults() {
    let mut expanded = Expand("#def m$(a$; b$ = 10; c$ = 5) = a$|b$|c$\nm$(1)\n");
    assert_contains("1|10|5", &expanded);
}
#[test]
fn BracketedDefault_WithSemicolons() {
    let mut expanded = Expand("#def m$(a$; b$ = [1; 2]; c$ = {3; 4}) = a$|b$|c$\nm$(1)\n");
    assert_contains("1|[1; 2]|{3; 4}", &expanded);
}
#[test]
fn PositionalArgumentsOverrideDefaults() {
    let mut expanded = Expand("#def m$(a$; b$ = 10; c$ = 5) = a$|b$|c$\nm$(1; 2; 3)\n");
    assert_contains("1|2|3", &expanded);
}
#[test]
fn KeywordArguments_CanBeGivenInAnyOrder() {
    let mut expanded = Expand("#def m$(a$; b$ = 10; c$ = 5) = a$|b$|c$\nm$(a$ = 1; c$ = 7; b$ = 2)\n");
    assert_contains("1|2|7", &expanded);
}
#[test]
fn PositionalFollowedByKeyword_IsAllowed() {
    let mut expanded = Expand("#def m$(a$; b$ = 10; c$ = 5) = a$|b$|c$\nm$(1; c$ = 7)\n");
    assert_contains("1|10|7", &expanded);
}
#[test]
fn KeywordArgumentsWithoutSpaces_AreRecognized() {
    let mut expanded = Expand("#def m$(a$; b$ = 10) = a$|b$\nm$(a$=1;b$=2)\n");
    assert_contains("1|2", &expanded);
}
#[test]
fn MultilineMacro_SupportsDefaults() {
    let mut expanded = Expand("#def m$(a$; b$ = 4)\n\ta$ + b$\n#end def\nm$(3)\n");
    assert_contains("3 + 4", &expanded);
}
#[test]
fn DefaultContainingBrackets_IsPreserved() {
    let mut expanded = Expand("#def m$(a$ = f(1; 2)) = [a$]\nm$()\n");
    assert_contains("[f(1; 2)]", &expanded);
}
#[test]
fn MultilineMacro_DefaultDoesNotEndTheDefinition() {
    let mut expanded = Expand("#def m$(v$; u$ = mm)\n\tv$|u$\n#end def\nm$(2.5m)\n");
    assert_contains("2.5m|mm", &expanded);
}
#[test]
fn DefaultContainingAVector_IsPreserved() {
    let mut expanded = Expand("#def m$(a$ = [1; 2; 3]) = len(a$)\nm$()\n");
    assert_contains("len([1; 2; 3])", &expanded);
}
#[test]
fn EmptyArgumentList_UsesDefaultsWhenMacroHasOptionalParameters() {
    let mut expanded = Expand("#def m$(a$ = 7; b$ = 8) = a$|b$\nm$()\n");
    assert_contains("7|8", &expanded);
}
#[test]
fn EmptyArgument_StillMeansEmptyStringForRequiredParameters() {
    let mut expanded = Expand("#def m$(a$) = [a$]\nm$()\n");
    assert_contains("[]", &expanded);
}
#[test]
fn AllRequiredParameters_StillWork() {
    let mut expanded = Expand("#def m$(a$; b$) = a$+b$\nm$(1; 2)\n");
    assert_contains("1+2", &expanded);
}
#[test]
fn ParameterWithoutDollarSuffix_StillParses() {
    let mut expanded = Expand("#def m$(a) = [a]\nm$(1)\n");
    assert_contains("[1]", &expanded);
}
#[test]
fn ArgumentNamingAnExistingMacro_StaysPositional() {
    let mut expanded = Expand("#def y$ = q\n#def emit$(lhs$) = [lhs$]\nemit$(y$ = 5)\n");
    assert_contains("[q = 5]", &expanded);
}
#[test]
fn UnknownKeywordArgument_ReportsError() {
    let mut expanded = ExpandWithErrors("#def m$(a$; b$ = 1) = a$|b$\nm$(1; z$ = 2)\n");
    assert_contains("Unknown keyword argument", &expanded);
}
#[test]
fn MissingRequiredArgument_ReportsError() {
    let mut expanded = ExpandWithErrors("#def m$(a$; b$; c$ = 1) = a$|b$|c$\nm$(1)\n");
    assert_contains("Invalid number of arguments", &expanded);
}
#[test]
fn DuplicateArgument_ReportsError() {
    let mut expanded = ExpandWithErrors("#def m$(a$; b$ = 1) = a$|b$\nm$(1; a$ = 2)\n");
    assert_contains("Duplicate argument", &expanded);
}
#[test]
fn RequiredParameterAfterOptional_ReportsError() {
    let mut expanded = ExpandWithErrors("#def m$(a$ = 1; b$) = a$|b$\nm$(1; 2)\n");
    assert_contains("cannot follow an optional parameter", &expanded);
}
#[test]
fn PositionalAfterKeyword_ReportsError() {
    let mut expanded = ExpandWithErrors("#def m$(a$; b$ = 1) = a$|b$\nm$(a$ = 1; 2)\n");
    assert_contains("Invalid number of arguments", &expanded);
}
