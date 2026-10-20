//! Port of Vector/Vector.cs (with the storage of the derived classes).
//!
//! C# has a class hierarchy `Vector` > `LargeVector` > `HpVector`, plus `ColumnVector` and
//! `RowVector` (views into a matrix). Vectors are reference objects in C#: assignment shares
//! the object and several functions (`fill`, `resize`, unit conversion) mutate it in place.
//! The port keeps that with a shared handle (`Rc`) over interior mutable storage; the
//! storage is a [`VecData`] enum with one variant per C# class. Virtual methods are
//! dispatched with a `match` on the variant: the base implementation lives here, the
//! overrides in the module of the derived class (`large_vector`, `column_vector`,
//! `row_vector`, `hp_vector`).
pub mod column_vector;
pub mod hp_vector;
pub mod large_vector;
pub mod row_vector;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::base_types::real_value::{RealValue, LOGICAL_ZERO};
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::base_types::variable::ChangeEvent;
use crate::calculator::{self, RealFn, RealOp};
use crate::exceptions::{self, Items};
use crate::extension_methods::almost_equals;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::Matrix;

pub use hp_vector::HpData;

/// `Vector.MaxLength`
pub const MAX_LENGTH: i64 = 10_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Relation {
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

/// The storage of one vector object; one variant per C# class.
#[derive(Debug)]
pub enum VecData {
    /// `Vector`: `_values`, with `_size == _values.Length`.
    Dense { values: Vec<RealValue> },
    /// `LargeVector`: `_values` may be null (None) until the first non-zero element.
    Large {
        values: Option<Vec<RealValue>>,
        size: usize,
        capacity: usize,
        length: usize,
    },
    /// `ColumnVector`: a view of column `col` of `matrix`.
    Column { matrix: Matrix, col: usize, size: usize },
    /// `RowVector`: a view of row `row` of `matrix`.
    Row { matrix: Matrix, row: usize, size: usize },
    /// `HpVector`: doubles with one unit for all elements.
    Hp(HpData),
}

pub struct VectorInner {
    pub(crate) data: RefCell<VecData>,
    pub(crate) on_change: ChangeEvent,
}

/// A reference to a vector object (the C# `Vector` reference).
#[derive(Clone)]
pub struct Vector(pub(crate) Rc<VectorInner>);

impl std::fmt::Debug for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[", self.type_name())?;
        let n = self.len().min(20);
        for i in 0..n {
            if i > 0 {
                write!(f, ", ")?;
            }
            let v = self.get(i);
            write!(f, "{}", v.d)?;
            if let Some(u) = &v.units {
                write!(f, "{}", u.text())?;
            }
        }
        write!(f, "]")
    }
}

/// The C# `(int)d` conversion on x64: truncation, with NaN and out of range values
/// giving `int.MinValue`.
pub fn cs_int(d: f64) -> i64 {
    if d.is_nan() || d >= 2147483648.0 || d <= -2147483649.0 {
        i32::MIN as i64
    } else {
        d.trunc() as i64
    }
}

pub(crate) fn null_reference() -> MathParserException {
    MathParserException::unexpected("Object reference not set to an instance of an object.")
}

pub(crate) fn index_was_outside() -> MathParserException {
    MathParserException::unexpected("Index was outside the bounds of the array.")
}

/// `RealValue.AlmostEquals(in RealValue)`: throws (NullReferenceException) when exactly
/// one of the values has units.
pub fn real_almost_equals(a: &RealValue, b: &RealValue) -> CResult<bool> {
    match (&a.units, &b.units) {
        (None, None) => Ok(almost_equals(a.d, b.d)),
        (Some(ua), Some(ub)) => {
            if Rc::ptr_eq(ua, ub) {
                return Ok(almost_equals(a.d, b.d));
            }
            if !ua.is_consistent(ub) {
                return Ok(false);
            }
            let d = ua.convert_to(ub);
            Ok(almost_equals(a.d, b.d * d))
        }
        _ => Err(null_reference()),
    }
}

/// `RealValue.CompareTo(RealValue)`
pub fn real_compare_to(a: &RealValue, b: &RealValue) -> CResult<i32> {
    let d = Unit::convert(&a.units, &b.units, ',')?;
    Ok(double_compare_to(a.d, b.d * d))
}

/// `double.CompareTo(double)`: NaN is smaller than everything else.
pub fn double_compare_to(a: f64, b: f64) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else if a == b {
        0
    } else if a.is_nan() {
        if b.is_nan() {
            0
        } else {
            -1
        }
    } else {
        1
    }
}

fn min_max(n1: usize, n2: usize) -> (usize, usize) {
    if n1 <= n2 {
        (n1, n2)
    } else {
        (n2, n1)
    }
}

/// `new RealValue(0d, u)`
fn zero_with(u: &OptUnit) -> RealValue {
    RealValue::new(0.0, u.clone())
}

impl Vector {
    // ----- construction ---------------------------------------------------------------

    pub(crate) fn from_data(data: VecData) -> Vector {
        Vector(Rc::new(VectorInner {
            data: RefCell::new(data),
            on_change: ChangeEvent::default(),
        }))
    }

    /// `new Vector(RealValue[] values)`
    pub fn from_values(values: Vec<RealValue>) -> CResult<Vector> {
        if values.len() as i64 > MAX_LENGTH {
            return Err(exceptions::VectorSizeLimit());
        }
        Ok(Vector::from_data(VecData::Dense { values }))
    }

    /// `new Vector(int size)`
    pub fn new_size(size: usize) -> CResult<Vector> {
        if size as i64 > MAX_LENGTH {
            return Err(exceptions::VectorSizeLimit());
        }
        Ok(Vector::from_data(VecData::Dense {
            values: vec![RealValue::ZERO; size],
        }))
    }

    /// `new Vector(n)` for a size known to be valid.
    pub(crate) fn dense(size: usize) -> Vector {
        Vector::from_data(VecData::Dense {
            values: vec![RealValue::ZERO; size],
        })
    }

    pub(crate) fn data(&self) -> Ref<'_, VecData> {
        self.0.data.borrow()
    }

    pub(crate) fn data_mut(&self) -> RefMut<'_, VecData> {
        self.0.data.borrow_mut()
    }

    pub fn ptr_eq(a: &Vector, b: &Vector) -> bool {
        Rc::ptr_eq(&a.0, &b.0)
    }

    /// `Change()`: raises `OnChange`.
    pub fn change(&self) {
        self.0.on_change.invoke();
    }

    pub fn on_change(&self) -> &ChangeEvent {
        &self.0.on_change
    }

    // ----- type tests ------------------------------------------------------------------

    /// `is LargeVector` (true for HpVector too)
    pub fn is_large(&self) -> bool {
        matches!(*self.data(), VecData::Large { .. } | VecData::Hp(_))
    }

    /// `is HpVector`
    pub fn is_hp(&self) -> bool {
        matches!(*self.data(), VecData::Hp(_))
    }

    /// The C# class name, used by `ToString()` in error messages.
    pub fn type_name(&self) -> &'static str {
        match *self.data() {
            VecData::Dense { .. } => "Calcpad.Core.Vector",
            VecData::Large { .. } => "Calcpad.Core.LargeVector",
            VecData::Column { .. } => "Calcpad.Core.ColumnVector",
            VecData::Row { .. } => "Calcpad.Core.RowVector",
            VecData::Hp(_) => "Calcpad.Core.HpVector",
        }
    }

    /// `ToString()`
    pub fn to_cs_string(&self) -> String {
        self.type_name().to_string()
    }

    // ----- members ---------------------------------------------------------------------

    /// `Length`
    pub fn len(&self) -> usize {
        match &*self.data() {
            VecData::Dense { values } => values.len(),
            VecData::Large { length, .. } => *length,
            VecData::Column { size, .. } | VecData::Row { size, .. } => *size,
            VecData::Hp(h) => h.length,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `Size` (number of stored elements; the rest are zero)
    pub fn size(&self) -> usize {
        match &*self.data() {
            VecData::Dense { values } => values.len(),
            VecData::Large { size, .. } => *size,
            VecData::Column { size, .. } | VecData::Row { size, .. } => *size,
            VecData::Hp(h) => h.size,
        }
    }

    /// Indexer get: `this[index]`
    pub fn get(&self, index: usize) -> RealValue {
        let view = {
            let d = self.data();
            match &*d {
                VecData::Dense { values } => return values[index].clone(),
                VecData::Large { values, size, .. } => {
                    if index >= *size {
                        return RealValue::ZERO;
                    }
                    return values.as_ref().unwrap()[index].clone();
                }
                VecData::Hp(h) => return h.get(index),
                VecData::Column { matrix, col, .. } => (matrix.clone(), index, *col),
                VecData::Row { matrix, row, .. } => (matrix.clone(), *row, index),
            }
        };
        view.0.get(view.1, view.2)
    }

    /// Indexer get that reports an out of range index like a .NET array does.
    pub fn try_get(&self, index: usize) -> CResult<RealValue> {
        let ok = match &*self.data() {
            VecData::Dense { values } => index < values.len(),
            VecData::Large { .. } | VecData::Hp(_) => true,
            VecData::Column { matrix, .. } => index < matrix.row_count(),
            VecData::Row { matrix, .. } => index < matrix.col_count(),
        };
        if !ok {
            return Err(index_was_outside());
        }
        Ok(self.get(index))
    }

    /// Indexer set: `this[index] = value`
    pub fn set(&self, index: usize, value: RealValue) -> CResult<()> {
        let view = {
            let mut d = self.data_mut();
            match &mut *d {
                VecData::Dense { values } => {
                    if index >= values.len() {
                        return Err(index_was_outside());
                    }
                    values[index] = value;
                    return Ok(());
                }
                VecData::Large { .. } => {
                    large_vector::set(&mut d, index, value);
                    return Ok(());
                }
                VecData::Hp(h) => return h.set(index, &value),
                VecData::Column { matrix, col, .. } => (matrix.clone(), index, *col),
                VecData::Row { matrix, row, .. } => (matrix.clone(), *row, index),
            }
        };
        view.0.set(view.1, view.2, value)
    }

    /// Direct write into `_values[index]` (no size or zero handling), as the C# operators
    /// do on the vectors they create.
    pub(crate) fn raw_set(&self, index: usize, value: RealValue) {
        match &mut *self.data_mut() {
            VecData::Dense { values } => values[index] = value,
            VecData::Large { values, .. } => values.as_mut().unwrap()[index] = value,
            _ => unreachable!("raw_set on a vector without values"),
        }
    }

    /// `ref ValueByRef(index)` used to update an element in place.
    pub(crate) fn update_by_ref(&self, index: usize, f: &mut dyn FnMut(&mut RealValue) -> CResult<()>) -> CResult<()> {
        let view = {
            let mut d = self.data_mut();
            match &mut *d {
                VecData::Dense { values } => return f(&mut values[index]),
                VecData::Large { .. } => {
                    large_vector::value_by_ref_extend(&mut d, index);
                    if let VecData::Large { values, .. } = &mut *d {
                        return f(&mut values.as_mut().unwrap()[index]);
                    }
                    unreachable!()
                }
                VecData::Hp(_) => return Err(MathParserException::unexpected("ValueByRef on HpVector")),
                VecData::Column { matrix, col, .. } => (matrix.clone(), index, *col),
                VecData::Row { matrix, row, .. } => (matrix.clone(), *row, index),
            }
        };
        let mut v = view.0.get(view.1, view.2);
        f(&mut v)?;
        view.0.set(view.1, view.2, v)
    }

    /// `Raw`: the base `_values` array (None for views and HP vectors, or when not yet
    /// allocated).
    pub(crate) fn has_raw(&self) -> bool {
        match &*self.data() {
            VecData::Dense { .. } => true,
            VecData::Large { values, .. } => values.is_some(),
            _ => false,
        }
    }

    pub(crate) fn raw(&self) -> Option<Vec<RealValue>> {
        match &*self.data() {
            VecData::Dense { values } => Some(values.clone()),
            VecData::Large { values, .. } => values.clone(),
            _ => None,
        }
    }

    /// `Values`
    pub fn values(&self) -> Vec<RealValue> {
        let view = {
            let d = self.data();
            match &*d {
                VecData::Dense { values } => return values.clone(),
                VecData::Large { values, length, .. } => {
                    let mut v = vec![RealValue::ZERO; *length];
                    if let Some(vals) = values {
                        v[..vals.len()].clone_from_slice(vals);
                    }
                    return v;
                }
                VecData::Hp(h) => return h.values(),
                VecData::Column { .. } | VecData::Row { .. } => true,
            }
        };
        let _ = view;
        let n = self.len();
        (0..n).map(|i| self.get(i)).collect()
    }

    // ----- equality ----------------------------------------------------------------------

    /// `Equals(Vector)`
    pub fn equals(&self, other: &Vector) -> bool {
        if self.is_hp() {
            return self.hp_equals(other);
        }
        if self.len() != other.len() || self.size() != other.size() {
            return false;
        }
        for i in (0..self.size()).rev() {
            if !self.get(i).equals(&other.get(i)) {
                return false;
            }
        }
        true
    }

    // ----- Clone / Copy -------------------------------------------------------------------

    /// `Clone(Vector a)`
    pub(crate) fn clone_like(a: &Vector) -> Vector {
        if a.is_large() {
            large_vector::new_ls(a.len(), a.size())
        } else {
            Vector::dense(a.len())
        }
    }

    /// `Clone(Vector a, Vector b)`
    pub(crate) fn clone_like2(a: &Vector, b: &Vector) -> Vector {
        let size = a.size().max(b.size());
        let len_a = a.len();
        let len_b = b.len();
        if len_a > len_b {
            return if a.is_large() {
                large_vector::new_ls(len_a, size)
            } else {
                Vector::dense(len_a)
            };
        }
        if b.is_large() {
            large_vector::new_ls(len_b, size)
        } else {
            Vector::dense(len_b)
        }
    }

    /// `Copy()` (virtual)
    pub fn copy(&self) -> Vector {
        match &*self.data() {
            VecData::Large { .. } => return large_vector::copy(self),
            VecData::Hp(_) => return self.hp_copy(),
            _ => {}
        }
        let v = Vector::clone_like(self);
        for i in (0..self.size()).rev() {
            v.raw_set(i, self.get(i));
        }
        v
    }

    // ----- operators ---------------------------------------------------------------------

    /// `-a`
    pub fn neg(a: &Vector) -> Vector {
        let b = Vector::clone_like(a);
        for i in (0..a.size()).rev() {
            b.raw_set(i, a.get(i).neg());
        }
        b
    }

    fn zip(
        a: &Vector,
        b: &Vector,
        f: &dyn Fn(&RealValue, &RealValue) -> CResult<RealValue>,
        n1: usize,
    ) -> CResult<Vector> {
        let c = Vector::clone_like2(a, b);
        for i in (0..n1).rev() {
            c.raw_set(i, f(&a.get(i), &b.get(i))?);
        }
        Ok(c)
    }

    pub fn sub(a: &Vector, b: &Vector) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &RealValue::sub, n1)?;
        if na > nb {
            for i in (n1..n2).rev() {
                c.raw_set(i, a.get(i));
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                c.raw_set(i, b.get(i).neg());
            }
        }
        Ok(c)
    }

    pub fn sub_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::sub(&a.get(i), b)?);
        }
        if nc > na && !b.equals(&RealValue::ZERO) {
            c.fill_range(&b.neg(), na, nc - na)?;
        }
        Ok(c)
    }

    pub fn sub_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
        let c = Vector::clone_like(b);
        let nb = b.size();
        let nc = c.len();
        for i in (0..nb).rev() {
            c.raw_set(i, RealValue::sub(a, &b.get(i))?);
        }
        if nc > nb && !a.equals(&RealValue::ZERO) {
            c.fill_range(a, nb, nc - nb)?;
        }
        Ok(c)
    }

    pub fn add(a: &Vector, b: &Vector) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &RealValue::add, n1)?;
        if na > nb {
            for i in (n1..n2).rev() {
                c.raw_set(i, a.get(i));
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                c.raw_set(i, b.get(i));
            }
        }
        Ok(c)
    }

    pub fn add_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::add(&a.get(i), b)?);
        }
        if nc > na && !b.equals(&RealValue::ZERO) {
            c.fill_range(b, na, nc - na)?;
        }
        Ok(c)
    }

    pub fn mul(a: &Vector, b: &Vector) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &|x, y| Ok(RealValue::mul_op(x, y)), n1)?;
        if na > nb {
            for i in (n1..n2).rev() {
                c.raw_set(i, zero_with(&a.get(i).units));
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                c.raw_set(i, zero_with(&b.get(i).units));
            }
        }
        Ok(c)
    }

    pub fn mul_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::mul_op(&a.get(i), b));
        }
        if nc > na && b.units.is_some() {
            c.fill_range(&zero_with(&b.units), na, nc - na)?;
        }
        Ok(c)
    }

    pub fn div(a: &Vector, b: &Vector) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &|x, y| Ok(RealValue::div_op(x, y)), n1)?;
        let nc = c.len();
        if na > nb {
            for i in (n1..n2).rev() {
                c.raw_set(i, RealValue::div_op(&a.get(i), &RealValue::ZERO));
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                c.raw_set(i, RealValue::div_op(&RealValue::ZERO, &b.get(i)));
            }
        }
        if nc > n2 {
            c.fill_range(&RealValue::NAN, n2, nc - n2)?;
        }
        Ok(c)
    }

    pub fn div_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::div_op(&a.get(i), b));
        }
        if nc > na {
            let v = RealValue::div_op(&RealValue::ZERO, b);
            if !v.equals(&RealValue::ZERO) {
                c.fill_range(&v, na, nc - na)?;
            }
        }
        Ok(c)
    }

    pub fn div_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
        let c = Vector::clone_like(b);
        let nb = b.size();
        let nc = c.len();
        for i in (0..nb).rev() {
            c.raw_set(i, RealValue::div_op(a, &b.get(i)));
        }
        if nc > nb {
            c.fill_range(&RealValue::div_op(a, &RealValue::ZERO), nb, nc - nb)?;
        }
        Ok(c)
    }

    pub fn rem(a: &Vector, b: &Vector) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &RealValue::rem, n1)?;
        let nc = c.len();
        if na > nb {
            c.fill_range(&RealValue::NAN, n1, n2 - n1)?;
        } else if nb > na {
            for i in (n1..n2).rev() {
                c.raw_set(i, RealValue::rem(&RealValue::ZERO, &b.get(i))?);
            }
        }
        if nc > n2 {
            c.fill_range(&RealValue::NAN, n2, nc - n2)?;
        }
        Ok(c)
    }

    pub fn rem_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::rem(&a.get(i), b)?);
        }
        if nc > na && !b.equals(&RealValue::ZERO) {
            c.fill_range(&RealValue::NAN, na, nc - na)?;
        }
        Ok(c)
    }

    pub fn rem_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
        let c = Vector::clone_like(b);
        let nb = b.size();
        let nc = c.len();
        for i in (0..nb).rev() {
            c.raw_set(i, RealValue::rem(a, &b.get(i))?);
        }
        if nc > nb {
            c.fill_range(&RealValue::NAN, nb, nc - nb)?;
        }
        Ok(c)
    }

    /// The relational operators of two vectors: `fill_one` is true for `==`, `<=`, `>=`.
    fn compare(
        a: &Vector,
        b: &Vector,
        f: &dyn Fn(&RealValue, &RealValue) -> CResult<RealValue>,
        fill_one: bool,
    ) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, f, n1)?;
        let nc = c.len();
        let mut zero = RealValue::ZERO;
        if na > nb {
            for i in (n1..n2).rev() {
                let a_i = a.get(i);
                if !Unit::ptr_eq(&a_i.units, &zero.units) {
                    zero = zero_with(&a_i.units);
                }
                c.raw_set(i, f(&a_i, &zero)?);
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                let b_i = b.get(i);
                if !Unit::ptr_eq(&b_i.units, &zero.units) {
                    zero = zero_with(&b_i.units);
                }
                c.raw_set(i, f(&zero, &b_i)?);
            }
        }
        if fill_one && nc > n2 {
            c.fill_range(&RealValue::ONE, n2, nc - n2)?;
        }
        Ok(c)
    }

    /// Vector op RealValue for the relational operators: the tail is filled with
    /// `f(zero, b)` where zero carries the units of `b`.
    fn compare_vr(
        a: &Vector,
        b: &RealValue,
        f: &dyn Fn(&RealValue, &RealValue) -> CResult<RealValue>,
    ) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, f(&a.get(i), b)?);
        }
        if nc > na {
            let zero = if b.units.is_none() {
                RealValue::ZERO
            } else {
                zero_with(&b.units)
            };
            let v = f(&zero, b)?;
            if !v.equals(&RealValue::ZERO) {
                c.fill_range(&v, na, nc - na)?;
            }
        }
        Ok(c)
    }

    /// RealValue op Vector for the relational operators: the tail is filled with
    /// `f(a, zero)`.
    fn compare_rv(
        a: &RealValue,
        b: &Vector,
        f: &dyn Fn(&RealValue, &RealValue) -> CResult<RealValue>,
    ) -> CResult<Vector> {
        let c = Vector::clone_like(b);
        let nb = b.size();
        let nc = c.len();
        for i in (0..nb).rev() {
            c.raw_set(i, f(a, &b.get(i))?);
        }
        if nc > nb {
            let zero = if a.units.is_none() {
                RealValue::ZERO
            } else {
                zero_with(&a.units)
            };
            let v = f(a, &zero)?;
            if !v.equals(&RealValue::ZERO) {
                c.fill_range(&v, nb, nc - nb)?;
            }
        }
        Ok(c)
    }

    pub fn eq(a: &Vector, b: &Vector) -> CResult<Vector> {
        Vector::compare(a, b, &RealValue::eq_op, true)
    }

    /// `Vector == RealValue`; note the tail uses `zero == b`.
    pub fn eq_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        Vector::compare_vr(a, b, &RealValue::eq_op)
    }

    pub fn ne(a: &Vector, b: &Vector) -> CResult<Vector> {
        Vector::compare(a, b, &RealValue::ne_op, false)
    }

    /// `Vector != RealValue`; the tail uses `b != zero`.
    pub fn ne_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::ne_op(&a.get(i), b)?);
        }
        if nc > na {
            let zero = if b.units.is_none() {
                RealValue::ZERO
            } else {
                zero_with(&b.units)
            };
            let v = RealValue::ne_op(b, &zero)?;
            if !v.equals(&RealValue::ZERO) {
                c.fill_range(&v, na, nc - na)?;
            }
        }
        Ok(c)
    }

    pub fn lt(a: &Vector, b: &Vector) -> CResult<Vector> {
        Vector::compare(a, b, &RealValue::lt_op, false)
    }
    pub fn lt_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        Vector::compare_vr(a, b, &RealValue::lt_op)
    }
    pub fn lt_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
        Vector::compare_rv(a, b, &RealValue::lt_op)
    }
    pub fn gt(a: &Vector, b: &Vector) -> CResult<Vector> {
        Vector::compare(a, b, &RealValue::gt_op, false)
    }
    pub fn gt_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        Vector::compare_vr(a, b, &RealValue::gt_op)
    }
    pub fn gt_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
        Vector::compare_rv(a, b, &RealValue::gt_op)
    }
    pub fn le(a: &Vector, b: &Vector) -> CResult<Vector> {
        Vector::compare(a, b, &RealValue::le_op, true)
    }
    pub fn le_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        Vector::compare_vr(a, b, &RealValue::le_op)
    }
    pub fn le_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
        Vector::compare_rv(a, b, &RealValue::le_op)
    }
    pub fn ge(a: &Vector, b: &Vector) -> CResult<Vector> {
        Vector::compare(a, b, &RealValue::ge_op, true)
    }
    pub fn ge_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        Vector::compare_vr(a, b, &RealValue::ge_op)
    }
    pub fn ge_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
        Vector::compare_rv(a, b, &RealValue::ge_op)
    }

    pub fn and(a: &Vector, b: &Vector) -> CResult<Vector> {
        let n = a.size().min(b.size());
        Vector::zip(a, b, &|x, y| Ok(RealValue::and_op(x, y)), n)
    }

    pub fn and_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        for i in (0..a.size()).rev() {
            c.raw_set(i, RealValue::and_op(&a.get(i), b));
        }
        Ok(c)
    }

    pub fn or(a: &Vector, b: &Vector) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &|x, y| Ok(RealValue::or_op(x, y)), n1)?;
        if na > nb {
            for i in (n1..n2).rev() {
                c.raw_set(i, RealValue::or_op(&a.get(i), &RealValue::ZERO));
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                c.raw_set(i, RealValue::or_op(&RealValue::ZERO, &b.get(i)));
            }
        }
        Ok(c)
    }

    pub fn or_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::or_op(&a.get(i), b));
        }
        let v = RealValue::or_op(&RealValue::ZERO, b);
        if nc > na && !v.equals(&RealValue::ZERO) {
            c.fill_range(&v, na, nc - na)?;
        }
        Ok(c)
    }

    pub fn xor(a: &Vector, b: &Vector) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &|x, y| Ok(RealValue::xor_op(x, y)), n1)?;
        if na > nb {
            for i in (n1..n2).rev() {
                c.raw_set(i, RealValue::xor_op(&a.get(i), &RealValue::ZERO));
            }
        } else if nb > na {
            // C# reads vb[i] in both branches, which throws when b has no raw values.
            if !b.has_raw() {
                return Err(null_reference());
            }
            for i in (n1..n2).rev() {
                c.raw_set(i, RealValue::xor_op(&RealValue::ZERO, &b.get(i)));
            }
        }
        Ok(c)
    }

    pub fn xor_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, RealValue::xor_op(&a.get(i), b));
        }
        let v = RealValue::xor_op(&RealValue::ZERO, b);
        if nc > na && !v.equals(&RealValue::ZERO) {
            c.fill_range(&v, na, nc - na)?;
        }
        Ok(c)
    }

    /// `EvaluateOperator(op, Vector a, Vector b, index)`
    pub fn evaluate_operator(op: RealOp, a: &Vector, b: &Vector, index: i64) -> CResult<Vector> {
        match calculator::get_operator_symbol(index) {
            '/' | '\u{00F7}' => Vector::div(a, b),
            '\u{29BC}' => Vector::rem(a, b),
            '*' => Vector::mul(a, b),
            '-' => Vector::sub(a, b),
            '+' => Vector::add(a, b),
            '<' => Vector::lt(a, b),
            '>' => Vector::gt(a, b),
            '\u{2264}' => Vector::le(a, b),
            '\u{2265}' => Vector::ge(a, b),
            '\u{2261}' => Vector::eq(a, b),
            '\u{2260}' => Vector::ne(a, b),
            '\u{2227}' => Vector::and(a, b),
            '\u{2228}' => Vector::or(a, b),
            '\u{2295}' => Vector::xor(a, b),
            _ => Vector::evaluate_operator2(op, a, b, index),
        }
    }

    /// `EvaluateOperator(op, Vector a, RealValue b, index)`
    pub fn evaluate_operator_vr(op: RealOp, a: &Vector, b: &RealValue, index: i64) -> CResult<Vector> {
        match calculator::get_operator_symbol(index) {
            '/' | '\u{00F7}' => Vector::div_vr(a, b),
            '\u{29BC}' => Vector::rem_vr(a, b),
            '*' => Vector::mul_vr(a, b),
            '-' => Vector::sub_vr(a, b),
            '+' => Vector::add_vr(a, b),
            '<' => Vector::lt_vr(a, b),
            '>' => Vector::gt_vr(a, b),
            '\u{2264}' => Vector::le_vr(a, b),
            '\u{2265}' => Vector::ge_vr(a, b),
            '\u{2261}' => Vector::eq_vr(a, b),
            '\u{2260}' => Vector::ne_vr(a, b),
            '\u{2227}' => Vector::and_vr(a, b),
            '\u{2228}' => Vector::or_vr(a, b),
            '\u{2295}' => Vector::xor_vr(a, b),
            _ => Vector::evaluate_operator2_vr(op, a, b, index),
        }
    }

    /// `EvaluateOperator(op, RealValue a, Vector b, index)`
    pub fn evaluate_operator_rv(op: RealOp, a: &RealValue, b: &Vector, index: i64) -> CResult<Vector> {
        match calculator::get_operator_symbol(index) {
            '/' | '\u{00F7}' => Vector::div_rv(a, b),
            '\u{29BC}' => Vector::rem_rv(a, b),
            '*' => Vector::mul_vr(b, a),
            '-' => Vector::sub_rv(a, b),
            '+' => Vector::add_vr(b, a),
            '<' => Vector::lt_rv(a, b),
            '>' => Vector::gt_rv(a, b),
            '\u{2264}' => Vector::le_rv(a, b),
            '\u{2265}' => Vector::ge_rv(a, b),
            '\u{2261}' => Vector::eq_vr(b, a),
            '\u{2260}' => Vector::ne_vr(b, a),
            '\u{2227}' => Vector::and_vr(b, a),
            '\u{2228}' => Vector::or_vr(b, a),
            '\u{2295}' => Vector::xor_vr(b, a),
            _ => Vector::evaluate_operator2_rv(op, a, b, index),
        }
    }

    fn evaluate_operator2(op: RealOp, a: &Vector, b: &Vector, index: i64) -> CResult<Vector> {
        let (na, nb) = (a.size(), b.size());
        let (n1, n2) = min_max(na, nb);
        let c = Vector::zip(a, b, &|x, y| op.apply(x, y), n1)?;
        let nc = c.len();
        let mut zero = RealValue::ZERO;
        let require_consistent_units = calculator::operator_require_consistent_units(index);
        if na > nb {
            for i in (n1..n2).rev() {
                let a_i = a.get(i);
                if require_consistent_units && !Unit::ptr_eq(&a_i.units, &zero.units) {
                    zero = zero_with(&a_i.units);
                }
                c.raw_set(i, op.apply(&a_i, &zero)?);
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                let b_i = b.get(i);
                if require_consistent_units && !Unit::ptr_eq(&b_i.units, &zero.units) {
                    zero = zero_with(&b_i.units);
                }
                c.raw_set(i, op.apply(&zero, &b_i)?);
            }
        }
        if nc > n2 {
            let v = op.apply(&zero, &zero)?;
            if !v.equals(&zero) {
                c.fill_range(&v, n2, nc - n2)?;
            }
        }
        Ok(c)
    }

    fn evaluate_operator2_vr(op: RealOp, a: &Vector, b: &RealValue, index: i64) -> CResult<Vector> {
        let c = Vector::clone_like(a);
        let na = a.size();
        let nc = c.len();
        for i in (0..na).rev() {
            c.raw_set(i, op.apply(&a.get(i), b)?);
        }
        if nc > na {
            let zero = if calculator::operator_require_consistent_units(index) && b.units.is_some() {
                zero_with(&b.units)
            } else {
                RealValue::ZERO
            };
            let v = op.apply(&zero, b)?;
            if !v.equals(&RealValue::ZERO) {
                c.fill_range(&v, na, nc - na)?;
            }
        }
        Ok(c)
    }

    fn evaluate_operator2_rv(op: RealOp, a: &RealValue, b: &Vector, index: i64) -> CResult<Vector> {
        let c = Vector::clone_like(b);
        let nb = b.size();
        let nc = c.len();
        for i in (0..nb).rev() {
            c.raw_set(i, op.apply(a, &b.get(i))?);
        }
        if nc > nb {
            let zero = if calculator::operator_require_consistent_units(index) && a.units.is_some() {
                zero_with(&a.units)
            } else {
                RealValue::ZERO
            };
            let v = op.apply(a, &zero)?;
            if !v.equals(&RealValue::ZERO) {
                c.fill_range(&v, nb, nc - nb)?;
            }
        }
        Ok(c)
    }

    /// `EvaluateFunction(f, Vector a)`
    pub fn evaluate_function(f: RealFn, a: &Vector) -> CResult<Vector> {
        let b = Vector::clone_like(a);
        let na = a.size();
        let nb = b.len();
        for i in (0..na).rev() {
            b.raw_set(i, f.apply(&a.get(i))?);
        }
        if nb > na {
            let v = f.apply(&RealValue::ZERO)?;
            if !v.equals(&RealValue::ZERO) {
                b.fill_range(&v, na, nb - na)?;
            }
        }
        Ok(b)
    }

    /// `SetUnits(Unit)` (virtual)
    pub fn set_units(&self, units: &OptUnit) -> CResult<()> {
        if let VecData::Hp(h) = &mut *self.data_mut() {
            h.units = units.clone();
            return Ok(());
        }
        for i in (0..self.size()).rev() {
            let d = self.get(i).d;
            self.set(i, RealValue::new(d, units.clone()))?;
        }
        Ok(())
    }

    /// `CrossProduct(Vector a, Vector b)`
    pub fn cross_product(a: &Vector, b: &Vector) -> CResult<Vector> {
        let na = a.len();
        let nb = b.len();
        if !(2..=3).contains(&na) || !(2..=3).contains(&nb) {
            return Err(exceptions::CrossProductVectorDimensions());
        }
        let c = Vector::dense(3);
        let m = RealValue::mul_op;
        c.set(2, RealValue::sub(&m(&a.get(0), &b.get(1)), &m(&a.get(1), &b.get(0)))?)?;
        if na == 3 {
            c.set(0, m(&a.get(2).neg(), &b.get(1)))?;
            c.set(1, m(&a.get(2), &b.get(0)))?;
        }
        if nb == 3 {
            c.set(0, RealValue::add(&c.get(0), &m(&a.get(1), &b.get(2)))?)?;
            c.set(1, RealValue::sub(&c.get(1), &m(&a.get(0), &b.get(2)))?)?;
        }
        Ok(c)
    }

    /// `DotProduct(Vector a, Vector b)`
    pub fn dot_product(a: &Vector, b: &Vector) -> CResult<RealValue> {
        let n = a.size().min(b.size());
        let mut sum = RealValue::mul_op(&a.try_get(0)?, &b.try_get(0)?);
        for i in 1..n {
            sum = RealValue::add(&sum, &RealValue::mul_op(&a.get(i), &b.get(i)))?;
        }
        Ok(sum)
    }

    // ----- sorting and searching ---------------------------------------------------------

    /// `Sort(bool reverse)` (virtual)
    pub fn sort(&self, reverse: bool) -> CResult<Vector> {
        if self.is_hp() {
            return self.hp_sort(reverse);
        }
        if !self.has_raw() && !matches!(*self.data(), VecData::Large { .. }) {
            return Err(null_reference());
        }
        let n = self.len();
        let size = self.size();
        let mut values = vec![RealValue::ZERO; n];
        for (i, v) in values.iter_mut().enumerate().take(size) {
            *v = self.get(i);
        }
        if reverse {
            crate::sort::sort_by::<RealValue, ()>(&mut values, None, &mut |x, y| Ok(-real_compare_to(x, y)?))?;
        } else {
            crate::sort::sort_default::<RealValue, ()>(&mut values, None, &mut real_compare_to)?;
        }
        Ok(Vector::from_data(VecData::Dense { values }))
    }

    /// `Reverse()` (virtual)
    pub fn reverse(&self) -> CResult<Vector> {
        match &*self.data() {
            VecData::Large { .. } => return large_vector::reverse(self),
            VecData::Hp(_) => return self.hp_reverse(),
            _ => {}
        }
        let n = self.len();
        let vector = Vector::dense(n);
        for i in 0..n {
            vector.set(i, self.get(n - 1 - i))?;
        }
        Ok(vector)
    }

    /// `Order(bool reverse)` (virtual): returns an HpVector of 1 based indexes.
    pub fn order(&self, reverse: bool) -> CResult<Vector> {
        if self.is_hp() {
            return self.hp_order(reverse);
        }
        let idx = self.get_order_indexes(reverse, None)?;
        Ok(hp_vector::from_indexes(&idx))
    }

    /// `GetOrderIndexes(bool reverse, int len = -1)` (virtual)
    pub fn get_order_indexes(&self, reverse: bool, len: Option<usize>) -> CResult<Vec<i32>> {
        if self.is_hp() {
            return self.hp_get_order_indexes(reverse, len);
        }
        let len = len.unwrap_or_else(|| self.len());
        let mut values = vec![RealValue::ZERO; len];
        for (i, v) in values.iter_mut().enumerate().take(self.size()) {
            *v = self.get(i);
        }
        let mut indexes: Vec<i32> = (0..len as i32).collect();
        if reverse {
            crate::sort::sort_by(&mut values, Some(&mut indexes), &mut |x, y| Ok(-real_compare_to(x, y)?))?;
        } else {
            crate::sort::sort_default(&mut values, Some(&mut indexes), &mut real_compare_to)?;
        }
        Ok(indexes)
    }

    /// `First(int length)` (virtual)
    pub fn first(&self, length: usize) -> CResult<Vector> {
        if self.is_hp() {
            return self.hp_first(length);
        }
        let mut length = length;
        if length < 1 {
            length = 1;
        } else if length >= self.len() {
            return Ok(self.copy());
        }
        let vector = Vector::dense(length);
        let size = self.size();
        if length > size {
            length = size;
        }
        if length > 0 && !self.has_raw() {
            return Err(null_reference());
        }
        for i in 0..length {
            vector.raw_set(i, self.get(i));
        }
        Ok(vector)
    }

    /// `Last(int length)` (virtual)
    pub fn last(&self, length: usize) -> CResult<Vector> {
        if self.is_hp() {
            return self.hp_last(length);
        }
        let mut length = length;
        if length < 1 {
            length = 1;
        } else if length >= self.len() {
            return Ok(self.copy());
        }
        let vector = Vector::dense(length);
        let start = self.len() - length;
        let size = self.size();
        if start < size {
            if !self.has_raw() {
                return Err(null_reference());
            }
            for i in start..size {
                vector.raw_set(i - start, self.get(i));
            }
        }
        Ok(vector)
    }

    /// `CheckBounds(ref start, ref end)`: converts 1 based inclusive bounds to a 0 based
    /// range; returns true when the range is the whole vector.
    pub(crate) fn check_bounds(&self, start: &mut i64, end: &mut i64) -> bool {
        let len = self.len() as i64;
        if *start > *end {
            std::mem::swap(start, end);
        }
        if *start > len {
            *start = len;
        }
        if *end > len {
            *end = len;
        }
        *start -= 1;
        *start == 0 && *end == len
    }

    /// `Slice(int start, int end)` (virtual)
    pub fn slice(&self, start: i64, end: i64) -> CResult<Vector> {
        match &*self.data() {
            VecData::Large { .. } => return large_vector::slice(self, start, end),
            VecData::Hp(_) => return self.hp_slice(start, end),
            _ => {}
        }
        let (mut start, mut end) = (start, end);
        if self.check_bounds(&mut start, &mut end) {
            return Ok(self.copy());
        }
        let vector = Vector::new_size((end - start).max(0) as usize).map_err(|_| index_was_outside())?;
        let size = self.size() as i64;
        if end > size {
            end = size;
        }
        if start < end {
            if !self.has_raw() {
                return Err(null_reference());
            }
            for i in start..end {
                vector.raw_set((i - start) as usize, self.get(i as usize));
            }
        }
        Ok(vector)
    }

    /// `Fill(RealValue value)` (virtual): fills in place and returns this vector.
    pub fn fill(&self, value: &RealValue) -> CResult<Vector> {
        let kind = match &*self.data() {
            VecData::Dense { .. } => 0,
            VecData::Large { .. } => 1,
            VecData::Hp(_) => 2,
            VecData::Column { .. } => 3,
            VecData::Row { .. } => 4,
        };
        match kind {
            0 => {
                if let VecData::Dense { values } = &mut *self.data_mut() {
                    for v in values.iter_mut() {
                        *v = value.clone();
                    }
                }
                Ok(self.clone())
            }
            1 => large_vector::fill(self, value),
            2 => self.hp_fill(value),
            3 => column_vector::fill(self, value),
            _ => row_vector::fill(self, value),
        }
    }

    /// `Fill(RealValue value, int start, int len)` (protected virtual)
    pub(crate) fn fill_range(&self, value: &RealValue, start: usize, len: usize) -> CResult<()> {
        match &mut *self.data_mut() {
            VecData::Dense { values } => {
                if start + len > values.len() {
                    return Err(MathParserException::unexpected(
                        "Specified argument was out of the range of valid values.",
                    ));
                }
                for v in &mut values[start..start + len] {
                    *v = value.clone();
                }
                Ok(())
            }
            d @ VecData::Large { .. } => {
                large_vector::fill_range(d, value, start, len);
                Ok(())
            }
            VecData::Hp(h) => {
                let d = Unit::convert(&h.units, &value.units, '.')?;
                h.fill_range(value.d * d, start, len);
                Ok(())
            }
            _ => Err(null_reference()),
        }
    }

    /// `Join(IValue[] items)`
    pub fn join(items: &[Value]) -> CResult<Vector> {
        let mut len: i64 = 0;
        let mut is_hp = true;
        for item in items {
            match item {
                Value::Vector(v) => len += v.len() as i64,
                Value::Real(_) => len += 1,
                Value::Matrix(m) => len += (m.col_count() * m.row_count()) as i64,
                _ => {}
            }
            let hp_like = match item {
                Value::Matrix(m) => m.is_hp(),
                Value::Vector(v) => v.is_hp(),
                Value::Real(_) => true,
                _ => false,
            };
            if is_hp && !hp_like {
                is_hp = false;
            }
        }
        if len > MAX_LENGTH {
            return Err(exceptions::VectorSizeLimit());
        }
        if is_hp {
            if let Some(result) = hp_vector::try_join(items, len as usize)? {
                return Ok(result);
            }
        }
        let mut values = vec![RealValue::ZERO; len as usize];
        let mut index = 0usize;
        for item in items {
            match item {
                Value::Real(real) => {
                    values[index] = real.clone();
                    index += 1;
                }
                Value::Vector(vector) => {
                    let vals = vector.values();
                    let size = vector.size();
                    values[index..index + size].clone_from_slice(&vals[..size]);
                    index += vector.len();
                }
                Value::Matrix(matrix) => {
                    let n = matrix.col_count();
                    for i in 0..matrix.row_count() {
                        for j in 0..n {
                            values[index] = matrix.get(i, j);
                            index += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        Vector::from_values(values)
    }

    /// `Range(RealValue start, RealValue end, RealValue step)`
    pub fn range(start: &RealValue, end: &RealValue, step: &RealValue) -> CResult<Vector> {
        if almost_equals(step.d, 0.0) {
            return Err(exceptions::StepCannotBeZero());
        }
        let mut len = RealValue::div_op(&RealValue::sub(end, start)?, step).d + 1.0;
        if len <= 1.0 {
            return Vector::from_values(vec![start.clone(), end.clone()]);
        }
        if len > MAX_LENGTH as f64 {
            return Err(exceptions::VectorSizeLimit());
        }
        len = len.trunc();
        let n = len as usize;
        let mut values = Vec::with_capacity(n);
        for i in 0..n {
            values.push(RealValue::add(
                start,
                &RealValue::mul_op(step, &RealValue::num(i as f64)),
            )?);
        }
        Vector::from_values(values)
    }

    /// `Search(RealValue value, int start)` (virtual)
    pub fn search(&self, value: &RealValue, start: usize) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_search(value, start);
        }
        let n = self.len();
        let size = self.size();
        let mut start = start;
        if start < 1 {
            start = 1;
        } else if start > size {
            if start > n {
                return Ok(RealValue::ZERO);
            }
            if real_almost_equals(value, &RealValue::ZERO)? {
                return Ok(RealValue::num(start as f64));
            }
            return Ok(RealValue::ZERO);
        }
        for i in start - 1..size {
            if real_almost_equals(value, &self.get(i))? {
                return Ok(RealValue::num((i + 1) as f64));
            }
        }
        if size < n && real_almost_equals(value, &RealValue::ZERO)? {
            return Ok(RealValue::num((size + 1) as f64));
        }
        Ok(RealValue::ZERO)
    }

    /// `FindAll(RealValue value, int start, Relation rel)` (virtual)
    pub fn find_all(&self, value: &RealValue, start: usize, rel: Relation) -> CResult<Vector> {
        if self.is_hp() {
            return self.hp_find_all(value, start, rel);
        }
        let idx = self.find_all_indexes(value, start, rel)?;
        Ok(hp_vector::from_indexes(&idx))
    }

    /// `Lookup(Vector dest, RealValue value, Relation rel)`
    pub fn lookup(&self, dest: &Vector, value: &RealValue, rel: Relation) -> CResult<Vector> {
        let indexes = self.find_all_indexes(value, 1, rel)?;
        let vector = Vector::dense(indexes.len());
        for (j, &i) in indexes.iter().enumerate() {
            let i = i as usize;
            if i > dest.len() {
                return Err(exceptions::IndexOutOfRange(&(i + 1).to_string()));
            }
            vector.set(j, dest.try_get(i)?)?;
        }
        Ok(vector)
    }

    /// `Relate(RealValue a, RealValue b, Relation rel)`
    pub fn relate(a: &RealValue, b: &RealValue, rel: Relation) -> CResult<bool> {
        Ok(match rel {
            Relation::Equal => real_almost_equals(a, b)?,
            Relation::NotEqual => !real_almost_equals(a, b)?,
            Relation::LessThan => real_compare_to(a, b)? < 0 && !real_almost_equals(a, b)?,
            Relation::LessOrEqual => real_compare_to(a, b)? <= 0 || real_almost_equals(a, b)?,
            Relation::GreaterThan => real_compare_to(a, b)? > 0 && !real_almost_equals(a, b)?,
            Relation::GreaterOrEqual => real_compare_to(a, b)? >= 0 || real_almost_equals(a, b)?,
        })
    }

    fn find_all_indexes(&self, value: &RealValue, start: usize, rel: Relation) -> CResult<Vec<i32>> {
        let n = self.len();
        let size = self.size();
        let mut start = start;
        if start < 1 {
            start = 1;
        } else if start > size {
            if start > n {
                return Ok(vec![]);
            }
            if Vector::relate(value, &RealValue::ZERO, rel)? {
                return Ok((size as i32..n as i32).collect());
            }
            return Ok(vec![]);
        }
        let mut indexes = Vec::new();
        for i in start - 1..size {
            if Vector::relate(&self.get(i), value, rel)? {
                indexes.push(i as i32);
            }
        }
        if size < n && Vector::relate(&RealValue::ZERO, value, rel)? {
            indexes.extend(size as i32..n as i32);
        }
        Ok(indexes)
    }

    /// `Extract(Vector indexes)` (virtual)
    pub fn extract(&self, indexes: &Vector) -> CResult<Vector> {
        if self.is_hp() {
            return self.hp_extract(indexes);
        }
        let n = self.len();
        let ni = indexes.len();
        let vector = Vector::dense(ni);
        for i in 0..ni {
            let d = indexes.get(i).d;
            if d < 1.0 || d > i32::MAX as f64 {
                return Err(exceptions::MustBePositiveInteger(Items::Index));
            }
            if d.is_nan() {
                return Err(index_was_outside());
            }
            let j = d as usize;
            if j > n {
                return Err(exceptions::IndexOutOfRange(&j.to_string()));
            }
            vector.set(i, self.get(j - 1))?;
        }
        Ok(vector)
    }

    /// `Count(RealValue value, int start)` (virtual)
    pub fn count(&self, value: &RealValue, start: usize) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_count(value, start);
        }
        let mut count: i64 = 0;
        let size = self.size();
        let s = start as i64 - 1;
        let mut i = s;
        while i < size as i64 {
            if i < 0 {
                return Err(index_was_outside());
            }
            if real_almost_equals(value, &self.get(i as usize))? {
                count += 1;
            }
            i += 1;
        }
        if value.equals(&RealValue::ZERO) {
            count += (self.len() - size) as i64;
        }
        Ok(RealValue::num(count as f64))
    }

    /// `Resize(int newSize)` (virtual): resizes in place (or returns a new vector for views).
    pub fn resize(&self, new_size: usize) -> CResult<Vector> {
        let kind = match &*self.data() {
            VecData::Dense { .. } => 0,
            VecData::Large { .. } => 1,
            VecData::Hp(_) => 2,
            VecData::Column { .. } => 3,
            VecData::Row { .. } => 4,
        };
        match kind {
            0 => {
                if new_size as i64 > MAX_LENGTH {
                    return Err(exceptions::VectorSizeLimit());
                }
                if let VecData::Dense { values } = &mut *self.data_mut() {
                    if new_size != values.len() {
                        values.resize(new_size, RealValue::ZERO);
                    }
                }
                Ok(self.clone())
            }
            1 => large_vector::resize(self, new_size),
            2 => self.hp_resize(new_size),
            3 => column_vector::resize(self, new_size),
            _ => row_vector::resize(self, new_size),
        }
    }

    // ----- norms and aggregates ------------------------------------------------------------

    /// The first element as `_values[0]` reads it (null reference for views).
    fn raw0(&self) -> CResult<RealValue> {
        if !self.has_raw() {
            return Err(null_reference());
        }
        Ok(self.get(0))
    }

    /// `L1Norm()` (virtual)
    pub fn l1_norm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_l1_norm();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.get(0);
        let u = v.units.clone();
        let mut norm = v.d.abs();
        for i in 1..size {
            let v = self.get(i);
            norm += v.d.abs() * Unit::convert(&u, &v.units, '+')?;
        }
        Ok(RealValue::new(norm, u))
    }

    /// `Norm()` (virtual): the Euclidean norm.
    pub fn norm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_norm();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.raw0()?;
        let mut norm = v.d;
        norm *= norm;
        let u = v.units.clone();
        for i in 1..size {
            let v = self.get(i);
            let d = v.d * Unit::convert(&u, &v.units, ',')?;
            norm += d * d;
        }
        Ok(RealValue::new(norm.sqrt(), u))
    }

    /// `LpNorm(int p)` (virtual)
    pub fn lp_norm(&self, p: i32) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_lp_norm(p);
        }
        if p < 1 {
            return Err(exceptions::InvalidLpNormArgument());
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.raw0()?;
        let u = v.units.clone();
        let pf = p as f64;
        let mut norm = v.d.abs().powf(pf);
        for i in 1..size {
            let v = self.get(i);
            let d = Unit::convert(&u, &v.units, ',')?;
            norm += (v.d.abs() * d).powf(pf);
        }
        Ok(RealValue::new(norm.powf(1.0 / pf), u))
    }

    /// `InfNorm()` (virtual)
    pub fn inf_norm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_inf_norm();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.raw0()?;
        let u = v.units.clone();
        let mut norm = v.d.abs();
        for i in 1..size {
            let v = self.get(i);
            let c = v.d.abs() * Unit::convert(&u, &v.units, ',')?;
            if c > norm {
                norm = c;
            }
        }
        Ok(RealValue::new(norm, u))
    }

    /// `Normalize()` (virtual)
    pub fn normalize(&self) -> CResult<Vector> {
        if self.is_hp() {
            return self.hp_normalize();
        }
        Vector::div_vr(self, &self.norm()?)
    }

    /// `Min()` (virtual)
    pub fn min(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_min();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.get(0);
        let mut min = v.d;
        let u = v.units.clone();
        for i in 1..size {
            let v = self.get(i);
            let b = v.d * Unit::convert(&u, &v.units, ',')?;
            if b < min {
                min = b;
            }
        }
        if self.len() > size && min > 0.0 {
            min = 0.0;
        }
        Ok(RealValue::new(min, u))
    }

    /// `Max()` (virtual)
    pub fn max(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_max();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.get(0);
        let mut max = v.d;
        let u = v.units.clone();
        for i in 1..size {
            let v = self.get(i);
            let b = v.d * Unit::convert(&u, &v.units, ',')?;
            if b > max {
                max = b;
            }
        }
        if self.len() > size && max < 0.0 {
            max = 0.0;
        }
        Ok(RealValue::new(max, u))
    }

    /// `Sum()` (virtual)
    pub fn sum(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_sum();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.get(0);
        let mut sum = v.d;
        let u = v.units.clone();
        for i in 1..size {
            let v = self.get(i);
            sum += v.d * Unit::convert(&u, &v.units, ',')?;
        }
        Ok(RealValue::new(sum, u))
    }

    /// `SumSq()` (virtual)
    pub fn sum_sq(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_sum_sq();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.get(0);
        let mut sumsq = v.d;
        let u = v.units.clone();
        sumsq *= sumsq;
        for i in 1..size {
            let v = self.get(i);
            let b = v.d * Unit::convert(&u, &v.units, ',')?;
            sumsq += b * b;
        }
        Ok(RealValue::new(sumsq, u.and_then(|u| u.pow(2.0))))
    }

    /// `Srss()` (virtual)
    pub fn srss(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_srss();
        }
        if self.size() == 0 {
            return Ok(RealValue::ZERO);
        }
        let sumsq = self.sum_sq()?;
        let u = sumsq.units.clone();
        Ok(RealValue::new(sumsq.d.sqrt(), u.and_then(|u| u.pow(0.5))))
    }

    /// `Average()` (virtual)
    pub fn average(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_average();
        }
        if self.size() == 0 {
            return Ok(RealValue::ZERO);
        }
        Ok(self.sum()?.mul_f(1.0 / self.len() as f64))
    }

    /// `Product()` (virtual)
    pub fn product(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_product();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ZERO);
        }
        let v = self.get(0);
        let mut product = v.d;
        let mut u = v.units.clone();
        for i in 1..size {
            let v = self.get(i);
            let (uc, b) = Unit::multiply(&u, &v.units, false);
            u = uc;
            product *= v.d * b;
        }
        if self.len() > size {
            return Ok(RealValue::new(0.0, u));
        }
        Ok(RealValue::new(product, u))
    }

    /// `Mean()` (virtual): the geometric mean.
    pub fn mean(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_mean();
        }
        if self.size() == 0 {
            return Ok(RealValue::ZERO);
        }
        let product = self.product()?;
        let len = self.len();
        let result = product.d.powf(1.0 / len as f64);
        match &product.units {
            None => Ok(RealValue::num(result)),
            Some(u) => Ok(RealValue::new(result, Unit::root(u, len as i32, false))),
        }
    }

    /// `And()` (virtual)
    pub fn and_all(&self) -> RealValue {
        let size = self.size();
        if size == 0 || self.len() > size {
            return RealValue::ZERO;
        }
        for i in 0..size {
            if self.get(i).d.abs() < LOGICAL_ZERO {
                return RealValue::ZERO;
            }
        }
        RealValue::ONE
    }

    /// `Or()` (virtual)
    pub fn or_all(&self) -> RealValue {
        let size = self.size();
        if size == 0 {
            return RealValue::ZERO;
        }
        for i in 0..size {
            if self.get(i).d.abs() >= LOGICAL_ZERO {
                return RealValue::ONE;
            }
        }
        RealValue::ZERO
    }

    /// `Xor()` (virtual)
    pub fn xor_all(&self) -> RealValue {
        let size = self.size();
        if size == 0 {
            return RealValue::ZERO;
        }
        let mut b = self.get(0).d.abs() >= LOGICAL_ZERO;
        for i in 1..size {
            b = b != (self.get(i).d.abs() >= LOGICAL_ZERO);
        }
        if b {
            RealValue::ONE
        } else {
            RealValue::ZERO
        }
    }

    /// `Gcd()` (virtual)
    pub fn gcd(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_gcd();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ONE);
        }
        let v = self.get(0);
        let mut a = calculator::as_long(v.d)?;
        let u = v.units.clone();
        for i in 1..size {
            let v = self.get(i);
            let b = calculator::as_long(v.d * Unit::convert(&u, &v.units, ',')?)?;
            a = calculator::gcd_l(a, b);
        }
        Ok(RealValue::num(a as f64))
    }

    /// `Lcm()` (virtual)
    pub fn lcm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return self.hp_lcm();
        }
        let size = self.size();
        if size == 0 {
            return Ok(RealValue::ONE);
        }
        let v = self.get(0);
        let mut a = calculator::as_long(v.d)?;
        let u = v.units.clone();
        for i in 1..size {
            let v = self.get(i);
            let b = calculator::as_long(v.d * Unit::convert(&u, &v.units, ',')?)?;
            let g = calculator::gcd_l(a, b);
            if g == 0 {
                return Err(MathParserException::unexpected("Attempted to divide by zero."));
            }
            a = a.wrapping_mul(b) / g;
        }
        Ok(RealValue::num(a as f64))
    }

    /// `Take(in RealValue x)` (virtual)
    pub fn take(&self, x: &RealValue) -> CResult<RealValue> {
        let d = x.d.round();
        let len = self.len() as f64;
        if !d.is_normal() || d < calculator::DELTA_MINUS || d >= len * calculator::DELTA_PLUS {
            return Ok(RealValue::NAN);
        }
        Ok(self.get(d as usize - 1))
    }

    /// `Line(in RealValue x)` (virtual)
    pub fn line(&self, x: &RealValue) -> CResult<RealValue> {
        let d = x.d;
        let len = self.len();
        if !d.is_normal() || d < calculator::DELTA_MINUS || d >= len as f64 * calculator::DELTA_PLUS {
            return Ok(RealValue::NAN);
        }
        let i = d.floor() as i64;
        if i < 1 {
            return Err(index_was_outside());
        }
        let v1 = self.get(i as usize - 1);
        if i as f64 == d || d >= len as f64 {
            return Ok(v1);
        }
        let dv = RealValue::sub(&self.get(i as usize), &v1)?;
        RealValue::add(&v1, &dv.mul_f(d - i as f64))
    }

    /// `Spline(in RealValue x)` (virtual)
    pub fn spline(&self, x: &RealValue) -> CResult<RealValue> {
        let mut d = x.d;
        let len = self.len() as i64;
        if !d.is_normal() || d < calculator::DELTA_MINUS || d >= len as f64 * calculator::DELTA_PLUS {
            return Ok(RealValue::NAN);
        }
        let i = d.floor() as i64 - 1;
        if i < 0 {
            return Err(index_was_outside());
        }
        let v = self.get(i as usize);
        if i as f64 == d || d >= len as f64 {
            return Ok(v);
        }
        let u = v.units.clone();
        let y0 = v.d;
        let v = self.get(i as usize + 1);
        let y1 = v.d * Unit::convert(&u, &v.units, ',')?;
        let mut dy = y1 - y0;
        let mut a = dy;
        let mut b = dy;
        dy = crate::base_types::complex::math_sign(dy)? as f64;
        if i > 0 {
            let v = self.get(i as usize - 1);
            let y2 = v.d * Unit::convert(&u, &v.units, ',')?;
            a = (y1 - y2)
                * if crate::base_types::complex::math_sign(y0 - y2)? as f64 == dy {
                    0.5
                } else {
                    0.25
                };
        }
        if i < len - 2 {
            let v = self.get(i as usize + 2);
            let y2 = v.d * Unit::convert(&u, &v.units, ',')?;
            b = (y2 - y0)
                * if crate::base_types::complex::math_sign(y2 - y1)? as f64 == dy {
                    0.5
                } else {
                    0.25
                };
        }
        if i == 0 {
            a += (a - b) / 2.0;
        }
        if i == len - 2 {
            b += (b - a) / 2.0;
        }
        let t = d - i as f64 - 1.0;
        d = y0 + ((y1 - y0) * (3.0 - 2.0 * t) * t + ((a + b) * t - a) * (t - 1.0)) * t;
        Ok(RealValue::new(d, u))
    }
}
