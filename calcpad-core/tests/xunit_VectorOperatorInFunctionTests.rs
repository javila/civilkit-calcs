#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Vectors/VectorOperatorInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn VectorAddition() {
    let mut vector1 = "[1; 2; 3; -1; 0; 4]";
    let mut vector2 = "[4; 3; 3; -1; 2; 2]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", vector1, vector2)]);
}
#[test]
fn VectorSubtraction() {
    let mut vector1 = "[2; 5; 7; 3; 6; 8]";
    let mut vector2 = "[1; 4; 3; -2; 5; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 1 4 5 1 7]", &calc.ToString());
}
#[test]
fn VectorExponentiation() {
    let mut vector = "[2; 3; -1; 4; 0; 6]";
    let mut exp_vector = "[2; 1; 3; 2; 0; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", vector, exp_vector)]);
    assert_equal_str("[4 3 -1 16 1 6]", &calc.ToString());
}
#[test]
fn VectorDivision() {
    let mut vector1 = "[8; 2; -6; 4; 10; 8]";
    let mut vector2 = "[2; 1; 3; 10; 3; 4]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{}/{}", vector1, vector2));
    assert_equal_str("[4 2 -2 0.4 3.33 2]", &calc.ToString());
}
#[test]
fn VectorMultiplication() {
    let mut vector1 = "[2; -1; 3; 4; 0; -5]";
    let mut vector2 = "[7; 1; 9; 22; 1231; -90]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[14 -1 27 88 0 450]", &calc.ToString());
}
#[test]
fn VectorIntegerDivision() {
    let mut vector1 = "[9; 3; 1; 7; 5]";
    let mut vector2 = "[2; 1; 2; 4; 5]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[4 3 0 1 1]", &calc.ToString());
}
#[test]
fn VectorDivisionBar() {
    let mut vector1 = "[10; 5; 2; 8; 6]";
    let mut vector2 = "[3; 4; 1; 2; 3]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[3.33 1.25 2 4 2]", &calc.ToString());
}
#[test]
fn VectorRemainder() {
    let mut vector1 = "[-4.3; 0; 6.9; -2.8; 3]";
    let mut vector2 = "[8.2; -5; 0; 2.7; -1.4]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[-4.3 0  Undefined  -0.1 0.2]", &calc.ToString());
}
#[test]
fn VectorEqual() {
    let mut vector1 = "[8.2; 0; 6.9; -2.8; 3]";
    let mut vector2 = "[8.2; -5; 6.9; 2.7; 3]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", vector1, vector2)]);
    let mut actualResult = calc.ToString();
    let mut expectedResult = "[1 0 1 0 1]";
    assert_equal(expectedResult.clone(), actualResult.clone());
}
#[test]
fn VectorNotEqual() {
    let mut vector1 = "[2; 0; 6.9; -2.8; 3]";
    let mut vector2 = "[2; 1; 6.9; -2.8; 3]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[0 1 0 0 0]", &calc.ToString());
}
#[test]
fn VectorLessThan() {
    let mut vector1 = "[0; -6.7; 3.8; -2; 9.5]";
    let mut vector2 = "[4; 7.2; 0; -1.3; 5]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 1 0 1 0]", &calc.ToString());
}
#[test]
fn VectorFactorial() {
    let mut vector = "[4; 2; 6; 2; 3]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = a!", &format!("f({})", vector)]);
    let mut expected = "[24 2 720 2 6]";
    assert_equal(expected.clone(), calc.ToString());
}
#[test]
fn VectorGreaterThan() {
    let mut vector1 = "[4.7; -8; 0; 2.3; -6.5]";
    let mut vector2 = "[-3; 6.2; 0; -5.1; 7.4]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 0 0 1 0]", &calc.ToString());
}
#[test]
fn VectorLessOrEqual() {
    let mut vector1 = "[5; -2.1; 0; 7.8; -4.6]";
    let mut vector2 = "[-7; 1.3; 0; 4.5; -9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[0 1 1 0 0]", &calc.ToString());
}
#[test]
fn VectorGreaterOrEqual() {
    let mut vector1 = "[20; 16.7; 0; 22.3; 13.5]";
    let mut vector2 = "[-18; 21.2; 0; -19.3; 25]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 0 1 1 0]", &calc.ToString());
}
#[test]
fn VectorLogicalAnd() {
    let mut vector1 = "[0; 16.7; 0; 22.3; 13.5]";
    let mut vector2 = "[-18; 21.2; 0; -19.3; 25]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[0 1 0 1 1]", &calc.ToString());
}
#[test]
fn VectorLogicalOr() {
    let mut vector1 = "[30; 25.8; 0; 28.1; 31.9]";
    let mut vector2 = "[31.9; 0; 37.8; 27.6]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 1 1 1 1]", &calc.ToString());
}
#[test]
fn VectorLogicalXor() {
    let mut vector1 = "[6; -2.5; 0; 4.3; -8.1]";
    let mut vector2 = "[0; 8.6; 0; -5.2; 1.9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", vector1, vector2)]);
    assert_equal_str("[1 0 0 0 0]", &calc.ToString());
}
#[test]
fn VectorScalarAddition() {
    let mut vector = "[5; 12.3; -44; 13; 21.4]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[12 19.3 -37 20 28.4]", &calc.ToString());
}
#[test]
fn VectorScalarSubtraction() {
    let mut vector = "[-3; 0; 4.5; -7; 2.25]";
    let mut x = "-1.5";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[-1.5 1.5 6 -5.5 3.75]", &calc.ToString());
}
#[test]
fn VectorScalarExponentiation() {
    let mut vector = "[-4.25; 0; 7.89; -10; 3.14159]";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[-76.77 0 491.17 -1000 31.01]", &calc.ToString());
}
#[test]
fn VectorScalarDivision() {
    let mut vector = "[0.1; -0.2; 0.3; -0.4; 0.5]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[0.05 -0.1 0.15 -0.2 0.25]", &calc.ToString());
}
#[test]
fn VectorScalarMultiplication() {
    let mut vector = "[0.1; -0.2; 0.3; -0.4; 0.5]";
    let mut x = "3.5";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", vector, x)]);
    assert_equal_str("[0.35 -0.7 1.05 -1.4 1.75]", &calc.ToString());
}
#[test]
fn VectorScalarIntegerDivision() {
    let mut vector = "[-0.5; 3; 0; -7.8; 5.6; 4.2; -6; 1.8; -3.1]";
    let mut x = "2.5";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", vector, x));
    assert_equal_str("[0 1 0 -3 2 1 -2 0 -1]", &calc.ToString());
}
#[test]
fn VectorScalarDivisionBar() {
    let mut vector = "[0.1; -0.2; 0.3; -0.4; 0.5; -5; 8.4; 0]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", vector, x));
    assert_equal_str("[0.025 -0.05 0.075 -0.1 0.125 -1.25 2.1 0]", &calc.ToString());
}
#[test]
fn VectorScalarRemainder() {
    let mut vector = "[-4.3; 0; 6.9; -2.8; 3; -3; 6.5; 0]";
    let mut x = "2.7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", vector, x));
    assert_equal_str("[-1.6 0 1.5 -0.1 0.3 -0.3 1.1 0]", &calc.ToString());
}
#[test]
fn VectorScalarEqual() {
    let mut vector = "[8.2; 0; 6.9; -2.8; 3; 8.2; -7; 4.6]";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≡ {}", vector, x));
    let mut actualResult = calc.ToString();
    let mut expectedResult = "[0 0 0 0 1 0 0 0]";
    assert_equal(expectedResult.clone(), actualResult.clone());
}
#[test]
fn VectorScalarNotEqual() {
    let mut vector = "[-4.3; 0; 6.9; -2.8; 3; -5; 0; 2.7; -1.4]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≠ {}", vector, x));
    assert_equal_str("[1 1 1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn VectorScalarLessThan() {
    let mut vector = "[0; -6.7; 3.8; -2; 9.5; 7.2; 0; -1.3; 5]";
    let mut x = "8";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", vector, x));
    assert_equal_str("[1 1 1 1 0 1 1 1 1]", &calc.ToString());
}
#[test]
fn VectorScalarGreaterThan() {
    let mut vector = "[4.7; -8; 0; 2.3; -6.5; -3; 6.2; 0; -5.1; 7.4]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", vector, x));
    assert_equal_str("[1 0 0 1 0 0 1 0 0 1]", &calc.ToString());
}
#[test]
fn VectorScalarLessOrEqual() {
    let mut vector = "[5; -2.1; 0; 7.8; -4.6]";
    let mut x = "6";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", vector, x));
    assert_equal_str("[1 1 1 0 1]", &calc.ToString());
}
#[test]
fn VectorScalarGreaterOrEqual() {
    let mut vector = "[-40; 16.7; 0; 22.3; 13.5; 21.2; 0; -19.3; 25]";
    let mut x = "10";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", vector, x));
    assert_equal_str("[0 1 0 1 1 1 0 0 1]", &calc.ToString());
}
#[test]
fn VectorScalarLogicalAnd() {
    let mut vector = "[0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25]";
    let mut x = "5";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", vector, x));
    assert_equal_str("[0 1 0 1 1 1 1 0 1 1]", &calc.ToString());
}
#[test]
fn VectorScalarLogicalOr() {
    let mut vector = "[0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25]";
    let mut x = "-10";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", vector, x));
    assert_equal_str("[1 1 1 1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn VectorScalarLogicalXor() {
    let mut vector = "[6; -2.5; 0; 4.3; -8.1; 8.6; 0; -5.2; 1.9]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", vector, x));
    assert_equal_str("[0 0 1 0 0 0 1 0 0]", &calc.ToString());
}
#[test]
fn ScalarVectorAddition() {
    let mut x = "7";
    let mut vector = "[5; 12.3; -44; 13; 21.4]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[12 19.3 -37 20 28.4]", &calc.ToString());
}
#[test]
fn ScalarVectorSubtraction() {
    let mut x = "5.5";
    let mut vector = "[-3; 0; 4.5; -7; 2.25]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[8.5 5.5 1 12.5 3.25]", &calc.ToString());
}
#[test]
fn ScalarVectorExponentiation() {
    let mut x = "3";
    let mut vector = "[-4.25; 0; 7.89; 2; 3.14159]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0.00938 1 5814.16 9 31.54]", &calc.ToString());
}
#[test]
fn ScalarVectorDivision() {
    let mut x = "5";
    let mut vector = "[0.1; -0.2; 0.3; -0.4; 0.5]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[50 -25 16.67 -12.5 10]", &calc.ToString());
}
#[test]
fn ScalarVectorMultiplication() {
    let mut x = "4.5";
    let mut vector = "[0.1; -0.2; 0.3; -0.4; 0.5]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0.45 -0.9 1.35 -1.8 2.25]", &calc.ToString());
}
#[test]
fn ScalarVectorIntegerDivision() {
    let mut x = "8.25";
    let mut vector = "[-0.5; 3; 0; -7.8; 5.6; 4.2; -6; 1.8; -3.1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[-16 2  Undefined  -1 1 1 -1 4 -2]", &calc.ToString());
}
#[test]
fn ScalarVectorDivisionBar() {
    let mut x = "5";
    let mut vector = "[0.1; -0.2; 0.3; -0.4; 0.5; -5; 8.4; 0]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[50 -25 16.67 -12.5 10 -1 0.595 +∞]", &calc.ToString());
}
#[test]
fn ScalarVectorRemainder() {
    let mut x = "3.2";
    let mut vector = "[-4.3; 0; 6.9; -2.8; 3; -3; 6.5; 0]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[3.2  Undefined  3.2 0.4 0.2 0.2 3.2  Undefined ]", &calc.ToString());
}
#[test]
fn ScalarVectorEqual() {
    let mut x = "5.5";
    let mut vector = "[5.5; 2.1; 7.9; -1.5; 5.5; 6.7; -5; 5.5]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", x, vector)]);
    let mut actualResult = calc.ToString();
    let mut expectedResult = "[1 0 0 0 1 0 0 1]";
    assert_equal(expectedResult.clone(), actualResult.clone());
}
#[test]
fn ScalarVectorNotEqual() {
    let mut x = "3";
    let mut vector = "[-4.3; 0; 6.9; -2.8; 3; -5; 0; 2.7; -1.4]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 1 1 1 0 1 1 1 1]", &calc.ToString());
}
#[test]
fn ScalarVectorLessThan() {
    let mut x = "6";
    let mut vector = "[0; -6.7; 3.8; -2; 9.5; 7.2; 0; -1.3; 5]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 0 0 0 1 1 0 0 0]", &calc.ToString());
}
#[test]
fn ScalarVectorGreaterThan() {
    let mut x = "-1";
    let mut vector = "[4.7; -8; 0; 2.3; -6.5; -3; 6.2; 0; -5.1; 7.4]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 1 0 0 1 1 0 0 1 0]", &calc.ToString());
}
#[test]
fn ScalarVectorLessOrEqual() {
    let mut x = "3";
    let mut vector = "[5; -2.1; 0; 7.8; -4.6]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 0 0 1 0]", &calc.ToString());
}
#[test]
fn ScalarVectorGreaterOrEqual() {
    let mut x = "0";
    let mut vector = "[-40; 16.7; 0; 22.3; 13.5; 21.2; 0; -19.3; 25]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 0 1 0 0 0 1 1 0]", &calc.ToString());
}
#[test]
fn ScalarVectorLogicalAnd() {
    let mut x = "5";
    let mut vector = "[0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 1 0 1 1 1 1 0 1 1]", &calc.ToString());
}
#[test]
fn ScalarVectorLogicalOr() {
    let mut x = "5";
    let mut vector = "[0; 16.7; 0; 22.3; 13.5; -18; 21.2; 0; -19.3; 25]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[1 1 1 1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn ScalarVectorLogicalXor() {
    let mut x = "5";
    let mut vector = "[6; -2.5; 0; 4.3; -8.1; 8.6; 0; -5.2; 1.9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", x, vector)]);
    assert_equal_str("[0 0 1 0 0 0 1 0 0]", &calc.ToString());
}
