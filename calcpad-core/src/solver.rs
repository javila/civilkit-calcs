//! Port of Solver.cs (root finding, extrema, quadrature, slope, sums and products).
use std::rc::Rc;

use crate::base_types::complex::{math_sign, Complex};
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::base_types::variable::VarRef;
use crate::exceptions::{self, Items};
use crate::math_parser_exception::CResult;
use crate::netfmt::double_to_string;
use crate::parsers::math_parser::compiler::Expr;
use crate::parsers::math_parser::MathParser;

const LIMITS: f64 = 1E8;
const TANH_SINH_DEPTH: usize = 11;
const M: [usize; 11] = [6, 7, 13, 26, 53, 106, 212, 423, 846, 1693, 3385];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuadratureMethods {
    AdaptiveLobatto,
    TanhSinh,
}

struct TanhSinhTable {
    r: Vec<Vec<f64>>,
    w: Vec<Vec<f64>>,
}

thread_local! {
    static TABLE: TanhSinhTable = {
        let mut r = Vec::with_capacity(TANH_SINH_DEPTH);
        let mut w = Vec::with_capacity(TANH_SINH_DEPTH);
        let mut h = 2.0f64;
        for (i, &m) in M.iter().enumerate() {
            h /= 2.0;
            let mut eh = h.exp();
            let mut t = eh;
            let mut ri = vec![0.0; m];
            let mut wi = vec![0.0; m];
            if i > 0 {
                eh *= eh;
            }
            for j in 0..m {
                let u = (1.0 / t - t).exp();
                let d = 2.0 * u / (1.0 + u);
                ri[j] = d;
                wi[j] = (1.0 / t + t) * d / (1.0 + u);
                t *= eh;
            }
            r.push(ri);
            w.push(wi);
        }
        TanhSinhTable { r, w }
    };
}

pub struct Solver<'a> {
    pub parser: &'a MathParser,
    pub is_complex: bool,
    pub precision: f64,
    pub quadrature_method: QuadratureMethods,
    pub units: OptUnit,
    pub function: Rc<Expr>,
    pub variable: VarRef,
}

impl<'a> Solver<'a> {
    fn function(&self) -> CResult<Value> {
        self.parser.eval_expr(&self.function)
    }

    fn fd(&mut self, x: f64) -> CResult<f64> {
        self.variable.set_number(x);
        let result = self.function()?;
        if self.is_complex && !result.is_real() {
            return Err(exceptions::CannotEvaluateFunction(&double_to_string(x)));
        }
        let real = result.as_real_checked(Items::Result)?;
        self.units = real.units.clone();
        if real.d.is_nan() || real.d.is_infinite() {
            return Err(exceptions::FunctionNotDefined(&double_to_string(x)));
        }
        Ok(real.d)
    }

    fn fc(&mut self, x: Complex) -> CResult<Complex> {
        self.variable.set_number_c(x);
        let result = self.function()?;
        self.units = result.units().clone();
        if result.re().is_nan() && result.im().is_nan() {
            use crate::output::OutputWriter;
            let w = crate::output::text_writer::TextWriter::with_decimals(15);
            let s = if x.is_real() {
                w.format_real(x.re(), None, false)
            } else {
                w.format_complex(x.re(), x.im(), None)
            };
            return Err(exceptions::CannotEvaluateFunction(&s));
        }
        Ok(result.complex())
    }

    fn fi(&mut self, x: f64) -> CResult<Value> {
        self.variable.set_number(x);
        let result = self.function()?;
        if result.re().is_nan() || result.re().is_infinite() {
            return Err(exceptions::FunctionNotDefined(&double_to_string(x)));
        }
        self.units = result.units().clone();
        Ok(result)
    }

    pub fn mod_ab(&mut self, left: f64, right: f64, target: f64, precision: f64) -> CResult<(f64, f64)> {
        let mut err = 0.0;
        let mut x1 = left.min(right);
        let mut y1 = self.fd(x1)? - target;
        if y1.abs() <= precision {
            return Ok((x1, err));
        }
        let mut x2 = left.max(right);
        let mut y2 = self.fd(x2)? - target;
        if y2.abs() <= precision {
            return Ok((x2, err));
        }
        let mut eps1 = precision * 1e-3;
        let eps2 = precision * (x2 - x1);
        if target.abs() >= 1.0 {
            eps1 *= target.abs();
        } else {
            eps1 *= eps1;
        }
        let mut side = 0;
        let mut x0 = x1;
        let mut bisection = true;
        let mut threshold = x2 - x1;
        const C: f64 = 16.0;
        for _ in 1..=100 {
            let x3;
            let y3;
            if bisection {
                x3 = (x1 + x2) / 2.0;
                y3 = self.fd(x3)? - target;
                let ym = (y1 + y2) / 2.0;
                let r = 1.0 - (ym / (y2 - y1)).abs();
                let k = r * r;
                if (ym - y3).abs() < k * (y3.abs() + ym.abs()) {
                    bisection = false;
                    threshold = (x2 - x1) * C;
                }
            } else {
                let mut xx = (x1 * y2 - y1 * x2) / (y2 - y1);
                let yy;
                if xx <= x1 {
                    xx = x1;
                    yy = y1;
                } else if xx >= x2 {
                    xx = x2;
                    yy = y2;
                } else {
                    yy = self.fd(xx)? - target;
                }
                x3 = xx;
                y3 = yy;
                threshold /= 2.0;
            }
            err = y3.abs();
            if err <= eps1 || x2 - x1 <= eps2 {
                return Ok((x3, err));
            }
            x0 = x3;
            if math_sign(y1)? == math_sign(y3)? {
                if side == 1 {
                    let m = 1.0 - y3 / y1;
                    if m <= 0.0 {
                        y2 /= 2.0;
                    } else {
                        y2 *= m;
                    }
                } else if !bisection {
                    side = 1;
                }
                x1 = x3;
                y1 = y3;
            } else {
                if side == -1 {
                    let m = 1.0 - y3 / y2;
                    if m <= 0.0 {
                        y1 /= 2.0;
                    } else {
                        y1 *= m;
                    }
                } else if !bisection {
                    side = -1;
                }
                x2 = x3;
                y2 = y3;
            }
            if x2 - x1 > threshold {
                bisection = true;
            }
        }
        Ok((x0, err))
    }

    pub fn root(&mut self, left: f64, right: f64, target: f64) -> CResult<f64> {
        let u = self.variable.value().and_then(|v| v.units().clone());
        let (x, err) = self.mod_ab(left, right, target, self.precision)?;
        self.units = u;
        let mut eps = self.precision.sqrt();
        if target != 0.0 {
            eps *= target.abs();
        }
        if err > eps {
            return Ok(f64::NAN);
        }
        Ok(x)
    }

    pub fn find(&mut self, left: f64, right: f64) -> CResult<f64> {
        let u = self.variable.value().and_then(|v| v.units().clone());
        let (x, _) = self.mod_ab(left, right, 0.0, self.precision)?;
        self.units = u;
        Ok(x)
    }

    pub fn sup(&mut self, left: f64, right: f64) -> CResult<f64> {
        self.extremum(left, right, false)
    }

    pub fn inf(&mut self, left: f64, right: f64) -> CResult<f64> {
        self.extremum(left, right, true)
    }

    fn extremum(&mut self, left: f64, right: f64, is_min: bool) -> CResult<f64> {
        const K: f64 = 0.6180339887498948482;
        let (mut x1, mut x2) = if left < right { (left, right) } else { (right, left) };
        let mut d = x2 - x1;
        let d0 = 0.1 * d;
        let mut x3 = x2 - K * d;
        let mut x4 = x1 + K * d;
        let mut eps = self.precision * (x3.abs() + x4.abs()) / 2.0;
        let tol2 = 1e-30;
        let mut y3 = self.fd(x3)?;
        let mut y4 = self.fd(x4)?;
        while d > eps {
            if y3 == y4 && d < d0 {
                break;
            }
            if is_min == (y3 < y4) {
                x2 = x4;
                x4 = x3;
                y4 = y3;
                d = x2 - x1;
                x3 = x2 - K * d;
                y3 = self.fd(x3)?;
            } else {
                x1 = x3;
                x3 = x4;
                y3 = y4;
                d = x2 - x1;
                x4 = x1 + K * d;
                y4 = self.fd(x4)?;
            }
            eps = self.precision * (x3.abs() + x4.abs());
            if eps < tol2 {
                eps = tol2;
            }
        }
        if x1 == left {
            return self.fd(left);
        }
        if x2 == right {
            return self.fd(right);
        }
        self.fd((x1 + x2) / 2.0)
    }

    pub fn area(&mut self, left: f64, right: f64) -> CResult<f64> {
        let (mut left, mut right) = (left, right);
        let mut k = 1.0;
        if left > right {
            std::mem::swap(&mut left, &mut right);
            k = -1.0;
        }
        let area = if right - left > 1e-14 * (left.abs() + right.abs()) {
            if self.quadrature_method == QuadratureMethods::AdaptiveLobatto {
                self.adaptive_lobatto(left, right)? * k
            } else {
                self.tanh_sinh(left, right)? * k
            }
        } else {
            let y = self.fd((left + right) / 2.0)?;
            (right - left) * y * k
        };
        let u = self.variable.value().and_then(|v| v.units().clone());
        if u.is_none() {
            return Ok(area);
        }
        if self.units.is_none() {
            self.units = u;
            return Ok(area);
        }
        let (nu, factor) = Unit::multiply(&self.units, &u, false);
        self.units = nu;
        Ok(area * factor)
    }

    fn adaptive_lobatto(&mut self, left: f64, right: f64) -> CResult<f64> {
        self.parser.solver_eps.set(self.precision.clamp(1e-14, 1e-4) / 2.0);
        let y1 = self.fd(left)?;
        let y3 = self.fd(right)?;
        self.lobatto(left, right, y1, y3, 1)
    }

    fn lobatto(&mut self, x1: f64, x3: f64, y1: f64, y3: f64, depth: i32) -> CResult<f64> {
        let alpha = (2.0f64 / 3.0).sqrt();
        let beta = (1.0f64 / 5.0).sqrt();
        const K1: f64 = 1.0 / 1470.0;
        const K2: f64 = 1.0 / 6.0;
        let h = (x3 - x1) / 2.0;
        let x2 = (x1 + x3) / 2.0;
        let ah = alpha * h;
        let bh = beta * h;
        let x4 = x2 - ah;
        let x5 = x2 - bh;
        let x6 = x2 + bh;
        let x7 = x2 + ah;
        let y4 = self.fd(x4)?;
        let y5 = self.fd(x5)?;
        let y2 = self.fd(x2)?;
        let y6 = self.fd(x6)?;
        let y7 = self.fd(x7)?;
        let y13 = y1 + y3;
        let y56 = y5 + y6;
        let a1 = h * K1 * (77.0 * y13 + 432.0 * (y4 + y7) + 625.0 * y56 + 672.0 * y2);
        let a2 = h * K2 * (y13 + 5.0 * y56);
        if depth == 1 {
            if a1.is_finite() && a1 > 1.0 {
                self.parser.solver_eps.set(self.parser.solver_eps.get() * a1);
            }
        } else if (a1 - a2).abs() < self.parser.solver_eps.get() || depth > 15 {
            return Ok(a1);
        }
        let depth = depth + 1;
        Ok(self.lobatto(x1, x4, y1, y4, depth)?
            + self.lobatto(x4, x5, y4, y5, depth)?
            + self.lobatto(x5, x2, y5, y2, depth)?
            + self.lobatto(x2, x6, y2, y6, depth)?
            + self.lobatto(x6, x7, y6, y7, depth)?
            + self.lobatto(x7, x3, y7, y3, depth)?)
    }

    fn tanh_sinh(&mut self, left: f64, right: f64) -> CResult<f64> {
        let precision = self.precision;
        let c = (left + right) / 2.0;
        let d = (right - left) / 2.0;
        let mut s = self.fd(c)?;
        let mut err;
        let mut i = 0usize;
        let eps = (precision * 0.1).clamp(1e-15, 1e-8);
        let tol = 10.0 * precision;
        loop {
            let mut q;
            let mut p = 0.0;
            let mut fp = 0.0;
            let mut fm = 0.0;
            let mut j = 0;
            loop {
                let (rij, wij) = TABLE.with(|t| (t.r[i][j], t.w[i][j]));
                let x = rij * d;
                if left + x > left {
                    let y = self.fd(left + x)?;
                    if y.is_finite() {
                        fp = y;
                    }
                }
                if right - x < right {
                    let y = self.fd(right - x)?;
                    if y.is_finite() {
                        fm = y;
                    }
                }
                q = wij * (fp + fm);
                p += q;
                j += 1;
                if !(q.abs() > eps * p.abs() && j < M[i]) {
                    break;
                }
            }
            err = 2.0 * s;
            s += p;
            err = (err - s).abs();
            i += 1;
            if !(err > tol * s.abs() && i < TANH_SINH_DEPTH) {
                break;
            }
        }
        if s.abs() > 1.0 {
            err /= s.abs();
        }
        if err > 10.0 * tol {
            return Ok(f64::NAN);
        }
        Ok(d * s * 2f64.powf(1.0 - i as f64))
    }

    pub fn slope(&mut self, x: f64) -> CResult<f64> {
        let delta = self.precision.sqrt().min(1e-3);
        let max_err = (50.0 * self.precision).max(1e-3);
        const N: usize = 7;
        let a = if x.abs() < 1.0 { 1.0 } else { x };
        let eps = (bit_increment(a) - a).cbrt();
        let mut h = 2f64.powf(N as f64) * eps;
        let mut h2 = 2.0 * h;
        let mut r = [0f64; N];
        let mut err = delta / 2.0;
        for i in 0..N {
            let x1 = x - h;
            let x2 = x1 + h2;
            let mut d = 1.0;
            let r0 = r[0];
            r[i] = (self.fd(x2)? - self.fd(x1)?) / h2;
            let mut k = i as isize - 1;
            while k >= 0 {
                d *= 4.0;
                let ku = k as usize;
                let k1 = ku + 1;
                r[ku] = r[k1] + (r[k1] - r[ku]) / (d - 1.0);
                k -= 1;
            }
            if i >= 1 {
                err = if r[0].abs() <= delta {
                    (r[0] - r0).abs()
                } else {
                    ((r[0] - r0) / r[0]).abs()
                };
                if err < delta {
                    break;
                }
            }
            h2 = h;
            h = h2 / 2.0;
        }
        let u = self.variable.value().and_then(|v| v.units().clone());
        let slope = if err > max_err { f64::NAN } else { r[0] };
        if u.is_none() {
            return Ok(slope);
        }
        if self.units.is_none() {
            self.units = u;
            return Ok(slope);
        }
        let (nu, factor) = Unit::divide(&self.units, &u, false);
        self.units = nu;
        Ok(slope * factor)
    }

    pub fn repeat(&mut self, start: f64, end: f64) -> CResult<Value> {
        let (n1, n2) = get_bounds(start, end)?;
        let mut result = Value::nan();
        let mut i = n1;
        while i <= n2 {
            result = self.fi(i as f64)?;
            if result.re().is_infinite() {
                break;
            }
            i += 1;
        }
        Ok(result)
    }

    pub fn sum(&mut self, start: f64, end: f64) -> CResult<f64> {
        let (n1, n2) = get_bounds(start, end)?;
        let mut sum = self.fd(n1 as f64)?;
        let units = self.units.clone();
        let mut c = 0.0;
        let mut i = n1 + 1;
        while i <= n2 {
            let mut d = self.fd(i as f64)?;
            if !Unit::ptr_eq(&units, &self.units) {
                self.check_units(&units)?;
                d *= self.units.as_ref().unwrap().convert_to(units.as_ref().unwrap());
            }
            let y = d - c;
            let t = sum + y;
            c = (t - sum) - y;
            sum = t;
            if sum.is_infinite() {
                break;
            }
            i += 1;
        }
        self.units = units;
        Ok(sum)
    }

    fn check_units(&self, units: &OptUnit) -> CResult<()> {
        if !Unit::is_consistent_opt(units, &self.units) {
            return Err(exceptions::InconsistentUnits(
                &Unit::get_text_of(units),
                &Unit::get_text_of(&self.units),
            ));
        }
        Ok(())
    }

    pub fn product(&mut self, start: f64, end: f64) -> CResult<f64> {
        let (n1, n2) = get_bounds(start, end)?;
        let mut number = self.fd(n1 as f64)?;
        let mut units = self.units.clone();
        let has_units = units.is_some();
        let mut i = n1 + 1;
        while i <= n2 {
            number *= self.fd(i as f64)?;
            if has_units && self.units.is_some() {
                let (nu, factor) = Unit::multiply(&units, &self.units, false);
                units = nu;
                number *= factor;
            }
            if number.is_infinite() {
                break;
            }
            i += 1;
        }
        self.units = units;
        Ok(number)
    }

    pub fn complex_repeat(&mut self, start: f64, end: f64) -> CResult<Complex> {
        let (n1, n2) = get_bounds(start, end)?;
        let mut number = Complex::real(0.0);
        let mut i = n1;
        while i <= n2 {
            number = self.fc(Complex::real(i as f64))?;
            if number.re().is_infinite() || number.im().is_infinite() {
                break;
            }
            i += 1;
        }
        Ok(number)
    }

    pub fn complex_sum(&mut self, start: f64, end: f64) -> CResult<Complex> {
        let (n1, n2) = get_bounds(start, end)?;
        let mut sum = self.fc(Complex::real(n1 as f64))?;
        let units = self.units.clone();
        let mut c = Complex::ZERO;
        let mut i = n1 + 1;
        while i <= n2 {
            let mut d = self.fc(Complex::real(i as f64))?;
            if !Unit::ptr_eq(&units, &self.units) {
                self.check_units(&units)?;
                d = d * self.units.as_ref().unwrap().convert_to(units.as_ref().unwrap());
            }
            let y = d - c;
            let t = sum + y;
            c = (t - sum) - y;
            sum = t;
            if sum.re().is_infinite() || sum.im().is_infinite() {
                break;
            }
            i += 1;
        }
        self.units = units;
        Ok(sum)
    }

    pub fn complex_product(&mut self, start: f64, end: f64) -> CResult<Complex> {
        let (n1, n2) = get_bounds(start, end)?;
        let mut number = self.fc(Complex::real(n1 as f64))?;
        let mut units = self.units.clone();
        let has_units = units.is_some();
        let mut i = n1 + 1;
        while i <= n2 {
            number = number * self.fc(Complex::real(i as f64))?;
            if has_units && self.units.is_some() {
                let (nu, factor) = Unit::multiply(&units, &self.units, false);
                units = nu;
                number = number * factor;
            }
            if number.re().is_infinite() || number.im().is_infinite() {
                break;
            }
            i += 1;
        }
        self.units = units;
        Ok(number)
    }
}

fn get_bounds(start: f64, end: f64) -> CResult<(i64, i64)> {
    if start.abs() > LIMITS || end.abs() > LIMITS {
        return Err(exceptions::IterationLimits(
            &double_to_string(-LIMITS),
            &double_to_string(LIMITS),
        ));
    }
    Ok((start.round() as i32 as i64, end.round() as i32 as i64))
}

/// `Math.BitIncrement`
fn bit_increment(x: f64) -> f64 {
    if x.is_nan() || x == f64::INFINITY {
        return x;
    }
    if x == 0.0 {
        return f64::from_bits(1);
    }
    let bits = x.to_bits();
    if x > 0.0 {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    }
}
