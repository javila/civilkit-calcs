//! SettingsVariableTests.cs
mod common;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}

fn unused_warnings(r: &LinterResult, name: &str) -> usize {
    r.diagnostics
        .iter()
        .filter(|d| d.code == "CPD-3312" && d.message.contains(&format!("'{name}'")))
        .count()
}

#[test]
fn settings_variable_unused_no_warning() {
    for name in [
        "PlotHeight",
        "PlotWidth",
        "PlotStep",
        "Precision",
        "Tol",
        "ReturnAngleUnits",
    ] {
        let result = lint(&format!("{name} = 100\n"));
        assert_eq!(0, unused_warnings(&result, name), "{name}");
    }
}

#[test]
fn ordinary_variable_unused_still_warns() {
    assert!(unused_warnings(&lint("myVar = 100\n"), "myVar") > 0);
}

#[test]
fn settings_variable_wrong_case_still_warns() {
    // Core's settings lookup is case-sensitive, so a lowercase name is a plain variable.
    assert!(unused_warnings(&lint("precision = 100\n"), "precision") > 0);
}
