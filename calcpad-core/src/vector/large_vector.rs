//! Port of Vector/LargeVector.cs: a vector that stores only the leading `_size` elements
//! (the rest are zero) and grows its storage on demand.
use super::{VecData, Vector, MAX_LENGTH};
use crate::base_types::real_value::RealValue;
use crate::exceptions;
use crate::math_parser_exception::CResult;

/// `new LargeVector(RealValue[] values)`
pub fn from_values(values: Vec<RealValue>) -> CResult<Vector> {
    if values.len() as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    let length = values.len();
    let mut size = 0;
    for i in (0..length).rev() {
        if !values[i].equals(&RealValue::ZERO) {
            size = i + 1;
            break;
        }
    }
    Ok(Vector::from_data(VecData::Large {
        values: Some(values),
        size,
        capacity: 0,
        length,
    }))
}

/// `new LargeVector(int length)`
pub fn new_length(length: usize) -> CResult<Vector> {
    if length as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    Ok(new_l(length))
}

pub(crate) fn new_l(length: usize) -> Vector {
    Vector::from_data(VecData::Large {
        values: None,
        size: 0,
        capacity: 0,
        length,
    })
}

/// `new LargeVector(int length, int size)`
pub(crate) fn new_ls(length: usize, size: usize) -> Vector {
    Vector::from_data(VecData::Large {
        values: Some(vec![RealValue::ZERO; size]),
        size,
        capacity: size,
        length,
    })
}

/// `ExpandCapacity(int index)`
fn expand_capacity(values: &mut Option<Vec<RealValue>>, capacity: &mut usize, length: usize, index: usize) {
    *capacity = 125 * index / 100 + 1;
    if *capacity > length {
        *capacity = length;
    }
    match values {
        None => {
            if length <= 10 {
                *capacity = length;
            } else if *capacity < length / 100 {
                *capacity = length / 100;
            }
            *values = Some(vec![RealValue::ZERO; *capacity]);
        }
        Some(v) => v.resize(*capacity, RealValue::ZERO),
    }
}

/// Indexer set
pub(super) fn set(d: &mut VecData, index: usize, value: RealValue) {
    if let VecData::Large {
        values,
        size,
        capacity,
        length,
    } = d
    {
        if index >= *size {
            if value.equals(&RealValue::ZERO) {
                return;
            }
            *size = index + 1;
            if index >= *capacity {
                expand_capacity(values, capacity, *length, index);
            }
        }
        values.as_mut().unwrap()[index] = value;
    }
}

/// The size and capacity update of `ValueByRef(int index)`.
pub(super) fn value_by_ref_extend(d: &mut VecData, index: usize) {
    if let VecData::Large {
        values,
        size,
        capacity,
        length,
    } = d
    {
        if index >= *size {
            *size = index + 1;
            if index >= *capacity {
                expand_capacity(values, capacity, *length, index);
            }
        }
    }
}

/// `Copy()`
pub(super) fn copy(v: &Vector) -> Vector {
    let (length, size, vals) = match &*v.data() {
        VecData::Large {
            values, size, length, ..
        } => (
            *length,
            *size,
            values.as_ref().map(|x| x[..*size].to_vec()).unwrap_or_default(),
        ),
        _ => unreachable!(),
    };
    let c = new_ls(length, size);
    if let VecData::Large { values, .. } = &mut *c.data_mut() {
        values.as_mut().unwrap()[..size].clone_from_slice(&vals);
    }
    c
}

/// `Resize(int newSize)`
pub(super) fn resize(v: &Vector, new_size: usize) -> CResult<Vector> {
    if new_size as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    if let VecData::Large {
        values,
        size,
        capacity,
        length,
    } = &mut *v.data_mut()
    {
        if new_size != *length {
            *length = new_size;
            if *size > *length {
                *size = *length;
            }
            if *capacity > *length {
                *capacity = *length;
                if let Some(vals) = values {
                    vals.resize(*length, RealValue::ZERO);
                } else {
                    *values = Some(vec![RealValue::ZERO; *length]);
                }
            }
        }
    }
    Ok(v.clone())
}

/// `Slice(int start, int end)`
pub(super) fn slice(v: &Vector, start: i64, end: i64) -> CResult<Vector> {
    let (mut start, mut end) = (start, end);
    if v.check_bounds(&mut start, &mut end) {
        return Ok(v.copy());
    }
    let length = (end - start) as usize;
    let size_v = v.size() as i64;
    if end > size_v {
        end = size_v;
    }
    let size = if start < end { (end - start) as usize } else { 0 };
    let vector = new_ls(length, size);
    for i in 0..size {
        vector.raw_set(i, v.get(start as usize + i));
    }
    Ok(vector)
}

/// `Reverse()`
pub(super) fn reverse(v: &Vector) -> CResult<Vector> {
    let length = v.len();
    let size = v.size();
    let vector = new_l(length);
    let mut start = 0;
    while start < size {
        if !v.get(start).equals(&RealValue::ZERO) {
            break;
        }
        start += 1;
    }
    for i in start..size {
        vector.set(length - i - 1, v.get(i))?;
    }
    Ok(vector)
}

/// `Fill(RealValue value)`
pub(super) fn fill(v: &Vector, value: &RealValue) -> CResult<Vector> {
    if let VecData::Large {
        values,
        size,
        capacity,
        length,
    } = &mut *v.data_mut()
    {
        if value.equals(&RealValue::ZERO) {
            *size = 0;
            *capacity = 0;
            *values = Some(vec![]);
            return Ok(v.clone());
        }
        if *size < *length {
            *size = *length;
            if *capacity < *length {
                *capacity = *length;
                *values = Some(vec![RealValue::ZERO; *capacity]);
            }
        }
        if let Some(vals) = values {
            for x in vals.iter_mut() {
                *x = value.clone();
            }
        }
    }
    Ok(v.clone())
}

/// `Fill(RealValue value, int start, int len)`
pub(super) fn fill_range(d: &mut VecData, value: &RealValue, start: usize, len: usize) {
    if let VecData::Large {
        values, size, capacity, ..
    } = d
    {
        let end = start + len;
        if *size < end {
            *size = end;
        }
        if *capacity < end {
            *capacity = end;
            match values {
                Some(v) => v.resize(end, RealValue::ZERO),
                None => *values = Some(vec![RealValue::ZERO; end]),
            }
        }
        let vals = values.as_mut().unwrap();
        for x in &mut vals[start..end] {
            *x = value.clone();
        }
    }
}
