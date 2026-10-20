#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/UtriangMatrixOperatorTests.cs by tools/transcribe_xunit.py.
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

#[test]
fn UtriangMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([4; 7; 5|0; 1; 1|0; 0; 2]; utriang(3); 1; 1)",
        "D3 = D2 + D1",
    ]);
    assert_equal_str("[6 10 9|0 6 7|0 0 9]", &calc.ToString());
}
#[test]
fn UtriangMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; utriang(3); 1; 1)",
        "D3 = D2 - D1",
    ]);
    assert_equal_str("[-1 -1 -1|0 -1 -1|0 0 -1]", &calc.ToString());
}
#[test]
fn UtriangMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; utriang(3); 1; 1)",
        "D3 = D2 ^ D1",
    ]);
    assert_equal_str("[4 8 16|1 32 64|1 1 128]", &calc.ToString());
}
#[test]
fn UtriangMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; utriang(3); 1; 1)",
        "D3 = D2 / D1",
    ]);
    assert_equal_str(
        "[0.5 0.667 0.75| Undefined  0.8 0.833| Undefined   Undefined  0.857]",
        &calc.ToString(),
    );
}
#[test]
fn UtriangMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([2; 4; 6|0; 1; 3|0; 0; 5]; utriang(3); 1; 1)",
        "D3 = D2 * D1",
    ]);
    assert_equal_str("[4 26 74|0 5 27|0 0 35]", &calc.ToString());
}
#[test]
fn UtriangMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 2; 1|0; 3; 2|0; 0; 3]; utriang(3); 1; 1)",
        "D3 = D2 \\ D1",
    ]);
    assert_equal_str("[0 0 0| Undefined  0 0| Undefined   Undefined  0]", &calc.ToString());
}
#[test]
fn UtriangMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([4; 8; 12|0; 5; 10|0; 0; 14]; utriang(3); 1; 1)",
        "D3 = D2 ÷ D1",
    ]);
    assert_equal_str(
        "[2 2.67 3| Undefined  1 1.67| Undefined   Undefined  2]",
        &calc.ToString(),
    );
}
#[test]
fn UtriangMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([4; 7; 5|0; 3; 6|0; 0; 8]; utriang(3); 1; 1)",
        "D3 = D2 ⦼ D1",
    ]);
    assert_equal_str("[0 1 1| Undefined  3 0| Undefined   Undefined  1]", &calc.ToString());
}
#[test]
fn UtriangMatrixFactorial() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)", "D3 = D1!"]);
    assert_equal_str("[2 6 24|1 120 720|1 1 5040]", &calc.ToString());
}
#[test]
fn UtriangMatrixEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D3 = D2 ≡ D1",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; utriang(3); 1; 1)",
        "D3 = D2 ≠ D1",
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([6; 5; 4|0; 4; 3|0; 0; 2]; utriang(3); 1; 1)",
        "D3 = D2 < D1",
    ]);
    assert_equal_str("[0 0 0|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 6; 5|0; 0; 7]; utriang(3); 1; 1)",
        "D3 = D2 > D1",
    ]);
    assert_equal_str("[0 0 0|0 1 0|0 0 0]", &calc.ToString());
}
#[test]
fn UtriangMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([5; 5; 5|0; 5; 5|0; 0; 5]; utriang(3); 1; 1)",
        "D3 = D2 ≤ D1",
    ]);
    assert_equal_str("[0 0 0|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D3 = D2 ≥ D1",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 0; 1|0; 0; 1]; utriang(3); 1; 1)",
        "D3 = D2 ∧ D1",
    ]);
    assert_equal_str("[1 1 1|0 0 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 0; 1|0; 0; 0]; utriang(3); 1; 1)",
        "D3 = D2 ∨ D1",
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = copy([1; 0; 1|0; 1; 0|0; 0; 1]; utriang(3); 1; 1)",
        "D3 = D2 ⊕ D1",
    ]);
    assert_equal_str("[0 1 0|0 0 1|0 0 0]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 + D2",
    ]);
    assert_equal_str("[7 8 9|5 10 11|5 5 12]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 - D2",
    ]);
    assert_equal_str("[-1 0 1|-3 2 3|-3 -3 4]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ^ D2",
    ]);
    assert_equal_str("[4 9 16|0 25 36|0 0 49]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 / D2",
    ]);
    assert_equal_str("[1 1.5 2|0 2.5 3|0 0 3.5]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 * D2",
    ]);
    assert_equal_str("[6 9 12|0 15 18|0 0 21]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 \\ D2",
    ]);
    assert_equal_str("[0 0 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ÷ D2",
    ]);
    assert_equal_str("[1 1.5 2|0 2.5 3|0 0 3.5]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ⦼ D2",
    ]);
    assert_equal_str("[2 3 0|0 1 2|0 0 3]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ≡ D2",
    ]);
    assert_equal_str("[1 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ≠ D2",
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 < D2",
    ]);
    assert_equal_str("[1 1 1|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 1",
        "D3 = D1 > D2",
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 ≤ D2",
    ]);
    assert_equal_str("[1 1 0|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ≥ D2",
    ]);
    assert_equal_str("[0 0 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∧ D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∨ D2",
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn UtriangMatrixScalarLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 1",
        "D3 = D1 ⊕ D2",
    ]);
    assert_equal_str("[0 0 0|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D2 + D1",
    ]);
    assert_equal_str("[7 8 9|5 10 11|5 5 12]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 - D1",
    ]);
    assert_equal_str("[1 0 -1|3 -2 -3|3 3 -4]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 ^ D1",
    ]);
    assert_equal_str("[4 8 16|1 32 64|1 1 128]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 12",
        "D3 = D2 / D1",
    ]);
    assert_equal_str("[6 4 3|+∞ 2.4 2|+∞ +∞ 1.71]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 * D1",
    ]);
    assert_equal_str("[6 9 12|0 15 18|0 0 21]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 8",
        "D3 = D2 \\ D1",
    ]);
    assert_equal_str("[4 2 2| Undefined  1 1| Undefined   Undefined  1]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 10",
        "D3 = D2 ÷ D1",
    ]);
    assert_equal_str("[5 3.33 2.5|+∞ 2 1.67|+∞ +∞ 1.43]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 9",
        "D3 = D2 ⦼ D1",
    ]);
    assert_equal_str("[1 0 1| Undefined  4 3| Undefined   Undefined  2]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D2 < D1",
    ]);
    assert_equal_str("[0 0 0|0 0 1|0 0 1]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 4",
        "D3 = D2 > D1",
    ]);
    assert_equal_str("[1 1 0|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 ≤ D1",
    ]);
    assert_equal_str("[0 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn ScalarUtriangMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D2 ≥ D1",
    ]);
    assert_equal_str("[1 1 1|1 1 0|1 1 0]", &calc.ToString());
}
