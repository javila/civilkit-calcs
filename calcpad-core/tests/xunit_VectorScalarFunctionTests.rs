#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Vectors/VectorScalarFunctionTests.cs by tools/transcribe_xunit.py.
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
fn VectorSin() {
    let mut vector = "[30; 60; 45; 90; 120; 180; 270]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(Sin({})*1000)/1000", vector));
    assert_equal_str("[0.5 0.866 0.707 1 0.866 0 -1]", &calc.ToString());
}
#[test]
fn VectorCos() {
    let mut vector = "[30; 60; 45; 90; 120; 180; 270]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(Cos({})*1000)/1000", vector));
    assert_equal_str("[0.866 0.5 0.707 0 -0.5 -1 0]", &calc.ToString());
}
#[test]
fn VectorTan() {
    let mut vector = "[0; 15; 30; 45; 60; 75; 89]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(Tan({})*1000)/1000", vector));
    assert_equal_str("[0 0.268 0.577 1 1.73 3.73 57.29]", &calc.ToString());
}
#[test]
fn VectorCot() {
    let mut vector = "[1; 45; 135; 30; 60; 150; 210]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Cot({})", vector));
    assert_equal_str("[57.29 1 -1 1.73 0.577 -1.73 1.73]", &calc.ToString());
}
#[test]
fn VectorSec() {
    let mut vector = "[1; 60; 120; 180; 45; 30; 150]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round(Sec({})*100)/100", vector));
    assert_equal_str("[1 2 -2 -1 1.41 1.15 -1.15]", &calc.ToString());
}
#[test]
fn VectorCsc() {
    let mut vector = "[1; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Csc({})", vector));
    assert_equal_str("[57.3 28.65 19.11 14.34 11.47 9.57 8.21]", &calc.ToString());
}
#[test]
fn VectorSinh() {
    let mut vector = "[0; 1; 2; 3; 4; 5; 6]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Sinh({})", vector));
    assert_equal_str("[0 1.18 3.63 10.02 27.29 74.2 201.71]", &calc.ToString());
}
#[test]
fn VectorCosh() {
    let mut vector = "[0; 1; 2; 3; 4; 5; 6]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Cosh({})", vector));
    assert_equal_str("[1 1.54 3.76 10.07 27.31 74.21 201.72]", &calc.ToString());
}
#[test]
fn VectorTanh() {
    let mut vector = "[30; 60; 45; 90; 120; 180; 270]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Tanh({})", vector));
    assert_equal_str("[1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn VectorCoth() {
    let mut vector = "[30; 60; 45; 90; 120; 180; 270]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Coth({})", vector));
    assert_equal_str("[1 1 1 1 1 1 1]", &calc.ToString());
}
#[test]
fn VectorSech() {
    let mut vector = "[0; 1; 2; 3; 4; 5; 6]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Sech({})", vector));
    assert_equal_str("[1 0.648 0.266 0.0993 0.0366 0.0135 0.00496]", &calc.ToString());
}
#[test]
fn VectorCsch() {
    let mut vector = "[1; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Csch({})", vector));
    assert_equal_str("[0.851 0.276 0.0998 0.0366 0.0135 0.00496 0.00182]", &calc.ToString());
}
#[test]
fn VectorAsin() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asin({})", vector));
    assert_equal_str("[30 60 90 -30 -60 -90 0]", &calc.ToString());
}
#[test]
fn VectorAcos() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acos({})", vector));
    assert_equal_str("[60 30 0 120 150 180 90]", &calc.ToString());
}
#[test]
fn VectorAtan() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Atan({})", vector));
    assert_equal_str("[26.57 40.89 45 -26.57 -40.89 -45 0]", &calc.ToString());
}
#[test]
fn VectorAcsc() {
    let mut vector = "[-3.5; 7.8; 4.25; π/3; 0; 10; -2.3; 6.75]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acsc({})", vector));
    assert_equal_str("[-16.6 7.37 13.61 72.73 +∞ 5.74 -25.77 8.52]", &calc.ToString());
}
#[test]
fn VectorAsec() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asec({})", vector));
    assert_equal_str(
        "[ Undefined   Undefined  0  Undefined   Undefined  180 +∞]",
        &calc.ToString(),
    );
}
#[test]
fn VectorAcot() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acot({})", vector));
    assert_equal_str("[63.43 49.11 45 -63.43 -49.11 -45 90]", &calc.ToString());
}
#[test]
fn VectorAtan2() {
    let mut vector = "[-3.5; 7.8; 4.25; 0; 10; -2.3; 6.75]";
    let mut vector2 = "[-3.5; 7.8; 4.25; π/4; 0; 10; -2.3; 6.75]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Atan2({}; {})", vector, vector2));
    assert_equal_str("[-135 45 45 90 0 102.95 -18.82 90]", &calc.ToString());
}
#[test]
fn VectorAsinh() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asinh({})", vector));
    assert_equal_str("[0.481 0.783 0.881 -0.481 -0.783 -0.881 0]", &calc.ToString());
}
#[test]
fn VectorAcosh() {
    let mut vector = "[1; 1.5; 2; 3; 4; 5; 6]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acosh({})", vector));
    assert_equal_str("[0 0.962 1.32 1.76 2.06 2.29 2.48]", &calc.ToString());
}
#[test]
fn VectorAtanh() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Atanh({})", vector));
    assert_equal_str("[0.549 1.32 +∞ -0.549 -1.32 -∞ 0]", &calc.ToString());
}
#[test]
fn VectorAcsch() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acsch({})", vector));
    assert_equal_str("[1.44 0.987 0.881 -1.44 -0.987 -0.881 +∞]", &calc.ToString());
}
#[test]
fn VectorAsech() {
    let mut vector = "[0.5; 0.866; 1; -0.5; -0.866; -1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Asech({})", vector));
    assert_equal_str(
        "[1.32 0.549 0  Undefined   Undefined   Undefined  +∞]",
        &calc.ToString(),
    );
}
#[test]
fn VectorAcoth() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Acoth({})", vector));
    assert_equal_str("[0.805 0.549 0.347 0.255 0.203 0.168 0.144]", &calc.ToString());
}
#[test]
fn VectorLn() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("ln({})", vector));
    assert_equal_str("[0.405 0.693 1.1 1.39 1.61 1.79 1.95]", &calc.ToString());
}
#[test]
fn VectorLog() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("log({})", vector));
    assert_equal_str("[0.176 0.301 0.477 0.602 0.699 0.778 0.845]", &calc.ToString());
}
#[test]
fn VectorLog2() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("log_2({})", vector));
    assert_equal_str("[0.585 1 1.58 2 2.32 2.58 2.81]", &calc.ToString());
}
#[test]
fn VectorExp() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("exp({})", vector));
    assert_equal_str("[4.48 7.39 20.09 54.6 148.41 403.43 1096.63]", &calc.ToString());
}
#[test]
fn VectorSqr() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sqr({})", vector));
    assert_equal_str("[1.22 1.41 1.73 2 2.24 2.45 2.65]", &calc.ToString());
}
#[test]
fn VectorSqrt() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sqrt({})", vector));
    assert_equal_str("[1.22 1.41 1.73 2 2.24 2.45 2.65]", &calc.ToString());
}
#[test]
fn VectorCbrt() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cbrt({})", vector));
    assert_equal_str("[1.14 1.26 1.44 1.59 1.71 1.82 1.91]", &calc.ToString());
}
#[test]
fn VectorRoot() {
    let mut vector = "[1.5; 2; 3; 4; 5; 6; 7]";
    let mut x = 3.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("root({}; {})", vector, x));
    assert_equal_str("[1.14 1.26 1.44 1.59 1.71 1.82 1.91]", &calc.ToString());
}
#[test]
fn VectorRound() {
    let mut vector = "[1.14; 1.26; 1.44; 1.59; 1.71; 1.82; 1.91]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("round({})", vector));
    assert_equal_str("[1 1 1 2 2 2 2]", &calc.ToString());
}
#[test]
fn VectorFloor() {
    let mut vector = "[1.7; 2.8; 3.4; 4.9; 5.12; 6.91; 7.25]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("floor({})", vector));
    assert_equal_str("[1 2 3 4 5 6 7]", &calc.ToString());
}
#[test]
fn VectorCeiling() {
    let mut vector = "[1.5; 2.1; 3.2; 4.4; 5.5; 6.9; 7.111]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("ceiling({})", vector));
    assert_equal_str("[2 3 4 5 6 7 8]", &calc.ToString());
}
#[test]
fn VectorTrunc() {
    let mut vector = "[3.7; -4.9; 5.0; -2.0; 0.1; -0.32; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("trunc({})", vector));
    assert_equal_str("[3 -4 5 -2 0 0 7]", &calc.ToString());
}
#[test]
fn VectorMod() {
    let mut vector = "[10; -15; 20; -25; 30; -35; 40]";
    let mut n = 7.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mod({}; {})", vector, n));
    assert_equal_str("[3 -1 6 -4 2 0 5]", &calc.ToString());
}
#[test]
fn VectorGcd() {
    let mut vector = "[12; 15; 9; 18; 21; 24; 30]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("gcd({}; 6)", vector));
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn VectorLcm() {
    let mut vector = "[4; 5; 6; 7; 8; 9; 10]";
    let mut n = 12.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lcm({}; {})", vector, n));
    assert_equal_str("2520", &calc.ToString());
}
#[test]
fn VectorAbs() {
    let mut vector = "[3.7; -4.9; 5.0; -2.0; 0.1; -0.32; 7.5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("abs({})", vector));
    assert_equal_str("[3.7 4.9 5 2 0.1 0.32 7.5]", &calc.ToString());
}
#[test]
fn VectorSign() {
    let mut vector = "[3.7; -4.9; 5.0; -2.0; 0.1; -0.32; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sign({})", vector));
    assert_equal_str("[1 -1 1 -1 1 -1 0]", &calc.ToString());
}
