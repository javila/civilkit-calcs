//! Port of Matrix/SymmetricMatrix.cs: row `i` stores columns `i..n` of the upper half.
//!
//! The numerical members (Cholesky, LDLT, determinant, solving, inverse, eigenvalues,
//! condition, adjoint) are in `matrix_solvers.rs`.
use super::{Matrix, MatrixType, MAX_SIZE};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::Unit;
use crate::calculator::real_calculator;
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::vector::{self, large_vector};

/// `new SymmetricMatrix(int length)`
pub fn new(length: usize) -> Matrix {
    let rows = (0..length).map(|i| large_vector::new_l(length - i)).collect();
    Matrix::from_parts(MatrixType::Symmetric, false, length, length, rows, None)
}

pub fn new_checked(length: usize) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    Ok(new(length))
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

/// `Resize(int n)`
pub(super) fn resize(m: &Matrix, n: usize) -> CResult<Matrix> {
    super::upper_triangular_matrix::resize(m, n)
}

/// `Count(RealValue value)`
pub(super) fn count(m: &Matrix, value: &RealValue) -> CResult<RealValue> {
    let rows = m.rows();
    if rows.is_empty() {
        return Ok(RealValue::ZERO);
    }
    let mut count = 0.0;
    for row in &rows {
        count += 2.0 * row.count(value, 2)?.d;
        if vector::real_almost_equals(&row.get(0), value)? {
            count += 1.0;
        }
    }
    Ok(RealValue::num(count))
}

/// `Sum()`
pub(super) fn sum(m: &Matrix) -> CResult<RealValue> {
    let rows = m.rows();
    if rows.is_empty() {
        return Ok(RealValue::ZERO);
    }
    let v = rows[0].sum()?;
    let mut sum = 2.0 * v.d;
    let u = v.units.clone();
    let d0 = rows[0].get(0);
    sum -= d0.d * Unit::convert(&u, &d0.units, ',')?;
    for row in &rows[1..] {
        let v = row.sum()?;
        sum += 2.0 * v.d * Unit::convert(&u, &v.units, ',')?;
        let d = row.get(0);
        sum -= d.d * Unit::convert(&u, &d.units, ',')?;
    }
    Ok(RealValue::new(sum, u))
}

/// `SumSq()`
pub(super) fn sum_sq(m: &Matrix) -> CResult<RealValue> {
    let rows = m.rows();
    if rows.is_empty() {
        return Ok(RealValue::ZERO);
    }
    let v = rows[0].sum_sq()?;
    let mut sumsq = 2.0 * v.d;
    let u = v.units.clone();
    let restore = |row: &vector::Vector, s: &mut f64| -> CResult<()> {
        let r0 = row.get(0);
        let v = RealValue::mul_op(&r0, &r0);
        *s -= v.d * Unit::convert(&u, &v.units, ',')?;
        Ok(())
    };
    restore(&rows[0], &mut sumsq)?;
    for row in &rows[1..] {
        let v = row.sum_sq()?;
        sumsq += 2.0 * v.d * Unit::convert(&u, &v.units, ',')?;
        restore(row, &mut sumsq)?;
    }
    Ok(RealValue::new(sumsq, u))
}

/// `Product()`
pub(super) fn product(m: &Matrix) -> CResult<RealValue> {
    let rows = m.rows();
    if rows.is_empty() {
        return Ok(RealValue::ZERO);
    }
    let p = rows[0].product()?;
    let mut product = p.d * p.d;
    let mut u = p.units.as_ref().and_then(|u| u.pow(2.0));
    let restore = |row: &vector::Vector, u: &mut crate::base_types::unit::OptUnit, product: &mut f64| {
        let p = row.get(0);
        let (uc, b) = Unit::divide(u, &p.units, false);
        *u = uc;
        *product *= b;
        if p.d != 0.0 {
            *product /= p.d;
        }
    };
    restore(&rows[0], &mut u, &mut product);
    for row in &rows[1..] {
        let p = row.product()?;
        let (uc, b) = Unit::multiply(&u, &p.units.as_ref().and_then(|u| u.pow(2.0)), false);
        u = uc;
        product *= p.d * p.d * b;
        restore(row, &mut u, &mut product);
    }
    Ok(RealValue::new(product, u))
}

/// `Transpose()`
pub(super) fn transpose(m: &Matrix) -> CResult<Matrix> {
    Ok(raw_copy(m))
}

/// `InfNorm()`
pub(super) fn inf_norm(m: &Matrix) -> CResult<RealValue> {
    let mut norm = RealValue::ZERO;
    for i in 0..m.row_count() {
        let mut sum_abs = real_calculator::abs(&m.get(i, 0));
        for j in 1..m.col_count() {
            sum_abs = RealValue::add(&sum_abs, &real_calculator::abs(&m.get(i, j)))?;
        }
        if i == 0 || vector::real_compare_to(&sum_abs, &norm)? > 0 {
            norm = sum_abs;
        }
    }
    Ok(norm)
}
