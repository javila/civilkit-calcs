//! Port of Matrix/UpperTriangularMatrix.cs: row `i` stores columns `i..n`.
//!
//! The numerical members (determinant, LU, inverse, solving) are in `matrix_solvers.rs`.
use super::{Matrix, MatrixType, MAX_SIZE};
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::vector::{large_vector, Vector};

/// `new UpperTriangularMatrix(int length)`
pub fn new(length: usize) -> Matrix {
    let rows = (0..length).map(|i| large_vector::new_l(length - i)).collect();
    Matrix::from_parts(MatrixType::UpperTriangular, false, length, length, rows, None)
}

pub fn new_checked(length: usize) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    Ok(new(length))
}

/// `Resize(int n)` (shared with SymmetricMatrix)
pub(super) fn resize(m: &Matrix, n: usize) -> CResult<Matrix> {
    let rc = m.row_count();
    if n != rc {
        let n1 = n.min(rc);
        {
            let mut d = m.data_mut();
            d.rows.truncate(n);
            for i in rc..n {
                d.rows.push(large_vector::new_l(n - i));
            }
        }
        for i in 0..n1 {
            m.row_ref(i).resize(n - i)?;
        }
        m.set_dims(n, n);
    }
    Ok(m.clone())
}

/// `RawCol(int col)`
pub(crate) fn raw_col(m: &Matrix, col: usize) -> CResult<Vector> {
    let v = large_vector::new_l(col + 1);
    for i in (0..=col).rev() {
        v.set(i, m.row_ref(i).get(col - i))?;
    }
    Ok(v)
}

/// `Transpose()`
pub(super) fn transpose(m: &Matrix) -> CResult<Matrix> {
    let n = m.row_count();
    let l = super::lower_triangular_matrix::new(n);
    for i in (0..n).rev() {
        l.set_row_ref(i, raw_col(m, i)?);
    }
    Ok(l)
}

/// `RawCopy()`: note that C# returns a LowerTriangularMatrix here.
pub(crate) fn raw_copy(m: &Matrix) -> Matrix {
    let n = m.rows_len();
    let c = super::lower_triangular_matrix::new(n);
    for i in (0..n).rev() {
        c.set_row_ref(i, m.row_ref(i).copy());
    }
    c
}
