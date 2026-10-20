//! Transcribed from Calcpad.Tests/Macros/EmptyMacroArgumentTests.cs by tools/transcribe_xunit.py.
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

#[test]
fn LeadingEmptyArguments_ExpandToEmptyStrings() {
    let mut expanded = Expand("#def m$(a$;b$;c$) = a$|b$|c$\nm$(;;2)\n");
    assert_contains("||2", &expanded);
}
#[test]
fn TrailingEmptyArguments_ExpandToEmptyStrings() {
    let mut expanded = Expand("#def m$(a$;b$;c$) = a$|b$|c$\nm$(1;;)\n");
    assert_contains("1||", &expanded);
}
#[test]
fn MiddleEmptyArgument_ExpandsToEmptyString() {
    let mut expanded = Expand("#def m$(a$;b$;c$) = a$|b$|c$\nm$(1;;3)\n");
    assert_contains("1||3", &expanded);
}
#[test]
fn WhitespaceOnlyArgument_TreatedAsEmpty() {
    let mut expanded = Expand("#def m$(a$;b$;c$) = a$|b$|c$\nm$( ; ;2)\n");
    assert_contains("||2", &expanded);
}
#[test]
fn AllEmptyArguments_ExpandToEmptyStrings() {
    let mut expanded = Expand("#def m$(a$;b$) = [a$b$]\nm$(;)\n");
    assert_contains("[]", &expanded);
}
