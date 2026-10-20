#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/ColumnMatrixOperatorTests.cs by tools/transcribe_xunit.py.
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
fn ColumnMatrixAddition() {
    let mut matrix1 = "[1| 4| 7]";
    let mut matrix2 = "[5| 1| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", matrix1, matrix2));
    assert_equal_str("[6|5|14]", &calc.ToString());
}
#[test]
fn ColumnMatrixSubtraction() {
    let mut matrix1 = "[8| 2| 9]";
    let mut matrix2 = "[5| 3| 1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", matrix1, matrix2));
    assert_equal_str("[3|-1|8]", &calc.ToString());
}
#[test]
fn ColumnMatrixExponentiation() {
    let mut matrix1 = "[2| 1| 3]";
    let mut matrix2 = "[3| 5| 2]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", matrix1, matrix2));
    assert_equal_str("[8|1|9]", &calc.ToString());
}
#[test]
fn ColumnMatrixDivision() {
    let mut matrix1 = "[10| 40| 70]";
    let mut matrix2 = "[2| 10| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", matrix1, matrix2));
    assert_equal_str("[5|4|10]", &calc.ToString());
}
#[test]
fn ColumnMatrixIntegerDivision() {
    let mut matrix1 = "[10| 40| 70]";
    let mut matrix2 = "[2| 10| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", matrix1, matrix2));
    assert_equal_str("[5|4|10]", &calc.ToString());
}
#[test]
fn ColumnMatrixDivisionBar() {
    let mut matrix1 = "[18| 12| 6]";
    let mut matrix2 = "[2| 3| 1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", matrix1, matrix2));
    assert_equal_str("[9|4|6]", &calc.ToString());
}
#[test]
fn ColumnMatrixRemainder() {
    let mut matrix1 = "[11| 67| 78]";
    let mut matrix2 = "[2| 6| 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", matrix1, matrix2));
    assert_equal_str("[1|1|6]", &calc.ToString());
}
#[test]
fn ColumnMatrixFactorial() {
    let mut matrix = "[4| 2| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{}!", matrix));
    let mut expected = "[24|2|5040]";
    assert_equal(expected.clone(), calc.ToString());
}
#[test]
fn ColumnMatrixEqual() {
    let mut matrix1 = "[1| 4| 7]";
    let mut matrix2 = "[1| 4| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≡ {}", matrix1, matrix2));
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixNotEqual() {
    let mut matrix1 = "[1| 4| 7]";
    let mut matrix2 = "[9| 6| 3]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≠ {}", matrix1, matrix2));
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixLessThan() {
    let mut matrix1 = "[1| 4| 7]";
    let mut matrix2 = "[9| 6| 3]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", matrix1, matrix2));
    assert_equal_str("[1|1|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixGreaterThan() {
    let mut matrix1 = "[9| 6| 3]";
    let mut matrix2 = "[1| 4| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", matrix1, matrix2));
    assert_equal_str("[1|1|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixLessOrEqual() {
    let mut matrix1 = "[1| 4| 7]";
    let mut matrix2 = "[1| 4| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", matrix1, matrix2));
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixGreaterOrEqual() {
    let mut matrix1 = "[9| 6| 3]";
    let mut matrix2 = "[1| 4| 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", matrix1, matrix2));
    assert_equal_str("[1|1|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixLogicalAnd() {
    let mut matrix1 = "[1| 0| 1]";
    let mut matrix2 = "[1| 1| 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", matrix1, matrix2));
    assert_equal_str("[1|0|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixLogicalOr() {
    let mut matrix1 = "[1| 0| 1]";
    let mut matrix2 = "[1| 1| 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", matrix1, matrix2));
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixLogicalXor() {
    let mut matrix1 = "[1| 0| 1]";
    let mut matrix2 = "[1| 1| 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", matrix1, matrix2));
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorAddition() {
    let mut matrix = "[1| 2| 3| 4| 5]";
    let mut vector = "[1; 1; 2; 4; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", matrix, vector));
    assert_equal_str("[2|3|5|8|12]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorSubtraction() {
    let mut matrix = "[5| 6| 7| 8| 9]";
    let mut vector = "[1; 1; 2; 4; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", matrix, vector));
    assert_equal_str("[4|5|5|4|2]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorExponentiation() {
    let mut matrix = "[2| 3| 4| 5| 6]";
    let mut vector = "[1; 2; 3; 4; 5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", matrix, vector));
    assert_equal_str("[2|9|64|625|7776]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorDivision() {
    let mut matrix = "[8| 18| 24| 36| 42]";
    let mut vector = "[2; 3; 4; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", matrix, vector));
    assert_equal_str("[4|6|6|6|6]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorMultiplication() {
    let mut matrix = "[4.5| -5.7| 7.1]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "vec = matrix(1; 3)",
        "vec.(1; 1) = -9.2",
        "vec.(1; 2) = 6.3",
        "vec.(1; 3) = 1",
        &format!("{} * vec", matrix),
    ]);
    assert_equal_str("[-41.4 28.35 4.5|52.44 -35.91 -5.7|-65.32 44.73 7.1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorIntegerDivision() {
    let mut matrix = "[9| 12| 15| 18| 21]";
    let mut vector = "[3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", matrix, vector));
    assert_equal_str("[3|3|3|3|3]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorDivisionBar() {
    let mut matrix = "[9| 18| 27| 36| 45]";
    let mut vector = "[1; 2; 3; 4; 5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", matrix, vector));
    assert_equal_str("[9|9|9|9|9]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorRemainder() {
    let mut matrix = "[10| 20| 30| 40| 50]";
    let mut vector = "[3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", matrix, vector));
    assert_equal_str("[1|0|0|4|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorEqual() {
    let mut matrix = "[7| 7| 7| 7| 7]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≡ {}", matrix, vector));
    assert_equal_str("[1|1|1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorNotEqual() {
    let mut matrix = "[7| 8| 9| 10| 11]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≠ {}", matrix, vector));
    assert_equal_str("[0|1|1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorGreaterThan() {
    let mut matrix = "[8| 9| 10| 11| 12]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", matrix, vector));
    assert_equal_str("[1|1|1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorLessThan() {
    let mut matrix = "[5| 6| 7| 8| 9]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", matrix, vector));
    assert_equal_str("[1|1|0|0|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorLessOrEqual() {
    let mut matrix = "[5| 7| 9| 11| 13]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", matrix, vector));
    assert_equal_str("[1|1|0|0|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorGreaterOrEqual() {
    let mut matrix = "[7| 8| 9| 10| 11]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", matrix, vector));
    assert_equal_str("[1|1|1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorLogicalAnd() {
    let mut matrix = "[1| 0| 1| 0| 1]";
    let mut vector = "[1; 1; 0; 0; 1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", matrix, vector));
    assert_equal_str("[1|0|0|0|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorLogicalOr() {
    let mut matrix = "[0| 0| 0| 0| 1]";
    let mut vector = "[1; 1; 1; 0; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", matrix, vector));
    assert_equal_str("[1|1|1|0|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixVectorLogicalXor() {
    let mut matrix = "[1| 1| 0| 0| 1]";
    let mut vector = "[1; 0; 1; 0; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", matrix, vector));
    assert_equal_str("[0|1|1|0|1]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixAddition() {
    let mut matrix = "[1| 2| 3| 4| 5]";
    let mut vector = "[1; 1; 2; 4; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", vector, matrix));
    assert_equal_str("[2|3|5|8|12]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixSubtraction() {
    let mut matrix = "[5| 6| 7| 8| 9]";
    let mut vector = "[1; 1; 2; 4; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", vector, matrix));
    assert_equal_str("[-4|-5|-5|-4|-2]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixExponentiation() {
    let mut matrix = "[2| 3| 4| 5| 6]";
    let mut vector = "[1; 2; 3; 4; 5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", vector, matrix));
    assert_equal_str("[1|8|81|1024|15625]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixDivision() {
    let mut matrix = "[8| 18| 24| 36| 42]";
    let mut vector = "[2; 3; 4; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", vector, matrix));
    assert_equal_str("[0.25|0.167|0.167|0.167|0.167]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixMultiplication() {
    let mut matrix = "[4.5; -3.2| -5.7; 3.4]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "vec = matrix(1; 2)",
        "vec.(1; 1) = -9.2",
        "vec.(1; 2) = 6.3",
        &format!("vec * {}", matrix),
    ]);
    assert_equal_str("[-77.31 50.86]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixIntegerDivision() {
    let mut matrix = "[9| 2| 15| 18| 21]";
    let mut vector = "[3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", vector, matrix));
    assert_equal_str("[0|2|0|0|0]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixDivisionBar() {
    let mut matrix = "[9| 18| 27| 36| 45]";
    let mut vector = "[1; 2; 3; 4; 5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", vector, matrix));
    assert_equal_str("[0.111|0.111|0.111|0.111|0.111]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixRemainder() {
    let mut matrix = "[10| 20| 30| 40| 50]";
    let mut vector = "[3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", vector, matrix));
    assert_equal_str("[3|4|5|6|7]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixGreaterThan() {
    let mut matrix = "[2| 9| 10| 11| 12]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", vector, matrix));
    assert_equal_str("[1|0|0|0|0]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixLessThan() {
    let mut matrix = "[5| 6| 7| 8| 9]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", vector, matrix));
    assert_equal_str("[0|0|0|1|1]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixLessOrEqual() {
    let mut matrix = "[5| 7| 9| 11| 13]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", vector, matrix));
    assert_equal_str("[0|1|1|1|1]", &calc.ToString());
}
#[test]
fn VectorColumnMatrixGreaterOrEqual() {
    let mut matrix = "[7| 8| 9| 10| 11]";
    let mut vector = "[7; 7; 7; 7; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", vector, matrix));
    assert_equal_str("[1|0|0|0|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarAddition() {
    let mut matrix = "[1| 4| 7]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", matrix, x));
    assert_equal_str("[5|8|11]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarSubtraction() {
    let mut matrix = "[5| 8| 11]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", matrix, x));
    assert_equal_str("[1|4|7]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarExponentiation() {
    let mut matrix = "[2| 5| 8]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", matrix, x));
    assert_equal_str("[4|25|64]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarDivision() {
    let mut matrix = "[8| 32| 56]";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", matrix, x));
    assert_equal_str("[1|4|7]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarMultiplication() {
    let mut matrix = "[1| 4| 7]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} * {}", matrix, x));
    assert_equal_str("[4|16|28]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarIntegerDivision() {
    let mut matrix = "[9| 18| 27]";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", matrix, x));
    assert_equal_str("[3|6|9]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarDivisionBar() {
    let mut matrix = "[9| 36| 63]";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", matrix, x));
    assert_equal_str("[1|4|7]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarRemainder() {
    let mut matrix = "[10| 25| 40]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", matrix, x));
    assert_equal_str("[3|4|5]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarEqual() {
    let mut matrix = "[1| 7| 7]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≡ {}", matrix, x));
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarNotEqual() {
    let mut matrix = "[7| 10| 13]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≠ {}", matrix, x));
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarLessThan() {
    let mut matrix = "[5| 8| 11]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", matrix, x));
    assert_equal_str("[1|0|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarGreaterThan() {
    let mut matrix = "[2| 11| 14]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", matrix, x));
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarLessOrEqual() {
    let mut matrix = "[5| 11| 17]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", matrix, x));
    assert_equal_str("[1|0|0]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarGreaterOrEqual() {
    let mut matrix = "[7| 10| 13]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", matrix, x));
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarLogicalAnd() {
    let mut matrix = "[1| 0| 1]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∧ {}", matrix, x));
    assert_equal_str("[1|0|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarLogicalOr() {
    let mut matrix = "[0| 1| 0]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ∨ {}", matrix, x));
    assert_equal_str("[1|1|1]", &calc.ToString());
}
#[test]
fn ColumnMatrixScalarLogicalXor() {
    let mut matrix = "[1| 0| 1]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⊕ {}", matrix, x));
    assert_equal_str("[0|1|0]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixAddition() {
    let mut matrix = "[1| 4| 7]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} + {}", x, matrix));
    assert_equal_str("[5|8|11]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixSubtraction() {
    let mut matrix = "[5| 8| 11]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} - {}", x, matrix));
    assert_equal_str("[-1|-4|-7]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixExponentiation() {
    let mut matrix = "[2| 5| 8]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ^ {}", x, matrix));
    assert_equal_str("[4|32|256]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixDivision() {
    let mut matrix = "[8| 32| 56]";
    let mut x = "8.0";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} / {}", x, matrix));
    assert_equal_str("[1|0.25|0.143]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixMultiplication() {
    let mut matrix = "[1| 4| 7]";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} * {}", x, matrix));
    assert_equal_str("[4|16|28]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixIntegerDivision() {
    let mut matrix = "[9| 18| 27]";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} \\ {}", x, matrix));
    assert_equal_str("[0|0|0]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixDivisionBar() {
    let mut matrix = "[9| 36| 63]";
    let mut x = "9";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ÷ {}", x, matrix));
    assert_equal_str("[1|0.25|0.143]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixRemainder() {
    let mut matrix = "[10| 25| 40]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ⦼ {}", x, matrix));
    assert_equal_str("[7|7|7]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixLessThan() {
    let mut matrix = "[5| 8| 11]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} < {}", x, matrix));
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixGreaterThan() {
    let mut matrix = "[8| 11| 14]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} > {}", x, matrix));
    assert_equal_str("[0|0|0]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixLessOrEqual() {
    let mut matrix = "[5| 11| 17]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≤ {}", x, matrix));
    assert_equal_str("[0|1|1]", &calc.ToString());
}
#[test]
fn ScalarColumnMatrixGreaterOrEqual() {
    let mut matrix = "[7| 10| 13]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("{} ≥ {}", x, matrix));
    assert_equal_str("[1|0|0]", &calc.ToString());
}
