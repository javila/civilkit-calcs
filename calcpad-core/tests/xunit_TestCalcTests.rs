//! Transcribed from Calcpad.Tests/TestCalcTests.cs by tools/transcribe_xunit.py.
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

#[test]
fn CompareWithTolerance_UsesAbsoluteTolerance_NearZero() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "c = 10^-8",
        "c_hp = c + 5*10^-13",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn CompareWithTolerance_UsesRelativeTolerance_ForLargerValues() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "c = 10^6",
        "c_hp = c + 5*10^-7",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn CompareWithTolerance_Fails_WhenAbsoluteToleranceIsExceeded_NearZero() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "c = 10^-8",
        "c_hp = c + 2*10^-12",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-12"),
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn CompareWithTolerance_Fails_WhenRelativeToleranceIsExceeded_ForLargerValues() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "c = 10^6",
        "c_hp = c + 2*10^-6",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-12"),
    ]);
    assert_equal(0.0, result.clone());
}
