//! Transcribed from Calcpad.Tests/Scalars/ComplexOperatorTest.cs by tools/transcribe_xunit.py.
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
fn _sqrt2() -> f64 {
    Math::Sqrt(2.0)
}
fn _sqrt3() -> f64 {
    Math::Sqrt(3.0)
}

#[test]
fn Factorial() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms().IsComplex(true)).Run("1i!");
    });
    assert_contains("The argument of n! cannot be complex.", &e.message);
}
#[test]
fn Pow_i_2() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i^2");
    let mut im = calc.Imaginary();
    assert_equal_tol(-1.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Pow_i_i() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i^1i");
    let mut im = calc.Imaginary();
    assert_equal_tol(Math::Exp(-Math::PI / 2.0), re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Pow_i__Minus_1() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i^-1");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-1.0, im, Tol);
}
#[test]
fn Pow_i_0() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i^0");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Pow_e__2Pi_i() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("e^(2*πi)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Minus_Pow_e__Pi_i_Plus_1() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("-e^(πi + 1)");
    let mut im = calc.Imaginary();
    assert_equal_tol(Math::E, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Pow_Minus_1__1_Div_2() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(-1)^(1/2)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(1.0, im, Tol);
}
#[test]
fn Pow_Minus_8__1_Div_3() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(-8)^(1/3)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(_sqrt3(), im, Tol);
}
#[test]
fn Pow_Minus_16__1_Div_4() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(-16)^(1/4)");
    let mut im = calc.Imaginary();
    assert_equal_tol(_sqrt2(), re, Tol);
    assert_equal_tol(_sqrt2(), im, Tol);
}
#[test]
fn Pow_Minus_32__1_Div_5() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(-32)^(1/5)");
    let mut im = calc.Imaginary();
    let mut sqrt5 = Math::Sqrt(5.0);
    assert_equal_tol((1.0 + sqrt5) / 2.0, re, Tol);
    assert_equal_tol(Math::Sqrt((5.0 - sqrt5) / 2.0), im, Tol);
}
#[test]
fn Pow_2_Plus_3i__2() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(2 + 3i)^2");
    let mut im = calc.Imaginary();
    assert_equal_tol(-5.0, re, Tol);
    assert_equal_tol(12.0, im, Tol);
}
#[test]
fn Pow_1_Plus_i__Minus_2() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(1 + 1i)^-2");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-1.0 / 2.0, im, Tol);
}
#[test]
fn Pow_1_Plus_i__3() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(1 + 1i)^3");
    let mut im = calc.Imaginary();
    assert_equal_tol(-2.0, re, Tol);
    assert_equal_tol(2.0, im, Tol);
}
#[test]
fn Pow_3_Minus_2i__3() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(3 - 2i)^3");
    let mut im = calc.Imaginary();
    assert_equal_tol(-9.0, re, Tol);
    assert_equal_tol(-46.0, im, Tol);
}
#[test]
fn Div_I_0() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i/0");
    let mut im = calc.Imaginary();
    assert_equal_tol(f64::INFINITY, re, Tol);
    assert_equal_tol(f64::INFINITY, im, Tol);
}
#[test]
fn Div_3_Plus_4i__1_Minus_2i() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(3 + 4i)/(1 - 2i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(-1.0, re, Tol);
    assert_equal_tol(2.0, im, Tol);
}
#[test]
fn Mult_i_i() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i*1i");
    let mut im = calc.Imaginary();
    assert_equal_tol(-1.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Mult_3_Plus_4i__3_Minus_4i() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(3 + 4i)*(3 - 4i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(25.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Add_3_Plus_4i__2_Minus_3i() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(3 + 4i) + (2 - 3i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(5.0, re, Tol);
    assert_equal_tol(1.0, im, Tol);
}
#[test]
fn Subt_3_Plus_4i__2_Plus_3i() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(3 + 4i) - (2 + 3i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(1.0, im, Tol);
}
#[test]
fn Equals_1l__Sqrt_Minus_1() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i ≡ sqrt(-1)");
    assert_equal(1.0, re);
}
#[test]
fn Equals_e_Pow_Pi_i__Minus_1() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("e^πi ≡ -1");
    assert_equal(1.0, re);
}
#[test]
fn Equals_3_Plus_4i__4i_Plus_3() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("3 + 4i ≡ 4i + 3");
    assert_equal(1.0, re);
}
#[test]
fn NotEquals_i_1() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("1i ≠ 1");
    assert_equal(1.0, re);
}
#[test]
fn GreaterThan() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(3 + 4i) > (2 + 3i)");
    assert_equal(f64::NAN, re);
}
#[test]
fn GreaterOrEquals() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("(3 + 4i) ≥ (2 + 3i)");
    assert_equal(f64::NAN, re);
}
#[test]
fn LessThan() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("-(3 + 2i) < (4 + 3i)");
    assert_equal(f64::NAN, re);
}
#[test]
fn LessOrEquals_Real() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("2 + 2 ≤ 2*2");
    assert_equal(1.0, re);
}
#[test]
fn LessOrEquals_WhenEqual() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("3 + 4i ≤ 4i + 3");
    assert_equal(f64::NAN, re);
}
