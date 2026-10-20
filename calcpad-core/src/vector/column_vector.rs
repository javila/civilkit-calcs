//! Port of Vector/ColumnVector.cs: a view of one column of a matrix.
use super::{VecData, Vector};
use crate::base_types::real_value::RealValue;
use crate::math_parser_exception::CResult;
use crate::matrix::Matrix;

/// `new ColumnVector(Matrix matrix, int column)`
pub fn new(matrix: &Matrix, column: usize) -> Vector {
    let size = matrix.row_count();
    Vector::from_data(VecData::Column {
        matrix: matrix.clone(),
        col: column,
        size,
    })
}

fn parts(v: &Vector) -> (Matrix, usize) {
    match &*v.data() {
        VecData::Column { matrix, col, .. } => (matrix.clone(), *col),
        _ => unreachable!(),
    }
}

/// `Fill(RealValue value)`
pub(super) fn fill(v: &Vector, value: &RealValue) -> CResult<Vector> {
    let (m, col) = parts(v);
    for i in (0..v.len()).rev() {
        m.set(i, col, value.clone())?;
    }
    Ok(v.clone())
}

/// `Resize(int newSize)`
pub(super) fn resize(v: &Vector, new_size: usize) -> CResult<Vector> {
    if new_size == v.len() {
        return Ok(v.clone());
    }
    let (m, col) = parts(v);
    let vector = Vector::new_size(new_size)?;
    let n = new_size.min(v.len());
    for i in (0..n).rev() {
        vector.set(i, m.get(i, col))?;
    }
    Ok(vector)
}
