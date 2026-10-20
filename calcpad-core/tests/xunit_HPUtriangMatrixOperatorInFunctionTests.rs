#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Small/HPUtriangMatrixOperatorInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPUtriangMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([4; 7; 5|0; 1; 1|0; 0; 2]; utriang_hp(3); 1; 1)",
        "f(a; b) = a + b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[6 10 9|0 6 7|0 0 9]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; utriang_hp(3); 1; 1)",
        "f(a; b) = a - b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[-1 -1 -1|0 -1 -1|0 0 -1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([2; 2; 2|0; 2; 2|0; 0; 2]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ^ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 8 16|1 32 64|1 1 128]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; utriang_hp(3); 1; 1)",
        "f(a; b) = a / b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str(
        "[0.5 0.667 0.75| Undefined  0.8 0.833| Undefined   Undefined  0.857]",
        &calc.ToString(),
    );
}
#[test]
fn HPUtriangMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([2; 4; 6|0; 1; 3|0; 0; 5]; utriang_hp(3); 1; 1)",
        "f(a; b) = a * b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 26 74|0 5 27|0 0 35]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 2; 1|0; 3; 2|0; 0; 3]; utriang_hp(3); 1; 1)",
        "f(a; b) = a \\ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0| Undefined  0 0| Undefined   Undefined  0]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([4; 8; 12|0; 5; 10|0; 0; 14]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ÷ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str(
        "[2 2.67 3| Undefined  1 1.67| Undefined   Undefined  2]",
        &calc.ToString(),
    );
}
#[test]
fn HPUtriangMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([4; 7; 5|0; 3; 6|0; 0; 8]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ⦼ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 1 1| Undefined  3 0| Undefined   Undefined  1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixFactorial() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "f(a) = a!",
        &format!("f(D1)"),
    ]);
    assert_equal_str("[2 6 24|1 120 720|1 1 5040]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ≡ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 4; 5|0; 0; 6]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ≠ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([6; 5; 4|0; 4; 3|0; 0; 2]; utriang_hp(3); 1; 1)",
        "f(a; b) = a < b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 6; 5|0; 0; 7]; utriang_hp(3); 1; 1)",
        "f(a; b) = a > b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|0 1 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([5; 5; 5|0; 5; 5|0; 0; 5]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ≤ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ≥ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 0; 1|0; 0; 1]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ∧ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|0 0 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 2; 3|0; 0; 1|0; 0; 0]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ∨ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = copy([1; 0; 1|0; 1; 0|0; 0; 1]; utriang_hp(3); 1; 1)",
        "f(a; b) = a ⊕ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 1 0|0 0 1|0 0 0]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 5",
        "f(a; b) = a + b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[7 8 9|5 10 11|5 5 12]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 3",
        "f(a; b) = a - b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[-1 0 1|-3 2 3|-3 -3 4]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ^ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[4 9 16|0 25 36|0 0 49]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 2",
        "f(a; b) = a / b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1.5 2|0 2.5 3|0 0 3.5]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 3",
        "f(a; b) = a * b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[6 9 12|0 15 18|0 0 21]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 4",
        "f(a; b) = a \\ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ÷ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1.5 2|0 2.5 3|0 0 3.5]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 4",
        "f(a; b) = a ⦼ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[2 3 0|0 1 2|0 0 3]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≡ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarNotEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ≠ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 5",
        "f(a; b) = a < b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 1",
        "f(a; b) = a > b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ≤ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 0|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 4",
        "f(a; b) = a ≥ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarLogicalAnd() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 0",
        "f(a; b) = a ∧ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarLogicalOr() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 0",
        "f(a; b) = a ∨ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[1 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixScalarLogicalXor() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 1",
        "f(a; b) = a ⊕ b",
        &format!("f(D1;D2)"),
    ]);
    assert_equal_str("[0 0 0|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 5",
        "f(a; b) = a + b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[7 8 9|5 10 11|5 5 12]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 3",
        "f(a; b) = a - b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 0 -1|3 -2 -3|3 3 -4]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixExponentiation() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 2",
        "f(a; b) = a ^ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 8 16|1 32 64|1 1 128]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 12",
        "f(a; b) = a / b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[6 4 3|+∞ 2.4 2|+∞ +∞ 1.71]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 3",
        "f(a; b) = a * b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[6 9 12|0 15 18|0 0 21]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 8",
        "f(a; b) = a \\ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[4 2 2| Undefined  1 1| Undefined   Undefined  1]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixDivisionBar() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 10",
        "f(a; b) = a ÷ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[5 3.33 2.5|+∞ 2 1.67|+∞ +∞ 1.43]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixRemainder() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 9",
        "f(a; b) = a ⦼ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 0 1| Undefined  4 3| Undefined   Undefined  2]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixLessThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 5",
        "f(a; b) = a < b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 0 0|0 0 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixGreaterThan() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 4",
        "f(a; b) = a > b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 0|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixLessOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 3",
        "f(a; b) = a ≤ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[0 1 1|0 1 1|0 0 1]", &calc.ToString());
}
#[test]
fn HPScalarUtriangMatrixGreaterOrEqual() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "D1 = copy([2; 3; 4|0; 5; 6|0; 0; 7]; utriang_hp(3); 1; 1)",
        "D2 = 5",
        "f(a; b) = a ≥ b",
        &format!("f(D2;D1)"),
    ]);
    assert_equal_str("[1 1 1|1 1 0|1 1 0]", &calc.ToString());
}
