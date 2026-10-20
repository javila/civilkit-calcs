//! Port of Matrix/HpMatrix/HpSymmetricMatrix.cs: the aggregates and the numerical members
//! of the HP symmetric matrix (skyline Cholesky and LDLT, solving, inverse, eigenvalues).
//! Constructors, element access and Resize are in `mod.rs`.
use super::{create_from_cols, hp_eigen_solver, hp_linear_solver, units, vectorized, HpMat};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::exceptions;
use crate::extension_methods::almost_equals;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::{Matrix, MatrixType, PARALLEL_THRESHOLD};
use crate::vector::{hp_vector, HpData, Vector};

const VEC: usize = vectorized::VEC;

fn out_of_range() -> MathParserException {
    MathParserException::unexpected("Index was outside the bounds of the array.")
}

/// `RawCopy()` (also `Transpose()`)
pub(crate) fn raw_copy(m: &Matrix) -> Matrix {
    let a = HpMat::from(m);
    let n = a.rows.len();
    let mut c = HpMat::empty(MatrixType::Symmetric, n, n, a.units.clone());
    for i in (0..n).rev() {
        c.rows[i] = a.rows[i].copy();
    }
    c.into_matrix()
}

/// `Count(RealValue value)`
pub(super) fn count(m: &Matrix, value: &RealValue) -> CResult<RealValue> {
    let u = units(m);
    if !Unit::is_consistent_opt(&u, &value.units) {
        return Ok(RealValue::ZERO);
    }
    let d = value.d * Unit::convert(&u, &value.units, ',')?;
    let rows = m.rows();
    let len = rows.len();
    if len > PARALLEL_THRESHOLD {
        let mut values = vec![0.0; len];
        for (i, row) in rows.iter().enumerate() {
            let mut count = 2.0 * row.hp_count(value, 2)?.d;
            if almost_equals(row.with_hp(|h| h.get_value(0)), d) {
                count += 1.0;
            }
            values[i] = count;
        }
        return Ok(hp_vector::sum_data(&full(values, None)));
    }
    let mut count = 0.0;
    for row in &rows {
        count += 2.0 * row.hp_count(value, 2)?.d;
        if almost_equals(row.with_hp(|h| h.get_value(0)), d) {
            count += 1.0;
        }
    }
    Ok(RealValue::num(count))
}

/// `new HpVector(len, len, units)` with the given values.
fn full(values: Vec<f64>, units: OptUnit) -> HpData {
    let n = values.len();
    HpData {
        values: Some(values),
        size: n,
        capacity: n,
        length: n,
        units,
    }
}

/// `Sum()`
pub(super) fn sum(m: &Matrix) -> CResult<RealValue> {
    let a = HpMat::from(m);
    let len = a.rows.len();
    if len > PARALLEL_THRESHOLD {
        let values: Vec<f64> = a
            .rows
            .iter()
            .map(|r| 2.0 * hp_vector::sum_data(r).d - r.get_value(0))
            .collect();
        return Ok(hp_vector::sum_data(&full(values, a.units.clone())));
    }
    if len == 0 {
        return Ok(RealValue::new(0.0, a.units.clone()));
    }
    let mut sum = 2.0 * hp_vector::sum_data(&a.rows[0]).d - a.rows[0].get_value(0);
    for r in &a.rows[1..] {
        sum += 2.0 * hp_vector::sum_data(r).d - r.get_value(0);
    }
    Ok(RealValue::new(sum, a.units))
}

/// `SumSq()`
pub(super) fn sum_sq(m: &Matrix) -> CResult<RealValue> {
    let a = HpMat::from(m);
    let len = a.rows.len();
    let u = a.units.as_ref().and_then(|u| u.pow(2.0));
    if len > PARALLEL_THRESHOLD {
        let values: Vec<f64> = a
            .rows
            .iter()
            .map(|r| {
                let d = r.get_value(0);
                2.0 * hp_vector::sum_sq_data(r).d - d * d
            })
            .collect();
        return Ok(hp_vector::sum_data(&full(values, u)));
    }
    if len == 0 {
        return Ok(RealValue::new(0.0, u));
    }
    let d = a.rows[0].get_value(0);
    let mut sumsq = 2.0 * hp_vector::sum_sq_data(&a.rows[0]).d - d * d;
    for r in &a.rows[1..] {
        let d = r.get_value(0);
        sumsq += 2.0 * hp_vector::sum_sq_data(r).d - d * d;
    }
    Ok(RealValue::new(sumsq, u))
}

/// `Product()`
pub(super) fn product(m: &Matrix) -> CResult<RealValue> {
    let a = HpMat::from(m);
    let n = a.row_count * a.col_count;
    let len = a.rows.len();
    let u = a.units.as_ref().and_then(|u| u.pow(n as f32));
    if len == 0 {
        return Ok(RealValue::new(0.0, u));
    }
    if len > PARALLEL_THRESHOLD {
        let mut values = vec![0.0; len];
        for (i, row) in a.rows.iter().enumerate() {
            let p = hp_vector::product_data(row).d;
            if p != 0.0 {
                values[i] = p * p / row.get_value(0);
            }
        }
        return Ok(hp_vector::product_data(&full(values, u)));
    }
    let row = &a.rows[0];
    let p = hp_vector::product_data(row).d;
    let mut product = p * p;
    if p != 0.0 {
        product /= row.get_value(0);
    }
    for row in &a.rows[1..] {
        let p = hp_vector::product_data(row).d;
        product *= p * p;
        if p != 0.0 {
            product /= row.get_value(0);
        }
    }
    Ok(RealValue::new(product, u))
}

/// `InfNorm()`
pub(super) fn inf_norm(m: &Matrix) -> CResult<RealValue> {
    let a = HpMat::from(m);
    let mut norm = 0.0;
    for i in 0..a.row_count {
        let mut sum_abs = a.get_value(i, 0).abs();
        for j in 1..a.col_count {
            sum_abs += a.get_value(i, j).abs();
        }
        if i == 0 || crate::vector::double_compare_to(sum_abs, norm) > 0 {
            norm = sum_abs;
        }
    }
    Ok(RealValue::new(norm, a.units))
}

// ----- skyline factorizations ------------------------------------------------------------------

/// `GetRowsStart()`: the first non-zero row of each column, rounded down to a multiple
/// of the SIMD width.
fn get_rows_start(a: &HpMat) -> Vec<usize> {
    let n = a.row_count;
    let mut start = vec![n; n];
    for i in 0..n {
        let len = i + a.rows[i].size;
        for s in start.iter_mut().take(len).skip(i) {
            if i < *s {
                *s = i;
            }
        }
    }
    for j in 0..a.col_count {
        let mut i0 = start[j];
        let mut i = i0;
        while i < j {
            if a.get_value(i, j) != 0.0 {
                break;
            }
            i0 += 1;
            i += 1;
        }
        start[j] = i0 - (i0 % VEC);
    }
    start
}

/// `GetCholesky(out int[] start)`: the rows of L from their start column, or None when
/// the matrix is not positive definite.
fn get_cholesky(a: &HpMat) -> Option<(Vec<Vec<f64>>, Vec<usize>)> {
    let start = get_rows_start(a);
    let n = a.row_count;
    let mut l: Vec<Vec<f64>> = Vec::with_capacity(n);
    for (i, &s) in start.iter().enumerate() {
        let len = i as i64 - s as i64 + 1;
        if len < 1 {
            return None;
        }
        l.push(vec![0.0; len as usize]);
    }
    // The block parallel algorithm of C# computes the same elements in the same way.
    for i in 0..n {
        let row = &a.rows[i];
        if row.size == 0 {
            return None;
        }
        let d0 = row.raw_all()[0];
        let i0 = start[i];
        for j in start[i]..=i {
            let mut sum = 0.0;
            if i == j {
                let sa = &l[i];
                let len = i - i0;
                let nv = len / VEC;
                for k in 0..nv {
                    sum += vectorized::vdot(sa, k * VEC, sa, k * VEC);
                }
                for k in nv * VEC..len {
                    sum += sa[k] * sa[k];
                }
                let d = d0 - sum;
                if d <= 0.0 {
                    return None;
                }
                l[i][i - i0] = d.sqrt();
            } else {
                let j0 = start[j];
                let k0 = i0.max(j0);
                let len = j as i64 - k0 as i64;
                let len = len.max(0) as usize;
                let nv = len / VEC;
                let sb_off = k0 - j0;
                let k0 = k0 - i0;
                let (sa, sb) = (&l[i], &l[j]);
                if nv > 0 {
                    for k in 0..nv {
                        sum += vectorized::vdot(sa, k0 + k * VEC, sb, sb_off + k * VEC);
                    }
                }
                for k in nv * VEC..len {
                    sum += sa[k0 + k] * sb[sb_off + k];
                }
                let v = (a.get_value(i, j) - sum) / l[j][j - j0];
                l[i][j - i0] = v;
            }
        }
    }
    Some((l, start))
}

/// `GetLDLT(out int[] start)`: D is stored on the diagonal of L.
fn get_ldlt(a: &HpMat) -> Option<(Vec<Vec<f64>>, Vec<usize>)> {
    let start = get_rows_start(a);
    let n = a.row_count;
    let mut l: Vec<Vec<f64>> = Vec::with_capacity(n);
    let mut d = vec![0.0; n];
    for (i, &s) in start.iter().enumerate() {
        let len = i as i64 - s as i64 + 1;
        if len < 1 {
            return None;
        }
        l.push(vec![0.0; len as usize]);
    }
    for i in 0..n {
        let row = &a.rows[i];
        if row.size == 0 {
            return None;
        }
        let d0 = row.raw_all()[0];
        let i0 = start[i];
        for j in start[i]..=i {
            let j0 = start[j];
            let k0 = i0.max(j0);
            let len = j + 1;
            let mut sum = 0.0;
            for k in k0..len {
                sum += l[i][k - i0] * l[j][k - j0] * d[k];
            }
            if i == j {
                d[i] = d0 - sum;
                l[j][j - j0] = d[i];
            } else {
                let v = (a.get_value(i, j) - sum) / l[j][j - j0];
                l[i][j - i0] = v;
            }
        }
    }
    Some((l, start))
}

/// `GetUFromLRows(lRows, start)`
fn get_u_from_l_rows(a: &HpMat, l: &[Vec<f64>], start: &[usize]) -> HpMat {
    let n = a.row_count;
    let mut u = HpMat::empty(
        MatrixType::UpperTriangular,
        n,
        n,
        a.units.as_ref().and_then(|u| u.pow(0.5)),
    );
    let mut maxsize: i64 = 1;
    for i in 0..n {
        let row = &a.rows[i];
        maxsize -= 1;
        if row.size as i64 > maxsize {
            maxsize = row.size as i64;
        }
        let mut nn = (a.col_count - i) as i64;
        if nn > maxsize {
            nn = maxsize;
        }
        for j in (0..nn.max(0) as usize).rev() {
            let ll = i + j;
            let l_j = &l[ll];
            let k = i as i64 - start[ll] as i64;
            if k >= 0 {
                u.rows[i].set_value(l_j[k as usize], j);
            }
        }
    }
    u
}

/// `CholeskyDecomposition()`
pub fn cholesky_decomposition(m: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (l, start) = get_cholesky(&a).ok_or_else(exceptions::MatrixNotPositiveDefinite)?;
    Ok(get_u_from_l_rows(&a, &l, &start).into_matrix())
}

fn get_determinant(l: &[Vec<f64>]) -> f64 {
    let mut det = 1.0;
    for l_i in l {
        det *= l_i[l_i.len() - 1];
    }
    det
}

/// `Determinant()`
pub fn determinant(m: &Matrix) -> CResult<RealValue> {
    let a = HpMat::from(m);
    let n = a.row_count;
    let un = a.units.as_ref().and_then(|u| u.pow(n as f32));
    match get_ldlt(&a) {
        None => Ok(if a.units.is_none() {
            RealValue::ZERO
        } else {
            RealValue::new(0.0, un)
        }),
        Some((l, _)) => Ok(RealValue::new(get_determinant(&l), un)),
    }
}

/// `FwdAndBackSubst(L, start, v, x, Cholesky)`
fn fwd_and_back_subst(l: &[Vec<f64>], start: &[usize], v: &HpData, x: &mut [f64], cholesky: bool) {
    let m = l.len();
    let mut s0: i64 = -1;
    for i in 0..m {
        let mut sum = v.get_value(i);
        let l_i = &l[i];
        if s0 >= 0 {
            let s0u = s0 as usize;
            let j0 = start[i];
            let j1 = s0u.max(j0);
            let n = i as i64 - j1 as i64;
            let n = n.max(0) as usize;
            let sl_off = j1 - j0;
            let mut nv = n / VEC;
            if nv > 0 {
                for j in 0..nv {
                    sum -= vectorized::vdot(l_i, sl_off + j * VEC, x, j1 + j * VEC);
                }
                nv *= VEC;
            }
            let mut j = j1 + nv;
            while j < i {
                sum -= l_i[sl_off + j - j1] * x[j];
                j += 1;
            }
        } else if sum != 0.0 {
            s0 = ((i / VEC) * VEC) as i64;
        }
        x[i] = if cholesky { sum / l_i[l_i.len() - 1] } else { sum };
    }
    if !cholesky {
        for i in 0..m {
            x[i] /= l[i][l[i].len() - 1];
        }
    }
    for i in (0..m).rev() {
        let mut sum = x[i];
        for k in i + 1..m {
            let i0 = start[k];
            if i >= i0 {
                sum -= l[k][i - i0] * x[k];
            }
        }
        x[i] = if cholesky { sum / l[i][l[i].len() - 1] } else { sum };
    }
}

fn solve_with(a: &HpMat, l: &[Vec<f64>], start: &[usize], vector: &Vector, cholesky: bool) -> CResult<Vector> {
    let v = hp_vector::snap(vector);
    let mut x = vec![0.0; a.row_count];
    fwd_and_back_subst(l, start, &v, &mut x, cholesky);
    let (u, d) = Unit::divide(&v.units, &a.units, true);
    if d != 1.0 {
        vectorized::scale(&mut x, d);
    }
    hp_vector::from_doubles(x, u)
}

/// `LSolve(HpVector vector)`
pub fn lsolve(m: &Matrix, vector: &Vector) -> CResult<Vector> {
    let a = HpMat::from(m);
    let (l, start) = get_ldlt(&a).ok_or_else(exceptions::MatrixSingular)?;
    solve_with(&a, &l, &start, vector, false)
}

/// `ClSolve(HpVector v)`
pub fn cl_solve(m: &Matrix, vector: &Vector) -> CResult<Vector> {
    let a = HpMat::from(m);
    let (l, start) = get_cholesky(&a).ok_or_else(exceptions::MatrixNotPositiveDefinite)?;
    solve_with(&a, &l, &start, vector, true)
}

/// `SlSolve(HpVector v, double tol)`
pub fn sl_solve(m: &Matrix, vector: &Vector, tol: f64) -> CResult<Vector> {
    let a = HpMat::from(m);
    let v = hp_vector::snap(vector);
    let mut x = hp_linear_solver::solve_pcg(&a, v.raw_all(), tol)?;
    let (u, d) = Unit::divide(&v.units, &a.units, true);
    if d != 1.0 {
        vectorized::scale(&mut x, d);
    }
    hp_vector::from_doubles(x, u)
}

fn msolve_for_l(l: &[Vec<f64>], start: &[usize], b: &Matrix, units_a: &OptUnit, cholesky: bool) -> CResult<Matrix> {
    let m = l.len();
    let n = b.col_count();
    let (u, d) = Unit::divide(&units(b), units_a, true);
    let mut v = Vec::with_capacity(n);
    for j in 0..n {
        let col = hp_vector::snap(&super::col(b, j as i64 + 1)?);
        let mut x = vec![0.0; m];
        fwd_and_back_subst(l, start, &col, &mut x, cholesky);
        if d != 1.0 {
            vectorized::scale(&mut x, d);
        }
        v.push(hp_vector::from_doubles(x, u.clone())?);
    }
    create_from_cols(&v, m)
}

/// `MSolve(HpMatrix M)`
pub fn msolve(m: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (l, start) = get_ldlt(&a).ok_or_else(exceptions::MatrixSingular)?;
    msolve_for_l(&l, &start, b, &a.units, false)
}

/// `CmSolve(HpMatrix M)`
pub fn cm_solve(m: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (l, start) = get_cholesky(&a).ok_or_else(exceptions::MatrixNotPositiveDefinite)?;
    msolve_for_l(&l, &start, b, &a.units, true)
}

/// `SmSolve(HpMatrix M, double tol)`
pub fn sm_solve(m: &Matrix, b: &Matrix, tol: f64) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let bt = super::as_jagged_array_transposed(&HpMat::from(b))?;
    let mut x = hp_linear_solver::msolve_pcg(&a, &bt, tol)?;
    let (u, d) = Unit::divide(&units(b), &a.units, true);
    if d != 1.0 {
        for xi in x.iter_mut() {
            vectorized::scale(xi, d);
        }
    }
    super::from_jagged_array_transposed(&x, u)
}

/// `MSolveForL(L, start, units)`: the inverse, column by column. C# builds its right hand
/// sides from a symmetric "identity" whose rows hold a single 1.
fn msolve_for_l_inverse(l: &[Vec<f64>], start: &[usize], units: OptUnit) -> CResult<Matrix> {
    let m = l.len();
    // Identity(m): M.SetValue(1d, i, n - 1)
    let mut mm = HpMat::empty(MatrixType::Symmetric, m, m, None);
    for i in 0..m {
        mm.set_value(1.0, i, m - 1)?;
    }
    let mut result = Vec::with_capacity(m);
    for j in 0..m {
        let mut x = vec![0.0; m];
        let rhs = mm.rows[m - j - 1].clone();
        fwd_and_back_subst(l, start, &rhs, &mut x, false);
        result.push(hp_vector::from_doubles(x, units.clone())?);
    }
    for j in (0..m).rev() {
        let col = hp_vector::snap(&result[j]);
        for i in (j..m).rev() {
            let v = col.get_value(i);
            mm.rows[j].set_value(v, i - j);
        }
    }
    Ok(mm.into_matrix())
}

/// `GetInverse(L, start)`
fn get_inverse(a: &HpMat, l: &[Vec<f64>], start: &[usize]) -> CResult<Matrix> {
    let units = a.units.as_ref().and_then(|u| u.pow(-1.0));
    let n = a.row_count;
    if n > PARALLEL_THRESHOLD {
        return msolve_for_l_inverse(l, start, units);
    }
    let mut vector = HpData::new(n, n, None);
    let mut x = vec![0.0; n];
    let mut mm = HpMat::empty(MatrixType::Symmetric, n, n, units);
    for j in 0..n {
        vector.values.as_mut().unwrap()[j] = 1.0;
        fwd_and_back_subst(l, start, &vector, &mut x, false);
        for i in j..n {
            mm.set_value(x[i], i, j)?;
        }
        vector.values.as_mut().unwrap()[j] = 0.0;
    }
    Ok(mm.into_matrix())
}

/// `Invert()`
pub fn invert(m: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (l, start) = get_ldlt(&a).ok_or_else(exceptions::MatrixSingular)?;
    get_inverse(&a, &l, &start)
}

/// `Condition(norm)`
pub fn condition(m: &Matrix, norm: &dyn Fn(&Matrix) -> CResult<RealValue>) -> CResult<RealValue> {
    let a = HpMat::from(m);
    let (l, start) = match get_ldlt(&a) {
        None => return Ok(RealValue::POSITIVE_INFINITY),
        Some(x) => x,
    };
    let inv = get_inverse(&a, &l, &start)?;
    Ok(RealValue::mul_op(&norm(m)?, &norm(&inv)?))
}

/// `Adjoint()`
pub fn adjoint(m: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (l, start) = get_ldlt(&a).ok_or_else(exceptions::MatrixSingular)?;
    let det = get_determinant(&l);
    let inv = get_inverse(&a, &l, &start)?;
    super::mul_mr(&inv, &RealValue::num(det))
}

/// `GetLU`: the U of the LDLT decomposition in a symmetric matrix.
pub(crate) fn get_lu(m: &Matrix) -> CResult<Option<(HpMat, Vec<i32>, f64, f64)>> {
    let a = HpMat::from(m);
    let n = a.row_count;
    let indexes: Vec<i32> = (0..n as i32).collect();
    let (l, start) = match get_ldlt(&a) {
        None => return Ok(None),
        Some(x) => x,
    };
    let u = get_u_from_l_rows(&a, &l, &start);
    let mut lu = HpMat::empty(MatrixType::Symmetric, n, n, a.units.clone());
    lu.rows = u.rows;
    Ok(Some((lu, indexes, f64::MAX, 1.0)))
}

// ----- eigenvalues ---------------------------------------------------------------------------

fn check_count(n: usize, count: &mut i64) -> bool {
    if *count < 0 {
        *count = -*count;
        return true;
    }
    if *count == 0 || *count > n as i64 {
        *count = n as i64;
    }
    false
}

fn is_direct(n: usize, count: usize) -> bool {
    n <= 200 || n <= 1000 && count > n / 5
}

/// `SortEigenValues(d, units, count, reverse)`
fn sort_eigen_values(d: &mut [f64], units: OptUnit, count: usize, reverse: bool) -> CResult<Vector> {
    if reverse {
        crate::sort::sort_by::<f64, ()>(d, None, &mut |a, b| Ok(crate::vector::double_compare_to(*b, *a)))?;
    } else {
        crate::sort::sort_doubles::<()>(d, None);
    }
    hp_vector::from_doubles(d[..count].to_vec(), units)
}

/// `SortEigenVectors(d, Q, count, sort, reverse, combine, units)`
fn sort_eigen_vectors(
    d: &mut [f64],
    q: &[Vec<f64>],
    count: usize,
    sort: bool,
    reverse: bool,
    combine: bool,
    units: OptUnit,
) -> CResult<Matrix> {
    let len = d.len();
    let count = count.min(len);
    let mut indexes: Vec<i32> = Vec::new();
    if sort {
        indexes = (0..len as i32).collect();
        crate::sort::sort_doubles(d, Some(&mut indexes));
    }
    let j0 = if combine { 1 } else { 0 };
    let n = q.len();
    let index_of = |i: usize| -> usize {
        if sort {
            if reverse {
                indexes[len - 1 - i] as usize
            } else {
                indexes[i] as usize
            }
        } else {
            i
        }
    };
    if combine && units.is_some() {
        let mm = Matrix::new(count, n + j0)?;
        for i in 0..count {
            let index = index_of(i);
            let row = mm.row_ref(i);
            for j in (0..n).rev() {
                row.set(j + j0, RealValue::num(q[j][index]))?;
            }
            row.set(0, RealValue::new(d[i], units.clone()))?;
        }
        return Ok(mm);
    }
    let mut hm = HpMat::empty(MatrixType::Full, count, n + j0, None);
    for i in 0..count {
        let index = index_of(i);
        for j in (0..n).rev() {
            hm.rows[i].set_value(q[j][index], j + j0);
        }
        if combine {
            hm.rows[i].set_value(d[i], 0);
        }
    }
    Ok(hm.into_matrix())
}

/// `EigenVectors(int count, double tol)`
pub fn eigen_vectors(m: &Matrix, count: i64, tol: f64) -> CResult<Matrix> {
    eigen_impl(m, count, tol, false)
}

/// `Eigen(int count, double tol)`
pub fn eigen(m: &Matrix, count: i64, tol: f64) -> CResult<Matrix> {
    eigen_impl(m, count, tol, true)
}

fn eigen_impl(m: &Matrix, count: i64, tol: f64, combine: bool) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let n = a.row_count;
    let mut count = count;
    let reverse = check_count(n, &mut count);
    let count = count as usize;
    let jagged = super::as_jagged_array(&a);
    if is_direct(n, count) {
        let (mut q, mut d, mut e) = hp_eigen_solver::tridiagonalize(&jagged, true);
        hp_eigen_solver::implicit_ql(&mut d, &mut e, &mut q, true)?;
        sort_eigen_vectors(&mut d, &q, count, true, reverse, combine, a.units)
    } else {
        let (q, mut d) = hp_eigen_solver::lanczos(&jagged, count, reverse, tol)?;
        sort_eigen_vectors(&mut d, &q, count, false, false, combine, a.units)
    }
}

/// `EigenValues(int count, double tol)`
pub fn eigen_values(m: &Matrix, count: i64, tol: f64) -> CResult<Vector> {
    let a = HpMat::from(m);
    let n = a.row_count;
    let mut count = count;
    let reverse = check_count(n, &mut count);
    let count = count as usize;
    let jagged = super::as_jagged_array(&a);
    if is_direct(n, count) {
        let (mut q, mut d, mut e) = hp_eigen_solver::tridiagonalize(&jagged, false);
        hp_eigen_solver::implicit_ql(&mut d, &mut e, &mut q, false)?;
        sort_eigen_values(&mut d, a.units, count, reverse)
    } else {
        let (_, d) = hp_eigen_solver::lanczos(&jagged, count, reverse, tol)?;
        hp_vector::from_doubles(d, a.units)
    }
}

#[allow(dead_code)]
fn _unused() -> MathParserException {
    out_of_range()
}
