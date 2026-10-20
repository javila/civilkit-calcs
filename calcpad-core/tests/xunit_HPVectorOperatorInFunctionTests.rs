#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Vectors/HP/HPVectorOperatorInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPVectorAddition() {
    let mut vector1 = "hp([1; 2; 3; -1; 0; 4])";
    let mut vector2 = "hp([4; 3; 3; -1; 2; 2])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", vector1, vector2)]);
}
#[test]
fn HPVectorSubtraction() {
    let mut vector1 = "hp([2; 5; 7; 3; 6; 8])";
    let mut vector2 = "hp([1; 4; 3; -2; 5; 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 1 4 5 1 7]", &calc.ToString());
}
#[test]
fn HPVectorExponentiation() {
    let mut vector = "hp([2; 3; -1; 4; 0; 6])";
    let mut exp_vector = "hp([2; 1; 3; 2; 0; 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", vector, exp_vector)]);
    assert_equal_str("[4 3 -1 16 1 6]", &calc.ToString());
}
#[test]
fn HPVectorDivision() {
    let mut vector1 = "hp([8; 2; -6; 4; 10; 8])";
    let mut vector2 = "hp([2; 1; 3; 10; 3; 4])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{}/{}", vector1, vector2));
    assert_equal_str("[4 2 -2 0.4 3.33 2]", &calc.ToString());
}
#[test]
fn HPVectorMultiplication() {
    let mut vector1 = "hp([2; -1; 3; 4; 0; -5])";
    let mut vector2 = "hp([7; 1; 9; 22; 1231; -90])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[14 -1 27 88 0 450]", &calc.ToString());
}
#[test]
fn HPVectorIntegerDivision() {
    let mut vector1 = "hp([9; 3; 1; 7; 5])";
    let mut vector2 = "hp([2; 1; 2; 4; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[4 3 0 1 1]", &calc.ToString());
}
#[test]
fn HPVectorDivisionBar() {
    let mut vector1 = "hp([10; 5; 2; 8; 6])";
    let mut vector2 = "hp([3; 4; 1; 2; 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[3.33 1.25 2 4 2]", &calc.ToString());
}
#[test]
fn HPVectorRemainder() {
    let mut vector1 = "hp([-4.3; 0; 6.9; -2.8; 3])";
    let mut vector2 = "hp([8.2; -5; 0; 2.7; -1.4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[-4.3 0  Undefined  -0.1 0.2]", &calc.ToString());
}
#[test]
fn HPVectorEqual() {
    let mut vector1 = "hp([8.2; 0; 6.9; -2.8; 3])";
    let mut vector2 = "hp([8.2; -5; 6.9; 2.7; 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", vector1, vector2)]);
    let mut actualResult = calc.ToString();
    let mut expectedResult = "[1 0 1 0 1]";
    assert_equal(expectedResult.clone(), actualResult.clone());
}
#[test]
fn HPVectorNotEqual() {
    let mut vector1 = "hp([2; 0; 6.9; -2.8; 3])";
    let mut vector2 = "hp([2; 1; 6.9; -2.8; 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[0 1 0 0 0]", &calc.ToString());
}
#[test]
fn HPVectorLessThan() {
    let mut vector1 = "hp([0; -6.7; 3.8; -2; 9.5])";
    let mut vector2 = "hp([4; 7.2; 0; -1.3; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 1 0 1 0]", &calc.ToString());
}
#[test]
fn HPVectorFactorial() {
    let mut vector = "hp([4; 2; 6; 2; 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = a!", &format!("f({})", vector)]);
    let mut expected = "[24 2 720 2 6]";
    assert_equal(expected.clone(), calc.ToString());
}
#[test]
fn HPVectorGreaterThan() {
    let mut vector1 = "hp([4.7; -8; 0; 2.3; -6.5])";
    let mut vector2 = "hp([-3; 6.2; 0; -5.1; 7.4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 0 0 1 0]", &calc.ToString());
}
#[test]
fn HPVectorLessOrEqual() {
    let mut vector1 = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut vector2 = "hp([-7; 1.3; 0; 4.5; -9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[0 1 1 0 0]", &calc.ToString());
}
#[test]
fn HPVectorGreaterOrEqual() {
    let mut vector1 = "hp([20; 16.7; 0; 22.3; 13.5])";
    let mut vector2 = "hp([-18; 21.2; 0; -19.3; 25])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 0 1 1 0]", &calc.ToString());
}
#[test]
fn HPVectorLogicalAnd() {
    let mut vector1 = "hp([0; 16.7; 0; 22.3; 13.5])";
    let mut vector2 = "hp([-18; 21.2; 0; -19.3; 25])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[0 1 0 1 1]", &calc.ToString());
}
#[test]
fn HPVectorLogicalOr() {
    let mut vector1 = "hp([30; 25.8; 0; 28.1; 31.9])";
    let mut vector2 = "hp([31.9; 0; 37.8; 27.6])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPVectorLogicalXor() {
    let mut vector1 = "hp([6; -2.5; 0; 4.3; -8.1])";
    let mut vector2 = "hp([0; 8.6; 0; -5.2; 1.9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 0 0 0 0]", &calc.ToString());
}
#[test]
fn HPVectorScalarAddition() {
    let mut vector = "hp([5; 12.3; -44; 13; 21.4])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[12 19.3 -37 20 28.4]", &calc.ToString());
}
#[test]
fn HPVectorScalarSubtraction() {
    let mut vector = "hp([-3; 0; 4.5; -7; 2.25])";
    let mut x = "-1.5";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[-1.5 1.5 6 -5.5 3.75]", &calc.ToString());
}
#[test]
fn HPVectorScalarExponentiation() {
    let mut vector = "hp([-4.25; 0; 7.89; -10; 3.14159])";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[-76.77 0 491.17 -1000 31.01]", &calc.ToString());
}
#[test]
fn HPVectorScalarDivision() {
    let mut vector = "hp([0.1; -0.2; 0.3; -0.4; 0.5])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[0.05 -0.1 0.15 -0.2 0.25]", &calc.ToString());
}
#[test]
fn HPVectorScalarMultiplication() {
    let mut vector = "hp([0.1; -0.2; 0.3; -0.4; 0.5])";
    let mut x = "3.5";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[0.35 -0.7 1.05 -1.4 1.75]", &calc.ToString());
}
#[test]
fn HPVectorScalarIntegerDivision() {
    let mut vector = "hp([-0.5; 3; 0; -7.8; 5.6; 4.2; -6; 1.8; -3.1])";
    let mut x = "2.5";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", vector, x));
    assert_equal_str("[0 1 0 -3 2 1 -2 0 -1]", &calc.ToString());
}
#[test]
fn HPVectorScalarDivisionBar() {
    let mut vector = "hp([0.1; -0.2; 0.3; -0.4; 0.5; -5; 8.4; 0])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", vector, x));
    assert_equal_str("[0.025 -0.05 0.075 -0.1 0.125 -1.25 2.1 0]", &calc.ToString());
}
#[test]
fn HPVectorScalarRemainder() {
    let mut vector = "hp([-4.3; 0; 6.9; -2.8; 3; -3; 6.5; 0])";
    let mut x = "2.7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", vector, x));
    assert_equal_str("[-1.6 0 1.5 -0.1 0.3 -0.3 1.1 0]", &calc.ToString());
}
#[test]
fn HPVectorScalarEqual() {
    let mut vector = "hp([8.2; 0; 6.9; -2.8; 3; 8.2; -7; 4.6])";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≡ {}", vector, x));
    let mut actualResult = calc.ToString();
    let mut expectedResult = "[0 0 0 0 1 0 0 0]";
    assert_equal(expectedResult.clone(), actualResult.clone());
}
#[test]
fn HPVectorScalarNotEqual() {
    let mut vector = "hp([-4.3; 0; 6.9; -2.8; 3; -5; 0; 2.7; -1.4])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≠ {}", vector, x));
    assert_equal_str("[1 1 1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPVectorScalarLessThan() {
    let mut vector = "hp([0; -6.7; 3.8; -2; 9.5; 7.2; 0; -1.3; 5])";
    let mut x = "8";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", vector, x));
    assert_equal_str("[1 1 1 1 0 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPVectorScalarGreaterThan() {
    let mut vector = "hp([4.7; -8; 0; 2.3; -6.5; -3; 6.2; 0; -5.1; 7.4])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", vector, x));
    assert_equal_str("[1 0 0 1 0 0 1 0 0 1]", &calc.ToString());
}
#[test]
fn HPVectorScalarLessOrEqual() {
    let mut vector = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut x = "6";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", vector, x));
    assert_equal_str("[1 1 1 0 1]", &calc.ToString());
}
#[test]
fn HPVectorScalarGreaterOrEqual() {
    let mut vector = "hp([-40; 16.7; 0; 22.3; 13.5; 21.2; 0; -19.3; 25])";
    let mut x = "10";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", vector, x));
    assert_equal_str("[0 1 0 1 1 1 0 0 1]", &calc.ToString());
}
#[test]
fn HPVectorScalarLogicalAnd() {
    let mut vector = "hp([0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25])";
    let mut x = "5";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", vector, x));
    assert_equal_str("[0 1 0 1 1 1 1 0 1 1]", &calc.ToString());
}
#[test]
fn HPVectorScalarLogicalOr() {
    let mut vector = "hp([0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25])";
    let mut x = "-10";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", vector, x));
    assert_equal_str("[1 1 1 1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPVectorScalarLogicalXor() {
    let mut vector = "hp([6; -2.5; 0; 4.3; -8.1; 8.6; 0; -5.2; 1.9])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", vector, x));
    assert_equal_str("[0 0 1 0 0 0 1 0 0]", &calc.ToString());
}
#[test]
fn HPScalarVectorAddition() {
    let mut x = "7";
    let mut vector = "hp([5; 12.3; -44; 13; 21.4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[12 19.3 -37 20 28.4]", &calc.ToString());
}
#[test]
fn HPScalarVectorSubtraction() {
    let mut x = "5.5";
    let mut vector = "hp([-3; 0; 4.5; -7; 2.25])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[8.5 5.5 1 12.5 3.25]", &calc.ToString());
}
#[test]
fn HPScalarVectorExponentiation() {
    let mut x = "3";
    let mut vector = "hp([-4.25; 0; 7.89; 2; 3.14159])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0.00938 1 5814.16 9 31.54]", &calc.ToString());
}
#[test]
fn HPScalarVectorDivision() {
    let mut x = "5";
    let mut vector = "hp([0.1; -0.2; 0.3; -0.4; 0.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[50 -25 16.67 -12.5 10]", &calc.ToString());
}
#[test]
fn HPScalarVectorMultiplication() {
    let mut x = "4.5";
    let mut vector = "hp([0.1; -0.2; 0.3; -0.4; 0.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0.45 -0.9 1.35 -1.8 2.25]", &calc.ToString());
}
#[test]
fn HPScalarVectorIntegerDivision() {
    let mut x = "8.25";
    let mut vector = "hp([-0.5; 3; 0; -7.8; 5.6; 4.2; -6; 1.8; -3.1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[-16 2  Undefined  -1 1 1 -1 4 -2]", &calc.ToString());
}
#[test]
fn HPScalarVectorDivisionBar() {
    let mut x = "5";
    let mut vector = "hp([0.1; -0.2; 0.3; -0.4; 0.5; -5; 8.4; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[50 -25 16.67 -12.5 10 -1 0.595 +∞]", &calc.ToString());
}
#[test]
fn HPScalarVectorRemainder() {
    let mut x = "3.2";
    let mut vector = "hp([-4.3; 0; 6.9; -2.8; 3; -3; 6.5; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[3.2  Undefined  3.2 0.4 0.2 0.2 3.2  Undefined ]", &calc.ToString());
}
#[test]
fn HPScalarVectorEqual() {
    let mut x = "5.5";
    let mut vector = "hp([5.5; 2.1; 7.9; -1.5; 5.5; 6.7; -5; 5.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", x, vector)]);
    let mut actualResult = calc.ToString();
    let mut expectedResult = "[1 0 0 0 1 0 0 1]";
    assert_equal(expectedResult.clone(), actualResult.clone());
}
#[test]
fn HPScalarVectorNotEqual() {
    let mut x = "3";
    let mut vector = "hp([-4.3; 0; 6.9; -2.8; 3; -5; 0; 2.7; -1.4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 1 1 1 0 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPScalarVectorLessThan() {
    let mut x = "6";
    let mut vector = "hp([0; -6.7; 3.8; -2; 9.5; 7.2; 0; -1.3; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 0 0 0 1 1 0 0 0]", &calc.ToString());
}
#[test]
fn HPScalarVectorGreaterThan() {
    let mut x = "-1";
    let mut vector = "hp([4.7; -8; 0; 2.3; -6.5; -3; 6.2; 0; -5.1; 7.4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 1 0 0 1 1 0 0 1 0]", &calc.ToString());
}
#[test]
fn HPScalarVectorLessOrEqual() {
    let mut x = "3";
    let mut vector = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 0 0 1 0]", &calc.ToString());
}
#[test]
fn HPScalarVectorGreaterOrEqual() {
    let mut x = "0";
    let mut vector = "hp([-40; 16.7; 0; 22.3; 13.5; 21.2; 0; -19.3; 25])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 0 1 0 0 0 1 1 0]", &calc.ToString());
}
#[test]
fn HPScalarVectorLogicalAnd() {
    let mut x = "5";
    let mut vector = "hp([0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 1 0 1 1 1 1 0 1 1]", &calc.ToString());
}
#[test]
fn HPScalarVectorLogicalOr() {
    let mut x = "5";
    let mut vector = "hp([0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 1 1 1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPScalarVectorLogicalXor() {
    let mut x = "5";
    let mut vector = "hp([6; -2.5; 0; 4.3; -8.1; 8.6; 0; -5.2; 1.9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 0 1 0 0 0 1 0 0]", &calc.ToString());
}
