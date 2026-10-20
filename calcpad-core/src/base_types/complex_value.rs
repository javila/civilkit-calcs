//! Port of BaseTypes/ComplexValue.cs.
use crate::base_types::complex::{dbl_equals, Complex, Types};
use crate::base_types::real_value::{RealValue, LOGICAL_ZERO};
use crate::base_types::unit::{OptUnit, Unit};
use crate::exceptions::{self, Items};
use crate::math_parser_exception::CResult;
use crate::netfmt;

#[derive(Clone, Debug)]
pub struct ComplexValue {
    pub a: f64,
    pub b: f64,
    pub units: OptUnit,
    pub is_unit: bool,
}

impl ComplexValue {
    pub const COMPLEX_INFINITY: ComplexValue = ComplexValue {
        a: f64::INFINITY,
        b: f64::INFINITY,
        units: None,
        is_unit: false,
    };

    pub fn new(re: f64, im: f64, units: OptUnit) -> ComplexValue {
        ComplexValue {
            a: re,
            b: im,
            units,
            is_unit: false,
        }
    }

    pub fn from_c(c: Complex, units: OptUnit) -> ComplexValue {
        ComplexValue::new(c.re(), c.im(), units)
    }

    pub fn num(d: f64) -> ComplexValue {
        ComplexValue::new(d, 0.0, None)
    }

    pub fn c(c: Complex) -> ComplexValue {
        ComplexValue::new(c.re(), c.im(), None)
    }

    pub fn unit(units: OptUnit) -> ComplexValue {
        ComplexValue {
            a: 1.0,
            b: 0.0,
            units,
            is_unit: true,
        }
    }

    pub fn with_flag(c: Complex, units: OptUnit, is_unit: bool) -> ComplexValue {
        ComplexValue {
            a: c.re(),
            b: c.im(),
            units,
            is_unit,
        }
    }

    pub fn is_real(&self) -> bool {
        Complex::get_type(self.a, self.b) == Types::Real
    }

    pub fn is_complex(&self) -> bool {
        Complex::get_type(self.a, self.b) == Types::Complex
    }

    pub fn complex(&self) -> Complex {
        Complex::new(self.a, self.b)
    }

    pub fn is_zero(&self) -> bool {
        self.a == 0.0 && self.b == 0.0
    }

    pub fn equals(&self, other: &ComplexValue) -> bool {
        match (&self.units, &other.units) {
            (None, None) => dbl_equals(self.a, other.a) && dbl_equals(self.b, other.b),
            (Some(x), Some(y)) => dbl_equals(self.a, other.a) && dbl_equals(self.b, other.b) && Unit::equals(x, y),
            _ => false,
        }
    }

    pub fn is_composite(&self) -> bool {
        Unit::is_composite(self.a, &self.units)
    }

    pub fn neg(&self) -> ComplexValue {
        ComplexValue {
            a: -self.a,
            b: -self.b,
            units: self.units.clone(),
            is_unit: self.is_unit,
        }
    }

    pub fn add(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        let c = a.complex() + b.complex() * Unit::convert(&a.units, &b.units, '+')?;
        Ok(ComplexValue::from_c(c, a.units.clone()))
    }

    pub fn sub(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        let c = a.complex() - b.complex() * Unit::convert(&a.units, &b.units, '+')?;
        Ok(ComplexValue::from_c(c, a.units.clone()))
    }

    pub fn mul(a: &ComplexValue, b: &ComplexValue) -> ComplexValue {
        if a.units.is_none() {
            if let Some(bu) = &b.units {
                if bu.is_dimensionless() && !b.is_unit {
                    return ComplexValue::from_c(a.complex() * b.complex() * bu.get_dimensionless_factor(), None);
                }
            }
            return ComplexValue::from_c(a.complex() * b.complex(), b.units.clone());
        }
        let (uc, d) = Unit::multiply(&a.units, &b.units, b.is_unit);
        let c = a.complex() * b.complex() * d;
        ComplexValue::with_flag(c, uc, a.is_unit && b.is_unit)
    }

    pub fn mul_f(&self, b: f64) -> ComplexValue {
        ComplexValue::new(self.a * b, self.b * b, self.units.clone())
    }

    pub fn div(a: &ComplexValue, b: &ComplexValue) -> ComplexValue {
        let (uc, d) = Unit::divide(&a.units, &b.units, b.is_unit);
        let c = (a.complex() / b.complex()) * d;
        ComplexValue::with_flag(c, uc, a.is_unit && b.is_unit)
    }

    pub fn rem(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        if b.units.is_some() {
            return Err(exceptions::RemainderUnits(
                &Unit::get_text_of(&a.units),
                &Unit::get_text_of(&b.units),
            ));
        }
        Ok(ComplexValue::from_c(
            Complex::rem(a.complex(), b.complex()),
            a.units.clone(),
        ))
    }

    pub fn int_div(a: &ComplexValue, b: &ComplexValue) -> ComplexValue {
        let (uc, d) = Unit::divide(&a.units, &b.units, false);
        let c = Complex::int_div(a.complex() * d, b.complex());
        ComplexValue::with_flag(c, uc, a.is_unit && b.is_unit)
    }

    pub fn eq_op(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        Ok(ComplexValue::num(Complex::eq_op(
            a.complex(),
            b.complex() * Unit::convert(&a.units, &b.units, '\u{2261}')?,
        )))
    }

    pub fn ne_op(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        Ok(ComplexValue::num(Complex::ne_op(
            a.complex(),
            b.complex() * Unit::convert(&a.units, &b.units, '\u{2260}')?,
        )))
    }

    pub fn lt_op(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        Ok(ComplexValue::num(Complex::lt_op(
            a.complex(),
            b.complex() * Unit::convert(&a.units, &b.units, '<')?,
        )))
    }

    pub fn gt_op(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        Ok(ComplexValue::num(Complex::gt_op(
            a.complex(),
            b.complex() * Unit::convert(&a.units, &b.units, '>')?,
        )))
    }

    pub fn le_op(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        Ok(ComplexValue::num(Complex::le_op(
            a.complex(),
            b.complex() * Unit::convert(&a.units, &b.units, '\u{2264}')?,
        )))
    }

    pub fn ge_op(a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
        Ok(ComplexValue::num(Complex::ge_op(
            a.complex(),
            b.complex() * Unit::convert(&a.units, &b.units, '\u{2265}')?,
        )))
    }

    pub fn and_op(a: &ComplexValue, b: &ComplexValue) -> RealValue {
        if a.a.abs() < LOGICAL_ZERO || b.a.abs() < LOGICAL_ZERO {
            RealValue::ZERO
        } else {
            RealValue::ONE
        }
    }

    pub fn or_op(a: &ComplexValue, b: &ComplexValue) -> RealValue {
        if a.a.abs() >= LOGICAL_ZERO || b.a.abs() >= LOGICAL_ZERO {
            RealValue::ONE
        } else {
            RealValue::ZERO
        }
    }

    pub fn xor_op(a: &ComplexValue, b: &ComplexValue) -> RealValue {
        if (a.a.abs() >= LOGICAL_ZERO) != (b.a.abs() >= LOGICAL_ZERO) {
            RealValue::ONE
        } else {
            RealValue::ZERO
        }
    }

    /// implicit RealValue -> ComplexValue
    pub fn from_real(v: &RealValue) -> ComplexValue {
        ComplexValue::new(v.d, 0.0, v.units.clone())
    }

    /// explicit ComplexValue -> RealValue
    pub fn to_real(&self) -> CResult<RealValue> {
        if !self.is_real() {
            return Err(exceptions::MustBeReal(Items::Value));
        }
        Ok(RealValue::new(self.a, self.units.clone()))
    }

    pub fn to_cs_string(&self) -> String {
        let s = match &self.units {
            None => String::new(),
            Some(u) => format!(" {}", u.text()),
        };
        // {B: + 0i; - 0i;#}
        let im = if self.b > 0.0 {
            format!(" + {}i", netfmt::format_custom(self.b, "0"))
        } else if self.b < 0.0 {
            format!(" - {}i", netfmt::format_custom(-self.b, "0"))
        } else {
            String::new()
        };
        format!("{}{}{}", netfmt::double_to_string(self.a), im, s)
    }
}
