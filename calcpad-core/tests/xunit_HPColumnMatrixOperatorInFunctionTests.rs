#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Small/HPColumnMatrixOperatorInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPColumnMatrixAddition() {
    let mut matrix1 = "hp([1| 4| 7])";
    let mut matrix2 = "hp([5| 1| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[6|5|14]", &calc.ToString());
}
#[test]
fn HPColumnMatrixSubtraction() {
    let mut matrix1 = "hp([8| 2| 9])";
    let mut matrix2 = "hp([5| 3| 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[3|-1|8]", &calc.ToString());
}
#[test]
fn HPColumnMatrixExponentiation() {
    let mut matrix1 = "hp([2| 1| 3])";
    let mut matrix2 = "hp([3| 5| 2])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[8|1|9]", &calc.ToString());
}
#[test]
fn HPColumnMatrixDivision() {
    let mut matrix1 = "hp([10| 40| 70])";
    let mut matrix2 = "hp([2| 10| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[5|4|10]", &calc.ToString());
}
#[test]
fn HPColumnMatrixIntegerDivision() {
    let mut matrix1 = "hp([10| 40| 70])";
    let mut matrix2 = "hp([2| 10| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[5|4|10]", &calc.ToString());
}
#[test]
fn HPColumnMatrixDivisionBar() {
    let mut matrix1 = "hp([18| 12| 6])";
    let mut matrix2 = "hp([2| 3| 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[9|4|6]", &calc.ToString());
}
#[test]
fn HPColumnMatrixRemainder() {
    let mut matrix1 = "hp([11| 67| 78])";
    let mut matrix2 = "hp([2| 6| 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|6]", &calc.ToString());
}
#[test]
fn HPColumnMatrixFactorial() {
    let mut matrix = "hp([4| 2| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = a!", &format!("f({})", matrix)]);
    let mut expected = "[24|2|5040]";
    assert_equal(expected.clone(), calc.ToString());
}
#[test]
fn HPColumnMatrixEqual() {
    let mut matrix1 = "hp([1| 4| 7])";
    let mut matrix2 = "hp([1| 4| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixNotEqual() {
    let mut matrix1 = "hp([1| 4| 7])";
    let mut matrix2 = "hp([9| 6| 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixLessThan() {
    let mut matrix1 = "hp([1| 4| 7])";
    let mut matrix2 = "hp([9| 6| 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixGreaterThan() {
    let mut matrix1 = "hp([9| 6| 3])";
    let mut matrix2 = "hp([1| 4| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixLessOrEqual() {
    let mut matrix1 = "hp([1| 4| 7])";
    let mut matrix2 = "hp([1| 4| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixGreaterOrEqual() {
    let mut matrix1 = "hp([9| 6| 3])";
    let mut matrix2 = "hp([1| 4| 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixLogicalAnd() {
    let mut matrix1 = "hp([1| 0| 1])";
    let mut matrix2 = "hp([1| 1| 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|0|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixLogicalOr() {
    let mut matrix1 = "hp([1| 0| 1])";
    let mut matrix2 = "hp([1| 1| 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixLogicalXor() {
    let mut matrix1 = "hp([1| 0| 1])";
    let mut matrix2 = "hp([1| 1| 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", matrix1, matrix2)]);
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorAddition() {
    let mut matrix = "hp([1| 2| 3| 4| 5])";
    let mut vector = "hp([1; 1; 2; 4; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[2|3|5|8|12]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorSubtraction() {
    let mut matrix = "hp([5| 6| 7| 8| 9])";
    let mut vector = "hp([1; 1; 2; 4; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[4|5|5|4|2]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorExponentiation() {
    let mut matrix = "hp([2| 3| 4| 5| 6])";
    let mut vector = "hp([1; 2; 3; 4; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[2|9|64|625|7776]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorDivision() {
    let mut matrix = "hp([8| 18| 24| 36| 42])";
    let mut vector = "hp([2; 3; 4; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[4|6|6|6|6]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorMultiplication() {
    let mut matrix = "hp([4.5| -5.7| 7.1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "vec = matrix_hp(1; 3)",
        "vec.(1; 1) = -9.2",
        "vec.(1; 2) = 6.3",
        "vec.(1; 3) = 1",
        "f(a; b) = a * b",
        &format!("f({};vec)", matrix),
    ]);
    assert_equal_str("[-41.4 28.35 4.5|52.44 -35.91 -5.7|-65.32 44.73 7.1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorIntegerDivision() {
    let mut matrix = "hp([9| 12| 15| 18| 21])";
    let mut vector = "hp([3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[3|3|3|3|3]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorDivisionBar() {
    let mut matrix = "hp([9| 18| 27| 36| 45])";
    let mut vector = "hp([1; 2; 3; 4; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[9|9|9|9|9]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorRemainder() {
    let mut matrix = "hp([10| 20| 30| 40| 50])";
    let mut vector = "hp([3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|0|0|4|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorEqual() {
    let mut matrix = "hp([7| 7| 7| 7| 7])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|1|1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorNotEqual() {
    let mut matrix = "hp([7| 8| 9| 10| 11])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[0|1|1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorGreaterThan() {
    let mut matrix = "hp([8| 9| 10| 11| 12])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|1|1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorLessThan() {
    let mut matrix = "hp([5| 6| 7| 8| 9])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|1|0|0|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorLessOrEqual() {
    let mut matrix = "hp([5| 7| 9| 11| 13])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|1|0|0|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorGreaterOrEqual() {
    let mut matrix = "hp([7| 8| 9| 10| 11])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|1|1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorLogicalAnd() {
    let mut matrix = "hp([1| 0| 1| 0| 1])";
    let mut vector = "hp([1; 1; 0; 0; 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|0|0|0|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorLogicalOr() {
    let mut matrix = "hp([0| 0| 0| 0| 1])";
    let mut vector = "hp([1; 1; 1; 0; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[1|1|1|0|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixVectorLogicalXor() {
    let mut matrix = "hp([1| 1| 0| 0| 1])";
    let mut vector = "hp([1; 0; 1; 0; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", matrix, vector)]);
    assert_equal_str("[0|1|1|0|1]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixAddition() {
    let mut matrix = "hp([1| 2| 3| 4| 5])";
    let mut vector = "hp([1; 1; 2; 4; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[2|3|5|8|12]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixSubtraction() {
    let mut matrix = "hp([5| 6| 7| 8| 9])";
    let mut vector = "hp([1; 1; 2; 4; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[-4|-5|-5|-4|-2]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixExponentiation() {
    let mut matrix = "hp([2| 3| 4| 5| 6])";
    let mut vector = "hp([1; 2; 3; 4; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[1|8|81|1024|15625]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixDivision() {
    let mut matrix = "hp([8| 18| 24| 36| 42])";
    let mut vector = "hp([2; 3; 4; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[0.25|0.167|0.167|0.167|0.167]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixMultiplication() {
    let mut matrix = "hp([4.5; -3.2| -5.7; 3.4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "vec = matrix_hp(1; 2)",
        "vec.(1; 1) = -9.2",
        "vec.(1; 2) = 6.3",
        "f(a; b) = a * b",
        &format!("f(vec;{})", matrix),
    ]);
    assert_equal_str("[-77.31 50.86]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixIntegerDivision() {
    let mut matrix = "hp([9| 2| 15| 18| 21])";
    let mut vector = "hp([3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[0|2|0|0|0]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixDivisionBar() {
    let mut matrix = "hp([9| 18| 27| 36| 45])";
    let mut vector = "hp([1; 2; 3; 4; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[0.111|0.111|0.111|0.111|0.111]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixRemainder() {
    let mut matrix = "hp([10| 20| 30| 40| 50])";
    let mut vector = "hp([3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[3|4|5|6|7]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixGreaterThan() {
    let mut matrix = "hp([2| 9| 10| 11| 12])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[1|0|0|0|0]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixLessThan() {
    let mut matrix = "hp([5| 6| 7| 8| 9])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[0|0|0|1|1]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixLessOrEqual() {
    let mut matrix = "hp([5| 7| 9| 11| 13])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[0|1|1|1|1]", &calc.ToString());
}
#[test]
fn HPVectorColumnMatrixGreaterOrEqual() {
    let mut matrix = "hp([7| 8| 9| 10| 11])";
    let mut vector = "hp([7; 7; 7; 7; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", vector, matrix)]);
    assert_equal_str("[1|0|0|0|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarAddition() {
    let mut matrix = "hp([1| 4| 7])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[5|8|11]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarSubtraction() {
    let mut matrix = "hp([5| 8| 11])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|4|7]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarExponentiation() {
    let mut matrix = "hp([2| 5| 8])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[4|25|64]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarDivision() {
    let mut matrix = "hp([8| 32| 56])";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|4|7]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarMultiplication() {
    let mut matrix = "hp([1| 4| 7])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[4|16|28]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarIntegerDivision() {
    let mut matrix = "hp([9| 18| 27])";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[3|6|9]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarDivisionBar() {
    let mut matrix = "hp([9| 36| 63])";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|4|7]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarRemainder() {
    let mut matrix = "hp([10| 25| 40])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[3|4|5]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarEqual() {
    let mut matrix = "hp([1| 7| 7])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≡ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarNotEqual() {
    let mut matrix = "hp([7| 10| 13])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≠ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarLessThan() {
    let mut matrix = "hp([5| 8| 11])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|0|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarGreaterThan() {
    let mut matrix = "hp([2| 11| 14])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarLessOrEqual() {
    let mut matrix = "hp([5| 11| 17])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|0|0]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarGreaterOrEqual() {
    let mut matrix = "hp([7| 10| 13])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarLogicalAnd() {
    let mut matrix = "hp([1| 0| 1])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∧ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|0|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarLogicalOr() {
    let mut matrix = "hp([0| 1| 0])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ∨ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn HPColumnMatrixScalarLogicalXor() {
    let mut matrix = "hp([1| 0| 1])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⊕ b", &format!("f({};{})", matrix, x)]);
    assert_equal_str("[0|1|0]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixAddition() {
    let mut matrix = "hp([1| 4| 7])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a + b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[5|8|11]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixSubtraction() {
    let mut matrix = "hp([5| 8| 11])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a - b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[-1|-4|-7]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixExponentiation() {
    let mut matrix = "hp([2| 5| 8])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ^ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[4|32|256]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixDivision() {
    let mut matrix = "hp([8| 32| 56])";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a / b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[1|0.25|0.143]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixMultiplication() {
    let mut matrix = "hp([1| 4| 7])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a * b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[4|16|28]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixIntegerDivision() {
    let mut matrix = "hp([9| 18| 27])";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a \\ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0|0|0]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixDivisionBar() {
    let mut matrix = "hp([9| 36| 63])";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ÷ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[1|0.25|0.143]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixRemainder() {
    let mut matrix = "hp([10| 25| 40])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ⦼ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[7|7|7]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixLessThan() {
    let mut matrix = "hp([5| 8| 11])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a < b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixGreaterThan() {
    let mut matrix = "hp([8| 11| 14])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a > b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0|0|0]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixLessOrEqual() {
    let mut matrix = "hp([5| 11| 17])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≤ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn HPScalarColumnMatrixGreaterOrEqual() {
    let mut matrix = "hp([7| 10| 13])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = a ≥ b", &format!("f({};{})", x, matrix)]);
    assert_equal_str("[1|0|0]", &calc.ToString());
}
