//! Port of Matrix/DiagonalMatrix.cs: the diagonal is stored in `rows[0]`.
//!
//! The numerical members (LU, inverse, solving, eigenvalues, Cholesky, condition,
//! adjoint) are in `matrix_solvers.rs`.
use super::{Matrix, MatrixType, MAX_SIZE};
use crate::base_types::real_value::RealValue;
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::vector::{large_vector, row_vector, Vector};

/// `new DiagonalMatrix(int length)`
pub fn new(length: usize) -> Matrix {
    Matrix::from_parts(
        MatrixType::Diagonal,
        false,
        length,
        length,
        vec![large_vector::new_l(length)],
        None,
    )
}

/// `base(length)` size check of the square matrix constructors
pub fn new_checked(length: usize) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    Ok(new(length))
}

/// `new DiagonalMatrix(int length, in RealValue value)`
pub fn with_value(length: usize, value: &RealValue) -> CResult<Matrix> {
    let m = new_checked(length)?;
    m.row_ref(0).fill(value)?;
    Ok(m)
}

/// `new DiagonalMatrix(Vector a)`: shares the vector when it is a LargeVector.
pub fn from_vector(a: &Vector) -> CResult<Matrix> {
    let len = a.len();
    if len > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let row = Matrix::as_large_row(a)?;
    Ok(Matrix::from_parts(
        MatrixType::Diagonal,
        false,
        len,
        len,
        vec![row],
        None,
    ))
}

/// `RawCopy()`
pub(crate) fn raw_copy(m: &Matrix) -> Matrix {
    let c = new(m.row_count());
    c.set_row_ref(0, m.row_ref(0).copy());
    c
}

/// `Resize(int n)`
pub(super) fn resize(m: &Matrix, n: usize) -> CResult<Matrix> {
    if n != m.row_count() {
        m.row_ref(0).resize(n)?;
        m.set_dims(n, n);
    }
    Ok(m.clone())
}

/// `DiagonalMatrix * DiagonalMatrix`
pub(super) fn mul_dd(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    from_vector(&Vector::mul(&a.row_ref(0), &b.row_ref(0))?)
}

/// `DiagonalMatrix * LowerTriangularMatrix`
pub(super) fn mul_dl(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let mut n = b.row_count();
    let c = super::lower_triangular_matrix::new(n);
    let diag = a.row_ref(0);
    if n > diag.size() {
        n = diag.size();
    }
    for i in (0..n).rev() {
        c.set_row_ref(i, Vector::mul_vr(&b.row_ref(i), &diag.get(i))?);
    }
    Ok(c)
}

/// `DiagonalMatrix * UpperTriangularMatrix`
pub(super) fn mul_du(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let mut n = b.row_count();
    let c = super::upper_triangular_matrix::new(n);
    let diag = a.row_ref(0);
    if n > diag.size() {
        n = diag.size();
    }
    for i in (0..n).rev() {
        c.set_row_ref(i, Vector::mul_vr(&b.row_ref(i), &diag.get(i))?);
    }
    Ok(c)
}

/// `DiagonalMatrix * SymmetricMatrix`
pub(super) fn mul_ds(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let mut n = b.row_count();
    let c = Matrix::new(n, n)?;
    let diag = a.row_ref(0);
    if n > diag.size() {
        n = diag.size();
    }
    for i in (0..n).rev() {
        let c_row = Vector::mul_vr(&row_vector::new(b, i), &diag.get(i))?;
        c.set_row_ref(i, large_vector::from_values(c_row.values())?);
    }
    Ok(c)
}

/// `DiagonalMatrix * Matrix`
pub(super) fn mul_dm(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let mut n = b.row_count();
    let c = Matrix::new(n, b.col_count())?;
    let diag = a.row_ref(0);
    if n > diag.size() {
        n = diag.size();
    }
    if b.mtype() == MatrixType::Full {
        for i in (0..n).rev() {
            c.set_row_ref(i, Vector::mul_vr(&b.row_ref(i), &diag.get(i))?);
        }
    } else {
        for i in (0..n).rev() {
            let c_row = Vector::mul_vr(&row_vector::new(b, i), &diag.get(i))?;
            c.set_row_ref(i, large_vector::from_values(c_row.values())?);
        }
    }
    Ok(c)
}

/// `LowerTriangularMatrix * DiagonalMatrix`
pub(super) fn mul_ld(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let n = a.row_count();
    let c = super::lower_triangular_matrix::new(n);
    let diag = b.row_ref(0);
    for i in (0..n).rev() {
        c.set_row_ref(i, Vector::mul(&a.row_ref(i), &diag)?);
    }
    Ok(c)
}

/// `UpperTriangularMatrix * DiagonalMatrix`
pub(super) fn mul_ud(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let n = a.row_count();
    let c = super::upper_triangular_matrix::new(n);
    let diag_v = b.row_ref(0);
    let diag = diag_v.raw().unwrap_or_default();
    let diag_size = diag_v.size() as i64;
    for i in (0..n).rev() {
        let a_row = a.row_ref(i);
        let len = a_row.len();
        let size = (a_row.size() as i64).min(diag_size - i as i64);
        let mut c_raw = vec![RealValue::ZERO; len];
        let mut j = size - 1;
        while j >= 0 {
            let ju = j as usize;
            c_raw[ju] = RealValue::mul_op(&a_row.get(ju), &diag[i + ju]);
            j -= 1;
        }
        c.set_row_ref(i, large_vector::from_values(c_raw)?);
    }
    Ok(c)
}

/// `SymmetricMatrix * DiagonalMatrix`
pub(super) fn mul_sd(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let n = a.row_count();
    let c = Matrix::new(n, n)?;
    let diag = b.row_ref(0);
    for i in (0..n).rev() {
        let c_row = Vector::mul(&row_vector::new(a, i), &diag)?;
        c.set_row_ref(i, large_vector::from_values(c_row.values())?);
    }
    Ok(c)
}

/// `Matrix * DiagonalMatrix`
pub(super) fn mul_md(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let c = Matrix::new(a.row_count(), a.col_count())?;
    let diag = b.row_ref(0);
    if a.mtype() == MatrixType::Full {
        for i in (0..a.row_count()).rev() {
            c.set_row_ref(i, Vector::mul(&a.row_ref(i), &diag)?);
        }
    } else {
        for i in (0..a.row_count()).rev() {
            let c_row = Vector::mul(&row_vector::new(a, i), &diag)?;
            c.set_row_ref(i, large_vector::from_values(c_row.values())?);
        }
    }
    Ok(c)
}

/// `Transpose()`
pub(super) fn transpose(m: &Matrix) -> CResult<Matrix> {
    Ok(raw_copy(m))
}
