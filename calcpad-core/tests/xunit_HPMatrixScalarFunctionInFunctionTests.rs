#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Small/HPMatrixScalarFunctionInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPMatrixSin() {
    let mut matrix = "hp([30; 45; 60| 90; 120; 135| 150; 180; 210])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(sin(a)*1000)/1000", &format!("f({})", matrix)]);
    assert_equal_str("[0.5 0.707 0.866|1 0.866 0.707|0.5 0 -0.5]", &calc.ToString());
}
#[test]
fn HPMatrixCos() {
    let mut matrix = "hp([0; 30; 45| 60; 90; 120| 150; 180; 210])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(cos(a)*1000)/1000", &format!("f({})", matrix)]);
    assert_equal_str("[1 0.866 0.707|0.5 0 -0.5|-0.866 -1 -0.866]", &calc.ToString());
}
#[test]
fn HPMatrixTan() {
    let mut matrix = "hp([0; 30; 45| 60; 120; 150| 180; 210; 360])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(tan(a)*1000)/1000", &format!("f({})", matrix)]);
    assert_equal_str("[0 0.577 1|1.73 -1.73 -0.577|0 0.577 0]", &calc.ToString());
}
#[test]
fn HPMatrixCot() {
    let mut matrix = "hp([30; 45; 60| 0; 120; 135| 150; 0; 210])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cot(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1.73 1 0.577|+∞ -0.577 -1|-1.73 +∞ 1.73]", &calc.ToString());
}
#[test]
fn HPMatrixSec() {
    let mut matrix = "hp([0; 30; 45| 60; 120; 150| 180; 210; 360])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(sec(a)*100)/100", &format!("f({})", matrix)]);
    assert_equal_str("[1 1.15 1.41|2 -2 -1.15|-1 -1.15 1]", &calc.ToString());
}
#[test]
fn HPMatrixCsc() {
    let mut matrix = "hp([0; 30; 60| 45; 90; 120| 150; 0; 270])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = csc(a)", &format!("f({})", matrix)]);
    assert_equal_str("[+∞ 2 1.15|1.41 1 1.15|2 +∞ -1]", &calc.ToString());
}
#[test]
fn HPMatrixSinh() {
    let mut matrix = "hp([0.1; 0.3; 0.6| 1.2; -1.3; -0.7| 0.4; -0.4; 0.9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sinh(a)", &format!("f({})", matrix)]);
    assert_equal_str("[0.1 0.305 0.637|1.51 -1.7 -0.759|0.411 -0.411 1.03]", &calc.ToString());
}
#[test]
fn HPMatrixCosh() {
    let mut matrix = "hp([-2;-1; 0|1; 1.5; -1.5|0.5; -0.5; 2])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cosh(a)", &format!("f({})", matrix)]);
    assert_equal_str("[3.76 1.54 1|1.54 2.35 2.35|1.13 1.13 3.76]", &calc.ToString());
}
#[test]
fn HPMatrixTanh() {
    let mut matrix = "hp([0; 30; 45| 60; 90; 120| 150; 180; 210])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = tanh(a)", &format!("f({})", matrix)]);
    assert_equal_str("[0 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixCoth() {
    let mut matrix = "hp([0; 30; 45| 60; 90; 120| 150; 180; 210])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = coth(a)", &format!("f({})", matrix)]);
    assert_equal_str("[+∞ 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixSech() {
    let mut matrix = "hp([0.1; 0.3; 0.6| 1.2; -1.3; -0.7| 0.4; -0.4; 0.9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sech(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[0.995 0.957 0.844|0.552 0.507 0.797|0.925 0.925 0.698]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixCsch() {
    let mut matrix = "hp([0; 0.5; 1| -0.5; -1; 0.707| 0.866; -0.866; 1.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = csch(a)", &format!("f({})", matrix)]);
    assert_equal_str("[+∞ 1.92 0.851|-1.92 -0.851 1.3|1.02 -1.02 0.47]", &calc.ToString());
}
#[test]
fn HPMatrixAsin() {
    let mut matrix = "hp([0.5; -0.5; 1| 0.866; -0.866; -1| 0.707; -0.707; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = asin(a)", &format!("f({})", matrix)]);
    assert_equal_str("[30 -30 90|60 -60 -90|44.99 -44.99 0]", &calc.ToString());
}
#[test]
fn HPMatrixAcos() {
    let mut matrix = "hp([0.5; -0.5; 1| 0.866; -0.866; -1| 0.707; -0.707; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = acos(a)", &format!("f({})", matrix)]);
    assert_equal_str("[60 120 0|30 150 180|45.01 134.99 90]", &calc.ToString());
}
#[test]
fn HPMatrixAtan() {
    let mut matrix = "hp([0.2588; 0.5; -0.2588| 0.866; -0.866; -0.5| 0.707; 0; -0.707])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = atan(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[14.51 26.57 -14.51|40.89 -40.89 -26.57|35.26 0 -35.26]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAcsc() {
    let mut matrix = "hp([0.2588; 0.5; -0.2588| 0.866; -0.866; -0.5| 0.707; 0; -0.707])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = acsc(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[ Undefined   Undefined   Undefined | Undefined   Undefined   Undefined | Undefined  +∞  Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAsec() {
    let mut matrix = "hp([0.577; -0.577; 1| 0.577; 0.707; -1| 0; -1; 0.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = asec(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[ Undefined   Undefined  0| Undefined   Undefined  180|+∞ 180  Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAcot() {
    let mut matrix = "hp([0.577; -0.577; 1| 0.577; 0.707; -1| 0; -1; 0.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = acot(a)", &format!("f({})", matrix)]);
    assert_equal_str("[60.02 -60.02 45|60.02 54.74 -45|90 -45 63.43]", &calc.ToString());
}
#[test]
fn HPMatrixAtan2() {
    let mut matrix1 = "hp([0.577; -0.577; 1| 0.577; 0.707; -1| 0; -1; 0.5])";
    let mut matrix2 = "hp([0.5; -0.5; 1| 0.866; -0.866; -1| 0.707; -0.707; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = atan2(a; b)", &format!("f({}; {})", matrix1, matrix2)]);
    assert_equal_str("[40.91 -139.09 45|56.33 -50.77 -135|90 -144.74 0]", &calc.ToString());
}
#[test]
fn HPMatrixAsinh() {
    let mut matrix = "hp([1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = asinh(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[1.02 0.481 -0.481|0.95 -1.02 0.733|0.809 -0.809 1.19]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAcosh() {
    let mut matrix = "hp([1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = acosh(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[0.622  Undefined   Undefined |0.444  Undefined   Undefined | Undefined   Undefined  0.962]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAtanh() {
    let mut matrix = "hp([1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = atanh(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[ Undefined  0.549 -0.549| Undefined   Undefined  1.1|1.47 -1.47  Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAcsch() {
    let mut matrix = "hp([1.2; 0.5; -0.5| 1.1; -1.2; 0.8| 0.9; -0.9; 1.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = acsch(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[0.758 1.44 -1.44|0.816 -0.758 1.05|0.958 -0.958 0.625]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAsech() {
    let mut matrix = "hp([0.7; 0.6; -0.6| 1.3; -1.5; 0.4| 0.2; -0.3; 1.1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = asech(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[0.896 1.1  Undefined | Undefined   Undefined  1.57|2.29  Undefined   Undefined ]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixAcoth() {
    let mut matrix = "hp([0.7; 0.6; -0.6| 1.3; -1.5; 0.4| 0.2; -0.3; 1.1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = acoth(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[ Undefined   Undefined   Undefined |1.02 -0.805  Undefined | Undefined   Undefined  1.52]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixLn() {
    let mut matrix = "hp([0.1; 0.5; 0.9| 1.5; 2.5; 5.0| 7.5; 10; 15])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = ln(a)", &format!("f({})", matrix)]);
    assert_equal_str("[-2.3 -0.693 -0.105|0.405 0.916 1.61|2.01 2.3 2.71]", &calc.ToString());
}
#[test]
fn HPMatrixLog() {
    let mut matrix = "hp([1; 10; 100| 2; 20; 200| 5; 50; 500])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = log(a)", &format!("f({})", matrix)]);
    assert_equal_str("[0 1 2|0.301 1.3 2.3|0.699 1.7 2.7]", &calc.ToString());
}
#[test]
fn HPMatrixLog2() {
    let mut matrix = "hp([1; 2; 4| 8; 16; 32| 64; 128; 256])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = log_2(a)", &format!("f({})", matrix)]);
    assert_equal_str("[0 1 2|3 4 5|6 7 8]", &calc.ToString());
}
#[test]
fn HPMatrixExp() {
    let mut matrix = "hp([2; 3; 4| 1; 5; 6| 2.5; 3.5; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = exp(a)", &format!("f({})", matrix)]);
    assert_equal_str(
        "[7.39 20.09 54.6|2.72 148.41 403.43|12.18 33.12 1096.63]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixSqr() {
    let mut matrix = "hp([1; 2; 3| 4; 5; 6| 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sqr(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 1.41 1.73|2 2.24 2.45|2.65 2.83 3]", &calc.ToString());
}
#[test]
fn HPMatrixSqrt() {
    let mut matrix = "hp([1; 2; 3| 4; 5; 6| 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sqrt(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 1.41 1.73|2 2.24 2.45|2.65 2.83 3]", &calc.ToString());
}
#[test]
fn HPMatrixCbrt() {
    let mut matrix = "hp([1; 2; 3| 4; 5; 6| 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cbrt(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 1.26 1.44|1.59 1.71 1.82|1.91 2 2.08]", &calc.ToString());
}
#[test]
fn HPMatrixRoot() {
    let mut matrix = "hp([1; 2; 3| 4; 5; 6| 7; 8; 9])";
    let mut x = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = root(a; b)", &format!("f({}; {})", matrix, x)]);
    assert_equal_str("[1 1.26 1.44|1.59 1.71 1.82|1.91 2 2.08]", &calc.ToString());
}
#[test]
fn HPMatrixRound() {
    let mut matrix = "hp([1.4; 2.5; 3.5| 4.7; 5.8; 6.11| 7.12; 8.34; 9.7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 3 4|5 6 6|7 8 10]", &calc.ToString());
}
#[test]
fn HPMatrixFloor() {
    let mut matrix = "hp([1.7; 2.8; 3.9| 4.1; 5.3; 6.4| 7.7; 8.9; 9.99])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = floor(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn HPMatrixCeiling() {
    let mut matrix = "hp([1.4; 2.3; 3.7| 4.5; 5.4; 6.9| 7.8; 8.2; 9.1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = ceiling(a)", &format!("f({})", matrix)]);
    assert_equal_str("[2 3 4|5 6 7|8 9 10]", &calc.ToString());
}
#[test]
fn HPMatrixTrunc() {
    let mut matrix = "hp([1.4; 2.3; 3.2| 4.8; 5.9; 6.12| 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = trunc(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 2 3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn HPMatrixMod() {
    let mut matrix = "hp([1; 2; 3| 4; 5; 6| 7; 8; 9])";
    let mut n = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mod(a; b)", &format!("f({}; {})", matrix, n)]);
    assert_equal_str("[1 2 3|4 5 6|0 1 2]", &calc.ToString());
}
#[test]
fn HPMatrixGcd() {
    let mut matrix = "hp([1; 2; 3| 4; 5; 6| 7; 8; 9])";
    let mut n = "6";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = gcd(a; b)", &format!("f({}; {})", matrix, n)]);
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn HPMatrixLcm() {
    let mut matrix = "hp([1; 2; 3| 4; 5; 6| 7; 8; 9])";
    let mut n = "12";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = lcm(a; b)", &format!("f({}; {})", matrix, n)]);
    assert_equal_str("2520", &calc.ToString());
}
#[test]
fn HPMatrixSign() {
    let mut matrix = "hp([1; -2; -3| 4; 5; 6| 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sign(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 -1 -1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixGetUnits() {
    let mut matrix = "hp([1dm; -2dm; -3dm| 4dm; 5dm; 6dm| 7dm; 8dm; 9dm])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = getunits(a)", &format!("f({})", matrix)]);
    assert_equal_str("1dm", &calc.ToString());
}
#[test]
fn HPMatrixSetUnits() {
    let mut matrix = "hp([1; -2; -3| 4; 5; 6| 7; 8; 9])";
    let mut unit = "cm";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = setunits(a; b)", &format!("f({};{})", matrix, unit)]);
    assert_equal_str("[1cm -2cm -3cm|4cm 5cm 6cm|7cm 8cm 9cm]", &calc.ToString());
}
#[test]
fn HPMatrixClrUnits() {
    let mut matrix = "hp([1dm; -2dm; -3dm| 4dm; 5dm; 6dm| 7dm; 8dm; 9dm])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = clrunits(a)", &format!("f({})", matrix)]);
    assert_equal_str("[1 -2 -3|4 5 6|7 8 9]", &calc.ToString());
}
#[test]
fn HPMatrixIsHP() {
    let mut matrix = "hp([1; -2; -3| 4; 5; 6| 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = isHp(a)", &format!("f({})", matrix)]);
    assert_equal_str("1", &calc.ToString());
}
