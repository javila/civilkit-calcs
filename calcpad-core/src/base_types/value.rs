//! Port of BaseTypes/IValue.cs and BaseTypes/IScalarValue.cs.
//!
//! `IValue` is an interface implemented by RealValue, ComplexValue, Vector and Matrix in
//! C#. Here it is the `Value` enum. Phase 1 covers the scalar variants; vectors and
//! matrices will be added as further variants.
use crate::base_types::complex::Complex;
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::{RealValue, LOGICAL_ZERO};
use crate::base_types::unit::{OptUnit, Unit};
use crate::calculator::Calculator;
use crate::exceptions::{self, Items};
use crate::extension_methods::almost_equals;
use crate::math_parser_exception::CResult;

#[derive(Clone, Debug)]
pub enum Value {
    Real(RealValue),
    Complex(ComplexValue),
}

impl From<RealValue> for Value {
    fn from(v: RealValue) -> Value {
        Value::Real(v)
    }
}

impl From<ComplexValue> for Value {
    fn from(v: ComplexValue) -> Value {
        Value::Complex(v)
    }
}

impl Value {
    pub fn zero() -> Value {
        Value::Real(RealValue::ZERO)
    }

    pub fn nan() -> Value {
        Value::Real(RealValue::NAN)
    }

    // ----- IScalarValue members ------------------------------------------------------

    pub fn re(&self) -> f64 {
        match self {
            Value::Real(r) => r.d,
            Value::Complex(c) => c.a,
        }
    }

    pub fn im(&self) -> f64 {
        match self {
            Value::Real(_) => 0.0,
            Value::Complex(c) => c.b,
        }
    }

    pub fn units(&self) -> &OptUnit {
        match self {
            Value::Real(r) => &r.units,
            Value::Complex(c) => &c.units,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            Value::Real(r) => r.is_unit,
            Value::Complex(c) => c.is_unit,
        }
    }

    pub fn is_real(&self) -> bool {
        match self {
            Value::Real(_) => true,
            Value::Complex(c) => c.is_real(),
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            Value::Real(_) => false,
            Value::Complex(c) => c.is_complex(),
        }
    }

    pub fn complex(&self) -> Complex {
        match self {
            Value::Real(r) => Complex::real(r.d),
            Value::Complex(c) => c.complex(),
        }
    }

    pub fn is_composite(&self) -> bool {
        match self {
            Value::Real(r) => r.is_composite(),
            Value::Complex(c) => c.is_composite(),
        }
    }

    /// `IScalarValue.AsReal()` (keeps the IsUnit flag)
    pub fn as_real(&self) -> RealValue {
        match self {
            Value::Real(r) => r.clone(),
            Value::Complex(c) => RealValue::with_flag(c.a, c.units.clone(), c.is_unit),
        }
    }

    /// `IScalarValue.AsComplex()` (keeps the IsUnit flag)
    pub fn as_complex(&self) -> ComplexValue {
        match self {
            Value::Real(r) => ComplexValue {
                a: r.d,
                b: 0.0,
                units: r.units.clone(),
                is_unit: r.is_unit,
            },
            Value::Complex(c) => c.clone(),
        }
    }

    /// Implicit (ComplexValue) conversion: drops the IsUnit flag of a RealValue.
    fn implicit_complex(&self) -> ComplexValue {
        match self {
            Value::Real(r) => ComplexValue::from_real(r),
            Value::Complex(c) => c.clone(),
        }
    }

    pub fn is_scalar(&self) -> bool {
        true
    }

    /// `Equals(object)`
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Real(a), Value::Real(b)) => a.equals(b),
            (Value::Complex(a), Value::Complex(b)) => a.equals(b),
            _ => false,
        }
    }

    /// `ToString()` of the boxed value (used in some error messages)
    pub fn to_cs_string(&self) -> String {
        match self {
            Value::Real(r) => r.to_cs_string(),
            Value::Complex(c) => c.to_cs_string(),
        }
    }

    // ----- IValue operators ------------------------------------------------------------

    pub fn neg(&self) -> Value {
        match self {
            Value::Real(r) => Value::Real(r.neg()),
            Value::Complex(c) => Value::Complex(c.neg()),
        }
    }

    /// `IValue + IValue`
    pub fn add(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::add(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::add(
            &a.as_complex(),
            &b.as_complex(),
        )?))
    }

    pub fn sub(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::sub(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::sub(
            &a.as_complex(),
            &b.as_complex(),
        )?))
    }

    pub fn mul(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::mul_op(ra, rb)));
        }
        Ok(Value::Complex(ComplexValue::mul(
            &a.as_complex(),
            &b.as_complex(),
        )))
    }

    pub fn div(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::div_op(ra, rb)));
        }
        Ok(Value::Complex(ComplexValue::div(
            &a.as_complex(),
            &b.as_complex(),
        )))
    }

    pub fn rem(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::rem(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::rem(
            &a.as_complex(),
            &b.as_complex(),
        )?))
    }

    pub fn lt(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::lt_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::lt_op(
            &a.as_complex(),
            &b.as_complex(),
        )?))
    }

    pub fn gt(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::gt_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::gt_op(
            &a.as_complex(),
            &b.as_complex(),
        )?))
    }

    pub fn le(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::le_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::le_op(
            &a.as_complex(),
            &b.as_complex(),
        )?))
    }

    pub fn ge(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::ge_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::ge_op(
            &a.as_complex(),
            &b.as_complex(),
        )?))
    }

    /// `IValue.Equal` (uses the IScalarValue static method)
    pub fn equal(a: &Value, b: &Value) -> CResult<Value> {
        match (a, b) {
            (Value::Real(ra), Value::Real(rb)) => Ok(Value::Real(RealValue::eq_op(ra, rb)?)),
            _ => Ok(Value::Complex(ComplexValue::eq_op(
                &a.as_complex(),
                &b.as_complex(),
            )?)),
        }
    }

    pub fn not_equal(a: &Value, b: &Value) -> CResult<Value> {
        match (a, b) {
            (Value::Real(ra), Value::Real(rb)) => Ok(Value::Real(RealValue::ne_op(ra, rb)?)),
            _ => Ok(Value::Complex(ComplexValue::ne_op(
                &a.as_complex(),
                &b.as_complex(),
            )?)),
        }
    }

    pub fn and(a: &Value, b: &Value) -> Value {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Value::Real(RealValue::and_op(ra, rb));
        }
        Value::Real(ComplexValue::and_op(&a.as_complex(), &b.as_complex()))
    }

    pub fn or(a: &Value, b: &Value) -> Value {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Value::Real(RealValue::or_op(ra, rb));
        }
        Value::Real(ComplexValue::or_op(&a.as_complex(), &b.as_complex()))
    }

    pub fn xor(a: &Value, b: &Value) -> Value {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Value::Real(RealValue::xor_op(ra, rb));
        }
        Value::Real(ComplexValue::xor_op(&a.as_complex(), &b.as_complex()))
    }

    // ----- IScalarValue operators (implicit (ComplexValue) conversion) ------------------

    pub fn s_add(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::add(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::add(
            &a.implicit_complex(),
            &b.implicit_complex(),
        )?))
    }

    pub fn s_sub(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::sub(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::sub(
            &a.implicit_complex(),
            &b.implicit_complex(),
        )?))
    }

    pub fn s_mul(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::mul_op(ra, rb)));
        }
        Ok(Value::Complex(ComplexValue::mul(
            &a.implicit_complex(),
            &b.implicit_complex(),
        )))
    }

    pub fn s_mul_f(a: &Value, b: f64) -> Value {
        match a {
            Value::Real(r) => Value::Real(r.mul_f(b)),
            Value::Complex(c) => Value::Complex(c.mul_f(b)),
        }
    }

    pub fn s_sub_neg(a: &Value) -> Value {
        a.neg()
    }

    // ----- static helpers ----------------------------------------------------------------

    pub fn as_value(&self, _item: Items) -> CResult<&Value> {
        Ok(self)
    }

    /// `IValue.AsReal(ival, item)`
    pub fn as_real_checked(&self, _item: Items) -> CResult<RealValue> {
        match self {
            Value::Real(r) => Ok(r.clone()),
            Value::Complex(c) => c.to_real(),
        }
    }

    /// `IValue.AsInt` / `IScalarValue.AsInt`
    pub fn as_int(&self, item: Items) -> CResult<i32> {
        if self.is_real() && self.units().is_none() {
            let d = self.re();
            if d > 0.0 && d <= i32::MAX as f64 && almost_equals(d, d.trunc()) {
                return Ok(d as i32);
            }
        }
        Err(exceptions::MustBePositiveInteger(item))
    }

    /// `IValue.EvaluateSwitch`
    pub fn evaluate_switch(values: &[Value]) -> Value {
        let len = values.len();
        let mut i = 0;
        while i + 1 < len {
            if values[i].re().abs() >= LOGICAL_ZERO {
                return values[i + 1].clone();
            }
            i += 2;
        }
        if len % 2 != 0 {
            return values[len - 1].clone();
        }
        Value::nan()
    }

    /// `IValue.EvaluateFunction(calc, index, a)` for scalars
    pub fn evaluate_function(calc: &Calculator, index: i64, a: &Value) -> CResult<Value> {
        calc.evaluate_function(index, a)
    }

    pub fn evaluate_operator(
        calc: &Calculator,
        index: i64,
        a: &Value,
        b: &Value,
    ) -> CResult<Value> {
        calc.evaluate_operator(index, a, b)
    }

    pub fn evaluate_function2(
        calc: &Calculator,
        index: i64,
        a: &Value,
        b: &Value,
    ) -> CResult<Value> {
        calc.evaluate_function2(index, a, b)
    }

    pub fn evaluate_interpolation(
        calc: &Calculator,
        index: i64,
        values: &[Value],
    ) -> CResult<Value> {
        calc.evaluate_interpolation(index, values)
    }

    pub fn evaluate_multi_function(
        calc: &Calculator,
        index: i64,
        values: &[Value],
    ) -> CResult<Value> {
        if index == crate::calculator::SWITCH_INDEX {
            return Ok(Value::evaluate_switch(values));
        }
        calc.evaluate_multi_function(index, values)
    }

    pub fn units_text(&self) -> String {
        Unit::get_text_of(self.units())
    }
}
