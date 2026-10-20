//! Port of Matrix/HpMatrix/HpDiagonalMatrix.cs: the products and the numerical members
//! of the HP diagonal matrix. (Constructors, element access and Resize are in `mod.rs`.)
use super::{diagonal_from_vector, new, new_diagonal, new_lower, new_upper, units, HpMat};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::Unit;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::{Matrix, MatrixType};
use crate::vector::{hp_vector, HpData, Vector};

/// `HpDiagonalMatrix * HpDiagonalMatrix`
pub fn mul_dd(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    diagonal_from_vector(&hp_vector::mul(&a.row_ref(0), &b.row_ref(0))?)
}

/// `HpDiagonalMatrix * HpLowerTriangularMatrix` and `* HpUpperTriangularMatrix`
pub fn mul_dt(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let mut n = b.row_count();
    let (unit, _) = Unit::multiply(&units(a), &units(b), true);
    let c = if b.mtype() == MatrixType::LowerTriangular {
        new_lower(n, unit)?
    } else {
        new_upper(n, unit)?
    };
    let diag = a.row_ref(0);
    let ds = diag.size();
    if n > ds {
        n = ds;
    }
    for i in (0..n).rev() {
        c.set_row_ref(i, hp_vector::mul_vr(&b.row_ref(i), &diag.get(i))?);
    }
    Ok(c)
}

/// `HpDiagonalMatrix * HpSymmetricMatrix`
pub fn mul_ds(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let mut n = b.row_count();
    let (unit, d) = Unit::multiply(&units(a), &units(b), true);
    let mut c = HpMat::empty(MatrixType::Full, n, n, unit);
    let diag = hp_vector::snap(&a.row_ref(0));
    if n > diag.size {
        n = diag.size;
    }
    let bb = HpMat::from(b);
    for i in (0..n).rev() {
        let k = diag.get_value(i) * d;
        for j in (0..n).rev() {
            let v = bb.get_value(i, j) * k;
            c.rows[i].set_value(v, j);
        }
    }
    Ok(c.into_matrix())
}

/// `HpDiagonalMatrix * HpMatrix`
pub fn mul_dm(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let mut n = b.row_count();
    let (unit, _) = Unit::multiply(&units(a), &units(b), true);
    let c = new(n, b.col_count(), unit)?;
    let diag = a.row_ref(0);
    let ds = diag.size();
    if n > ds {
        n = ds;
    }
    for i in (0..n).rev() {
        c.set_row_ref(i, hp_vector::mul_vr(&b.row_ref(i), &diag.get(i))?);
    }
    Ok(c)
}

/// `HpLowerTriangularMatrix * HpDiagonalMatrix`
pub fn mul_ld(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let n = a.row_count();
    let (unit, _) = Unit::multiply(&units(a), &units(b), true);
    let c = new_lower(n, unit)?;
    let diag = b.row_ref(0);
    for i in (0..n).rev() {
        c.set_row_ref(i, hp_vector::mul(&a.row_ref(i), &diag)?);
    }
    Ok(c)
}

/// `HpUpperTriangularMatrix * HpDiagonalMatrix`
pub fn mul_ud(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let n = a.row_count();
    let (unit, d) = Unit::multiply(&units(a), &units(b), true);
    let c = new_upper(n, unit.clone())?;
    let diag_h = hp_vector::snap(&b.row_ref(0));
    let diag = diag_h.raw_all();
    let diag_size = diag_h.size as i64;
    let aa = HpMat::from(a);
    for i in (0..n).rev() {
        let a_row = &aa.rows[i];
        let len = a_row.length;
        let a_raw = a_row.raw_all();
        let size = (a_row.size as i64).min(diag_size - i as i64);
        if size < 0 {
            return Err(MathParserException::unexpected(
                "Arithmetic operation resulted in an overflow.",
            ));
        }
        let size = size as usize;
        let mut row = HpData::new(len, size, unit.clone());
        {
            let c_raw = row.values.as_mut().unwrap();
            for j in (0..size).rev() {
                c_raw[j] = a_raw[j] * diag[i + j] * d;
            }
        }
        c.set_row_ref(i, super::hpd(row));
    }
    Ok(c)
}

/// `HpSymmetricMatrix * HpDiagonalMatrix`
pub fn mul_sd(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let n = a.row_count();
    let (unit, d) = Unit::multiply(&units(a), &units(b), true);
    let mut c = HpMat::empty(MatrixType::Full, n, n, unit);
    let diag_h = hp_vector::snap(&b.row_ref(0));
    let diag = diag_h.raw_all();
    let diag_size = diag_h.size;
    let aa = HpMat::from(a);
    for i in (0..n).rev() {
        for j in (0..diag_size).rev() {
            let v = aa.get_value(i, j) * diag[j] * d;
            c.rows[i].set_value(v, j);
        }
    }
    Ok(c.into_matrix())
}

/// `HpMatrix * HpDiagonalMatrix`
pub fn mul_md(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let (unit, _) = Unit::multiply(&units(a), &units(b), true);
    let c = new(a.row_count(), a.col_count(), unit)?;
    let diag = b.row_ref(0);
    for i in (0..a.row_count()).rev() {
        c.set_row_ref(i, hp_vector::mul(&a.row_ref(i), &diag)?);
    }
    Ok(c)
}

// ----- numerical members ---------------------------------------------------------------------

/// `RawCopy()`
pub(crate) fn raw_copy(m: &Matrix) -> CResult<Matrix> {
    let c = new_diagonal(m.row_count(), units(m))?;
    c.set_row_ref(0, m.row_ref(0).hp_copy());
    Ok(c)
}

/// `Determinant()`
pub fn determinant(m: &Matrix) -> CResult<RealValue> {
    m.row_ref(0).hp_product()
}

/// `GetLU`: the diagonal itself, or None when a diagonal element is zero.
pub(crate) fn get_lu(m: &Matrix) -> CResult<Option<(Matrix, Vec<i32>, f64, f64)>> {
    let n = m.row_count();
    let diag = hp_vector::snap(&m.row_ref(0));
    let mut indexes = vec![0; n];
    for (i, x) in indexes.iter_mut().enumerate() {
        if diag.get_value(i) == 0.0 {
            return Ok(None);
        }
        *x = i as i32;
    }
    Ok(Some((raw_copy(m)?, indexes, f64::MAX, 1.0)))
}

/// `CholeskyDecomposition()`
pub fn cholesky_decomposition(m: &Matrix) -> CResult<Matrix> {
    let n = m.row_count();
    let u = new_upper(n, units(m))?;
    let diag = hp_vector::snap(&m.row_ref(0));
    for i in 0..n {
        super::set_value(&u, diag.get_value(i), i, i)?;
    }
    Ok(u)
}

fn check_singular(m: &Matrix) -> CResult<()> {
    let diag = hp_vector::snap(&m.row_ref(0));
    for i in (0..m.row_count()).rev() {
        if diag.get_value(i) == 0.0 {
            return Err(exceptions::MatrixSingular());
        }
    }
    Ok(())
}

/// `Invert()`
pub fn invert(m: &Matrix) -> CResult<Matrix> {
    check_singular(m)?;
    diagonal_from_vector(&hp_vector::div_rv(&RealValue::ONE, &m.row_ref(0))?)
}

/// `LSolve(HpVector vector)`
pub fn lsolve(m: &Matrix, v: &Vector) -> CResult<Vector> {
    check_singular(m)?;
    hp_vector::div(v, &m.row_ref(0))
}

/// `MSolve(HpMatrix M)`
pub fn msolve(m: &Matrix, b: &Matrix) -> CResult<Matrix> {
    check_singular(m)?;
    let rc = m.row_count();
    let n = b.col_count();
    let mut result = Vec::with_capacity(n);
    for j in 0..n {
        result.push(hp_vector::div(&super::col(b, j as i64 + 1)?, &m.row_ref(0))?);
    }
    super::create_from_cols(&result, rc)
}

fn check_count(m: &Matrix, count: &mut i64) -> bool {
    if *count < 0 {
        *count = -*count;
        return true;
    }
    if *count == 0 || *count > m.row_count() as i64 {
        *count = m.row_count() as i64;
    }
    false
}

/// `EigenValues(int count)`
pub fn eigen_values(m: &Matrix, count: i64) -> CResult<Vector> {
    let mut count = count;
    let reverse = check_count(m, &mut count);
    m.row_ref(0).hp_sort(reverse)?.hp_first(count as usize)
}

/// `EigenVectors(int count)`: C# creates a count x n matrix and writes to row
/// `indexes[i]`, which fails when that row does not exist.
pub fn eigen_vectors(m: &Matrix, count: i64) -> CResult<Matrix> {
    let mut count = count;
    let reverse = check_count(m, &mut count);
    let count = count as usize;
    let indexes = m.row_ref(0).hp_get_order_indexes(reverse, None)?;
    let mut mm = HpMat::empty(MatrixType::Full, count, m.row_count(), None);
    for i in 0..count {
        let r = indexes[i] as usize;
        if r >= count {
            return Err(MathParserException::unexpected(
                "Index was outside the bounds of the array.",
            ));
        }
        mm.set_value(1.0, r, i)?;
    }
    Ok(mm.into_matrix())
}

/// `Eigen(int count)`
pub fn eigen(m: &Matrix, count: i64) -> CResult<Matrix> {
    let mut count = count;
    let reverse = check_count(m, &mut count);
    let count = count as usize;
    let indexes = m.row_ref(0).hp_get_order_indexes(reverse, None)?;
    let mm = Matrix::new(count, m.row_count() + 1)?;
    let diag = m.row_ref(0);
    for i in 0..count {
        let r = indexes[i] as usize;
        if r >= count {
            return Err(MathParserException::unexpected(
                "Index was outside the bounds of the array.",
            ));
        }
        mm.set(r, 0, diag.get(i))?;
        mm.set(i, r + 1, RealValue::ONE)?;
    }
    Ok(mm)
}

/// `Condition(norm)`
pub fn condition(m: &Matrix, norm: &dyn Fn(&Matrix) -> CResult<RealValue>) -> CResult<RealValue> {
    let det = determinant(m)?;
    if det.d == 0.0 {
        return Ok(RealValue::POSITIVE_INFINITY);
    }
    let inv = invert(m)?;
    Ok(RealValue::mul_op(&norm(m)?, &norm(&inv)?))
}

/// `Adjoint()`
pub fn adjoint(m: &Matrix) -> CResult<Matrix> {
    let det = determinant(m)?;
    if det.d == 0.0 {
        return Err(exceptions::MatrixSingular());
    }
    let inv = invert(m)?;
    super::mul_mr(&inv, &det)
}
