//! Transcribed from Calcpad.Tests/Matrices/MatrixOperatorInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn MatrixAddition() {
    let mut matrix1 = "[1; 2; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut matrix2 = "[5; 4; 3 | 1; 2; 2 | 7; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[6 6 6|5 7 8|14 9 18]", &calc.ToString());
}
#[test]
fn MatrixSubtraction() {
    let mut matrix1 = "[8; 3; 7 | 2; 6; 5 | 9; 4; 2]";
    let mut matrix2 = "[5; 2; 4 | 3; 1; 6 | 1; 8; 7]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[3 1 3|-1 5 -1|8 -4 -5]", &calc.ToString());
}
#[test]
fn MatrixExponentiation() {
    let mut matrix1 = "[2; 3; 2 | 1; 4; 5 | 3; 2; 2]";
    let mut matrix2 = "[3; 2; 4 | 5; 1; 2 | 2; 3; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[8 9 16|1 4 25|9 8 2]", &calc.ToString());
}
#[test]
fn MatrixDivision() {
    let mut matrix1 = "[10; 20; 30 | 40; 50; 60 | 70; 80; 90]";
    let mut matrix2 = "[2; 4; 5 | 10; 2; 6 | 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[5 5 6|4 25 10|10 10 10]", &calc.ToString());
}
#[test]
fn MatrixMultiplication() {
    let mut matrix1 = "[1; 4; 7 | 2; 5; 8 | 3; 6; 9]";
    let mut matrix2 = "[9; 8; 7 | 6; 5; 4 | 3; 2; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[54 42 30|72 57 42|90 72 54]", &calc.ToString());
}
#[test]
fn MatrixIntegerDivision() {
    let mut matrix1 = "[10; 21; 30 | 40; 55; 66 | 70; 88; 99]";
    let mut matrix2 = "[2; 4; 5 | 10; 2; 6 | 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[5 5 6|4 27 11|10 11 11]", &calc.ToString());
}
#[test]
fn MatrixDivisionBar() {
    let mut matrix1 = "[18; 16; 14 | 12; 10; 8 | 6; 4; 2]";
    let mut matrix2 = "[2; 4; 7 | 3; 5; 8 | 1; 2; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[9 4 2|4 2 1|6 2 2]", &calc.ToString());
}
#[test]
fn MatrixRemainder() {
    let mut matrix1 = "[11; 23; 34 | 45; 56; 67 | 78; 89; 90]";
    let mut matrix2 = "[2; 4; 5 | 6; 7; 8 | 9; 10; 11]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 3 4|3 0 3|6 9 2]", &calc.ToString());
}
#[test]
fn MatrixFactorial() {
    let mut matrix = "[4; 2; 6| 2; 3; 4| 7; 7; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = a!", &format!("f({})", matrix)]);
    assert_equal_str("[24 2 720|2 6 24|5040 5040 1]", &calc.ToString());
}
#[test]
fn MatrixEqual() {
    let mut matrix1 = "[1; 2; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut matrix2 = "[1; 4; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 0 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixNotEqual() {
    let mut matrix1 = "[1; 2; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut matrix2 = "[9; 8; 7 | 6; 5; 4 | 3; 2; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 1 1|1 0 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixLessThan() {
    let mut matrix1 = "[1; 2; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut matrix2 = "[9; 8; 7 | 6; 5; 4 | 3; 2; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 1 1|1 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn MatrixGreaterThan() {
    let mut matrix1 = "[9; 8; 7 | 6; 5; 4 | 3; 2; 1]";
    let mut matrix2 = "[1; 2; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 1 1|1 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn MatrixLessOrEqual() {
    let mut matrix1 = "[1; 2; 10 | 4; 5; 6 | 7; 8; 9]";
    let mut matrix2 = "[1; 2; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 1 0|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixGreaterOrEqual() {
    let mut matrix1 = "[9; 8; 7 | 6; 5; 4 | 3; 2; 1]";
    let mut matrix2 = "[1; 2; 3 | 4; 5; 6 | 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 1 1|1 1 0|0 0 0]", &calc.ToString());
}
#[test]
fn MatrixLogicalAnd() {
    let mut matrix1 = "[1; 1; 0 | 0; 1; 0 | 1; 0; 1]";
    let mut matrix2 = "[1; 0; 1 | 1; 1; 0 | 0; 1; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn MatrixLogicalOr() {
    let mut matrix1 = "[1; 1; 0 | 0; 1; 0 | 1; 0; 1]";
    let mut matrix2 = "[1; 0; 1 | 1; 1; 0 | 0; 1; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1 1 1|1 1 0|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixLogicalXor() {
    let mut matrix1 = "[1; 1; 0 | 0; 1; 0 | 1; 0; 1]";
    let mut matrix2 = "[1; 0; 1 | 1; 1; 0 | 0; 1; 1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[0 1 1|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn MatrixScalarAddition() {
    let mut matrix = "[1; 2; 3 | 4; 5 ; 6 | 7; 8; 9]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[5 6 7|8 9 10|11 12 13]", &calc.ToString());
}
#[test]
fn MatrixScalarSubtraction() {
    let mut matrix = "[5; 6; 7 | 8; 9 ; 10 | 11; 12; 13]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn MatrixScalarExponentiation() {
    let mut matrix = "[2; 3; 4 | 5; 6 ; 7 | 8; 9; 10]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[4 9 16|25 36 49|64 81 100]", &calc.ToString());
}
#[test]
fn MatrixScalarDivision() {
    let mut matrix = "[8; 16; 24 | 32; 40 ; 48 | 56; 64; 72]";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn MatrixScalarMultiplication() {
    let mut matrix = "[1; 2; 3 | 4; 5 ; 6 | 7; 8; 9]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[4 8 12|16 20 24|28 32 36]", &calc.ToString());
}
#[test]
fn MatrixScalarIntegerDivision() {
    let mut matrix = "[9; 12; 15 | 18; 21 ; 24 | 27; 30; 33]";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[3 4 5|6 7 8|9 10 11]", &calc.ToString());
}
#[test]
fn MatrixScalarDivisionBar() {
    let mut matrix = "[9; 18; 27 | 36; 45 ; 54 | 63; 72; 81]";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn MatrixScalarRemainder() {
    let mut matrix = "[10; 15; 20 | 25; 30 ; 35 | 40; 45; 50]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[3 1 6|4 2 0|5 3 1]", &calc.ToString());
}
#[test]
fn MatrixScalarEqual() {
    let mut matrix = "[1; 7; 7 | 7; 7 ; 7 | 7; 7; 7]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixScalarNotEqual() {
    let mut matrix = "[7; 8; 9 | 10; 11 ; 12 | 13; 14; 15]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixScalarLessThan() {
    let mut matrix = "[5; 6; 7 | 8; 9 ; 10 | 11; 12; 13]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 1 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn MatrixScalarGreaterThan() {
    let mut matrix = "[8; 9; 10 | 11; 12 ; 13 | 14; 15; 16]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixScalarLessOrEqual() {
    let mut matrix = "[5; 7; 9 | 11; 13 ; 15 | 17; 19; 21]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 1 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn MatrixScalarGreaterOrEqual() {
    let mut matrix = "[7; 8; 9 | 10; 11 ; 12 | 13; 14; 15]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixScalarLogicalAnd() {
    let mut matrix = "[1; 0; 1 | 0; 1 ; 0 | 1; 0; 1]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 0 1|0 1 0|1 0 1]", &calc.ToString());
}
#[test]
fn MatrixScalarLogicalOr() {
    let mut matrix = "[0; 0; 0 | 0; 1 ; 0 | 0; 0; 0]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixScalarLogicalXor() {
    let mut matrix = "[1; 1; 0 | 0; 1 ; 0 | 1; 0; 1]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[0 0 1|1 0 1|0 1 0]", &calc.ToString());
}
#[test]
fn ScalarMatrixAddition() {
    let mut matrix = "[1; 2; 3 | 4; 5 ; 6 | 7; 8; 9]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[5 6 7|8 9 10|11 12 13]", &calc.ToString());
}
#[test]
fn ScalarMatrixSubtraction() {
    let mut matrix = "[5; 6; 7 | 8; 9 ; 10 | 11; 12; 13]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[-1 -2 -3|-4 -5 -6|-7 -8 -9]", &calc.ToString());
}
#[test]
fn ScalarMatrixExponentiation() {
    let mut matrix = "[2; 3; 4 | 5; 6 ; 7 | 8; 9; 2]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[4 8 16|32 64 128|256 512 4]", &calc.ToString());
}
#[test]
fn ScalarMatrixDivision() {
    let mut matrix = "[8; 16; 24 | 32; 40 ; 48 | 56; 64; 72]";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[1 0.5 0.333|0.25 0.2 0.167|0.143 0.125 0.111]", &calc.ToString());
}
#[test]
fn ScalarMatrixMultiplication() {
    let mut matrix = "[1; 2; 3 | 4; 5 ; 6 | 7; 8; 9]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[4 8 12|16 20 24|28 32 36]", &calc.ToString());
}
#[test]
fn ScalarMatrixIntegerDivision() {
    let mut matrix = "[9; 12; 15 | 18; 21 ; 24 | 27; 30; 33]";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn ScalarMatrixDivisionBar() {
    let mut matrix = "[9; 18; 27 | 36; 45 ; 54 | 63; 72; 81]";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[1 0.5 0.333|0.25 0.2 0.167|0.143 0.125 0.111]", &calc.ToString());
}
#[test]
fn ScalarMatrixRemainder() {
    let mut matrix = "[10; 15; 20 | 25; 30 ; 35 | 40; 45; 50]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[7 7 7|7 7 7|7 7 7]", &calc.ToString());
}
#[test]
fn ScalarMatrixLessThan() {
    let mut matrix = "[5; 6; 7 | 8; 9 ; 10 | 11; 12; 13]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0 0 0|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn ScalarMatrixGreaterThan() {
    let mut matrix = "[8; 9; 10 | 11; 12 ; 13 | 14; 15; 16]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn ScalarMatrixLessOrEqual() {
    let mut matrix = "[5; 7; 9 | 11; 13 ; 15 | 17; 19; 21]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn ScalarMatrixGreaterOrEqual() {
    let mut matrix = "[7; 8; 9 | 10; 11 ; 12 | 13; 14; 15]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[1 0 0|0 0 0|0 0 0]", &calc.ToString());
}
