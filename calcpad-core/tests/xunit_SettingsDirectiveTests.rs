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
    noop_method_call,
    clippy::needless_borrows_for_generic_args
)]
mod common;
use common::*;

fn Render(source: &str) -> String {
    render(source)
}
// Regex.Match(Render(source), @"width:(\d+)pt"): the first "width:" followed by digits and "pt".
fn PlotWidthStyle(source: &str) -> Option<String> {
    let html = render(source);
    let mut from = 0;
    while let Some(i) = html[from..].find("width:") {
        let rest = &html[from + i + 6..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() && rest[digits.len()..].starts_with("pt") {
            return Some(digits);
        }
        from += i + 6;
    }
    None
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
#[ignore = "out of scope: raster (PNG) plots drawn by SkiaSharp (default VectorGraphics=false)"]
fn SettingsDirective_LaterDirective_OverridesEarlierVariable() {}
#[test]
#[ignore = "out of scope: raster (PNG) plots drawn by SkiaSharp (default VectorGraphics=false)"]
fn SettingsDirective_LaterVariable_OverridesEarlierDirective() {}
#[test]
fn SettingsDirective_SettingControllingVariable_StaysReadable() {
    let mut html = Render("#settings {\"plotWidth\": 400}\nw = PlotWidth");
    assert_contains("400", &html);
}
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
