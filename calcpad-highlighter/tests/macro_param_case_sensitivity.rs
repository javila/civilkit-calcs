//! MacroParamCaseSensitivityTests.cs
mod common;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}

fn duplicate_param_errors(r: &LinterResult) -> usize {
    r.diagnostics.iter().filter(|d| d.code == "CPD-2212").count()
}

#[test]
fn params_differing_only_by_case_are_not_duplicates() {
    assert_eq!(0, duplicate_param_errors(&lint("#def sum$(A$; a$) = A$ + a$\n")));
}

#[test]
fn multiline_params_differing_only_by_case_are_not_duplicates() {
    let src = "#def report$(X$; x$)\nX$ = x$\n#end def\n";
    assert_eq!(0, duplicate_param_errors(&lint(src)));
}

#[test]
fn identical_params_are_still_duplicates() {
    assert_eq!(1, duplicate_param_errors(&lint("#def sum$(a$; a$) = a$\n")));
}
