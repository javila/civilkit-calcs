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
use crate::matrix::{hp_matrix, Matrix};
use crate::vector::{hp_vector, Vector};

#[derive(Clone, Debug)]
pub enum Value {
    Real(RealValue),
    Complex(ComplexValue),
    Vector(Vector),
    Matrix(Matrix),
}

impl From<Vector> for Value {
    fn from(v: Vector) -> Value {
        Value::Vector(v)
    }
}

impl From<Matrix> for Value {
    fn from(m: Matrix) -> Value {
        Value::Matrix(m)
    }
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

const NO_UNITS: &OptUnit = &None;

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
            _ => f64::NAN,
        }
    }

    pub fn im(&self) -> f64 {
        match self {
            Value::Real(_) => 0.0,
            Value::Complex(c) => c.b,
            _ => 0.0,
        }
    }

    pub fn units(&self) -> &OptUnit {
        match self {
            Value::Real(r) => &r.units,
            Value::Complex(c) => &c.units,
            _ => NO_UNITS,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            Value::Real(r) => r.is_unit,
            Value::Complex(c) => c.is_unit,
            _ => false,
        }
    }

    pub fn is_real(&self) -> bool {
        match self {
            Value::Real(_) => true,
            Value::Complex(c) => c.is_real(),
            _ => false,
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            Value::Real(_) => false,
            Value::Complex(c) => c.is_complex(),
            _ => false,
        }
    }

    pub fn complex(&self) -> Complex {
        match self {
            Value::Real(r) => Complex::real(r.d),
            Value::Complex(c) => c.complex(),
            _ => Complex::real(f64::NAN),
        }
    }

    pub fn is_composite(&self) -> bool {
        match self {
            Value::Real(r) => r.is_composite(),
            Value::Complex(c) => c.is_composite(),
            _ => false,
        }
    }

    /// `IScalarValue.AsReal()` (keeps the IsUnit flag)
    pub fn as_real(&self) -> RealValue {
        match self {
            Value::Real(r) => r.clone(),
            Value::Complex(c) => RealValue::with_flag(c.a, c.units.clone(), c.is_unit),
            _ => RealValue::NAN,
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
            _ => ComplexValue::from_real(&RealValue::NAN),
        }
    }

    /// Implicit (ComplexValue) conversion: drops the IsUnit flag of a RealValue.
    fn implicit_complex(&self) -> ComplexValue {
        match self {
            Value::Real(r) => ComplexValue::from_real(r),
            Value::Complex(c) => c.clone(),
            _ => ComplexValue::from_real(&RealValue::NAN),
        }
    }

    /// `is IScalarValue`
    pub fn is_scalar(&self) -> bool {
        matches!(self, Value::Real(_) | Value::Complex(_))
    }

    pub fn is_vector(&self) -> bool {
        matches!(self, Value::Vector(_))
    }

    pub fn is_matrix(&self) -> bool {
        matches!(self, Value::Matrix(_))
    }

    /// `Equals(object)`
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Real(a), Value::Real(b)) => a.equals(b),
            (Value::Complex(a), Value::Complex(b)) => a.equals(b),
            (Value::Vector(a), Value::Vector(b)) => a.equals(b),
            (Value::Matrix(a), Value::Matrix(b)) => a.equals(b),
            _ => false,
        }
    }

    /// `ToString()` of the boxed value (used in some error messages)
    pub fn to_cs_string(&self) -> String {
        match self {
            Value::Real(r) => r.to_cs_string(),
            Value::Complex(c) => c.to_cs_string(),
            Value::Vector(v) => v.to_cs_string(),
            Value::Matrix(m) => m.to_cs_string(),
        }
    }

    // ----- IValue operators ------------------------------------------------------------

    fn scalar_neg(&self) -> Value {
        match self {
            Value::Real(r) => Value::Real(r.neg()),
            Value::Complex(c) => Value::Complex(c.neg()),
            _ => Value::nan(),
        }
    }

    /// `IValue + IValue`
    fn scalar_add(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::add(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::add(&a.as_complex(), &b.as_complex())?))
    }

    fn scalar_sub(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::sub(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::sub(&a.as_complex(), &b.as_complex())?))
    }

    fn scalar_mul(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::mul_op(ra, rb)));
        }
        Ok(Value::Complex(ComplexValue::mul(&a.as_complex(), &b.as_complex())))
    }

    fn scalar_div(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::div_op(ra, rb)));
        }
        Ok(Value::Complex(ComplexValue::div(&a.as_complex(), &b.as_complex())))
    }

    fn scalar_rem(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::rem(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::rem(&a.as_complex(), &b.as_complex())?))
    }

    fn scalar_lt(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::lt_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::lt_op(&a.as_complex(), &b.as_complex())?))
    }

    fn scalar_gt(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::gt_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::gt_op(&a.as_complex(), &b.as_complex())?))
    }

    fn scalar_le(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::le_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::le_op(&a.as_complex(), &b.as_complex())?))
    }

    fn scalar_ge(a: &Value, b: &Value) -> CResult<Value> {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Ok(Value::Real(RealValue::ge_op(ra, rb)?));
        }
        Ok(Value::Complex(ComplexValue::ge_op(&a.as_complex(), &b.as_complex())?))
    }

    /// `IValue.Equal` (uses the IScalarValue static method)
    fn scalar_equal(a: &Value, b: &Value) -> CResult<Value> {
        match (a, b) {
            (Value::Real(ra), Value::Real(rb)) => Ok(Value::Real(RealValue::eq_op(ra, rb)?)),
            _ => Ok(Value::Complex(ComplexValue::eq_op(&a.as_complex(), &b.as_complex())?)),
        }
    }

    fn scalar_not_equal(a: &Value, b: &Value) -> CResult<Value> {
        match (a, b) {
            (Value::Real(ra), Value::Real(rb)) => Ok(Value::Real(RealValue::ne_op(ra, rb)?)),
            _ => Ok(Value::Complex(ComplexValue::ne_op(&a.as_complex(), &b.as_complex())?)),
        }
    }

    fn scalar_and(a: &Value, b: &Value) -> Value {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Value::Real(RealValue::and_op(ra, rb));
        }
        Value::Real(ComplexValue::and_op(&a.as_complex(), &b.as_complex()))
    }

    fn scalar_or(a: &Value, b: &Value) -> Value {
        if let (Value::Real(ra), Value::Real(rb)) = (a, b) {
            return Value::Real(RealValue::or_op(ra, rb));
        }
        Value::Real(ComplexValue::or_op(&a.as_complex(), &b.as_complex()))
    }

    fn scalar_xor(a: &Value, b: &Value) -> Value {
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
            _ => a.clone(),
        }
    }

    pub fn s_sub_neg(a: &Value) -> Value {
        a.scalar_neg()
    }

    // ----- static helpers ----------------------------------------------------------------

    /// `IValue.AsValue(ival, item)`: the value as a scalar.
    pub fn as_value(&self, item: Items) -> CResult<&Value> {
        if self.is_scalar() {
            Ok(self)
        } else {
            Err(exceptions::MustBeScalar(item))
        }
    }

    /// `IValue.AsReal(ival, item)`
    pub fn as_real_checked(&self, item: Items) -> CResult<RealValue> {
        match self {
            Value::Real(r) => Ok(r.clone()),
            Value::Complex(c) => c.to_real(),
            _ => Err(exceptions::MustBeScalar(item)),
        }
    }

    /// `IValue.AsInt` / `IScalarValue.AsInt`
    pub fn as_int(&self, item: Items) -> CResult<i32> {
        self.as_value(item)?;
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

    pub fn evaluate_operator(calc: &Calculator, index: i64, a: &Value, b: &Value) -> CResult<Value> {
        calc.evaluate_operator(index, a, b)
    }

    pub fn evaluate_function2(calc: &Calculator, index: i64, a: &Value, b: &Value) -> CResult<Value> {
        calc.evaluate_function2(index, a, b)
    }

    pub fn evaluate_interpolation(calc: &Calculator, index: i64, values: &[Value]) -> CResult<Value> {
        calc.evaluate_interpolation(index, values)
    }

    pub fn evaluate_multi_function(calc: &Calculator, index: i64, values: &[Value]) -> CResult<Value> {
        if index == crate::calculator::SWITCH_INDEX {
            return Ok(Value::evaluate_switch(values));
        }
        calc.evaluate_multi_function(index, values)
    }

    pub fn units_text(&self) -> String {
        Unit::get_text_of(self.units())
    }
}

// ----- IValue operators with vector and matrix operands ------------------------------------

/// The binary operators of IValue.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Xor,
}

impl BinOp {
    /// True when `RealValue op Vector` is evaluated as `Vector op RealValue` in IValue.
    pub fn commutes(self) -> bool {
        matches!(
            self,
            BinOp::Add | BinOp::Mul | BinOp::Eq | BinOp::Ne | BinOp::And | BinOp::Or | BinOp::Xor
        )
    }

    /// The text of the InvalidOperand message, as `string.Format` in IValue.cs.
    fn invalid_operand(self, a: &Value, b: &Value) -> crate::math_parser_exception::MathParserException {
        let (a, b) = (a.to_cs_string(), b.to_cs_string());
        let s = match self {
            BinOp::Add => format!("{} + {}", a, b),
            BinOp::Sub => format!("{} - {}", a, b),
            BinOp::Mul => format!("{}*{}", a, b),
            BinOp::Div => format!("{}/{}", a, b),
            BinOp::Rem => format!("{}%{}", a, b),
            BinOp::Lt => format!("{} < {}", a, b),
            BinOp::Gt => format!("{} > {}", a, b),
            BinOp::Le => format!("{} \u{2264} {}", a, b),
            BinOp::Ge => format!("{} \u{2265} {}", a, b),
            BinOp::Eq => format!("{} \u{2261} {}", a, b),
            BinOp::Ne => format!("{} \u{2260} {}", a, b),
            BinOp::And => format!("{}\u{2227}{}.", a, b),
            BinOp::Or => format!("{}\u{2228}{}", a, b),
            BinOp::Xor => format!("{}\u{2295}{}", a, b),
        };
        exceptions::InvalidOperand(&s)
    }
}

/// `Vector op RealValue`
pub fn vector_op_vr(op: BinOp, a: &Vector, b: &RealValue) -> CResult<Vector> {
    match op {
        BinOp::Add => Vector::add_vr(a, b),
        BinOp::Sub => Vector::sub_vr(a, b),
        BinOp::Mul => Vector::mul_vr(a, b),
        BinOp::Div => Vector::div_vr(a, b),
        BinOp::Rem => Vector::rem_vr(a, b),
        BinOp::Lt => Vector::lt_vr(a, b),
        BinOp::Gt => Vector::gt_vr(a, b),
        BinOp::Le => Vector::le_vr(a, b),
        BinOp::Ge => Vector::ge_vr(a, b),
        BinOp::Eq => Vector::eq_vr(a, b),
        BinOp::Ne => Vector::ne_vr(a, b),
        BinOp::And => Vector::and_vr(a, b),
        BinOp::Or => Vector::or_vr(a, b),
        BinOp::Xor => Vector::xor_vr(a, b),
    }
}

/// `RealValue op Vector` (for the operators that do not commute)
pub fn vector_op_rv(op: BinOp, a: &RealValue, b: &Vector) -> CResult<Vector> {
    match op {
        BinOp::Sub => Vector::sub_rv(a, b),
        BinOp::Div => Vector::div_rv(a, b),
        BinOp::Rem => Vector::rem_rv(a, b),
        BinOp::Lt => Vector::lt_rv(a, b),
        BinOp::Gt => Vector::gt_rv(a, b),
        BinOp::Le => Vector::le_rv(a, b),
        BinOp::Ge => Vector::ge_rv(a, b),
        _ => vector_op_vr(op, b, a),
    }
}

/// `Vector op Vector`
pub fn vector_op_vv(op: BinOp, a: &Vector, b: &Vector) -> CResult<Vector> {
    match op {
        BinOp::Add => Vector::add(a, b),
        BinOp::Sub => Vector::sub(a, b),
        BinOp::Mul => Vector::mul(a, b),
        BinOp::Div => Vector::div(a, b),
        BinOp::Rem => Vector::rem(a, b),
        BinOp::Lt => Vector::lt(a, b),
        BinOp::Gt => Vector::gt(a, b),
        BinOp::Le => Vector::le(a, b),
        BinOp::Ge => Vector::ge(a, b),
        BinOp::Eq => Vector::eq(a, b),
        BinOp::Ne => Vector::ne(a, b),
        BinOp::And => Vector::and(a, b),
        BinOp::Or => Vector::or(a, b),
        BinOp::Xor => Vector::xor(a, b),
    }
}

/// `Matrix op RealValue`
pub fn matrix_op_mr(op: BinOp, a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    match op {
        BinOp::Add => Matrix::add_mr(a, b),
        BinOp::Sub => Matrix::sub_mr(a, b),
        BinOp::Mul => Matrix::mul_mr(a, b),
        BinOp::Div => Matrix::div_mr(a, b),
        BinOp::Rem => Matrix::rem_mr(a, b),
        BinOp::Lt => Matrix::lt_mr(a, b),
        BinOp::Gt => Matrix::gt_mr(a, b),
        BinOp::Le => Matrix::le_mr(a, b),
        BinOp::Ge => Matrix::ge_mr(a, b),
        BinOp::Eq => Matrix::eq_mr(a, b),
        BinOp::Ne => Matrix::ne_mr(a, b),
        BinOp::And => Matrix::and_mr(a, b),
        BinOp::Or => Matrix::or_mr(a, b),
        BinOp::Xor => Matrix::xor_mr(a, b),
    }
}

/// `RealValue op Matrix` (for the operators that do not commute)
pub fn matrix_op_rm(op: BinOp, a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    match op {
        BinOp::Sub => Matrix::sub_rm(a, b),
        BinOp::Div => Matrix::div_rm(a, b),
        BinOp::Rem => Matrix::rem_rm(a, b),
        BinOp::Lt => Matrix::lt_rm(a, b),
        BinOp::Gt => Matrix::gt_rm(a, b),
        BinOp::Le => Matrix::le_rm(a, b),
        BinOp::Ge => Matrix::ge_rm(a, b),
        _ => matrix_op_mr(op, b, a),
    }
}

/// `Matrix op Matrix`
pub fn matrix_op_mm(op: BinOp, a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    match op {
        BinOp::Add => Matrix::add(a, b),
        BinOp::Sub => Matrix::sub(a, b),
        BinOp::Mul => Matrix::mul(a, b),
        BinOp::Div => Matrix::div(a, b),
        BinOp::Rem => Matrix::rem(a, b),
        BinOp::Lt => Matrix::lt(a, b),
        BinOp::Gt => Matrix::gt(a, b),
        BinOp::Le => Matrix::le(a, b),
        BinOp::Ge => Matrix::ge(a, b),
        BinOp::Eq => Matrix::eq(a, b),
        BinOp::Ne => Matrix::ne(a, b),
        BinOp::And => Matrix::and(a, b),
        BinOp::Or => Matrix::or(a, b),
        BinOp::Xor => Matrix::xor(a, b),
    }
}

/// `c.RowCount == 1 ? c[0, 0] : c.ColRef()` after a matrix times column product.
fn column_result(c: Matrix) -> Value {
    if c.row_count() == 1 {
        return Value::Real(c.get(0, 0));
    }
    Value::Vector(c.row_ref(0))
}

impl Value {
    /// `-IValue`
    pub fn neg(&self) -> CResult<Value> {
        Ok(match self {
            Value::Real(_) | Value::Complex(_) => self.scalar_neg(),
            Value::Vector(v) => {
                if v.is_hp() {
                    Value::Vector(hp_vector::ops::neg(v)?)
                } else {
                    Value::Vector(Vector::neg(v))
                }
            }
            Value::Matrix(m) => {
                if m.is_hp() {
                    Value::Matrix(hp_matrix::ops::neg(m)?)
                } else {
                    Value::Matrix(Matrix::neg(m)?)
                }
            }
        })
    }

    /// The IValue binary operators.
    pub fn binary(op: BinOp, a: &Value, b: &Value) -> CResult<Value> {
        match a {
            Value::Real(ra) => match b {
                Value::Real(_) | Value::Complex(_) => {}
                Value::Vector(vb) => {
                    return Ok(Value::Vector(if vb.is_hp() {
                        if op.commutes() {
                            hp_vector::ops::op_vr(op, vb, ra)?
                        } else {
                            hp_vector::ops::op_rv(op, ra, vb)?
                        }
                    } else {
                        vector_op_rv(op, ra, vb)?
                    }))
                }
                Value::Matrix(mb) => {
                    return Ok(Value::Matrix(if mb.is_hp() {
                        if op.commutes() {
                            hp_matrix::ops::op_mr(op, mb, ra)?
                        } else {
                            hp_matrix::ops::op_rm(op, ra, mb)?
                        }
                    } else {
                        matrix_op_rm(op, ra, mb)?
                    }))
                }
            },
            Value::Vector(va) => {
                if va.is_hp() {
                    match b {
                        Value::Real(rb) => return Ok(Value::Vector(hp_vector::ops::op_vr(op, va, rb)?)),
                        Value::Vector(vb) if vb.is_hp() => {
                            return Ok(Value::Vector(hp_vector::ops::op_vv(op, va, vb)?))
                        }
                        Value::Matrix(mb) if mb.is_hp() => {
                            let ma = Matrix::from_vector(va)?;
                            return Ok(Value::Matrix(hp_matrix::ops::op_mm(op, &ma, mb)?));
                        }
                        _ => {}
                    }
                }
                match b {
                    Value::Real(rb) => return Ok(Value::Vector(vector_op_vr(op, va, rb)?)),
                    Value::Vector(vb) => return Ok(Value::Vector(vector_op_vv(op, va, vb)?)),
                    Value::Matrix(mb) => {
                        let ma = Matrix::from_vector(va)?;
                        return Ok(Value::Matrix(matrix_op_mm(op, &ma, mb)?));
                    }
                    Value::Complex(_) => {}
                }
            }
            Value::Matrix(ma) => {
                if ma.is_hp() {
                    match b {
                        Value::Real(rb) => return Ok(Value::Matrix(hp_matrix::ops::op_mr(op, ma, rb)?)),
                        Value::Vector(vb) if vb.is_hp() => {
                            let cm = Matrix::from_vector(vb)?;
                            let c = hp_matrix::ops::op_mm(op, ma, &cm)?;
                            if op == BinOp::Mul {
                                return Ok(column_result(c));
                            }
                            return Ok(Value::Matrix(c));
                        }
                        Value::Matrix(mb) if mb.is_hp() => {
                            return Ok(Value::Matrix(hp_matrix::ops::op_mm(op, ma, mb)?))
                        }
                        _ => {}
                    }
                }
                match b {
                    Value::Real(rb) => return Ok(Value::Matrix(matrix_op_mr(op, ma, rb)?)),
                    Value::Vector(vb) => {
                        if op == BinOp::Mul {
                            let cm = crate::matrix::column_matrix::from_vector(vb)?;
                            return Ok(column_result(Matrix::mul(ma, &cm)?));
                        }
                        let mb = Matrix::from_vector(vb)?;
                        return Ok(Value::Matrix(matrix_op_mm(op, ma, &mb)?));
                    }
                    Value::Matrix(mb) => return Ok(Value::Matrix(matrix_op_mm(op, ma, mb)?)),
                    Value::Complex(_) => {}
                }
            }
            Value::Complex(_) => {}
        }
        if a.is_scalar() && b.is_scalar() {
            return match op {
                BinOp::Add => Value::scalar_add(a, b),
                BinOp::Sub => Value::scalar_sub(a, b),
                BinOp::Mul => Value::scalar_mul(a, b),
                BinOp::Div => Value::scalar_div(a, b),
                BinOp::Rem => Value::scalar_rem(a, b),
                BinOp::Lt => Value::scalar_lt(a, b),
                BinOp::Gt => Value::scalar_gt(a, b),
                BinOp::Le => Value::scalar_le(a, b),
                BinOp::Ge => Value::scalar_ge(a, b),
                BinOp::Eq => Value::scalar_equal(a, b),
                BinOp::Ne => Value::scalar_not_equal(a, b),
                BinOp::And => Ok(Value::scalar_and(a, b)),
                BinOp::Or => Ok(Value::scalar_or(a, b)),
                BinOp::Xor => Ok(Value::scalar_xor(a, b)),
            };
        }
        Err(op.invalid_operand(a, b))
    }

    pub fn add(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Add, a, b)
    }
    pub fn sub(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Sub, a, b)
    }
    pub fn mul(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Mul, a, b)
    }
    pub fn div(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Div, a, b)
    }
    pub fn rem(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Rem, a, b)
    }
    pub fn lt(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Lt, a, b)
    }
    pub fn gt(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Gt, a, b)
    }
    pub fn le(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Le, a, b)
    }
    pub fn ge(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Ge, a, b)
    }
    /// `IValue.Equal`
    pub fn equal(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Eq, a, b)
    }
    /// `IValue.NotEqual`
    pub fn not_equal(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Ne, a, b)
    }
    pub fn and(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::And, a, b)
    }
    pub fn or(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Or, a, b)
    }
    pub fn xor(a: &Value, b: &Value) -> CResult<Value> {
        Value::binary(BinOp::Xor, a, b)
    }

    // ----- IValue static helpers ------------------------------------------------------------

    /// `IValue.ExpandValues(IValue[] values)`
    pub fn expand_values(values: &[Value]) -> CResult<Vec<Value>> {
        let mut list = Vec::with_capacity(values.len());
        for v in values {
            match v {
                Value::Real(_) | Value::Complex(_) => list.push(v.clone()),
                Value::Vector(vec) => list.extend(vec.values().into_iter().map(Value::Real)),
                Value::Matrix(m) => list.extend(m.values().into_iter().map(Value::Real)),
            }
        }
        Ok(list)
    }

    /// `IValue.ExpandRealValues(IValue[] values)`
    pub fn expand_real_values(values: &[Value]) -> CResult<Vec<RealValue>> {
        let mut list = Vec::with_capacity(values.len());
        for v in values {
            match v {
                Value::Real(r) => list.push(r.clone()),
                Value::Vector(vec) => list.extend(vec.values()),
                Value::Matrix(m) => list.extend(m.values()),
                Value::Complex(_) => return Err(exceptions::InvalidArgument(&v.to_cs_string())),
            }
        }
        Ok(list)
    }

    /// `IValue.AsVector(ival, item)`
    pub fn as_vector(&self, item: Items) -> CResult<Vector> {
        match self {
            Value::Vector(v) => Ok(v.clone()),
            _ => Err(exceptions::MustBeVector(item)),
        }
    }

    /// `IValue.AsMatrix(ival, item)`: a vector becomes a column matrix.
    pub fn as_matrix(&self, item: Items) -> CResult<Matrix> {
        match self {
            Value::Matrix(m) => Ok(m.clone()),
            Value::Vector(v) => Matrix::from_vector(v),
            _ => Err(exceptions::MustBeMatrix(item)),
        }
    }

    /// `IValue.AsVectorHp(ival, item)`
    pub fn as_vector_hp(&self, item: Items) -> CResult<Vector> {
        match self {
            Value::Vector(v) if v.is_hp() => Ok(v.clone()),
            _ => Err(exceptions::MustBeHpVector(item)),
        }
    }

    /// `IValue.AsMatrixHp(ival, item)`
    pub fn as_matrix_hp(&self, item: Items) -> CResult<Matrix> {
        match self {
            Value::Matrix(m) if m.is_hp() => Ok(m.clone()),
            Value::Vector(v) if v.is_hp() => hp_matrix::hp_column_from_vector(v),
            _ => Err(exceptions::MustBeHpMatrix(item)),
        }
    }
}
