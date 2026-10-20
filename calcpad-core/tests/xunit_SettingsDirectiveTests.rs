//! Transcribed from Calcpad.Tests/ExpressionParser/SettingsDirectiveTests.cs by tools/transcribe_xunit.py.
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

fn Render(source: &str) -> String {
    render(source)
}
// Regex width:(\d+)pt over the rendered plot; plots are out of scope for phase 1.
fn PlotWidthStyle(source: &str) -> Option<String> {
    let html = render(source);
    let i = html.find("width:")?;
    let rest = &html[i + 6..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() && rest[digits.len()..].starts_with("pt") {
        Some(digits)
    } else {
        None
    }
}

#[test]
fn SettingsDirective_AppliesDecimals() {
    let mut html = Render("#settings {\"decimals\": 4}\nx = 6.12345");
    assert_contains("6.1235", &html);
}
#[test]
fn SettingsDirective_ChangesSettingsMidFile() {
    let mut html = Render("#settings {\"decimals\": 4}\nx = 6.12345\n#settings {\"decimals\": 2}\ny = 8.7654");
    assert_contains("6.1235", &html);
    assert_contains("8.77", &html);
    assert_does_not_contain("8.7654", &html);
}
#[test]
#[ignore = "out of scope for phase 1: plots ($Plot)"]
fn SettingsDirective_LaterDirective_OverridesEarlierVariable() {}
#[test]
#[ignore = "out of scope for phase 1: plots ($Plot)"]
fn SettingsDirective_LaterVariable_OverridesEarlierDirective() {}
#[test]
#[ignore = "out of scope for phase 1: plot settings (plotWidth)"]
fn SettingsDirective_SettingControllingVariable_StaysReadable() {}
#[test]
fn SettingsDirective_MalformedJson_ReportsError() {
    let mut parser = new_expression_parser(&[("Debug", true)]);
    parser.parse("#settings {not valid}\nx = 1", true);
    assert_contains("Invalid JSON in #settings", &parser.html_result);
}
#[test]
fn SettingsDirective_UnknownKey_IsIgnored() {
    let mut parser = new_expression_parser(&[("Debug", true)]);
    parser.parse("#settings {\"nonsense\": 4}\nx = 1", true);
    assert_does_not_contain("Invalid JSON in #settings", &parser.html_result);
}
