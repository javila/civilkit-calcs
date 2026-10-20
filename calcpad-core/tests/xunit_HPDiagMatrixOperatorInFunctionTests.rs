//! Transcribed from Calcpad.Tests/Matrices/HP/Small/HPDiagMatrixOperatorInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPDiagMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([4|0; 5|0; 0; 6]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a + b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[5 0 0|0 7 0|0 0 9]", &calc.ToString());
}
#[test]
fn HPDiagMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9|0; 6|0; 0; 8]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([4|0; 3|0; 0; 5]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a - b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[5 0 0|0 3 0|0 0 3]", &calc.ToString());
}
#[test]
fn HPDiagMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([3|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ^ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[8 1 1|1 9 1|1 1 64]", &calc.ToString());
}
#[test]
fn HPDiagMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 7|0; 0; 6]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([4|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a / b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[2  Undefined   Undefined | Undefined  3.5  Undefined | Undefined   Undefined  2]",
        &calc.ToString(),
    );
}
#[test]
fn HPDiagMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5|0; 4|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([3|0; 6|0; 0; 2]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a * b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[15 0 0|0 24 0|0 0 6]", &calc.ToString());
}
#[test]
fn HPDiagMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9|0; 7|0; 0; 8]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([3|0; 2|0; 0; 4]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a \\ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[3  Undefined   Undefined | Undefined  3  Undefined | Undefined   Undefined  2]",
        &calc.ToString(),
    );
}
#[test]
fn HPDiagMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 6|0; 0; 9]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([4|0; 3|0; 0; 5]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ÷ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[2  Undefined   Undefined | Undefined  2  Undefined | Undefined   Undefined  1.8]",
        &calc.ToString(),
    );
}
#[test]
fn HPDiagMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([10|0; 9|0; 0; 7]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([4|0; 5|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ⦼ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str(
        "[2  Undefined   Undefined | Undefined  4  Undefined | Undefined   Undefined  1]",
        &calc.ToString(),
    );
}
#[test]
fn HPDiagMatrixFactorial() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3|0; 4|0; 0; 5]; diagonal_hp(3; 1); 1; 1)",
        "f(a) = a!",
        &format!("f(D1)"),
    ]);
    assert_equal_str("[6 1 1|1 24 1|1 1 120]", &calc.ToString());
}
#[test]
fn HPDiagMatrixEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 4|0; 0; 6]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([3|0; 5|0; 0; 7]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ≡ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 1 1|1 0 1|1 1 0]", &calc.ToString());
}
#[test]
fn HPDiagMatrixNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3|0; 5|0; 0; 7]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([2|0; 4|0; 0; 6]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ≠ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([5|0; 6|0; 0; 7]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a < b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7|0; 8|0; 0; 9]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([3|0; 4|0; 0; 5]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a > b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 4|0; 0; 6]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([5|0; 6|0; 0; 8]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ≤ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([6|0; 7|0; 0; 8]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([3|0; 4|0; 0; 5]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ≥ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([0|0; 1|0; 0; 1]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ∧ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([0|0; 1|0; 0; 1]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([1|0; 0|0; 0; 1]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ∨ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 1|0; 0; 1]; diagonal_hp(3; 1); 1; 1)",
        "D2 = copy([0|0; 1|0; 0; 0]; diagonal_hp(3; 1); 1; 1)",
        "f(a; b) = a ⊕ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 0 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 5",
        "f(a; b) = a + b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[7 5 5|5 8 5|5 5 9]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 9|0; 0; 10]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a - b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[5 -3 -3|-3 6 -3|-3 -3 7]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 3|0; 0; 4]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ^ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[8 0 0|0 27 0|0 0 64]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([6|0; 8|0; 0; 12]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a / b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[3 0 0|0 4 0|0 0 6]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2|0; 4|0; 0; 5]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a * b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[6 0 0|0 12 0|0 0 15]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8|0; 10|0; 0; 12]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a \\ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[2 0 0|0 2 0|0 0 3]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([6|0; 8|0; 0; 10]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ÷ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[3 0 0|0 4 0|0 0 5]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7|0; 10|0; 0; 13]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a ⦼ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[3 0 0|0 2 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 1",
        "f(a; b) = a ≡ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≠ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 0 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a < b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 11",
        "f(a; b) = a > b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≤ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ≥ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 1|0; 0; 1]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 0",
        "f(a; b) = a ∧ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 1|0; 0; 1]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 0",
        "f(a; b) = a ∨ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagMatrixScalarLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 0|0; 0; 1]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 1",
        "f(a; b) = a ⊕ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a + b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[5 4 4|4 6 4|4 4 7]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7|0; 8|0; 0; 9]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a - b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[-3 4 4|4 -4 4|4 4 -5]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a ^ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 1 1|1 16 1|1 1 64]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a / b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 +∞ +∞|+∞ 2 +∞|+∞ +∞ 1.33]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a * b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[2 0 0|0 4 0|0 0 6]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
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
fn HPScalarDiagMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 5",
        "f(a; b) = a ÷ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[5 +∞ +∞|+∞ 2.5 +∞|+∞ +∞ 1.67]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
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
fn HPScalarDiagMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 4",
        "f(a; b) = a < b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 11",
        "f(a; b) = a > b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≤ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPScalarDiagMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1|0; 2|0; 0; 3]; diagonal_hp(3; 1); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ≥ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
