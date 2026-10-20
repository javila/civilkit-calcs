//! Transcribed from Calcpad.Tests/Matrices/SymmetricMatrixOperatorTests.cs by tools/transcribe_xunit.py.
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
fn SymmetricMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric(3); 1; 1)",
        "D2 = copy([1; 1; 1|0; 1; 1|0; 0; 1]; symmetric(3); 1; 1)",
        "D3 = D1 + D2",
    ]);
    assert_equal_str("[3 4 5|4 6 7|5 7 8]", &calc.ToString());
}
#[test]
fn SymmetricMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 6; 5|0; 4; 3|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = copy([2; 1; 1|0; 2; 1|0; 0; 1]; symmetric(3); 1; 1)",
        "D3 = D1 - D2",
    ]);
    assert_equal_str("[5 5 4|5 2 2|4 2 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric(3); 1; 1)",
        "D3 = D1 ^ D2",
    ]);
    assert_equal_str("[1 4 9|4 16 25|9 25 36]", &calc.ToString());
}
#[test]
fn SymmetricMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 6; 4|0; 3; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric(3); 1; 1)",
        "D3 = D1 / D2",
    ]);
    assert_equal_str("[4 3 2|3 1.5 1|2 1 0.5]", &calc.ToString());
}
#[test]
fn SymmetricMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric(3); 1; 1)",
        "D2 = copy([3; 3; 3|0; 3; 3|0; 0; 3]; symmetric(3); 1; 1)",
        "D3 = D1 * D2",
    ]);
    assert_equal_str("[36 36 36|51 51 51|60 60 60]", &calc.ToString());
}
#[test]
fn SymmetricMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 7; 6|0; 5; 4|0; 0; 3]; symmetric(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric(3); 1; 1)",
        "D3 = D1 \\ D2",
    ]);
    assert_equal_str("[4 3 3|3 2 2|3 2 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 7; 5|0; 6; 4|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; symmetric(3); 1; 1)",
        "D3 = D1 ÷ D2",
    ]);
    assert_equal_str("[4.5 3.5 2.5|3.5 3 2|2.5 2 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric(3); 1; 1)",
        "D2 = copy([4; 4; 4|0; 4; 4|0; 0; 4]; symmetric(3); 1; 1)",
        "D3 = D1 ⦼ D2",
    ]);
    assert_equal_str("[3 0 1|0 1 2|1 2 3]", &calc.ToString());
}
#[test]
fn SymmetricMatrixFactorial() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric(3); 1; 1)",
        "D2 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric(3); 1; 1)",
        "D3 = D1!",
    ]);
    assert_equal_str("[6 24 120|24 720 5040|120 5040 40320]", &calc.ToString());
}
#[test]
fn SymmetricMatrixEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = copy([3; 3; 3|0; 3; 3|0; 0; 3]; symmetric(3); 1; 1)",
        "D3 = D1 ≡ D2",
    ]);
    assert_equal_str("[0 0 1|0 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn SymmetricMatrixNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = copy([4; 4; 4|0; 4; 4|0; 0; 4]; symmetric(3); 1; 1)",
        "D3 = D1 ≠ D2",
    ]);
    assert_equal_str("[1 0 1|0 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 6; 7|0; 4; 3|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = copy([5; 5; 5|0; 5; 5|0; 0; 5]; symmetric(3); 1; 1)",
        "D3 = D1 < D2",
    ]);
    assert_equal_str("[0 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 3; 2|0; 6; 5|0; 0; 7]; symmetric(3); 1; 1)",
        "D2 = copy([3; 3; 3|0; 3; 3|0; 0; 3]; symmetric(3); 1; 1)",
        "D3 = D1 > D2",
    ]);
    assert_equal_str("[1 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 6; 7|0; 3; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = copy([6; 6; 6|0; 6; 6|0; 0; 6]; symmetric(3); 1; 1)",
        "D3 = D1 ≤ D2",
    ]);
    assert_equal_str("[1 1 0|1 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 5; 7|0; 6; 4|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = copy([4; 4; 4|0; 4; 4|0; 0; 4]; symmetric(3); 1; 1)",
        "D3 = D1 ≥ D2",
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn SymmetricMatrixLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric(3); 1; 1)",
        "D2 = copy([1; 1; 1|0; 1; 1|0; 0; 1]; symmetric(3); 1; 1)",
        "D3 = D1 ∧ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 0; 0|0; 1; 0|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = copy([0; 1; 1|0; 1; 1|0; 0; 1]; symmetric(3); 1; 1)",
        "D3 = D1 ∨ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 1; 0|0; 1; 0|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = copy([1; 0; 1|0; 1; 1|0; 0; 1]; symmetric(3); 1; 1)",
        "D3 = D1 ⊕ D2",
    ]);
    assert_equal_str("[0 1 1|1 0 1|1 1 0]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 + D2",
    ]);
    assert_equal_str("[7 8 9|8 10 11|9 11 12]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 6; 5|0; 4; 3|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 - D2",
    ]);
    assert_equal_str("[4 3 2|3 1 0|2 0 -1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ^ D2",
    ]);
    assert_equal_str("[1 4 9|4 16 25|9 25 36]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 6; 4|0; 3; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 / D2",
    ]);
    assert_equal_str("[4 3 2|3 1.5 1|2 1 0.5]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 * D2",
    ]);
    assert_equal_str("[9 12 15|12 18 21|15 21 24]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 7; 6|0; 5; 4|0; 0; 3]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 \\ D2",
    ]);
    assert_equal_str("[4 3 3|3 2 2|3 2 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 7; 5|0; 6; 4|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ÷ D2",
    ]);
    assert_equal_str("[4.5 3.5 2.5|3.5 3 2|2.5 2 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ⦼ D2",
    ]);
    assert_equal_str("[3 0 1|0 1 2|1 2 3]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 ≡ D2",
    ]);
    assert_equal_str("[0 0 1|0 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 4; 3|0; 5; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ≠ D2",
    ]);
    assert_equal_str("[1 0 1|0 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 6; 7|0; 4; 3|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 < D2",
    ]);
    assert_equal_str("[0 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 3; 2|0; 6; 5|0; 0; 7]; symmetric(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 > D2",
    ]);
    assert_equal_str("[1 0 0|0 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 6; 7|0; 3; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = 6",
        "D3 = D1 ≤ D2",
    ]);
    assert_equal_str("[1 1 0|1 1 1|0 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 5; 7|0; 6; 4|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ≥ D2",
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 0]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∧ D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∨ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn SymmetricMatrixScalarLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric(3); 1; 1)",
        "D2 = 1",
        "D3 = D1 ⊕ D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric(3); 1; 1)",
        "D2 = 5",
        "D3 = D2 + D1",
    ]);
    assert_equal_str("[7 8 9|8 10 11|9 11 12]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 6; 5|0; 4; 3|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 - D1",
    ]);
    assert_equal_str("[-4 -3 -2|-3 -1 0|-2 0 1]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 ^ D1",
    ]);
    assert_equal_str("[2 4 8|4 16 32|8 32 64]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 6; 4|0; 3; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 / D1",
    ]);
    assert_equal_str("[0.25 0.333 0.5|0.333 0.667 1|0.5 1 2]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 4; 5|0; 6; 7|0; 0; 8]; symmetric(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 * D1",
    ]);
    assert_equal_str("[9 12 15|12 18 21|15 21 24]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 7; 6|0; 5; 4|0; 0; 3]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 \\ D1",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 7; 5|0; 6; 4|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 ÷ D1",
    ]);
    assert_equal_str("[0.222 0.286 0.4|0.286 0.333 0.5|0.4 0.5 1]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric(3); 1; 1)",
        "D2 = 4",
        "D3 = D2 ⦼ D1",
    ]);
    assert_equal_str("[4 4 4|4 4 4|4 4 1]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 6; 7|0; 4; 3|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 5",
        "D3 = D2 < D1",
    ]);
    assert_equal_str("[0 1 1|1 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 3; 2|0; 6; 5|0; 0; 7]; symmetric(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 > D1",
    ]);
    assert_equal_str("[0 0 1|0 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 6; 7|0; 3; 2|0; 0; 1]; symmetric(3); 1; 1)",
        "D2 = 6",
        "D3 = D2 ≤ D1",
    ]);
    assert_equal_str("[0 1 1|1 0 0|1 0 0]", &calc.ToString());
}
#[test]
fn ScalarSymmetricMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 5; 7|0; 6; 4|0; 0; 2]; symmetric(3); 1; 1)",
        "D2 = 4",
        "D3 = D2 ≥ D1",
    ]);
    assert_equal_str("[1 0 0|0 0 1|0 1 1]", &calc.ToString());
}
