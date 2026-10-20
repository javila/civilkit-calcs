#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Vectors/HP/HPVectorScalarFunctionInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPVectorSin() {
    let mut vector = "hp([30; 60; 45; 90; 120; 180; 270])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(sin(a)*1000)/1000", &format!("f({})", vector)]);
    assert_equal_str("[0.5 0.866 0.707 1 0.866 0 -1]", &calc.ToString());
}
#[test]
fn HPVectorCos() {
    let mut vector = "hp([30; 60; 45; 90; 120; 180; 270])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(cos(a)*1000)/1000", &format!("f({})", vector)]);
    assert_equal_str("[0.866 0.5 0.707 0 -0.5 -1 0]", &calc.ToString());
}
#[test]
fn HPVectorTan() {
    let mut vector = "hp([0; 15; 30; 45; 60; 75; 89])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(tan(a)*1000)/1000", &format!("f({})", vector)]);
    assert_equal_str("[0 0.268 0.577 1 1.73 3.73 57.29]", &calc.ToString());
}
#[test]
fn HPVectorCot() {
    let mut vector = "hp([1; 45; 135; 30; 60; 150; 210])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cot(a)", &format!("f({})", vector)]);
    assert_equal_str("[57.29 1 -1 1.73 0.577 -1.73 1.73]", &calc.ToString());
}
#[test]
fn HPVectorSec() {
    let mut vector = "hp([1; 60; 120; 180; 45; 30; 150])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(Sec(a)*100)/100", &format!("f({})", vector)]);
    assert_equal_str("[1 2 -2 -1 1.41 1.15 -1.15]", &calc.ToString());
}
#[test]
fn HPVectorCsc() {
    let mut vector = "hp([1; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = csc(a)", &format!("f({})", vector)]);
    assert_equal_str("[57.3 28.65 19.11 14.34 11.47 9.57 8.21]", &calc.ToString());
}
#[test]
fn HPVectorSinh() {
    let mut vector = "hp([0; 1; 2; 3; 4; 5; 6])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sinh(a)", &format!("f({})", vector)]);
    assert_equal_str("[0 1.18 3.63 10.02 27.29 74.2 201.71]", &calc.ToString());
}
#[test]
fn HPVectorCosh() {
    let mut vector = "hp([0; 1; 2; 3; 4; 5; 6])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cosh(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 1.54 3.76 10.07 27.31 74.21 201.72]", &calc.ToString());
}
#[test]
fn HPVectorTanh() {
    let mut vector = "hp([30; 60; 45; 90; 120; 180; 270])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Tanh(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPVectorCoth() {
    let mut vector = "hp([30; 60; 45; 90; 120; 180; 270])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Coth(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn HPVectorSech() {
    let mut vector = "hp([0; 1; 2; 3; 4; 5; 6])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Sech(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 0.648 0.266 0.0993 0.0366 0.0135 0.00496]", &calc.ToString());
}
#[test]
fn HPVectorCsch() {
    let mut vector = "hp([1; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Csch(a)", &format!("f({})", vector)]);
    assert_equal_str("[0.851 0.276 0.0998 0.0366 0.0135 0.00496 0.00182]", &calc.ToString());
}
#[test]
fn HPVectorAsin() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Asin(a)", &format!("f({})", vector)]);
    assert_equal_str("[30 60 90 -30 -60 -90 0]", &calc.ToString());
}
#[test]
fn HPVectorAcos() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Acos(a)", &format!("f({})", vector)]);
    assert_equal_str("[60 30 0 120 150 180 90]", &calc.ToString());
}
#[test]
fn HPVectorAtan() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Atan(a)", &format!("f({})", vector)]);
    assert_equal_str("[26.57 40.89 45 -26.57 -40.89 -45 0]", &calc.ToString());
}
#[test]
fn HPVectorAcsc() {
    let mut vector = "hp([-3.5; 7.8; 4.25; π/3; 0; 10; -2.3; 6.75])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Acsc(a)", &format!("f({})", vector)]);
    assert_equal_str("[-16.6 7.37 13.61 72.73 +∞ 5.74 -25.77 8.52]", &calc.ToString());
}
#[test]
fn HPVectorAsec() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Asec(a)", &format!("f({})", vector)]);
    assert_equal_str(
        "[ Undefined   Undefined  0  Undefined   Undefined  180 +∞]",
        &calc.ToString(),
    );
}
#[test]
fn HPVectorAcot() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Acot(a)", &format!("f({})", vector)]);
    assert_equal_str("[63.43 49.11 45 -63.43 -49.11 -45 90]", &calc.ToString());
}
#[test]
fn HPVectorAtan2() {
    let mut vector1 = "hp([-3.5; 7.8; 4.25; 0; 10; -2.3; 6.75])";
    let mut vector2 = "hp([-3.5; 7.8; 4.25; π/4; 0; 10; -2.3; 6.75])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = Atan2(a; b)", &format!("f({}; {})", vector1, vector2)]);
    assert_equal_str("[-135 45 45 90 0 102.95 -18.82 90]", &calc.ToString());
}
#[test]
fn HPVectorAsinh() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Asinh(a)", &format!("f({})", vector)]);
    assert_equal_str("[0.481 0.783 0.881 -0.481 -0.783 -0.881 0]", &calc.ToString());
}
#[test]
fn HPVectorAcosh() {
    let mut vector = "hp([1; 1.5; 2; 3; 4; 5; 6])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Acosh(a)", &format!("f({})", vector)]);
    assert_equal_str("[0 0.962 1.32 1.76 2.06 2.29 2.48]", &calc.ToString());
}
#[test]
fn HPVectorAtanh() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Atanh(a)", &format!("f({})", vector)]);
    assert_equal_str("[0.549 1.32 +∞ -0.549 -1.32 -∞ 0]", &calc.ToString());
}
#[test]
fn HPVectorAcsch() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Acsch(a)", &format!("f({})", vector)]);
    assert_equal_str("[1.44 0.987 0.881 -1.44 -0.987 -0.881 +∞]", &calc.ToString());
}
#[test]
fn HPVectorAsech() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Asech(a)", &format!("f({})", vector)]);
    assert_equal_str(
        "[1.32 0.549 0  Undefined   Undefined   Undefined  +∞]",
        &calc.ToString(),
    );
}
#[test]
fn HPVectorAcoth() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = Acoth(a)", &format!("f({})", vector)]);
    assert_equal_str("[0.805 0.549 0.347 0.255 0.203 0.168 0.144]", &calc.ToString());
}
#[test]
fn HPVectorLn() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = ln(a)", &format!("f({})", vector)]);
    assert_equal_str("[0.405 0.693 1.1 1.39 1.61 1.79 1.95]", &calc.ToString());
}
#[test]
fn HPVectorLog() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = log(a)", &format!("f({})", vector)]);
    assert_equal_str("[0.176 0.301 0.477 0.602 0.699 0.778 0.845]", &calc.ToString());
}
#[test]
fn HPVectorLog2() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = log_2(a)", &format!("f({})", vector)]);
    assert_equal_str("[0.585 1 1.58 2 2.32 2.58 2.81]", &calc.ToString());
}
#[test]
fn HPVectorExp() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = exp(a)", &format!("f({})", vector)]);
    assert_equal_str("[4.48 7.39 20.09 54.6 148.41 403.43 1096.63]", &calc.ToString());
}
#[test]
fn HPVectorSqr() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sqr(a)", &format!("f({})", vector)]);
    assert_equal_str("[1.22 1.41 1.73 2 2.24 2.45 2.65]", &calc.ToString());
}
#[test]
fn HPVectorSqrt() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sqrt(a)", &format!("f({})", vector)]);
    assert_equal_str("[1.22 1.41 1.73 2 2.24 2.45 2.65]", &calc.ToString());
}
#[test]
fn HPVectorCbrt() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cbrt(a)", &format!("f({})", vector)]);
    assert_equal_str("[1.14 1.26 1.44 1.59 1.71 1.82 1.91]", &calc.ToString());
}
#[test]
fn HPVectorRoot() {
    let mut vector = "hp([1.5; 2; 3; 4; 5; 6; 7])";
    let mut x = 3.0;
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = root(a; b)", &format!("f({}; {})", vector, x)]);
    assert_equal_str("[1.14 1.26 1.44 1.59 1.71 1.82 1.91]", &calc.ToString());
}
#[test]
fn HPVectorRound() {
    let mut vector = "hp([1.14; 1.26; 1.44; 1.59; 1.71; 1.82; 1.91])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = round(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 1 1 2 2 2 2]", &calc.ToString());
}
#[test]
fn HPVectorFloor() {
    let mut vector = "hp([1.7; 2.8; 3.4; 4.9; 5.12; 6.91; 7.25])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = floor(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 2 3 4 5 6 7]", &calc.ToString());
}
#[test]
fn HPVectorCeiling() {
    let mut vector = "hp([1.5; 2.1; 3.2; 4.4; 5.5; 6.9; 7.111])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = ceiling(a)", &format!("f({})", vector)]);
    assert_equal_str("[2 3 4 5 6 7 8]", &calc.ToString());
}
#[test]
fn HPVectorTrunc() {
    let mut vector = "hp([3.7; -4.9; 5.0; -2.0; 0.1; -0.32; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = trunc(a)", &format!("f({})", vector)]);
    assert_equal_str("[3 -4 5 -2 0 0 7]", &calc.ToString());
}
#[test]
fn HPVectorMod() {
    let mut vector = "hp([10; -15; 20; -25; 30; -35; 40])";
    let mut n = 7.0;
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mod(a; b)", &format!("f({}; {})", vector, n)]);
    assert_equal_str("[3 -1 6 -4 2 0 5]", &calc.ToString());
}
#[test]
fn HPVectorGcd() {
    let mut vector = "hp([12; 15; 9; 18; 21; 24; 30])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = gcd(a)", &format!("f({})", vector)]);
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn HPVectorLcm() {
    let mut vector = "hp([4; 5; 6; 7; 8; 9; 10])";
    let mut n = 12.0;
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = lcm(a; b)", &format!("f({}; {})", vector, n)]);
    assert_equal_str("2520", &calc.ToString());
}
#[test]
fn HPVectorAbs() {
    let mut vector = "hp([3.7; -4.9; 5.0; -2.0; 0.1; -0.32; 7.5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = abs(a)", &format!("f({})", vector)]);
    assert_equal_str("[3.7 4.9 5 2 0.1 0.32 7.5]", &calc.ToString());
}
#[test]
fn HPVectorSign() {
    let mut vector = "hp([3.7; -4.9; 5.0; -2.0; 0.1; -0.32; 0])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sign(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 -1 1 -1 1 -1 0]", &calc.ToString());
}
#[test]
fn HPVectorGetUnits() {
    let mut vector = "hp([1dm; -2dm; -3dm; 4dm; 5dm; 6dm; 7dm; 8dm; 9dm])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = getunits(a)", &format!("f({})", vector)]);
    assert_equal_str("1dm", &calc.ToString());
}
#[test]
fn HPVectorSetUnits() {
    let mut vector = "hp([1; -2; -3; 4; 5; 6; 7; 8; 9])";
    let mut unit = "cm";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = setunits(a; b)", &format!("f({};{})", vector, unit)]);
    assert_equal_str("[1cm -2cm -3cm 4cm 5cm 6cm 7cm 8cm 9cm]", &calc.ToString());
}
#[test]
fn HPVectorClrUnits() {
    let mut vector = "hp([1dm; -2dm; -3dm; 4dm; 5dm; 6dm; 7dm; 8dm; 9dm])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = clrunits(a)", &format!("f({})", vector)]);
    assert_equal_str("[1 -2 -3 4 5 6 7 8 9]", &calc.ToString());
}
#[test]
fn HPVectorIsHP() {
    let mut vector = "hp([1; -2; -3; 4; 5; 6; 7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = isHp(a)", &format!("f({})", vector)]);
    assert_equal_str("1", &calc.ToString());
}
