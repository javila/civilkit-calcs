//! Port of Matrix/ColumnMatrix.cs: a single column stored in `rows[0]`.
use super::{Matrix, MatrixType, MAX_SIZE};
use crate::base_types::real_value::RealValue;
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::vector::{large_vector, row_vector, Vector};

/// `new ColumnMatrix(int length)`
pub fn new(length: usize) -> Matrix {
    Matrix::from_parts(
        MatrixType::Column,
        false,
        length,
        1,
        vec![large_vector::new_l(length)],
        None,
    )
}

/// `new ColumnMatrix(int length)` with the size check
pub fn new_checked(length: usize) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatrixDimensions());
    }
    Ok(new(length))
}

/// `new ColumnMatrix(int length, RealValue value)`
pub fn with_value(length: usize, value: &RealValue) -> CResult<Matrix> {
    let m = new_checked(length)?;
    m.row_ref(0).fill(value)?;
    Ok(m)
}

/// `new ColumnMatrix(Vector v)`: shares the vector when it is a LargeVector.
pub fn from_vector(v: &Vector) -> CResult<Matrix> {
    let len = v.len();
    if len > MAX_SIZE {
        return Err(exceptions::MatrixDimensions());
    }
    let row = Matrix::as_large_row(v)?;
    Ok(Matrix::from_parts(MatrixType::Column, false, len, 1, vec![row], None))
}

/// `Resize(int n)`
pub(super) fn resize(m: &Matrix, n: usize) -> CResult<Matrix> {
    if n != m.row_count() {
        m.row_ref(0).resize(n)?;
        m.data_mut().row_count = n;
    }
    Ok(m.clone())
}

/// `operator *(Matrix a, ColumnMatrix b)`
pub(super) fn mul(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    if a.mtype() == MatrixType::Column && !a.is_hp() {
        if a.row_count() != b.row_count() {
            return Err(exceptions::MatrixDimensions());
        }
        return from_vector(&Vector::mul(&a.row_ref(0), &b.row_ref(0))?);
    }
    let m = a.row_count();
    let c = new_checked(m)?;
    let cr = c.row_ref(0);
    let br = b.row_ref(0);
    let t = a.mtype();
    if t == MatrixType::Full || t == MatrixType::LowerTriangular {
        for i in (0..m).rev() {
            cr.set(i, Vector::dot_product(&a.row_ref(i), &br)?)?;
        }
    } else {
        for i in (0..m).rev() {
            cr.set(i, Vector::dot_product(&row_vector::new(a, i), &br)?)?;
        }
    }
    Ok(c)
}

/// `Transpose()`
pub(super) fn transpose(m: &Matrix) -> CResult<Matrix> {
    let c = Matrix::new(1, m.row_count())?;
    c.set_row_ref(0, m.row_ref(0).copy());
    Ok(c)
}

/// `ColRef()`
pub fn col_ref(m: &Matrix) -> Vector {
    m.row_ref(0)
}
