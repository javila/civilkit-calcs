#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Small/HPSymmetricMatrixOperatorTests.cs by tools/transcribe_xunit.py.
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
fn HPSymmetricMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric_hp(3); 1; 1)",
        "D2 = copy([1; 1; 1|0; 1; 1|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D3 = D1 + D2",
    ]);
    assert_equal_str("[3 4 5|4 6 7|5 7 8]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 6; 5|0; 4; 3|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = copy([2; 1; 1|0; 2; 1|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D3 = D1 - D2",
    ]);
    assert_equal_str("[5 5 4|5 2 2|4 2 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ^ D2",
    ]);
    assert_equal_str("[1 4 9|4 16 25|9 25 36]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 6; 4|0; 3; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D3 = D1 / D2",
    ]);
    assert_equal_str("[4 3 2|3 1.5 1|2 1 0.5]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric_hp(3); 1; 1)",
        "D2 = copy([3; 3; 3|0; 3; 3|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D3 = D1 * D2",
    ]);
    assert_equal_str("[36 36 36|51 51 51|60 60 60]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 7; 6|0; 5; 4|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D3 = D1 \\ D2",
    ]);
    assert_equal_str("[4 3 3|3 2 2|3 2 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 7; 5|0; 6; 4|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ÷ D2",
    ]);
    assert_equal_str("[4.5 3.5 2.5|3.5 3 2|2.5 2 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D2 = copy([4; 4; 4|0; 4; 4|0; 0; 4]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ⦼ D2",
    ]);
    assert_equal_str("[3 0 1|0 1 2|1 2 3]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixFactorial() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric_hp(3); 1; 1)",
        "D3 = D1!",
    ]);
    assert_equal_str("[6 24 120|24 720 5040|120 5040 40320]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = copy([3; 3; 3|0; 3; 3|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ≡ D2",
    ]);
    assert_equal_str("[0 0 1|0 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = copy([4; 4; 4|0; 4; 4|0; 0; 4]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ≠ D2",
    ]);
    assert_equal_str("[1 0 1|0 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 6; 7|0; 4; 3|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = copy([5; 5; 5|0; 5; 5|0; 0; 5]; symmetric_hp(3); 1; 1)",
        "D3 = D1 < D2",
    ]);
    assert_equal_str("[0 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 3; 2|0; 6; 5|0; 0; 7]; symmetric_hp(3); 1; 1)",
        "D2 = copy([3; 3; 3|0; 3; 3|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D3 = D1 > D2",
    ]);
    assert_equal_str("[1 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 6; 7|0; 3; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = copy([6; 6; 6|0; 6; 6|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ≤ D2",
    ]);
    assert_equal_str("[1 1 0|1 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 5; 7|0; 6; 4|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = copy([4; 4; 4|0; 4; 4|0; 0; 4]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ≥ D2",
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D2 = copy([1; 1; 1|0; 1; 1|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ∧ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 0; 0|0; 1; 0|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = copy([0; 1; 1|0; 1; 1|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ∨ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 1; 0|0; 1; 0|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = copy([1; 0; 1|0; 1; 1|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D3 = D1 ⊕ D2",
    ]);
    assert_equal_str("[0 1 1|1 0 1|1 1 0]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric_hp(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 + D2",
    ]);
    assert_equal_str("[7 8 9|8 10 11|9 11 12]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 6; 5|0; 4; 3|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 - D2",
    ]);
    assert_equal_str("[4 3 2|3 1 0|2 0 -1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ^ D2",
    ]);
    assert_equal_str("[1 4 9|4 16 25|9 25 36]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 6; 4|0; 3; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 / D2",
    ]);
    assert_equal_str("[4 3 2|3 1.5 1|2 1 0.5]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric_hp(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 * D2",
    ]);
    assert_equal_str("[9 12 15|12 18 21|15 21 24]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 7; 6|0; 5; 4|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 \\ D2",
    ]);
    assert_equal_str("[4 3 3|3 2 2|3 2 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 7; 5|0; 6; 4|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ÷ D2",
    ]);
    assert_equal_str("[4.5 3.5 2.5|3.5 3 2|2.5 2 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ⦼ D2",
    ]);
    assert_equal_str("[3 0 1|0 1 2|1 2 3]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 ≡ D2",
    ]);
    assert_equal_str("[0 0 1|0 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ≠ D2",
    ]);
    assert_equal_str("[1 0 1|0 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 6; 7|0; 4; 3|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 < D2",
    ]);
    assert_equal_str("[0 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 3; 2|0; 6; 5|0; 0; 7]; symmetric_hp(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 > D2",
    ]);
    assert_equal_str("[1 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 6; 7|0; 3; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = 6",
        "D3 = D1 ≤ D2",
    ]);
    assert_equal_str("[1 1 0|1 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 5; 7|0; 6; 4|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ≥ D2",
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∧ D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∨ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixScalarLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D2 = 1",
        "D3 = D1 ⊕ D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric_hp(3); 1; 1)",
        "D2 = 5",
        "D3 = D2 + D1",
    ]);
    assert_equal_str("[7 8 9|8 10 11|9 11 12]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 6; 5|0; 4; 3|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 - D1",
    ]);
    assert_equal_str("[-4 -3 -2|-3 -1 0|-2 0 1]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 ^ D1",
    ]);
    assert_equal_str("[2 4 8|4 16 32|8 32 64]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 6; 4|0; 3; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 / D1",
    ]);
    assert_equal_str("[0.25 0.333 0.5|0.333 0.667 1|0.5 1 2]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric_hp(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 * D1",
    ]);
    assert_equal_str("[9 12 15|12 18 21|15 21 24]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 7; 6|0; 5; 4|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 \\ D1",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 7; 5|0; 6; 4|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 ÷ D1",
    ]);
    assert_equal_str("[0.222 0.286 0.4|0.286 0.333 0.5|0.4 0.5 1]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "D2 = 4",
        "D3 = D2 ⦼ D1",
    ]);
    assert_equal_str("[4 4 4|4 4 4|4 4 1]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 6; 7|0; 4; 3|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 5",
        "D3 = D2 < D1",
    ]);
    assert_equal_str("[0 1 1|1 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 3; 2|0; 6; 5|0; 0; 7]; symmetric_hp(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 > D1",
    ]);
    assert_equal_str("[0 0 1|0 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 6; 7|0; 3; 2|0; 0; 1]; symmetric_hp(3); 1; 1)",
        "D2 = 6",
        "D3 = D2 ≤ D1",
    ]);
    assert_equal_str("[0 1 1|1 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn HPScalarSymmetricMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 5; 7|0; 6; 4|0; 0; 2]; symmetric_hp(3); 1; 1)",
        "D2 = 4",
        "D3 = D2 ≥ D1",
    ]);
    assert_equal_str("[1 0 0|0 0 1|0 1 1]", &calc.ToString());
}
