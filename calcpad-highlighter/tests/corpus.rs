//! ComprehensiveValidTests.cs and ComprehensiveErrorTests.cs
mod common;
use calcpad_highlighter::linter::models::LinterSeverity;
use common::*;

#[test]
fn valid_examples_have_no_errors() {
    let mut failures = Vec::new();
    let files = cpd_files("valid");
    assert!(files.len() >= 29);
    for p in &files {
        let r = lint_file(p);
        let errors: Vec<_> = r
            .diagnostics
            .iter()
            .filter(|d| d.severity == LinterSeverity::Error)
            .collect();
        if !errors.is_empty() {
            let msg: Vec<String> = errors
                .iter()
                .map(|e| format!("[{}] Line {}: {}", e.code, e.line + 1, e.message))
                .collect();
            failures.push(format!(
                "{}: {}",
                p.file_name().unwrap().to_string_lossy(),
                msg.join("; ")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn error_examples_produce_at_least_one_error() {
    let mut failures = Vec::new();
    for p in cpd_files("errors") {
        let r = lint_file(&p);
        if !r.diagnostics.iter().any(|d| d.severity == LinterSeverity::Error) {
            failures.push(p.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(failures.is_empty(), "no error in: {failures:?}");
}

const EXPECTED: &[(&str, &str)] = &[
    ("balance_errors.cpd", "CPD-3101"),
    ("balance_errors.cpd", "CPD-3102"),
    ("balance_errors.cpd", "CPD-3103"),
    ("balance_errors.cpd", "CPD-3104"),
    ("balance_errors.cpd", "CPD-3105"),
    ("balance_errors.cpd", "CPD-3106"),
    ("include_errors.cpd", "CPD-1102"),
    ("include_errors.cpd", "CPD-1103"),
    ("macro_errors.cpd", "CPD-2201"),
    ("macro_errors.cpd", "CPD-2202"),
    ("macro_errors.cpd", "CPD-2206"),
    ("macro_errors.cpd", "CPD-2207"),
    ("macro_errors.cpd", "CPD-2209"),
    ("macro_errors.cpd", "CPD-2210"),
    ("macro_errors.cpd", "CPD-2212"),
    ("macro_errors.cpd", "CPD-2213"),
    ("macro_errors.cpd", "CPD-3314"),
    ("naming_errors.cpd", "CPD-3204"),
    ("naming_errors.cpd", "CPD-3205"),
    ("naming_errors.cpd", "CPD-3206"),
    ("naming_errors.cpd", "CPD-3207"),
    ("usage_errors.cpd", "CPD-3301"),
    ("usage_errors.cpd", "CPD-3302"),
    ("usage_errors.cpd", "CPD-3303"),
    ("usage_errors.cpd", "CPD-3307"),
    ("usage_errors.cpd", "CPD-3308"),
    ("usage_errors.cpd", "CPD-3310"),
    ("usage_errors.cpd", "CPD-3311"),
    ("usage_errors.cpd", "CPD-3315"),
    ("semantic_errors.cpd", "CPD-3401"),
    ("semantic_errors.cpd", "CPD-3403"),
    ("semantic_errors.cpd", "CPD-3405"),
    ("semantic_errors.cpd", "CPD-3407"),
    ("semantic_errors.cpd", "CPD-3408"),
    ("semantic_errors.cpd", "CPD-3410"),
    ("semantic_errors.cpd", "CPD-3419"),
    ("reassignment_errors.cpd", "CPD-3409"),
    ("reassignment_errors.cpd", "CPD-3410"),
    ("ui_errors.cpd", "CPD-3415"),
];

#[test]
fn error_examples_produce_expected_error_codes() {
    let mut failures = Vec::new();
    for (file, code) in EXPECTED {
        let r = lint_file(&base_dir().join("errors").join(file));
        if !r.diagnostics.iter().any(|d| d.code == *code) {
            let found: Vec<String> = r
                .diagnostics
                .iter()
                .map(|d| format!("[{}] {:?} Line {}", d.code, d.severity, d.line + 1))
                .collect();
            failures.push(format!("{file} {code}: found {}", found.join("; ")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
