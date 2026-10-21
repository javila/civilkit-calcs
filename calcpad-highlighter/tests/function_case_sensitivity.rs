//! FunctionCaseSensitivityTests.cs
mod common;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}

fn param_count_errors(r: &LinterResult) -> usize {
    r.diagnostics.iter().filter(|d| d.code == "CPD-3302").count()
}

fn undefined_errors(r: &LinterResult) -> usize {
    r.diagnostics
        .iter()
        .filter(|d| d.code == "CPD-3301" || d.code == "CPD-3305")
        .count()
}

#[test]
fn upper_and_lower_case_functions_are_distinct_no_param_count_error() {
    let result = lint("F(a; b) = a + b\nf(x) = x^2\nr = f(3)\n");
    assert_eq!(0, param_count_errors(&result));
}

#[test]
fn command_block_function_does_not_shadow_lower_case_function() {
    let src = "F(φ; k) = $Integral{1/sqrt(1 - k^2*sin(θ)^2) @ θ = 0 : φ}\n\
               f(θ) = sin(-θ) - θ\n\
               r = f(1)\n";
    assert_eq!(0, param_count_errors(&lint(src)));
}

#[test]
fn calling_wrong_case_function_is_flagged_undefined() {
    let result = lint("F(a; b) = a + b\nr = f(1)\n");
    assert!(undefined_errors(&result) > 0);
    assert_eq!(0, param_count_errors(&result));
}
