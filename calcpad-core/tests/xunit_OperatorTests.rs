//! Transcribed from Calcpad.Tests/Scalars/OperatorTests.cs by tools/transcribe_xunit.py.
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

const Tol: f64 = 1e-15;

#[test]
fn Factorial_0() {
    let mut result = TestCalc::new(ms()).Run("0!");
    assert_equal(1.0, result.clone());
}
#[test]
fn Factorial_5() {
    let mut result = TestCalc::new(ms()).Run("5!");
    assert_equal(120.0, result.clone());
}
#[test]
fn Factorial_180() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms()).Run("180!");
    });
    assert_contains("Argument out of range for n!", &e.message);
}
#[test]
fn Factorial_Minus_10() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms()).Run("(-10)!");
    });
    assert_contains("Argument out of range for n!", &e.message);
}
#[test]
fn Power_2_3() {
    let mut result = TestCalc::new(ms()).Run("2^3");
    assert_equal(8.0, result.clone());
}
#[test]
fn Power_3__Minus_2() {
    let mut result = TestCalc::new(ms()).Run("3^-2");
    assert_equal_tol(1.0 / 9.0, result, Tol);
}
#[test]
fn Power_Minus_3__2() {
    let mut result = TestCalc::new(ms()).Run("-3^2");
    assert_equal(-9.0, result.clone());
}
#[test]
fn Power_Minus_3__4_Brackets() {
    let mut result = TestCalc::new(ms()).Run("(-3)^4");
    assert_equal(81.0, result.clone());
}
#[test]
fn Power_4_3_2() {
    let mut result = TestCalc::new(ms()).Run("4^3^2");
    assert_equal(262144.0, result.clone());
}
#[test]
fn Power_10_310() {
    let mut result = TestCalc::new(ms()).Run("10^310");
    assert_equal(f64::INFINITY, result.clone());
}
#[test]
fn Power_10__Minus_324() {
    let mut result = TestCalc::new(ms()).Run("10^-324");
    assert_equal(0.0, result.clone());
}
#[test]
fn Power_10_0() {
    let mut result = TestCalc::new(ms()).Run("10^0");
    assert_equal(1.0, result.clone());
}
#[test]
fn Power_0_10() {
    let mut result = TestCalc::new(ms()).Run("0^10");
    assert_equal(0.0, result.clone());
}
#[test]
fn Power_0_0() {
    let mut result = TestCalc::new(ms()).Run("0^0");
    assert_equal(1.0, result.clone());
}
#[test]
fn Power_0_Inf() {
    let mut calc = TestCalc::new(ms());
    calc.Run("inf = 10^310");
    let mut result = calc.Run("0^inf");
    assert_equal(0.0, result.clone());
}
#[test]
fn Power_1_Inf() {
    let mut calc = TestCalc::new(ms());
    calc.Run("inf = 10^310");
    let mut result = calc.Run("1^inf");
    assert_equal(1.0, result.clone());
}
#[test]
fn Power_Inf_0() {
    let mut calc = TestCalc::new(ms());
    calc.Run("inf = 10^310");
    let mut result = calc.Run("inf^0");
    assert_equal(1.0, result.clone());
}
#[test]
fn Divide_7_5() {
    let mut result = TestCalc::new(ms()).Run("7/5");
    assert_equal_tol(1.4, result, Tol);
}
#[test]
fn Divide_2_3() {
    let mut result = TestCalc::new(ms()).Run("2÷3");
    assert_equal_tol(2.0 / 3.0, result, Tol);
}
#[test]
fn Divide_1_0() {
    let mut result = TestCalc::new(ms()).Run("1/0");
    assert_equal(f64::INFINITY, result.clone());
}
#[test]
fn Divide_0_0() {
    let mut result = TestCalc::new(ms()).Run("0/0");
    assert_equal(f64::NAN, result.clone());
}
#[test]
fn Divide_Inf_Inf() {
    let mut result = TestCalc::new(ms()).Run("10^311/10^310");
    assert_equal(f64::NAN, result.clone());
}
#[test]
fn Int_Div_10_3() {
    let mut result = TestCalc::new(ms()).Run("10\\3");
    assert_equal(3.0, result.clone());
}
#[test]
fn Mod_7_5() {
    let mut result = TestCalc::new(ms()).Run("7⦼5");
    assert_equal(2.0, result.clone());
}
#[test]
fn Mod_2_0() {
    let mut result = TestCalc::new(ms()).Run("2⦼0");
    assert_equal(f64::NAN, result.clone());
}
#[test]
fn Mutiply_2_Minus_3() {
    let mut result = TestCalc::new(ms()).Run("2*-3");
    assert_equal(-6.0, result.clone());
}
#[test]
fn Mutiply_Minus_3_Minus_2() {
    let mut result = TestCalc::new(ms()).Run("-3*-2");
    assert_equal(6.0, result.clone());
}
#[test]
fn Mutiply_125_080() {
    let mut result = TestCalc::new(ms()).Run("1.25*0.8");
    assert_equal(1.0, result.clone());
}
#[test]
fn Mutiply_10_0() {
    let mut result = TestCalc::new(ms()).Run("10*0");
    assert_equal(0.0, result.clone());
}
#[test]
fn Multiply_Inf_0() {
    let mut calc = TestCalc::new(ms());
    calc.Run("inf = 10^310");
    let mut result = calc.Run("inf*0");
    assert_equal(f64::NAN, result.clone());
}
#[test]
fn Multiply_2_3_Div_2_3() {
    let mut result = TestCalc::new(ms()).Run("2*3 / 2*3");
    assert_equal(9.0, result.clone());
}
#[test]
fn Multiply_2_3_Fact() {
    let mut result = TestCalc::new(ms()).Run("2*3!");
    assert_equal(12.0, result.clone());
}
#[test]
fn Multiply_2_Pow_3_2() {
    let mut result = TestCalc::new(ms()).Run("2^3*2");
    assert_equal(16.0, result.clone());
}
#[test]
fn Subtract_5_3() {
    let mut result = TestCalc::new(ms()).Run("5-3");
    assert_equal(2.0, result.clone());
}
#[test]
fn Subtract_Minus_5_3() {
    let mut result = TestCalc::new(ms()).Run("-5 - 3");
    assert_equal(-8.0, result.clone());
}
#[test]
fn Subtract_5_Plus_3_And_5_Plus_3() {
    let mut result = TestCalc::new(ms()).Run("5 + 3 - 5 + 3");
    assert_equal(6.0, result.clone());
}
#[test]
fn Subtract_5_Plus_3_And_5_Plus_3_Brackets() {
    let mut result = TestCalc::new(ms()).Run("5 + 3 - (5 + 3)");
    assert_equal(0.0, result.clone());
}
#[test]
fn Subtract_10_Minus_2() {
    let mut result = TestCalc::new(ms()).Run("10 - -2");
    assert_equal(12.0, result.clone());
}
#[test]
fn Subtract_Minus_2_2() {
    let mut result = TestCalc::new(ms()).Run("-2 - 2");
    assert_equal(-4.0, result.clone());
}
#[test]
fn Subtract_Minus_2_Thirds_1_Ningth() {
    let mut result = TestCalc::new(ms()).Run("2/3 - 1/9");
    assert_equal_tol(2.0 / 3.0 - 1.0 / 9.0, result, Tol);
}
#[test]
fn Subtract_Inf_Inf() {
    let mut calc = TestCalc::new(ms());
    calc.Run("inf = 10^310");
    let mut result = calc.Run("inf - inf");
    assert_equal(f64::NAN, result.clone());
}
#[test]
fn Add_5_3() {
    let mut result = TestCalc::new(ms()).Run("5 + 3");
    assert_equal(8.0, result.clone());
}
#[test]
fn Add_3_2_Mult_3() {
    let mut result = TestCalc::new(ms()).Run("3+2 * 3");
    assert_equal(9.0, result.clone());
}
#[test]
fn Add_5_10_div_2() {
    let mut result = TestCalc::new(ms()).Run("5+10 / 2");
    assert_equal(10.0, result.clone());
}
#[test]
fn Equals_01_1_Div_10() {
    let mut result = TestCalc::new(ms()).Run("0.1 ≡ 1/10");
    assert_equal(1.0, result.clone());
}
#[test]
fn Equals_1_Div_0_2_Div_0() {
    let mut result = TestCalc::new(ms()).Run("1/0 ≡ 2/0");
    assert_equal(1.0, result.clone());
}
#[test]
fn Equals_Sin_Small_Number() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("sin(10^-8) ≡ 0.00000001");
    assert_equal(1.0, result.clone());
}
#[test]
fn Equals_Inf_Inf() {
    let mut calc = TestCalc::new(ms());
    calc.Run("inf = 10^310");
    let mut result = calc.Run("inf ≡ inf");
    assert_equal(1.0, result.clone());
}
#[test]
fn Equals_Inf_1_Div_0() {
    let mut calc = TestCalc::new(ms());
    calc.Run("inf = 10^310");
    let mut result = calc.Run("inf ≡ 1/0");
    assert_equal(1.0, result.clone());
}
#[test]
fn NotEquals_1_2() {
    let mut result = TestCalc::new(ms()).Run("1 ≠ 2");
    assert_equal(1.0, result.clone());
}
#[test]
fn NotEquals_1_Div_9_0_11() {
    let mut result = TestCalc::new(ms()).Run("1/9 ≠ 0.1111111111111111");
    assert_equal(0.0, result.clone());
}
#[test]
fn LessThan_4_5() {
    let mut result = TestCalc::new(ms()).Run("4 < 5");
    assert_equal(1.0, result.clone());
}
#[test]
fn LessThan_Minus_4_Minus_5() {
    let mut result = TestCalc::new(ms()).Run("-4 < -5");
    assert_equal(0.0, result.clone());
}
#[test]
fn GreaterThan_Minus_2_1() {
    let mut result = TestCalc::new(ms()).Run("-2 > 1");
    assert_equal(0.0, result.clone());
}
#[test]
fn GreaterThan_Minus_1_Minus_2() {
    let mut result = TestCalc::new(ms()).Run("-1 > -2");
    assert_equal(1.0, result.clone());
}
#[test]
fn LessOrEqual_1_Minus_2() {
    let mut result = TestCalc::new(ms()).Run("1 ≤ -2");
    assert_equal(0.0, result.clone());
}
#[test]
fn GreaterOrEqual_2_1() {
    let mut result = TestCalc::new(ms()).Run("2 ≥ 1");
    assert_equal(1.0, result.clone());
}
#[test]
fn And_0_0() {
    let mut result = TestCalc::new(ms()).Run("0 ∧ 0");
    assert_equal(0.0, result.clone());
}
#[test]
fn And_0_1() {
    let mut result = TestCalc::new(ms()).Run("0 ∧ 1");
    assert_equal(0.0, result.clone());
}
#[test]
fn And_2_0() {
    let mut result = TestCalc::new(ms()).Run("2 ∧ 0");
    assert_equal(0.0, result.clone());
}
#[test]
fn And_Minus_1_1() {
    let mut result = TestCalc::new(ms()).Run("-1 ∧ 1");
    assert_equal(1.0, result.clone());
}
#[test]
fn Or_0_0() {
    let mut result = TestCalc::new(ms()).Run("0 ∨ 0");
    assert_equal(0.0, result.clone());
}
#[test]
fn Or_0_1() {
    let mut result = TestCalc::new(ms()).Run("0 ∨ 1");
    assert_equal(1.0, result.clone());
}
#[test]
fn Or_2_0() {
    let mut result = TestCalc::new(ms()).Run("2 ∨ 0");
    assert_equal(1.0, result.clone());
}
#[test]
fn Or_Minus_1_1() {
    let mut result = TestCalc::new(ms()).Run("-1 ∨ 1");
    assert_equal(1.0, result.clone());
}
#[test]
fn Xor_0_0() {
    let mut result = TestCalc::new(ms()).Run("0 ⊕ 0");
    assert_equal(0.0, result.clone());
}
#[test]
fn Xor_0_1() {
    let mut result = TestCalc::new(ms()).Run("0 ⊕ 1");
    assert_equal(1.0, result.clone());
}
#[test]
fn Xor_2_0() {
    let mut result = TestCalc::new(ms()).Run("2 ⊕ 0");
    assert_equal(1.0, result.clone());
}
#[test]
fn Xor_Minus_1_1() {
    let mut result = TestCalc::new(ms()).Run("-1 ⊕ 1");
    assert_equal(0.0, result.clone());
}
#[test]
fn Assign() {
    let mut calc = TestCalc::new(ms());
    calc.Run("x = 1");
    let mut result = calc.Run("x");
    assert_equal(1.0, result.clone());
}
