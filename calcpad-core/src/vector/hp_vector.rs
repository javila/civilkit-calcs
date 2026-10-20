//! Port of Vector/HpVector.cs: a high performance vector of doubles that share one unit.
//!
//! This file holds the storage and the members the other vector and matrix types use.
//! The HpVector overrides of the virtual `Vector` methods are dispatched from
//! `vector/mod.rs` to the `hp_*` methods at the end of this file.
use super::{VecData, Vector, MAX_LENGTH};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};

/// `System.Numerics.Vector<double>.Count` on the x64 (AVX2) machines that produced the
/// reference output.
pub const VEC_SIZE: usize = 4;

#[derive(Debug, Clone)]
pub struct HpData {
    /// `_values` (None while not allocated)
    pub values: Option<Vec<f64>>,
    pub size: usize,
    pub capacity: usize,
    pub length: usize,
    pub units: OptUnit,
}

impl HpData {
    pub fn new(length: usize, size: usize, units: OptUnit) -> HpData {
        HpData {
            values: Some(vec![0.0; size]),
            size,
            capacity: size,
            length,
            units,
        }
    }

    /// `GetValue(int index)`
    #[inline]
    pub fn get_value(&self, index: usize) -> f64 {
        if index >= self.size {
            0.0
        } else {
            self.values.as_ref().unwrap()[index]
        }
    }

    /// Indexer get
    pub fn get(&self, index: usize) -> RealValue {
        RealValue::new(self.get_value(index), self.units.clone())
    }

    fn expand_capacity(&mut self, index: usize) {
        self.capacity = 125 * index / 100 + 1;
        if self.capacity > self.length {
            self.capacity = self.length;
        }
        match &mut self.values {
            None => {
                if self.length <= 10 {
                    self.capacity = self.length;
                } else if index == 0 && self.capacity < self.length / 100 {
                    self.capacity = self.length / 100;
                }
                self.values = Some(vec![0.0; self.capacity]);
            }
            Some(v) => v.resize(self.capacity, 0.0),
        }
    }

    /// Indexer set (converts the value to the units of the vector)
    pub fn set(&mut self, index: usize, value: &RealValue) -> CResult<()> {
        if index >= self.size {
            if value.equals(&RealValue::ZERO) {
                return Ok(());
            }
            self.size = index + 1;
            if index >= self.capacity {
                self.expand_capacity(index);
            }
        }
        let d = Unit::convert(&self.units, &value.units, '.')?;
        let vals = self.values.as_mut().unwrap();
        if d == 1.0 {
            vals[index] = value.d;
        } else {
            vals[index] = value.d * d;
        }
        Ok(())
    }

    /// `SetValue(double value, int index)`
    pub fn set_value(&mut self, value: f64, index: usize) {
        if index >= self.size {
            if value == 0.0 {
                return;
            }
            self.size = index + 1;
            if index >= self.capacity {
                self.expand_capacity(index);
            }
        }
        self.values.as_mut().unwrap()[index] = value;
    }

    /// `Values`: note that C# leaves the element at `_size` with no units.
    pub fn values(&self) -> Vec<RealValue> {
        let mut values = vec![RealValue::ZERO; self.length];
        for (i, v) in values.iter_mut().enumerate().take(self.size) {
            *v = RealValue::new(self.values.as_ref().unwrap()[i], self.units.clone());
        }
        if self.units.is_some() {
            let zero = RealValue::new(0.0, self.units.clone());
            for v in values.iter_mut().skip(self.size + 1) {
                *v = zero.clone();
            }
        }
        values
    }

    /// `Fill(double value, int start, int len)`
    pub fn fill_range(&mut self, value: f64, start: usize, len: usize) {
        let end = start + len;
        if self.size < end {
            self.size = end;
        }
        if self.capacity < end {
            self.capacity = end;
            match &mut self.values {
                Some(v) => v.resize(end, 0.0),
                None => self.values = Some(vec![0.0; end]),
            }
        }
        for x in &mut self.values.as_mut().unwrap()[start..end] {
            *x = value;
        }
    }

    /// The stored elements `_values[0.._size]`.
    pub fn raw(&self) -> &[f64] {
        match &self.values {
            Some(v) => &v[..self.size.min(v.len())],
            None => &[],
        }
    }
}

// ----- constructors --------------------------------------------------------------------------

/// `new HpVector(double[] values, Unit units)`
pub fn from_doubles(values: Vec<f64>, units: OptUnit) -> CResult<Vector> {
    if values.len() as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    let length = values.len();
    let mut size = 0;
    for i in (0..length).rev() {
        if values[i] != 0.0 {
            size = i + 1;
            break;
        }
    }
    Ok(Vector::from_data(VecData::Hp(HpData {
        values: Some(values),
        size,
        capacity: 0,
        length,
        units,
    })))
}

/// `new HpVector(RealValue[] values)`
pub fn from_values(values: &[RealValue]) -> CResult<Vector> {
    if values.len() as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    let length = values.len();
    let mut h = HpData {
        values: None,
        size: 0,
        capacity: 0,
        length,
        units: None,
    };
    if length > 0 {
        h.units = values[0].units.clone();
        for i in (0..length).rev() {
            h.set(i, &values[i])?;
        }
    }
    Ok(Vector::from_data(VecData::Hp(h)))
}

/// `new HpVector(int length, Unit units)`
pub fn new_length(length: usize, units: OptUnit) -> CResult<Vector> {
    if length as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    Ok(Vector::from_data(VecData::Hp(HpData {
        values: None,
        size: 0,
        capacity: 0,
        length,
        units,
    })))
}

/// `new HpVector(int length, int size, Unit units)`
pub fn new_ls(length: usize, size: usize, units: OptUnit) -> Vector {
    Vector::from_data(VecData::Hp(HpData::new(length, size, units)))
}

/// `new HpVector(Vector vec, Unit units)`
pub fn from_vector(vec: &Vector, units: OptUnit) -> CResult<Vector> {
    let mut h = HpData::new(vec.len(), vec.size(), units);
    if h.size == 0 {
        return Ok(Vector::from_data(VecData::Hp(h)));
    }
    let values = vec.raw().ok_or_else(super::null_reference)?;
    if h.units.is_none() {
        h.units = values[0].units.clone();
    }
    for i in (0..h.size).rev() {
        let v = &values[i];
        let d = v.d * Unit::convert(&h.units, &v.units, ',')?;
        h.values.as_mut().unwrap()[i] = d;
    }
    Ok(Vector::from_data(VecData::Hp(h)))
}

/// `FromIndexes(IEnumerable<int> indexes)`: 1 based indexes as an HpVector.
pub fn from_indexes(indexes: &[i32]) -> Vector {
    let n = indexes.len();
    let mut h = HpData::new(n, n, None);
    let vals = h.values.as_mut().unwrap();
    for (i, &index) in indexes.iter().enumerate() {
        vals[i] = (index + 1) as f64;
    }
    Vector::from_data(VecData::Hp(h))
}

pub(crate) fn hp_not_ported() -> MathParserException {
    MathParserException::new("HP vectors are not supported by calcpad-rs yet.")
}

/// `TryJoin(IValue[] items, int len, out HpVector result)`
pub fn try_join(_items: &[Value], _len: usize) -> CResult<Option<Vector>> {
    Err(hp_not_ported())
}

// ----- HpVector overrides of the virtual Vector members ---------------------------------------

impl Vector {
    /// Runs `f` on the HP storage of this vector.
    pub(crate) fn with_hp<R>(&self, f: impl FnOnce(&HpData) -> R) -> R {
        match &*self.data() {
            VecData::Hp(h) => f(h),
            _ => unreachable!("not an HpVector"),
        }
    }

    pub(crate) fn with_hp_mut<R>(&self, f: impl FnOnce(&mut HpData) -> R) -> R {
        match &mut *self.data_mut() {
            VecData::Hp(h) => f(h),
            _ => unreachable!("not an HpVector"),
        }
    }

    /// `HpVector.Units`
    pub fn hp_units(&self) -> OptUnit {
        self.with_hp(|h| h.units.clone())
    }

    /// `HpVector.Units = units`
    pub fn set_hp_units(&self, units: OptUnit) {
        self.with_hp_mut(|h| h.units = units);
    }

    /// `Scale(double d)`: multiplies the stored values by `d`.
    pub fn hp_scale(&self, d: f64) {
        self.with_hp_mut(|h| {
            let size = h.size;
            if let Some(v) = h.values.as_mut() {
                for x in &mut v[..size] {
                    *x *= d;
                }
            }
        });
    }

    /// `Equals(HpVector)` (note the C# condition `_size != other._size`)
    pub(crate) fn hp_equals(&self, other: &Vector) -> bool {
        if !other.is_hp() {
            return false;
        }
        let (len_a, size_a, raw_a) = self.with_hp(|h| (h.length, h.size, h.raw().to_vec()));
        let (len_b, size_b, raw_b) = other.with_hp(|h| (h.length, h.size, h.raw().to_vec()));
        len_a == len_b && size_a != size_b && raw_a == raw_b
    }

    /// `Copy()`
    pub(crate) fn hp_copy(&self) -> Vector {
        self.with_hp(|h| {
            let mut c = HpData::new(h.length, h.size, h.units.clone());
            c.values.as_mut().unwrap()[..h.size].copy_from_slice(h.raw());
            Vector::from_data(VecData::Hp(c))
        })
    }

    pub(crate) fn hp_sort(&self, _reverse: bool) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_reverse(&self) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_order(&self, _reverse: bool) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_get_order_indexes(&self, _reverse: bool, _len: Option<usize>) -> CResult<Vec<i32>> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_first(&self, _length: usize) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_last(&self, _length: usize) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_slice(&self, _start: i64, _end: i64) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_fill(&self, _value: &RealValue) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_search(&self, _value: &RealValue, _start: usize) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_find_all(&self, _value: &RealValue, _start: usize, _rel: super::Relation) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_extract(&self, _indexes: &Vector) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_count(&self, _value: &RealValue, _start: usize) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_resize(&self, _new_size: usize) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_l1_norm(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_norm(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_lp_norm(&self, _p: i32) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_inf_norm(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_normalize(&self) -> CResult<Vector> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_min(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_max(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_sum(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_sum_sq(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_srss(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_average(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_product(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_mean(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_gcd(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
    pub(crate) fn hp_lcm(&self) -> CResult<RealValue> {
        Err(hp_not_ported())
    }
}

/// The HpVector operators (`HpVector op RealValue`, `RealValue op HpVector`,
/// `HpVector op HpVector`), dispatched from `Value::binary`.
pub mod ops {
    use super::hp_not_ported;
    use crate::base_types::real_value::RealValue;
    use crate::base_types::value::BinOp;
    use crate::math_parser_exception::CResult;
    use crate::vector::Vector;

    /// `-HpVector`
    pub fn neg(_a: &Vector) -> CResult<Vector> {
        Err(hp_not_ported())
    }

    /// `HpVector op RealValue`
    pub fn op_vr(_op: BinOp, _a: &Vector, _b: &RealValue) -> CResult<Vector> {
        Err(hp_not_ported())
    }

    /// `RealValue op HpVector` (only the operators that do not commute)
    pub fn op_rv(_op: BinOp, _a: &RealValue, _b: &Vector) -> CResult<Vector> {
        Err(hp_not_ported())
    }

    /// `HpVector op HpVector`
    pub fn op_vv(_op: BinOp, _a: &Vector, _b: &Vector) -> CResult<Vector> {
        Err(hp_not_ported())
    }
}
