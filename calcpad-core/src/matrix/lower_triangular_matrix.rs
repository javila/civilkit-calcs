//! Port of Matrix/LowerTriangularMatrix.cs: row `i` stores columns `0..=i`.
//!
//! The numerical members (determinant, LU, inverse, solving) are in `matrix_solvers.rs`.
use super::{Matrix, MatrixType, MAX_SIZE};
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::vector::{large_vector, Vector};

/// `new LowerTriangularMatrix(int length)`
pub fn new(length: usize) -> Matrix {
    let rows = (0..length).map(|i| large_vector::new_l(i + 1)).collect();
    Matrix::from_parts(MatrixType::LowerTriangular, false, length, length, rows, None)
}

pub fn new_checked(length: usize) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    Ok(new(length))
}

/// `Resize(int n)`
pub(super) fn resize(m: &Matrix, n: usize) -> CResult<Matrix> {
    let rc = m.row_count();
    if n != rc {
        let mut d = m.data_mut();
        d.rows.truncate(n);
        for i in rc..n {
            d.rows.push(large_vector::new_l(i + 1));
        }
        d.row_count = n;
        d.col_count = n;
    }
    Ok(m.clone())
}

/// `RawCol(int col)`
pub(crate) fn raw_col(m: &Matrix, col: usize) -> CResult<Vector> {
    let n = m.col_count() - col;
    let v = large_vector::new_l(n);
    for i in (0..n).rev() {
        v.set(i, m.row_ref(col + i).get(col))?;
    }
    Ok(v)
}

/// `Transpose()`
pub(super) fn transpose(m: &Matrix) -> CResult<Matrix> {
    let n = m.row_count();
    let u = super::upper_triangular_matrix::new(n);
    for i in (0..n).rev() {
        u.set_row_ref(i, raw_col(m, i)?);
    }
    Ok(u)
}

/// `RawCopy()`
pub(crate) fn raw_copy(m: &Matrix) -> Matrix {
    let n = m.rows_len();
    let c = new(n);
    for i in (0..n).rev() {
        c.set_row_ref(i, m.row_ref(i).copy());
    }
    c
}
