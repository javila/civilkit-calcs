//! MatrixVectorProductTypeTests.cs
mod common;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}

fn type_warnings(r: &LinterResult) -> usize {
    r.diagnostics.iter().filter(|d| d.code == "CPD-3309").count()
}

#[test]
fn matrix_times_vector_into_vector_param_no_type_warning() {
    let src = "X = [1; 2 | 3; 4]\n\
               y = [5; 6]\n\
               b = X*y\n\
               A = copy(X*transp(X); symmetric(n_rows(X)); 1; 1)\n\
               a = clsolve(A; b)\n";
    assert_eq!(0, type_warnings(&lint(src)));
}

#[test]
fn matrix_times_matrix_into_vector_param_still_warns() {
    let src = "X = [1; 2 | 3; 4]\nA = [1; 2 | 3; 4]\nM = X*A\nr = clsolve(A; M)\n";
    assert!(type_warnings(&lint(src)) > 0);
}
