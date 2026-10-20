//! Port of BaseTypes/RealValue.cs.
use crate::base_types::complex::dbl_equals;
use crate::base_types::unit::{OptUnit, Unit};
use crate::exceptions;
use crate::extension_methods::almost_equals;
use crate::math_parser_exception::CResult;
use crate::netfmt;

pub const LOGICAL_ZERO: f64 = 1e-12;

#[derive(Clone, Debug)]
pub struct RealValue {
    pub d: f64,
    pub units: OptUnit,
    pub is_unit: bool,
}

impl std::fmt::Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unit({})", self.text())
    }
}

impl RealValue {
    pub const ZERO: RealValue = RealValue::num(0.0);
    pub const ONE: RealValue = RealValue::num(1.0);
    pub const NAN: RealValue = RealValue::num(f64::NAN);
    pub const POSITIVE_INFINITY: RealValue = RealValue::num(f64::INFINITY);
    pub const NEGATIVE_INFINITY: RealValue = RealValue::num(f64::NEG_INFINITY);

    #[inline]
    pub const fn num(d: f64) -> RealValue {
        RealValue {
            d,
            units: None,
            is_unit: false,
        }
    }

    #[inline]
    pub fn new(d: f64, units: OptUnit) -> RealValue {
        RealValue {
            d,
            units,
            is_unit: false,
        }
    }

    /// `new RealValue(Unit units)`: a unit literal.
    pub fn unit(units: OptUnit) -> RealValue {
        RealValue {
            d: 1.0,
            units,
            is_unit: true,
        }
    }

    pub fn with_flag(d: f64, units: OptUnit, is_unit: bool) -> RealValue {
        RealValue { d, units, is_unit }
    }

    /// `Equals(RealValue)`
    pub fn equals(&self, other: &RealValue) -> bool {
        match (&self.units, &other.units) {
            (None, None) => dbl_equals(self.d, other.d),
            (Some(a), Some(b)) => dbl_equals(self.d, other.d) && Unit::equals(a, b),
            _ => false,
        }
    }

    pub fn is_composite(&self) -> bool {
        Unit::is_composite(self.d, &self.units)
    }

    pub fn neg(&self) -> RealValue {
        RealValue::with_flag(-self.d, self.units.clone(), self.is_unit)
    }

    pub fn add(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        Ok(RealValue::new(
            a.d + b.d * Unit::convert(&a.units, &b.units, '+')?,
            a.units.clone(),
        ))
    }

    pub fn sub(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        Ok(RealValue::new(
            a.d - b.d * Unit::convert(&a.units, &b.units, '-')?,
            a.units.clone(),
        ))
    }

    /// `operator *`
    pub fn mul_op(a: &RealValue, b: &RealValue) -> RealValue {
        if a.units.is_none() {
            if let Some(bu) = &b.units {
                if bu.is_dimensionless() && !b.is_unit {
                    return RealValue::new(a.d * b.d * bu.get_dimensionless_factor(), None);
                }
            }
            return RealValue::new(a.d * b.d, b.units.clone());
        }
        let (uc, d) = Unit::multiply(&a.units, &b.units, false);
        RealValue::new(a.d * b.d * d, uc)
    }

    /// `RealValue.Multiply`
    pub fn multiply(a: &RealValue, b: &RealValue) -> RealValue {
        if a.units.is_none() {
            if let Some(bu) = &b.units {
                if bu.is_dimensionless() && !b.is_unit {
                    return RealValue::new(a.d * b.d * bu.get_dimensionless_factor(), None);
                }
            }
            return RealValue::new(a.d * b.d, b.units.clone());
        }
        let (uc, d) = Unit::multiply(&a.units, &b.units, b.is_unit);
        let is_unit = a.is_unit && b.is_unit && uc.is_some();
        RealValue::with_flag(a.d * b.d * d, uc, is_unit)
    }

    /// `operator /`
    pub fn div_op(a: &RealValue, b: &RealValue) -> RealValue {
        let (uc, d) = Unit::divide(&a.units, &b.units, false);
        RealValue::new(a.d / b.d * d, uc)
    }

    /// `RealValue.Divide`
    pub fn divide(a: &RealValue, b: &RealValue) -> RealValue {
        let (uc, d) = Unit::divide(&a.units, &b.units, b.is_unit);
        let is_unit = a.is_unit && b.is_unit && uc.is_some();
        RealValue::with_flag(a.d / b.d * d, uc, is_unit)
    }

    pub fn mul_f(&self, b: f64) -> RealValue {
        RealValue::new(self.d * b, self.units.clone())
    }

    pub fn rem(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        if b.units.is_some() {
            return Err(exceptions::CannotEvaluateRemainder(
                &Unit::get_text_of(&a.units),
                &Unit::get_text_of(&b.units),
            ));
        }
        Ok(RealValue::new(a.d % b.d, a.units.clone()))
    }

    pub fn int_div(a: &RealValue, b: &RealValue) -> RealValue {
        let (uc, d) = Unit::divide(&a.units, &b.units, false);
        let is_unit = a.is_unit && b.is_unit && uc.is_some();
        let c = if b.d == 0.0 {
            f64::NAN
        } else {
            (a.d / b.d * d).trunc()
        };
        RealValue::with_flag(c, uc, is_unit)
    }

    fn bool_val(b: bool) -> RealValue {
        if b {
            RealValue::ONE
        } else {
            RealValue::ZERO
        }
    }

    pub fn eq_op(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        Ok(RealValue::bool_val(almost_equals(
            a.d,
            b.d * Unit::convert(&a.units, &b.units, '\u{2261}')?,
        )))
    }

    pub fn ne_op(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        Ok(RealValue::bool_val(!almost_equals(
            a.d,
            b.d * Unit::convert(&a.units, &b.units, '\u{2260}')?,
        )))
    }

    pub fn lt_op(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        let c = a.d;
        let d = b.d * Unit::convert(&a.units, &b.units, '<')?;
        Ok(RealValue::bool_val(c < d && !almost_equals(c, d)))
    }

    pub fn gt_op(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        let c = a.d;
        let d = b.d * Unit::convert(&a.units, &b.units, '>')?;
        Ok(RealValue::bool_val(c > d && !almost_equals(c, d)))
    }

    pub fn le_op(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        let c = a.d;
        let d = b.d * Unit::convert(&a.units, &b.units, '\u{2264}')?;
        Ok(RealValue::bool_val(c <= d || almost_equals(c, d)))
    }

    pub fn ge_op(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        let c = a.d;
        let d = b.d * Unit::convert(&a.units, &b.units, '\u{2265}')?;
        Ok(RealValue::bool_val(c >= d || almost_equals(c, d)))
    }

    pub fn and_op(a: &RealValue, b: &RealValue) -> RealValue {
        RealValue::bool_val(!(a.d.abs() < LOGICAL_ZERO || b.d.abs() < LOGICAL_ZERO))
    }

    pub fn or_op(a: &RealValue, b: &RealValue) -> RealValue {
        RealValue::bool_val(a.d.abs() >= LOGICAL_ZERO || b.d.abs() >= LOGICAL_ZERO)
    }

    pub fn xor_op(a: &RealValue, b: &RealValue) -> RealValue {
        RealValue::bool_val((a.d.abs() >= LOGICAL_ZERO) != (b.d.abs() >= LOGICAL_ZERO))
    }

    /// `ToString()`
    pub fn to_cs_string(&self) -> String {
        if self.is_unit {
            return self.units.as_ref().map(|u| u.text()).unwrap_or_default();
        }
        let s = match &self.units {
            None => String::new(),
            Some(u) => u.text(),
        };
        format!("{}{}", netfmt::double_to_string(self.d), s)
    }
}
