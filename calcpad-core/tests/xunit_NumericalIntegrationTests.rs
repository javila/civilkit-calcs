//! Transcribed from Calcpad.Tests/Scalars/NumericalIntegrationTests.cs by tools/transcribe_xunit.py.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::identity_op
)]
mod common;
use common::*;

const Tol: f64 = 1e-14;

#[test]
fn Integral_1() {
    let mut result = TestCalc::new(ms()).Run("$Integral{ln(x) @ x = 0 : 1}");
    assert_equal_tol(-1.0, result, Tol);
}
#[test]
fn Integral_2() {
    let mut result = TestCalc::new(ms()).Run("$Integral{ln(x)/sqrt(x) @ x = 0 : 1}");
    assert_equal_tol(-4.0, result, Tol);
}
#[test]
fn Integral_3() {
    let mut result = TestCalc::new(ms()).Run("$Integral{4/(1 + x^2) @ x = 0 : 1}");
    assert_equal_tol(Math::PI, result, Tol);
}
#[test]
fn Integral_4() {
    let mut result = TestCalc::new(ms()).Run("$Integral{ln(x)*ln(1 - x) @ x = 0 : 1}");
    assert_equal_tol(2.0 - Math::PI * Math::PI / 6.0, result, Tol);
}
#[test]
fn Integral_5() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Integral{sin(x)/x @ x = 0 : 1}");
    assert_equal_tol(0.946083070367183, result, Tol);
}
#[test]
fn Integral_6() {
    let mut result = TestCalc::new(ms()).Run("$Integral{x^2*ln(x)*ln(x + 1)/(x + 1) @ x = 0 : 1}");
    assert_equal_tol(-0.0302622016388888, result, Tol);
}
#[test]
fn Integral_7() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Integral{(sin(x)/x)^2 @ x = 0 : 10}");
    assert_equal_tol(1.51864580413411, result, Tol);
}
#[test]
fn Integral_8() {
    let mut result = TestCalc::new(ms()).Run("$Integral{(16*x - 16)/(x^4 - 2*x^3 + 4*x - 4) @ x = 0 : 1}");
    assert_equal_tol(Math::PI, result, Tol);
}
#[test]
fn Integral_9() {
    let mut result = TestCalc::new(ms()).Run("$Integral{1/sqrt(abs(x)) @ x = 0 : 1}");
    assert_equal_tol(2.0, result, Tol);
}
#[test]
fn Integral_10() {
    let mut result = TestCalc::new(ms()).Run("$Area{1/(x*(1 + ln(x)^2)) @ x = 1 : e}");
    assert_equal_tol(0.785398163397448, result, Tol);
}
#[test]
fn Area_1() {
    let mut result = TestCalc::new(ms()).Run("$Area{x*ln(1 + x) @ x = 0 : 1}");
    assert_equal_tol(0.25, result, Tol);
}
#[test]
fn Area_2() {
    let mut result = TestCalc::new(ms()).Run("$Area{sqrt(1 - x^2) @ x = 0 : 1}");
    assert_equal_tol(Math::PI / 4.0, result, Tol);
}
#[test]
fn Area_3() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Area{sqrt(tan(x)) @ x = 0 : π/3}");
    assert_equal_tol(0.787779048098542, result, Tol);
}
#[test]
fn Area_4() {
    let mut result = TestCalc::new(ms()).Run("$Area{1/(1 - 2*x + 2*x^2) @ x = 0 : 1}");
    assert_equal_tol(Math::PI / 2.0, result, Tol);
}
#[test]
fn Area_5() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Area{1/(1 + tan(x)^3) @ x = 0 : π/2}");
    assert_equal_tol(Math::PI / 4.0, result, Tol);
}
#[test]
fn Area_6() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Area{1/(1 + cos(x)^x) @ x = -π/2 : π/2}");
    assert_equal_tol(Math::PI / 2.0, result, Tol);
}
#[test]
fn Area_7() {
    let mut result = TestCalc::new(ms()).Run("$Area{x^x @ x = 0 : 1}");
    assert_equal_tol(0.783430510712134, result, Tol);
}
#[test]
fn Area_8() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Area{sin(x)/sqrt(1 - 4*cos(x) + 4) @ x = 0 : π}");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Area_9() {
    let mut result = TestCalc::new(ms()).Run("$Area{ln(x + sqrt(1 + x^2)) @ x = 0 : 1}");
    assert_equal_tol(0.467160024646448, result, Tol);
}
#[test]
fn Area_10() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Area{sin(x)/sqrt(1 - 8*cos(x) + 16) @ x = 0 : π}");
    assert_equal_tol(0.5, result, Tol);
}
