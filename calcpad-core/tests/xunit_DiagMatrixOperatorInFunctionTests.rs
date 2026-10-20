#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/DiagMatrixOperatorInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn DiagMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = copy([4|0; 5|0; 0; 6]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a + b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[5 0 0|0 7 0|0 0 9]", &calc.ToString());
}
#[test]
fn DiagMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9|0; 6|0; 0; 8]; diagonal(3; 1); 1; 1)",
        "D2 = copy([4|0; 3|0; 0; 5]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a - b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[5 0 0|0 3 0|0 0 3]", &calc.ToString());
}
#[test]
fn DiagMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal(3; 1); 1; 1)",
        "D2 = copy([3|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ^ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[8 1 1|1 9 1|1 1 64]", &calc.ToString());
}
#[test]
fn DiagMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 7|0; 0; 6]; diagonal(3; 1); 1; 1)",
        "D2 = copy([4|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a / b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[2  Undefined   Undefined | Undefined  3.5  Undefined | Undefined   Undefined  2]",
        &calc.ToString(),
    );
}
#[test]
fn DiagMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5|0; 4|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = copy([3|0; 6|0; 0; 2]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a * b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[15 0 0|0 24 0|0 0 6]", &calc.ToString());
}
#[test]
fn DiagMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9|0; 7|0; 0; 8]; diagonal(3; 1); 1; 1)",
        "D2 = copy([3|0; 2|0; 0; 4]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a \\ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[3  Undefined   Undefined | Undefined  3  Undefined | Undefined   Undefined  2]",
        &calc.ToString(),
    );
}
#[test]
fn DiagMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 6|0; 0; 9]; diagonal(3; 1); 1; 1)",
        "D2 = copy([4|0; 3|0; 0; 5]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ÷ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[2  Undefined   Undefined | Undefined  2  Undefined | Undefined   Undefined  1.8]",
        &calc.ToString(),
    );
}
#[test]
fn DiagMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([10|0; 9|0; 0; 7]; diagonal(3; 1); 1; 1)",
        "D2 = copy([4|0; 5|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ⦼ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[2  Undefined   Undefined | Undefined  4  Undefined | Undefined   Undefined  1]",
        &calc.ToString(),
    );
}
#[test]
fn DiagMatrixFactorial() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3|0; 4|0; 0; 5]; diagonal(3; 1); 1; 1)",
        "f(a) = a!",
        &format!("f(D1)"),
    ]);
    assert_equal_str("[6 1 1|1 24 1|1 1 120]", &calc.ToString());
}
#[test]
fn DiagMatrixEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 4|0; 0; 6]; diagonal(3; 1); 1; 1)",
        "D2 = copy([3|0; 5|0; 0; 7]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ≡ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 1 1|1 0 1|1 1 0]", &calc.ToString());
}
#[test]
fn DiagMatrixNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3|0; 5|0; 0; 7]; diagonal(3; 1); 1; 1)",
        "D2 = copy([2|0; 4|0; 0; 6]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ≠ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal(3; 1); 1; 1)",
        "D2 = copy([5|0; 6|0; 0; 7]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a < b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7|0; 8|0; 0; 9]; diagonal(3; 1); 1; 1)",
        "D2 = copy([3|0; 4|0; 0; 5]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a > b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 4|0; 0; 6]; diagonal(3; 1); 1; 1)",
        "D2 = copy([5|0; 6|0; 0; 8]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ≤ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn DiagMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([6|0; 7|0; 0; 8]; diagonal(3; 1); 1; 1)",
        "D2 = copy([3|0; 4|0; 0; 5]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ≥ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn DiagMatrixLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = copy([0|0; 1|0; 0; 1]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ∧ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([0|0; 1|0; 0; 1]; diagonal(3; 1); 1; 1)",
        "D2 = copy([1|0; 0|0; 0; 1]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ∨ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 1|0; 0; 1]; diagonal(3; 1); 1; 1)",
        "D2 = copy([0|0; 1|0; 0; 0]; diagonal(3; 1); 1; 1)",
        "f(a; b) = a ⊕ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 0 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal(3; 1); 1; 1)",
        "D2 = 5",
        "f(a; b) = a + b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[7 5 5|5 8 5|5 5 9]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 9|0; 0; 10]; diagonal(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a - b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[5 -3 -3|-3 6 -3|-3 -3 7]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ^ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[8 0 0|0 27 0|0 0 64]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([6|0; 8|0; 0; 12]; diagonal(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a / b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[3 0 0|0 4 0|0 0 6]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 4|0; 0; 5]; diagonal(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a * b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[6 0 0|0 12 0|0 0 15]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 10|0; 0; 12]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a \\ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[2 0 0|0 2 0|0 0 3]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([6|0; 8|0; 0; 10]; diagonal(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ÷ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[3 0 0|0 4 0|0 0 5]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7|0; 10|0; 0; 13]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a ⦼ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[3 0 0|0 2 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 1",
        "f(a; b) = a ≡ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≠ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 0 1|1 1 1]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a < b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 11",
        "f(a; b) = a > b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≤ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ≥ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 1|0; 0; 1]; diagonal(3; 1); 1; 1)",
        "D2 = 0",
        "f(a; b) = a ∧ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 1|0; 0; 1]; diagonal(3; 1); 1; 1)",
        "D2 = 0",
        "f(a; b) = a ∨ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagMatrixScalarLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 0|0; 0; 1]; diagonal(3; 1); 1; 1)",
        "D2 = 1",
        "f(a; b) = a ⊕ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a + b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[5 4 4|4 6 4|4 4 7]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7|0; 8|0; 0; 9]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a - b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[-3 4 4|4 -4 4|4 4 -5]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a ^ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 1 1|1 16 1|1 1 64]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a / b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 +∞ +∞|+∞ 2 +∞|+∞ +∞ 1.33]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a * b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[2 0 0|0 4 0|0 0 6]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a \\ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str(
        "[4  Undefined   Undefined | Undefined  2  Undefined | Undefined   Undefined  1]",
        &calc.ToString(),
    );
}
#[test]
fn ScalarDiagMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 5",
        "f(a; b) = a ÷ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[5 +∞ +∞|+∞ 2.5 +∞|+∞ +∞ 1.67]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ⦼ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str(
        "[0  Undefined   Undefined | Undefined  1  Undefined | Undefined   Undefined  0]",
        &calc.ToString(),
    );
}
#[test]
fn ScalarDiagMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a < b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 11",
        "f(a; b) = a > b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≤ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn ScalarDiagMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ≥ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
