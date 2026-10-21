//! ElementWiseFunctionTypeTests.cs
mod common;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}

fn type_warnings(result: &LinterResult) -> usize {
    result.diagnostics.iter().filter(|d| d.code == "CPD-3309").count()
}

#[test]
fn element_wise_function_accepts_matrix_no_type_warning() {
    for func in ["sin", "cos", "sqrt", "ln", "exp", "floor", "abs", "sign"] {
        let result = lint(&format!("M = [1; 2; 3 | 4; 5; 6]\nr = {func}(M)\n"));
        assert_eq!(0, type_warnings(&result), "{func}");
    }
}

#[test]
fn element_wise_function_accepts_vector_literal_no_type_warning() {
    assert_eq!(0, type_warnings(&lint("r = sin([1; 2; 3])\n")));
}

#[test]
fn non_element_wise_function_still_warns_on_shape_mismatch() {
    // n_rows expects a matrix and is NOT element-wise, so passing a scalar is a real mismatch.
    assert!(type_warnings(&lint("r = n_rows(5)\n")) > 0);
}

#[test]
fn matrix_function_accepts_vector_no_type_warning() {
    for call in [
        "augment(v; v)",
        "stack(v; v)",
        "hprod(v; v)",
        "transp(v)",
        "row(v; 1)",
        "col(v; 1)",
        "n_rows(v)",
    ] {
        let result = lint(&format!("v = [1; 2; 3]\nr = {call}\n"));
        assert_eq!(0, type_warnings(&result), "{call}");
    }
}

#[test]
fn interpolation_accepts_index_and_matrix_no_type_warning() {
    for func in ["take", "line", "spline"] {
        let result = lint(&format!("M = [1; 2; 3 | 4; 5; 6]\nr = {func}(2; M)\n"));
        assert_eq!(0, type_warnings(&result), "{func}");
    }
}

#[test]
fn multi_function_accepts_vector_and_matrix_no_type_warning() {
    for func in ["gcd", "lcm"] {
        let result = lint(&format!("v = [12; 18]\nM = [24; 30 | 36; 42]\nr = {func}(v; M; 6)\n"));
        assert_eq!(0, type_warnings(&result), "{func}");
    }
}
