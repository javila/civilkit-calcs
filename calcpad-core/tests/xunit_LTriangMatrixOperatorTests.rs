#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/LTriangMatrixOperatorTests.cs by tools/transcribe_xunit.py.
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
fn LtriangMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 0; 0|2; 3; 0|4; 5; 6]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 + D2",
    ]);
    assert_equal_str("[5 0 0|7 9 0|11 12 11]", &calc.ToString());
}
#[test]
fn LtriangMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 0; 0|7; 8; 0|9; 10; 11]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 - D2",
    ]);
    assert_equal_str("[1 0 0|2 2 0|2 3 6]", &calc.ToString());
}
#[test]
fn LtriangMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|3; 4; 0|5; 6; 7]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ^ D2",
    ]);
    assert_equal_str("[16 1 1|243 4096 1|78125 279936 16807]", &calc.ToString());
}
#[test]
fn LtriangMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 0; 0|10; 12; 0|14; 16; 18]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 / D2",
    ]);
    assert_equal_str(
        "[2  Undefined   Undefined |2 2  Undefined |2 2.29 3.6]",
        &calc.ToString(),
    );
}
#[test]
fn LtriangMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 * D2",
    ]);
    assert_equal_str("[12 0 0|41 30 0|115 98 40]", &calc.ToString());
}
#[test]
fn LtriangMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 0; 0|7; 9; 0|11; 13; 15]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 \\ D2",
    ]);
    assert_equal_str("[1  Undefined   Undefined |1 1  Undefined |1 1 3]", &calc.ToString());
}
#[test]
fn LtriangMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([10; 0; 0|12; 14; 0|16; 18; 20]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ÷ D2",
    ]);
    assert_equal_str(
        "[2.5  Undefined   Undefined |2.4 2.33  Undefined |2.29 2.57 4]",
        &calc.ToString(),
    );
}
#[test]
fn LtriangMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 0; 0|10; 12; 0|14; 16; 18]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ⦼ D2",
    ]);
    assert_equal_str("[1  Undefined   Undefined |0 0  Undefined |0 2 3]", &calc.ToString());
}
#[test]
fn LtriangMatrixFactorial() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["D1 = copy([2; 0; 0|3; 4; 0|5; 6; 7]; ltriang(3); 1; 1)", "D2 = D1!"]);
    assert_equal_str("[2 1 1|6 24 1|120 720 5040]", &calc.ToString());
}
#[test]
fn LtriangMatrixEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ≡ D2",
    ]);
    assert_equal_str("[0 1 1|0 0 1|0 1 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ≠ D2",
    ]);
    assert_equal_str("[1 0 0|1 1 0|1 0 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 < D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 > D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ≤ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ≥ D2",
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ∧ D2",
    ]);
    assert_equal_str("[1 0 0|1 1 0|1 1 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ∨ D2",
    ]);
    assert_equal_str("[1 0 0|1 1 0|1 1 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = copy([4; 0; 0|5; 6; 0|7; 7; 5]; ltriang(3); 1; 1)",
        "D3 = D1 ⊕ D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 0; 0|8; 9; 0|10; 11; 12]; ltriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 + D2",
    ]);
    assert_equal_str("[12 5 5|13 14 5|15 16 17]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 0; 0|6; 7; 0|8; 9; 10]; ltriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 - D2",
    ]);
    assert_equal_str("[1 -3 -3|3 4 -3|5 6 7]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|3; 4; 0|5; 6; 7]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ^ D2",
    ]);
    assert_equal_str("[4 0 0|9 16 0|25 36 49]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 0; 0|6; 7; 0|10; 12; 14]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 / D2",
    ]);
    assert_equal_str("[4 0 0|3 3.5 0|5 6 7]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|2; 5; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 * D2",
    ]);
    assert_equal_str("[9 0 0|6 15 0|21 24 27]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 0; 0|8; 9; 0|12; 14; 16]; ltriang(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 \\ D2",
    ]);
    assert_equal_str("[1 0 0|2 2 0|3 3 4]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([7; 0; 0|9; 10; 0|11; 13; 15]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D1 ÷ D2",
    ]);
    assert_equal_str("[3.5 0 0|4.5 5 0|5.5 6.5 7.5]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([11; 0; 0|13; 14; 0|15; 17; 19]; ltriang(3); 1; 1)",
        "D2 = 4",
        "D3 = D1 ⦼ D2",
    ]);
    assert_equal_str("[3 0 0|1 2 0|3 1 3]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 ≡ D2",
    ]);
    assert_equal_str("[0 0 0|0 1 0|0 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = 5",
        "D3 = D1 ≠ D2",
    ]);
    assert_equal_str("[1 1 1|1 0 1|1 1 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = 6",
        "D3 = D1 < D2",
    ]);
    assert_equal_str("[1 1 1|1 0 1|0 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D1 > D2",
    ]);
    assert_equal_str("[0 0 0|1 1 0|1 1 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = 7",
        "D3 = D1 ≤ D2",
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 0; 0|6; 7; 0|8; 9; 10]; ltriang(3); 1; 1)",
        "D2 = 6",
        "D3 = D1 ≥ D2",
    ]);
    assert_equal_str("[0 0 0|1 1 0|1 1 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∧ D2",
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|3; 4; 0|5; 6; 7]; ltriang(3); 1; 1)",
        "D2 = 0",
        "D3 = D1 ∨ D2",
    ]);
    assert_equal_str("[1 0 0|1 1 0|1 1 1]", &calc.ToString());
}
#[test]
fn LtriangMatrixScalarLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|3; 4; 0|5; 6; 7]; ltriang(3); 1; 1)",
        "D2 = 1",
        "D3 = D1 ⊕ D2",
    ]);
    assert_equal_str("[0 1 1|0 0 1|0 0 0]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([1; 0; 0|2; 3; 0|4; 5; 6]; ltriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 + D1",
    ]);
    assert_equal_str("[4 3 3|5 6 3|7 8 9]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 0; 0|7; 8; 0|9; 10; 11]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 - D1",
    ]);
    assert_equal_str("[-3 2 2|-5 -6 2|-7 -8 -9]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|3; 4; 0|5; 6; 7]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 ^ D1",
    ]);
    assert_equal_str("[4 1 1|8 16 1|32 64 128]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([8; 0; 0|10; 12; 0|14; 16; 18]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 / D1",
    ]);
    assert_equal_str("[0.25 +∞ +∞|0.2 0.167 +∞|0.143 0.125 0.111]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 * D1",
    ]);
    assert_equal_str("[6 0 0|8 10 0|12 14 16]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([5; 0; 0|7; 9; 0|11; 13; 15]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 \\ D1",
    ]);
    assert_equal_str("[0  Undefined   Undefined |0 0  Undefined |0 0 0]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([10; 0; 0|12; 14; 0|16; 18; 20]; ltriang(3); 1; 1)",
        "D2 = 2",
        "D3 = D2 ÷ D1",
    ]);
    assert_equal_str("[0.2 +∞ +∞|0.167 0.143 +∞|0.125 0.111 0.1]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([9; 0; 0|10; 12; 0|14; 16; 18]; ltriang(3); 1; 1)",
        "D2 = 4",
        "D3 = D2 ⦼ D1",
    ]);
    assert_equal_str("[4  Undefined   Undefined |4 4  Undefined |4 4 4]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([4; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = 6",
        "D3 = D2 < D1",
    ]);
    assert_equal_str("[0 0 0|0 0 0|1 1 1]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 0; 0|4; 5; 0|6; 7; 8]; ltriang(3); 1; 1)",
        "D2 = 3",
        "D3 = D2 > D1",
    ]);
    assert_equal_str("[1 1 1|0 0 1|0 0 0]", &calc.ToString());
}
#[test]
fn ScalarLtriangMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([3; 0; 0|5; 6; 0|7; 8; 9]; ltriang(3); 1; 1)",
        "D2 = 7",
        "D3 = D2 ≤ D1",
    ]);
    assert_equal_str("[0 0 0|0 0 0|1 1 1]", &calc.ToString());
}
