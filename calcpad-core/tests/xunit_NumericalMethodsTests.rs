//! Transcribed from Calcpad.Tests/Scalars/NumericalMethodsTests.cs by tools/transcribe_xunit.py.
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

const Tol: f64 = 1e-12;

#[test]
fn Inf_Quad() {
    let mut calc = TestCalc::new(ms());
    calc.Run("f(x) = x^2 - 2*x - 1");
    let mut result = calc.Run("$Inf{f(x) @ x = -1 : 2}");
    assert_equal_tol(-2.0, result, Tol);
    result = calc.Run("x_inf");
    assert_equal_tol(1.0, result, 1e-7);
}
#[test]
fn Sup_Quad() {
    let mut calc = TestCalc::new(ms());
    calc.Run("f(x) = x^2 - 2*x - 1");
    let mut result = calc.Run("$Sup{f(x) @ x = -1 : 2}");
    assert_equal_tol(2.0, result, Tol);
    result = calc.Run("x_sup");
    assert_equal_tol(-1.0, result, Tol);
}
#[test]
fn Inf_Cos() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run("f(x) = cos(x - 1)");
    let mut result = calc.Run("$Inf{f(x) @ x = -1 : 2}");
    assert_equal_tol(Math::Cos(-2.0), result, Tol);
    result = calc.Run("x_inf");
    assert_equal_tol(-1.0, result, Tol);
}
#[test]
fn Sup_Cos() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run("f(x) = cos(x - 1)");
    let mut result = calc.Run("$Sup{f(x) @ x = -1 : 2}");
    assert_equal_tol(1.0, result, Tol);
    result = calc.Run("x_sup");
    assert_equal_tol(1.0, result, 1e-7);
}
#[test]
fn Slope_Exp() {
    let mut result = TestCalc::new(ms()).Run("$Slope{e^x @ x = 101}");
    let mut expected = Math::Exp(101.0);
    assert!(Math::Abs(expected - result) / Math::Abs(expected) < 1e-10);
}
#[test]
fn Slope_Sin() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Slope{sin(x) @ x = 1009*π}");
    let mut expected = Math::Cos(1009.0 * Math::PI);
    assert_equal_tol(expected, result, 1e-10);
}
#[test]
fn Slope_Cos_Square() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Slope{cos(x^2) @ x = 1013}");
    let mut expected = -2.0 * 1013.0 * Math::Sin(1013.0 * 1013.0);
    assert_equal_tol(expected, result, 1e-6);
}
#[test]
fn Derivative_Exp() {
    let mut result = TestCalc::new(ms()).Run("$Derivative{e^x @ x = 101}");
    let mut expected = Math::Exp(101.0);
    assert!(Math::Abs(expected - result) / Math::Abs(expected) < Tol);
}
#[test]
fn Derivative_Sin() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Derivative{sin(x) @ x = 1009*π}");
    let mut expected = Math::Cos(1009.0 * Math::PI);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Derivative_Cos_Square() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("$Derivative{cos(x^2) @ x = 1013}");
    let mut expected = -2.0 * 1013.0 * Math::Sin(1013.0 * 1013.0);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Find_If() {
    let mut result = TestCalc::new(ms()).Run("$Find{if(x < 1; -1; 1) @ x = -2 : 2}");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Find_Hyp() {
    let mut result = TestCalc::new(ms()).Run("$Find{1/(x - 1) @ x = -1 : 2}");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Sum_Pi() {
    let mut result =
        TestCalc::new(ms()).Run("$Sum{1/16^k*(4/(8*k + 1) - 2/(8*k + 4) - 1/(8*k + 5) - 1/(8*k + 6)) @ k = 0 : 11}");
    assert_equal_tol(Math::PI, result, Tol);
}
#[test]
fn Sum_e() {
    let mut result = TestCalc::new(ms()).Run("$Sum{1/k! @ k = 0 : 20}");
    assert_equal_tol(Math::E, result, Tol);
}
#[test]
fn Sum_2() {
    let mut result = TestCalc::new(ms()).Run("$Sum{1/(2^k) @ k = 0 : 55}");
    assert_equal_tol(2.0, result, Tol);
}
#[test]
fn Factorial() {
    let mut calc = TestCalc::new(ms());
    calc.Run("F(n) = $Product{k @ k = 1 : n}");
    let mut result = calc.Run("F(5)");
    assert_equal_tol(120.0, result, Tol);
}
#[test]
fn Binomial() {
    let mut calc = TestCalc::new(ms());
    calc.Run("C(n; k) = $Product{(i + n - k)/i @ i = 1 : k}");
    let mut result = calc.Run("C(7; 5)");
    assert_equal_tol(21.0, result, Tol);
}
#[test]
fn Babylonian_Method() {
    let mut calc = TestCalc::new(ms());
    calc.Run("x = 1");
    let mut result = calc.Run("x = $Repeat{x ← 0.5*(x + 5/x) @ k = 1 : 6}");
    let mut expected = Math::Sqrt(5.0);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Golden_Ratio() {
    let mut calc = TestCalc::new(ms());
    calc.Run("x = 1");
    let mut result = calc.Run("$Repeat{x = 1/(1 + x) @ k = 1 : 40}");
    let mut expected = 2.0 / (1.0 + Math::Sqrt(5.0));
    assert_equal_tol(expected, result, Tol);
}
