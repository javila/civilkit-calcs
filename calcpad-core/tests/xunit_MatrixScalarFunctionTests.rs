#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/MatrixScalarFunctionTests.cs by tools/transcribe_xunit.py.
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
fn MatrixSin() {
    let mut matrix = "[30; 45; 60| 90; 120; 135| 150; 180; 210]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(Sin({})*1000)/1000", matrix));
    assert_equal_str("[0.5 0.707 0.866|1 0.866 0.707|0.5 0 -0.5]", &calc.ToString());
}
#[test]
fn MatrixCos() {
    let mut matrix = "[0; 30; 45| 60; 90; 120| 150; 180; 210]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(Cos({})*1000)/1000", matrix));
    assert_equal_str("[1 0.866 0.707|0.5 0 -0.5|-0.866 -1 -0.866]", &calc.ToString());
}
#[test]
fn MatrixTan() {
    let mut matrix = "[0; 30; 45| 60; 120; 150| 180; 210; 360]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(Tan({})*1000)/1000", matrix));
    assert_equal_str("[0 0.577 1|1.73 -1.73 -0.577|0 0.577 0]", &calc.ToString());
}
#[test]
fn MatrixCot() {
    let mut matrix = "[30; 45; 60| 0; 120; 135| 150; 0; 210]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Cot({})", matrix));
    assert_equal_str("[1.73 1 0.577|+∞ -0.577 -1|-1.73 +∞ 1.73]", &calc.ToString());
}
#[test]
fn MatrixSec() {
    let mut matrix = "[0; 30; 45| 60; 120; 150| 180; 210; 360]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(sec({})*100)/100", matrix));
    assert_equal_str("[1 1.15 1.41|2 -2 -1.15|-1 -1.15 1]", &calc.ToString());
}
#[test]
fn MatrixCsc() {
    let mut matrix = "[0; 30; 60| 45; 90; 120| 150; 0; 270]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Csc({})", matrix));
    assert_equal_str("[+∞ 2 1.15|1.41 1 1.15|2 +∞ -1]", &calc.ToString());
}
#[test]
fn MatrixSinh() {
    let mut matrix = "[0.1; 0.3; 0.6| 1.2; -1.3; -0.7| 0.4; -0.4; 0.9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Sinh({})", matrix));
    assert_equal_str("[0.1 0.305 0.637|1.51 -1.7 -0.759|0.411 -0.411 1.03]", &calc.ToString());
}
#[test]
fn MatrixCosh() {
    let mut matrix = "[-2;-1; 0|1; 1.5; -1.5|0.5; -0.5; 2]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Cosh({})", matrix));
    assert_equal_str("[3.76 1.54 1|1.54 2.35 2.35|1.13 1.13 3.76]", &calc.ToString());
}
#[test]
fn MatrixTanh() {
    let mut matrix = "[0.1; 0.3; 0.6| 1.2; -1.3; -0.7| 0.4; -0.4; 0.9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Tanh({})", matrix));
    assert_equal_str(
        "[0.0997 0.291 0.537|0.834 -0.862 -0.604|0.38 -0.38 0.716]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixCoth() {
    let mut matrix = "[0.1; 0.3; 0.6| 1.2; -1.3; -0.7| 0.4; -0.4; 0.9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Coth({})", matrix));
    assert_equal_str("[10.03 3.43 1.86|1.2 -1.16 -1.65|2.63 -2.63 1.4]", &calc.ToString());
}
#[test]
fn MatrixSech() {
    let mut matrix = "[0.1; 0.3; 0.6| 1.2; -1.3; -0.7| 0.4; -0.4; 0.9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Sech({})", matrix));
    assert_equal_str(
        "[0.995 0.957 0.844|0.552 0.507 0.797|0.925 0.925 0.698]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixCsch() {
    let mut matrix = "[0; 0.5; 1| -0.5; -1; 0.707| 0.866; -0.866; 1.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Csch({})", matrix));
    assert_equal_str("[+∞ 1.92 0.851|-1.92 -0.851 1.3|1.02 -1.02 0.47]", &calc.ToString());
}
#[test]
fn MatrixAsin() {
    let mut matrix = "[0.5; -0.5; 1| 0.866; -0.866; -1| 0.707; -0.707; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asin({})", matrix));
    assert_equal_str("[30 -30 90|60 -60 -90|44.99 -44.99 0]", &calc.ToString());
}
#[test]
fn MatrixAcos() {
    let mut matrix = "[0.5; -0.5; 1| 0.866; -0.866; -1| 0.707; -0.707; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acos({})", matrix));
    assert_equal_str("[60 120 0|30 150 180|45.01 134.99 90]", &calc.ToString());
}
#[test]
fn MatrixAtan() {
    let mut matrix = "[0.2588; 0.5; -0.2588| 0.866; -0.866; -0.5| 0.707; 0; -0.707]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Atan({})", matrix));
    assert_equal_str(
        "[14.51 26.57 -14.51|40.89 -40.89 -26.57|35.26 0 -35.26]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAcsc() {
    let mut matrix = "[0.2588; 0.5; -0.2588| 0.866; -0.866; -0.5| 0.707; 0; -0.707]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acsc({})", matrix));
    assert_equal_str(
        "[ Undefined   Undefined   Undefined | Undefined   Undefined   Undefined | Undefined  +∞  Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAsec() {
    let mut matrix = "[0.577; -0.577; 1| 0.577; 0.707; -1| 0; -1; 0.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asec({})", matrix));
    assert_equal_str(
        "[ Undefined   Undefined  0| Undefined   Undefined  180|+∞ 180  Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAcot() {
    let mut matrix = "[0.577; -0.577; 1| 0.577; 0.707; -1| 0; -1; 0.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acot({})", matrix));
    assert_equal_str("[60.02 -60.02 45|60.02 54.74 -45|90 -45 63.43]", &calc.ToString());
}
#[test]
fn MatrixAtan2() {
    let mut matrix1 = "[0.577; -0.577; 1| 0.577; 0.707; -1| 0; -1; 0.5]";
    let mut matrix2 = "[0.5; -0.5; 1| 0.866; -0.866; -1| 0.707; -0.707; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Atan2({}; {})", matrix1, matrix2));
    assert_equal_str("[40.91 -139.09 45|56.33 -50.77 -135|90 -144.74 0]", &calc.ToString());
}
#[test]
fn MatrixAsinh() {
    let mut matrix = "[1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asinh({})", matrix));
    assert_equal_str(
        "[1.02 0.481 -0.481|0.95 -1.02 0.733|0.809 -0.809 1.19]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAcosh() {
    let mut matrix = "[1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acosh({})", matrix));
    assert_equal_str(
        "[0.622  Undefined   Undefined |0.444  Undefined   Undefined | Undefined   Undefined  0.962]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAtanh() {
    let mut matrix = "[1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Atanh({})", matrix));
    assert_equal_str(
        "[ Undefined  0.549 -0.549| Undefined   Undefined  1.1|1.47 -1.47  Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAcsch() {
    let mut matrix = "[1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acsch({})", matrix));
    assert_equal_str(
        "[0.758 1.44 -1.44|0.816 -0.758 1.05|0.958 -0.958 0.625]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAsech() {
    let mut matrix = "[0.7; 0.6; -0.6| 1.3; -1.5; 0.4| 0.2; -0.3; 1.1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asech({})", matrix));
    assert_equal_str(
        "[0.896 1.1  Undefined | Undefined   Undefined  1.57|2.29  Undefined   Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixAcoth() {
    let mut matrix = "[0.7; 0.6; -0.6| 1.3; -1.5; 0.4| 0.2; -0.3; 1.1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acoth({})", matrix));
    assert_equal_str(
        "[ Undefined   Undefined   Undefined |1.02 -0.805  Undefined | Undefined   Undefined  1.52]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixLn() {
    let mut matrix = "[0.1; 0.5; 0.9| 1.5; 2.5; 5.0| 7.5; 10; 15]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("ln({})", matrix));
    assert_equal_str("[-2.3 -0.693 -0.105|0.405 0.916 1.61|2.01 2.3 2.71]", &calc.ToString());
}
#[test]
fn MatrixLog() {
    let mut matrix = "[1; 10; 100| 2; 20; 200| 5; 50; 500]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("log({})", matrix));
    assert_equal_str("[0 1 2|0.301 1.3 2.3|0.699 1.7 2.7]", &calc.ToString());
}
#[test]
fn MatrixLog2() {
    let mut matrix = "[1; 2; 4| 8; 16; 32| 64; 128; 256]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("log_2({})", matrix));
    assert_equal_str("[0 1 2|3 4 5|6 7 8]", &calc.ToString());
}
#[test]
fn MatrixExp() {
    let mut matrix = "[2; 3; 4| 1; 5; 6| 2.5; 3.5; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("exp({})", matrix));
    assert_equal_str(
        "[7.39 20.09 54.6|2.72 148.41 403.43|12.18 33.12 1096.63]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixSqr() {
    let mut matrix = "[1; 2; 3| 4; 5; 6| 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sqr({})", matrix));
    assert_equal_str("[1 1.41 1.73|2 2.24 2.45|2.65 2.83 3]", &calc.ToString());
}
#[test]
fn MatrixSqrt() {
    let mut matrix = "[1; 2; 3| 4; 5; 6| 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sqrt({})", matrix));
    assert_equal_str("[1 1.41 1.73|2 2.24 2.45|2.65 2.83 3]", &calc.ToString());
}
#[test]
fn MatrixCbrt() {
    let mut matrix = "[1; 2; 3| 4; 5; 6| 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cbrt({})", matrix));
    assert_equal_str("[1 1.26 1.44|1.59 1.71 1.82|1.91 2 2.08]", &calc.ToString());
}
#[test]
fn MatrixRoot() {
    let mut matrix = "[1; 2; 3| 4; 5; 6| 7; 8; 9]";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("root({}; {})", matrix, x));
    assert_equal_str("[1 1.26 1.44|1.59 1.71 1.82|1.91 2 2.08]", &calc.ToString());
}
#[test]
fn MatrixRound() {
    let mut matrix = "[1.4; 2.5; 3.5| 4.7; 5.8; 6.11| 7.12; 8.34; 9.7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round({})", matrix));
    assert_equal_str("[1 3 4|5 6 6|7 8 10]", &calc.ToString());
}
#[test]
fn MatrixFloor() {
    let mut matrix = "[1.7; 2.8; 3.9| 4.1; 5.3; 6.4| 7.7; 8.9; 9.99]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("floor({})", matrix));
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn MatrixCeiling() {
    let mut matrix = "[1.4; 2.3; 3.7| 4.5; 5.4; 6.9| 7.8; 8.2; 9.1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("ceiling({})", matrix));
    assert_equal_str("[2 3 4|5 6 7|8 9 10]", &calc.ToString());
}
#[test]
fn MatrixTrunc() {
    let mut matrix = "[1.4; 2.3; 3.2| 4.8; 5.9; 6.12| 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("trunc({})", matrix));
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn MatrixMod() {
    let mut matrix = "[1; 2; 3| 4; 5; 6| 7; 8; 9]";
    let mut n = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mod({}; {})", matrix, n));
    assert_equal_str("[1 2 3|4 5 6|0 1 2]", &calc.ToString());
}
#[test]
fn MatrixGcd() {
    let mut matrix = "[1; 2; 3| 4; 5; 6| 7; 8; 9]";
    let mut n = "6";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("gcd({}; {})", matrix, n));
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn MatrixLcm() {
    let mut matrix = "[1; 2; 3| 4; 5; 6| 7; 8; 9]";
    let mut n = "12";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lcm({}; {})", matrix, n));
    assert_equal_str("2520", &calc.ToString());
}
#[test]
fn MatrixSign() {
    let mut matrix = "[1; -2; -3| 4; 5; 6| 7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sign({})", matrix));
    assert_equal_str("[1 -1 -1|1 1 1|1 1 1]", &calc.ToString());
}
