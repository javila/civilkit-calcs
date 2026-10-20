#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Small/HPMatrixOperatorTests.cs by tools/transcribe_xunit.py.
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
fn HPMatrixAddition() {
    let mut matrix1 = "hp([1; 2; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut matrix2 = "hp([5; 4; 3 | 1; 2; 2 | 7; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", matrix1, matrix2));
    assert_equal_str("[6 6 6|5 7 8|14 9 18]", &calc.ToString());
}
#[test]
fn HPMatrixSubtraction() {
    let mut matrix1 = "hp([8; 3; 7 | 2; 6; 5 | 9; 4; 2])";
    let mut matrix2 = "hp([5; 2; 4 | 3; 1; 6 | 1; 8; 7])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", matrix1, matrix2));
    assert_equal_str("[3 1 3|-1 5 -1|8 -4 -5]", &calc.ToString());
}
#[test]
fn HPMatrixExponentiation() {
    let mut matrix1 = "hp([2; 3; 2 | 1; 4; 5 | 3; 2; 2])";
    let mut matrix2 = "hp([3; 2; 4 | 5; 1; 2 | 2; 3; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", matrix1, matrix2));
    assert_equal_str("[8 9 16|1 4 25|9 8 2]", &calc.ToString());
}
#[test]
fn HPMatrixDivision() {
    let mut matrix1 = "hp([10; 20; 30 | 40; 50; 60 | 70; 80; 90])";
    let mut matrix2 = "hp([2; 4; 5 | 10; 2; 6 | 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", matrix1, matrix2));
    assert_equal_str("[5 5 6|4 25 10|10 10 10]", &calc.ToString());
}
#[test]
fn HPMatrixMultiplication() {
    let mut matrix1 = "hp([1; 4; 7 | 2; 5; 8 | 3; 6; 9])";
    let mut matrix2 = "hp([9; 8; 7 | 6; 5; 4 | 3; 2; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} * {}", matrix1, matrix2));
    assert_equal_str("[54 42 30|72 57 42|90 72 54]", &calc.ToString());
}
#[test]
fn HPMatrixIntegerDivision() {
    let mut matrix1 = "hp([10; 21; 30 | 40; 55; 66 | 70; 88; 99])";
    let mut matrix2 = "hp([2; 4; 5 | 10; 2; 6 | 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", matrix1, matrix2));
    assert_equal_str("[5 5 6|4 27 11|10 11 11]", &calc.ToString());
}
#[test]
fn HPMatrixDivisionBar() {
    let mut matrix1 = "hp([18; 16; 14 | 12; 10; 8 | 6; 4; 2])";
    let mut matrix2 = "hp([2; 4; 7 | 3; 5; 8 | 1; 2; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", matrix1, matrix2));
    assert_equal_str("[9 4 2|4 2 1|6 2 2]", &calc.ToString());
}
#[test]
fn HPMatrixRemainder() {
    let mut matrix1 = "hp([11; 23; 34 | 45; 56; 67 | 78; 89; 90])";
    let mut matrix2 = "hp([2; 4; 5 | 6; 7; 8 | 9; 10; 11])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", matrix1, matrix2));
    assert_equal_str("[1 3 4|3 0 3|6 9 2]", &calc.ToString());
}
#[test]
fn HPMatrixFactorial() {
    let mut matrix = "hp([4; 2; 6| 2; 3; 4| 7; 7; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{}!", matrix));
    assert_equal_str("[24 2 720|2 6 24|5040 5040 1]", &calc.ToString());
}
#[test]
fn HPMatrixEqual() {
    let mut matrix1 = "hp([1; 2; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut matrix2 = "hp([1; 4; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≡ {}", matrix1, matrix2));
    assert_equal_str("[1 0 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixNotEqual() {
    let mut matrix1 = "hp([1; 2; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut matrix2 = "hp([9; 8; 7 | 6; 5; 4 | 3; 2; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≠ {}", matrix1, matrix2));
    assert_equal_str("[1 1 1|1 0 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixLessThan() {
    let mut matrix1 = "hp([1; 2; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut matrix2 = "hp([9; 8; 7 | 6; 5; 4 | 3; 2; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", matrix1, matrix2));
    assert_equal_str("[1 1 1|1 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPMatrixGreaterThan() {
    let mut matrix1 = "hp([9; 8; 7 | 6; 5; 4 | 3; 2; 1])";
    let mut matrix2 = "hp([1; 2; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", matrix1, matrix2));
    assert_equal_str("[1 1 1|1 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPMatrixLessOrEqual() {
    let mut matrix1 = "hp([1; 2; 10 | 4; 5; 6 | 7; 8; 9])";
    let mut matrix2 = "hp([1; 2; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", matrix1, matrix2));
    assert_equal_str("[1 1 0|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixGreaterOrEqual() {
    let mut matrix1 = "hp([9; 8; 7 | 6; 5; 4 | 3; 2; 1])";
    let mut matrix2 = "hp([1; 2; 3 | 4; 5; 6 | 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", matrix1, matrix2));
    assert_equal_str("[1 1 1|1 1 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPMatrixLogicalAnd() {
    let mut matrix1 = "hp([1; 1; 0 | 0; 1; 0 | 1; 0; 1])";
    let mut matrix2 = "hp([1; 0; 1 | 1; 1; 0 | 0; 1; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", matrix1, matrix2));
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPMatrixLogicalOr() {
    let mut matrix1 = "hp([1; 1; 0 | 0; 1; 0 | 1; 0; 1])";
    let mut matrix2 = "hp([1; 0; 1 | 1; 1; 0 | 0; 1; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", matrix1, matrix2));
    assert_equal_str("[1 1 1|1 1 0|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixLogicalXor() {
    let mut matrix1 = "hp([1; 1; 0 | 0; 1; 0 | 1; 0; 1])";
    let mut matrix2 = "hp([1; 0; 1 | 1; 1; 0 | 0; 1; 1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", matrix1, matrix2));
    assert_equal_str("[0 1 1|1 0 0|1 1 0]", &calc.ToString());
}
#[test]
fn HPMatrixScalarAddition() {
    let mut matrix = "hp([1; 2; 3 | 4; 5 ; 6 | 7; 8; 9])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", matrix, x));
    assert_equal_str("[5 6 7|8 9 10|11 12 13]", &calc.ToString());
}
#[test]
fn HPMatrixScalarSubtraction() {
    let mut matrix = "hp([5; 6; 7 | 8; 9 ; 10 | 11; 12; 13])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", matrix, x));
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn HPMatrixScalarExponentiation() {
    let mut matrix = "hp([2; 3; 4 | 5; 6 ; 7 | 8; 9; 10])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", matrix, x));
    assert_equal_str("[4 9 16|25 36 49|64 81 100]", &calc.ToString());
}
#[test]
fn HPMatrixScalarDivision() {
    let mut matrix = "hp([8; 16; 24 | 32; 40 ; 48 | 56; 64; 72])";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", matrix, x));
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn HPMatrixScalarMultiplication() {
    let mut matrix = "hp([1; 2; 3 | 4; 5 ; 6 | 7; 8; 9])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} * {}", matrix, x));
    assert_equal_str("[4 8 12|16 20 24|28 32 36]", &calc.ToString());
}
#[test]
fn HPMatrixScalarIntegerDivision() {
    let mut matrix = "hp([9; 12; 15 | 18; 21 ; 24 | 27; 30; 33])";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", matrix, x));
    assert_equal_str("[3 4 5|6 7 8|9 10 11]", &calc.ToString());
}
#[test]
fn HPMatrixScalarDivisionBar() {
    let mut matrix = "hp([9; 18; 27 | 36; 45 ; 54 | 63; 72; 81])";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", matrix, x));
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn HPMatrixScalarRemainder() {
    let mut matrix = "hp([10; 15; 20 | 25; 30 ; 35 | 40; 45; 50])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", matrix, x));
    assert_equal_str("[3 1 6|4 2 0|5 3 1]", &calc.ToString());
}
#[test]
fn HPMatrixScalarEqual() {
    let mut matrix = "hp([1; 7; 7 | 7; 7 ; 7 | 7; 7; 7])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≡ {}", matrix, x));
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixScalarNotEqual() {
    let mut matrix = "hp([7; 8; 9 | 10; 11 ; 12 | 13; 14; 15])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≠ {}", matrix, x));
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixScalarLessThan() {
    let mut matrix = "hp([5; 6; 7 | 8; 9 ; 10 | 11; 12; 13])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", matrix, x));
    assert_equal_str("[1 1 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPMatrixScalarGreaterThan() {
    let mut matrix = "hp([8; 9; 10 | 11; 12 ; 13 | 14; 15; 16])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", matrix, x));
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixScalarLessOrEqual() {
    let mut matrix = "hp([5; 7; 9 | 11; 13 ; 15 | 17; 19; 21])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", matrix, x));
    assert_equal_str("[1 1 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPMatrixScalarGreaterOrEqual() {
    let mut matrix = "hp([7; 8; 9 | 10; 11 ; 12 | 13; 14; 15])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", matrix, x));
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixScalarLogicalAnd() {
    let mut matrix = "hp([1; 0; 1 | 0; 1 ; 0 | 1; 0; 1])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", matrix, x));
    assert_equal_str("[1 0 1|0 1 0|1 0 1]", &calc.ToString());
}
#[test]
fn HPMatrixScalarLogicalOr() {
    let mut matrix = "hp([0; 0; 0 | 0; 1 ; 0 | 0; 0; 0])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", matrix, x));
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixScalarLogicalXor() {
    let mut matrix = "hp([1; 1; 0 | 0; 1 ; 0 | 1; 0; 1])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", matrix, x));
    assert_equal_str("[0 0 1|1 0 1|0 1 0]", &calc.ToString());
}
#[test]
fn HPScalarMatrixAddition() {
    let mut matrix = "hp([1; 2; 3 | 4; 5 ; 6 | 7; 8; 9])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", x, matrix));
    assert_equal_str("[5 6 7|8 9 10|11 12 13]", &calc.ToString());
}
#[test]
fn HPScalarMatrixSubtraction() {
    let mut matrix = "hp([5; 6; 7 | 8; 9 ; 10 | 11; 12; 13])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", x, matrix));
    assert_equal_str("[-1 -2 -3|-4 -5 -6|-7 -8 -9]", &calc.ToString());
}
#[test]
fn HPScalarMatrixExponentiation() {
    let mut matrix = "hp([2; 3; 4 | 5; 6 ; 7 | 8; 9; 2])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", x, matrix));
    assert_equal_str("[4 8 16|32 64 128|256 512 4]", &calc.ToString());
}
#[test]
fn HPScalarMatrixDivision() {
    let mut matrix = "hp([8; 16; 24 | 32; 40 ; 48 | 56; 64; 72])";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", x, matrix));
    assert_equal_str("[1 0.5 0.333|0.25 0.2 0.167|0.143 0.125 0.111]", &calc.ToString());
}
#[test]
fn HPScalarMatrixMultiplication() {
    let mut matrix = "hp([1; 2; 3 | 4; 5 ; 6 | 7; 8; 9])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} * {}", x, matrix));
    assert_equal_str("[4 8 12|16 20 24|28 32 36]", &calc.ToString());
}
#[test]
fn HPScalarMatrixIntegerDivision() {
    let mut matrix = "hp([9; 12; 15 | 18; 21 ; 24 | 27; 30; 33])";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", x, matrix));
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPScalarMatrixDivisionBar() {
    let mut matrix = "hp([9; 18; 27 | 36; 45 ; 54 | 63; 72; 81])";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", x, matrix));
    assert_equal_str("[1 0.5 0.333|0.25 0.2 0.167|0.143 0.125 0.111]", &calc.ToString());
}
#[test]
fn HPScalarMatrixRemainder() {
    let mut matrix = "hp([10; 15; 20 | 25; 30 ; 35 | 40; 45; 50])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", x, matrix));
    assert_equal_str("[7 7 7|7 7 7|7 7 7]", &calc.ToString());
}
#[test]
fn HPScalarMatrixLessThan() {
    let mut matrix = "hp([5; 6; 7 | 8; 9 ; 10 | 11; 12; 13])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", x, matrix));
    assert_equal_str("[0 0 0|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPScalarMatrixGreaterThan() {
    let mut matrix = "hp([8; 9; 10 | 11; 12 ; 13 | 14; 15; 16])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", x, matrix));
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPScalarMatrixLessOrEqual() {
    let mut matrix = "hp([5; 7; 9 | 11; 13 ; 15 | 17; 19; 21])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", x, matrix));
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPScalarMatrixGreaterOrEqual() {
    let mut matrix = "hp([7; 8; 9 | 10; 11 ; 12 | 13; 14; 15])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", x, matrix));
    assert_equal_str("[1 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPScalarMatrixLogicalAnd() {
    let mut matrix = "hp([1; 0; 1 | 0; 1 ; 0 | 1; 0; 1])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", x, matrix));
    assert_equal_str("[1 0 1|0 1 0|1 0 1]", &calc.ToString());
}
#[test]
fn HPScalarMatrixLogicalOr() {
    let mut matrix = "hp([1; 0; 1 | 0; 1 ; 0 | 1; 0; 1])";
    let mut x = "0";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", x, matrix));
    assert_equal_str("[1 0 1|0 1 0|1 0 1]", &calc.ToString());
}
#[test]
fn HPScalarMatrixLogicalXor() {
    let mut matrix = "hp([1; 0; 1 | 0; 1 ; 0 | 1; 0; 1])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", x, matrix));
    assert_equal_str("[0 1 0|1 0 1|0 1 0]", &calc.ToString());
}
