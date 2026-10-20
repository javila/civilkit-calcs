//! Port of BaseTypes/Complex.cs.
use std::cell::Cell;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::exceptions;
use crate::extension_methods::almost_equals;
use crate::math_parser_exception::{CResult, MathParserException};

const TRIG_MIN: f64 = -1e8;
const TRIG_MAX: f64 = 1e8;
const EPS: f64 = 1e-12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    a: f64,
    b: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Types {
    Real = 1,
    Imaginary = 2,
    Complex = 3,
}

thread_local! {
    static RAND_STATE: Cell<u64> = Cell::new({
        let t = (crate::clock::now_ms() * 1e6) as u64;
        let t = if t == 0 { 0x9E37_79B9_7F4A_7C15 } else { t };
        t | 1
    });
}

/// `Random.NextDouble()` replacement (xorshift64*), uniform in [0, 1).
pub fn next_double() -> f64 {
    RAND_STATE.with(|s| {
        let mut x = s.get();
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        s.set(x);
        let r = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (r >> 11) as f64 / (1u64 << 53) as f64
    })
}

/// `Math.Sign(double)`; throws ArithmeticException for NaN in .NET.
pub fn math_sign(d: f64) -> CResult<i32> {
    if d.is_nan() {
        return Err(MathParserException::unexpected(
            "Function does not accept floating point Not-a-Number values.",
        ));
    }
    Ok(if d > 0.0 {
        1
    } else if d < 0.0 {
        -1
    } else {
        0
    })
}

impl Complex {
    pub const ZERO: Complex = Complex { a: 0.0, b: 0.0 };
    pub const ONE: Complex = Complex { a: 1.0, b: 0.0 };
    pub const IMAGINARY_ONE: Complex = Complex { a: 0.0, b: 1.0 };
    pub const NAN: Complex = Complex { a: f64::NAN, b: 0.0 };
    pub const POSITIVE_INFINITY: Complex = Complex {
        a: f64::INFINITY,
        b: 0.0,
    };
    pub const NEGATIVE_INFINITY: Complex = Complex {
        a: f64::NEG_INFINITY,
        b: 0.0,
    };
    pub const COMPLEX_INFINITY: Complex = Complex {
        a: f64::INFINITY,
        b: f64::INFINITY,
    };

    #[inline]
    pub const fn new(real: f64, imaginary: f64) -> Complex {
        Complex { a: real, b: imaginary }
    }

    #[inline]
    pub const fn real(real: f64) -> Complex {
        Complex { a: real, b: 0.0 }
    }

    pub fn get_type(a: f64, b: f64) -> Types {
        if b == 0.0 {
            return Types::Real;
        }
        if a == 0.0 {
            return Types::Imaginary;
        }
        let re = a.abs();
        let im = b.abs();
        let d = (re + im) * EPS;
        if im < d {
            return Types::Real;
        }
        if re < d {
            return Types::Imaginary;
        }
        Types::Complex
    }

    pub fn type_(&self) -> Types {
        Complex::get_type(self.a, self.b)
    }
    pub fn is_real(&self) -> bool {
        self.type_() == Types::Real
    }
    pub fn is_imaginary(&self) -> bool {
        self.type_() == Types::Imaginary
    }
    pub fn is_complex(&self) -> bool {
        self.type_() == Types::Complex
    }
    #[inline]
    pub fn re(&self) -> f64 {
        self.a
    }
    #[inline]
    pub fn im(&self) -> f64 {
        self.b
    }

    pub fn phase(&self) -> f64 {
        if self.b == 0.0 {
            return if self.a >= 0.0 { 0.0 } else { std::f64::consts::PI };
        }
        self.b.atan2(self.a)
    }

    pub fn normal_phase(&self) -> f64 {
        let phi = self.b.atan2(self.a);
        if phi < 0.0 {
            return phi + 2.0 * std::f64::consts::PI;
        }
        phi
    }

    pub fn conjugate(&self) -> Complex {
        Complex::new(self.a, -self.b)
    }

    /// `Complex / Complex`
    pub fn div_c(left: Complex, right: Complex) -> Complex {
        let (a, b, c, d) = (left.a, left.b, right.a, right.b);
        if d == 0.0 {
            if c == 0.0 && (a != 0.0 || b != 0.0) {
                return Complex::COMPLEX_INFINITY;
            }
            return Complex::new(a / c, b / c);
        } else if (c.is_infinite() || d.is_infinite()) && !(a.is_infinite() || b.is_infinite()) {
            return Complex::ZERO;
        }
        if d.abs() < c.abs() {
            let e = d / c;
            let f = 1.0 / (c + d * e);
            return Complex::new((a + b * e) * f, (b - a * e) * f);
        }
        let e = c / d;
        let f = 1.0 / (d + c * e);
        Complex::new((b + a * e) * f, (-a + b * e) * f)
    }

    /// `double / Complex`
    pub fn f_div(left: f64, right: Complex) -> Complex {
        let (a, b) = (right.a, right.b);
        if b == 0.0 {
            if a == 0.0 && left != 0.0 {
                return Complex::POSITIVE_INFINITY;
            }
            return Complex::real(left / a);
        } else if (a.is_infinite() || b.is_infinite()) && !left.is_infinite() {
            return Complex::ZERO;
        }
        if b.abs() < a.abs() {
            let e = b / a;
            let f = 1.0 / (a + b * e);
            return Complex::new(left * f, -left * e * f);
        }
        let e = a / b;
        let f = 1.0 / (b + a * e);
        Complex::new(left * e * f, -left * f)
    }

    pub fn div_f(self, right: f64) -> Complex {
        Complex::new(self.a / right, self.b / right)
    }

    pub fn int_div(left: Complex, right: Complex) -> Complex {
        if right.is_real() && right.a != 0.0 {
            Complex::new((left.a / right.a).trunc(), (left.b / right.a).trunc())
        } else {
            Complex::NAN
        }
    }

    pub fn rem(left: Complex, right: Complex) -> Complex {
        if right.is_real() {
            Complex::new(left.a % right.a, left.b % right.a)
        } else {
            Complex::NAN
        }
    }

    pub fn eq_op(left: Complex, right: Complex) -> f64 {
        if Complex::almost_equals_c(left, right) {
            1.0
        } else {
            0.0
        }
    }

    pub fn ne_op(left: Complex, right: Complex) -> f64 {
        if Complex::almost_equals_c(left, right) {
            0.0
        } else {
            1.0
        }
    }

    pub fn lt_op(l: Complex, r: Complex) -> f64 {
        if l.is_real() && r.is_real() {
            if l.a < r.a && !almost_equals(l.a, r.a) {
                1.0
            } else {
                0.0
            }
        } else {
            f64::NAN
        }
    }

    pub fn gt_op(l: Complex, r: Complex) -> f64 {
        if l.is_real() && r.is_real() {
            if l.a > r.a && !almost_equals(l.a, r.a) {
                1.0
            } else {
                0.0
            }
        } else {
            f64::NAN
        }
    }

    pub fn le_op(l: Complex, r: Complex) -> f64 {
        if l.is_real() && r.is_real() {
            if l.a <= r.a || almost_equals(l.a, r.a) {
                1.0
            } else {
                0.0
            }
        } else {
            f64::NAN
        }
    }

    pub fn ge_op(l: Complex, r: Complex) -> f64 {
        if l.is_real() && r.is_real() {
            if l.a >= r.a || almost_equals(l.a, r.a) {
                1.0
            } else {
                0.0
            }
        } else {
            f64::NAN
        }
    }

    /// `Equals(Complex)`: exact component equality (double.Equals treats NaN == NaN).
    pub fn equals(&self, other: &Complex) -> bool {
        dbl_equals(self.a, other.a) && dbl_equals(self.b, other.b)
    }

    fn almost_equals_c(left: Complex, right: Complex) -> bool {
        let lt = left.type_();
        let rt = right.type_();
        if lt == Types::Real && rt == Types::Real {
            return almost_equals(left.a, right.a);
        }
        if lt == Types::Imaginary && rt == Types::Imaginary {
            return almost_equals(left.b, right.b);
        }
        almost_equals(left.a, right.a) && almost_equals(left.b, right.b)
    }

    pub fn abs_c(value: Complex) -> Complex {
        Complex::real(Complex::abs(value.a, value.b))
    }

    pub fn abs(a: f64, b: f64) -> f64 {
        if a.is_infinite() || b.is_infinite() {
            return f64::INFINITY;
        }
        let c = a.abs();
        if b == 0.0 {
            return c;
        }
        let d = b.abs();
        if c > d {
            let rc = d / c;
            return c * (1.0 + rc * rc).sqrt();
        }
        let rd = c / d;
        d * (1.0 + rd * rd).sqrt()
    }

    pub fn sign(value: Complex) -> CResult<Complex> {
        if value.is_real() {
            Ok(Complex::real(math_sign(value.a)? as f64))
        } else {
            Ok(Complex::NAN)
        }
    }

    fn check_trig_scope(angle: f64, func: &str) -> CResult<()> {
        // NaN passes, as in C# (both comparisons are false).
        if angle < TRIG_MIN || angle > TRIG_MAX {
            return Err(exceptions::ArgumentOutOfRange(func));
        }
        Ok(())
    }

    pub fn real_random(value: f64) -> f64 {
        next_double() * value
    }

    pub fn real_sin(value: f64) -> CResult<f64> {
        Complex::check_trig_scope(value, "sin")?;
        Ok(value.sin())
    }

    pub fn real_cos(value: f64) -> CResult<f64> {
        Complex::check_trig_scope(value, "cos")?;
        Ok(value.cos())
    }

    pub fn sin(value: Complex) -> CResult<Complex> {
        let (a, b) = (value.a, value.b);
        Complex::check_trig_scope(a, "sin")?;
        let (s, c) = a.sin_cos();
        Ok(Complex::new(s * b.cosh(), c * b.sinh()))
    }

    pub fn cos(value: Complex) -> CResult<Complex> {
        let (a, b) = (value.a, value.b);
        Complex::check_trig_scope(a, "cos")?;
        let (s, c) = a.sin_cos();
        Ok(Complex::new(c * b.cosh(), -s * b.sinh()))
    }

    pub fn tan(value: Complex) -> Complex {
        let ta = value.a.tan();
        if value.b == 0.0 {
            return Complex::real(ta);
        }
        let thb = value.b.tanh();
        Complex::div_c(Complex::new(ta, thb), Complex::new(1.0, -ta * thb))
    }

    pub fn cot(value: Complex) -> Complex {
        let ta = value.a.tan();
        if value.b == 0.0 {
            return Complex::real(1.0 / ta);
        }
        let thb = value.b.tanh();
        Complex::div_c(Complex::new(1.0, -ta * thb), Complex::new(ta, thb))
    }

    pub fn sinh(value: Complex) -> CResult<Complex> {
        let (a, b) = (value.a, value.b);
        Complex::check_trig_scope(b, "sinh")?;
        let (s, c) = b.sin_cos();
        Ok(Complex::new(a.sinh() * c, a.cosh() * s))
    }

    pub fn cosh(value: Complex) -> CResult<Complex> {
        let (a, b) = (value.a, value.b);
        Complex::check_trig_scope(b, "cosh")?;
        let (s, c) = b.sin_cos();
        Ok(Complex::new(a.cosh() * c, a.sinh() * s))
    }

    pub fn tanh(value: Complex) -> Complex {
        let tha = value.a.tanh();
        let tb = value.b.tan();
        if tb == 0.0 {
            Complex::real(tha)
        } else {
            Complex::div_c(Complex::new(tha, tb), Complex::new(1.0, tha * tb))
        }
    }

    pub fn coth(value: Complex) -> Complex {
        let tha = value.a.tanh();
        let tb = value.b.tan();
        if tb == 0.0 {
            Complex::real(1.0 / tha)
        } else {
            Complex::div_c(Complex::new(1.0, tha * tb), Complex::new(tha, tb))
        }
    }

    pub fn asin(value: Complex) -> Complex {
        let i = Complex::IMAGINARY_ONE;
        -i * Complex::log(i * value + Complex::sqrt(Complex::ONE - value * value))
    }

    pub fn acos(value: Complex) -> Complex {
        let i = Complex::IMAGINARY_ONE;
        -i * Complex::log(value + i * Complex::sqrt(Complex::ONE - value * value))
    }

    pub fn atan(value: Complex) -> Complex {
        let i = Complex::IMAGINARY_ONE;
        if value.equals(&i) {
            return Complex::new(0.0, f64::INFINITY);
        }
        if value.equals(&-i) {
            return Complex::new(0.0, f64::NEG_INFINITY);
        }
        (-i).div_f(2.0) * Complex::log(Complex::div_c(i - value, i + value))
    }

    pub fn acot(value: Complex) -> Complex {
        let i = Complex::IMAGINARY_ONE;
        if value.equals(&i) {
            return Complex::new(0.0, f64::NEG_INFINITY);
        }
        if value.equals(&-i) {
            return Complex::new(0.0, f64::INFINITY);
        }
        (-i).div_f(2.0) * Complex::log(Complex::div_c(value + i, value - i))
    }

    pub fn asinh(value: Complex) -> Complex {
        Complex::log(value + Complex::sqrt(value * value + Complex::ONE))
    }

    pub fn acosh(value: Complex) -> Complex {
        Complex::log(value + Complex::sqrt(value - Complex::ONE) * Complex::sqrt(value + Complex::ONE))
    }

    pub fn atanh(value: Complex) -> Complex {
        Complex::log(Complex::div_c(Complex::ONE + value, Complex::ONE - value)).div_f(2.0)
    }

    pub fn acoth(value: Complex) -> Complex {
        Complex::log(Complex::div_c(value + Complex::ONE, value - Complex::ONE)).div_f(2.0)
    }

    pub fn pow_f(value: Complex, power: f64) -> Complex {
        if power == 0.0 {
            return Complex::ONE;
        }
        let is_integer = power.is_finite() && power == power.trunc();
        if value.b == 0.0 && (value.a > 0.0 || is_integer) {
            return if value.a == 0.0 {
                Complex::ZERO
            } else {
                Complex::real(value.a.powf(power))
            };
        }
        if power == 2.0 {
            return value * value;
        }
        if is_integer && power > 0.0 && power < 6.0 {
            let mut result = value;
            let mut i = 1.0;
            while i < power {
                result = result * value;
                i += 1.0;
            }
            return result;
        }
        let r = Complex::abs(value.a, value.b);
        let theta = power * value.normal_phase();
        let t = r.powf(power);
        Complex::new(t * theta.cos(), t * theta.sin())
    }

    pub fn pow(value: Complex, power: Complex) -> Complex {
        let (c, d) = (power.a, power.b);
        if d == 0.0 {
            return Complex::pow_f(value, c);
        }
        let (a, b) = (value.a, value.b);
        if a == 0.0 && b == 0.0 {
            return Complex::ZERO;
        }
        let r = Complex::abs(a, b);
        let phi = value.normal_phase();
        let theta = c * phi + d * r.ln();
        let t = r.powf(c) * (-d * phi).exp();
        Complex::new(t * theta.cos(), t * theta.sin())
    }

    pub fn log(value: Complex) -> Complex {
        Complex::new(Complex::abs(value.a, value.b).ln(), value.phase())
    }

    pub fn log10(value: Complex) -> Complex {
        let inv = 1.0 / 10f64.ln();
        Complex::new(Complex::abs(value.a, value.b).ln() * inv, value.phase() * inv)
    }

    pub fn log2(value: Complex) -> Complex {
        let inv = 1.0 / 2f64.ln();
        Complex::new(Complex::abs(value.a, value.b).ln() * inv, value.phase() * inv)
    }

    pub fn exp(value: Complex) -> Complex {
        let r = value.a.exp();
        Complex::new(r * value.b.cos(), r * value.b.sin())
    }

    pub fn sqrt(value: Complex) -> Complex {
        let r = Complex::abs(value.a, value.b).sqrt();
        let theta = value.normal_phase() / 2.0;
        Complex::new(r * theta.cos(), r * theta.sin())
    }

    pub fn cbrt(value: Complex) -> Complex {
        let r = Complex::abs(value.a, value.b).cbrt();
        let theta = value.normal_phase() / 3.0;
        Complex::new(r * theta.cos(), r * theta.sin())
    }

    pub fn round(value: Complex) -> Complex {
        Complex::new(value.a.round(), value.b.round())
    }

    pub fn floor(value: Complex) -> Complex {
        Complex::new(value.a.floor(), value.b.floor())
    }

    pub fn ceiling(value: Complex) -> Complex {
        Complex::new(value.a.ceil(), value.b.ceil())
    }

    pub fn truncate(value: Complex) -> Complex {
        Complex::new(value.a.trunc(), value.b.trunc())
    }

    pub fn random(value: Complex) -> Complex {
        Complex::new(next_double(), next_double()) * value
    }

    pub fn atan2(left: Complex, right: Complex) -> Complex {
        if left.is_real() && right.is_real() {
            Complex::real(left.a.atan2(right.a))
        } else {
            Complex::NAN
        }
    }
}

/// `double.Equals(double)`: like ==, but NaN equals NaN.
#[inline]
pub fn dbl_equals(a: f64, b: f64) -> bool {
    a == b || a.is_nan() && b.is_nan()
}

impl Neg for Complex {
    type Output = Complex;
    fn neg(self) -> Complex {
        Complex::new(-self.a, -self.b)
    }
}

impl Add for Complex {
    type Output = Complex;
    fn add(self, r: Complex) -> Complex {
        Complex::new(self.a + r.a, self.b + r.b)
    }
}

impl Sub for Complex {
    type Output = Complex;
    fn sub(self, r: Complex) -> Complex {
        Complex::new(self.a - r.a, self.b - r.b)
    }
}

impl Mul for Complex {
    type Output = Complex;
    fn mul(self, r: Complex) -> Complex {
        Complex::new(self.a * r.a - self.b * r.b, self.b * r.a + self.a * r.b)
    }
}

impl Mul<f64> for Complex {
    type Output = Complex;
    fn mul(self, r: f64) -> Complex {
        Complex::new(self.a * r, self.b * r)
    }
}

impl Mul<Complex> for f64 {
    type Output = Complex;
    fn mul(self, r: Complex) -> Complex {
        r * self
    }
}

impl Div for Complex {
    type Output = Complex;
    fn div(self, r: Complex) -> Complex {
        Complex::div_c(self, r)
    }
}
