//! CommandScopeVariableTests.cs
mod common;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}

fn count(result: &LinterResult, code: &str) -> usize {
    result.diagnostics.iter().filter(|d| d.code == code).count()
}

#[test]
fn command_scope_variable_colliding_with_unit_no_syntax_error() {
    for src in [
        "#rad\nx(t) = $Integral{cos(s^2) @ s = 0 : t}",
        "$Sum{s^2 @ s = 1 : 5}",
        "q = $Integral{cos(m^2) @ m = 0 : 5}",
    ] {
        assert_eq!(0, count(&lint(src), "CPD-3406"), "{src}");
    }
}

#[test]
fn inf_defines_counter_inf_variable() {
    let src = "f(k) = k^2 - 3*k\nA_inf = $Inf{f(k) @ k = 0 : 5}\nk = k_inf\n";
    assert_eq!(0, count(&lint(src), "CPD-3301"));
}

#[test]
fn sup_defines_counter_sup_variable() {
    let src = "f(x) = -x^2 + 4*x\nA_sup = $Sup{f(x) @ x = 0 : 5}\nx_peak = x_sup\n";
    assert_eq!(0, count(&lint(src), "CPD-3301"));
}

#[test]
fn counter_inf_with_unrelated_name_still_undefined() {
    let src = "f(k) = k^2\nA_inf = $Inf{f(k) @ k = 0 : 5}\ny = j_inf\n";
    assert!(count(&lint(src), "CPD-3301") > 0);
}

#[test]
fn nested_inf_defines_both_counter_variables() {
    let src = "f(x; y) = (x^2 + y - 11)^2 + (x + y^2 - 7)^2\n\
               z_inf = $inf{$inf{f(x; y) @ x = -5 : 0} @ y = -5 : 0}\n\
               x_inf','y_inf\n";
    assert_eq!(0, count(&lint(src), "CPD-3301"));
}

#[test]
fn implicit_solver_variables_appear_in_definitions_for_autocomplete() {
    let src = "f(x; y) = x^2 + y^2\nz_inf = $inf{$inf{f(x; y) @ x = -5 : 0} @ y = -5 : 0}\n";
    let staged = ContentResolver::new().get_staged_content(src, None, None, None);
    let names: Vec<&str> = staged
        .stage3
        .variables_with_definitions
        .iter()
        .map(|v| v.name.as_str())
        .collect();
    assert!(names.contains(&"x_inf"), "{names:?}");
    assert!(names.contains(&"y_inf"), "{names:?}");
}

#[test]
fn non_inf_solver_does_not_define_implicit_variable() {
    let src = "f(x) = x^2\ns = $Sum{f(k) @ k = 1 : 5}\ny = k_inf\n";
    assert!(count(&lint(src), "CPD-3301") > 0);
}

#[test]
fn lu_decomposition_defines_ind_permutation_vector() {
    let src = "A = [4; 12; -16|12; 37; -43|-16; -43; 98]\nLU = lu(A)\np = ind\n";
    assert_eq!(0, count(&lint(src), "CPD-3301"));
}

#[test]
fn lu_ind_variable_appears_in_definitions_for_autocomplete() {
    let src = "A = [1; 2 | 3; 4]\nLU = lu(A)\n";
    let staged = ContentResolver::new().get_staged_content(src, None, None, None);
    assert!(staged.stage3.variables_with_definitions.iter().any(|v| v.name == "ind"));
}

#[test]
fn ind_without_lu_still_undefined() {
    assert!(count(&lint("x = ind\n"), "CPD-3301") > 0);
}
