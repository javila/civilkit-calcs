//! Port of Vector/RowVector.cs: a view of one row of a matrix.
use super::{VecData, Vector};
use crate::base_types::real_value::RealValue;
use crate::math_parser_exception::CResult;
use crate::matrix::Matrix;

/// `new RowVector(Matrix matrix, int row)`
pub fn new(matrix: &Matrix, row: usize) -> Vector {
    let size = matrix.col_count();
    Vector::from_data(VecData::Row {
        matrix: matrix.clone(),
        row,
        size,
    })
}

fn parts(v: &Vector) -> (Matrix, usize) {
    match &*v.data() {
        VecData::Row { matrix, row, .. } => (matrix.clone(), *row),
        _ => unreachable!(),
    }
}

/// `Fill(RealValue value)`
pub(super) fn fill(v: &Vector, value: &RealValue) -> CResult<Vector> {
    let (m, row) = parts(v);
    for i in (0..v.len()).rev() {
        m.set(row, i, value.clone())?;
    }
    Ok(v.clone())
}

/// `Resize(int newSize)`
pub(super) fn resize(v: &Vector, new_size: usize) -> CResult<Vector> {
    if new_size == v.len() {
        return Ok(v.clone());
    }
    let (m, row) = parts(v);
    let vector = Vector::new_size(new_size)?;
    let n = new_size.min(v.len());
    for i in (0..n).rev() {
        vector.set(i, m.get(row, i))?;
    }
    Ok(vector)
}
