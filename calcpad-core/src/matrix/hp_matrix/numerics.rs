//! The HpMatrix overrides of the numerical matrix members (HpMatrix.cs and its subclasses,
//! HpLinearSolver, HpEigenSolver, HpSvdSolver). Called from `matrix_solvers.rs` for HP
//! matrices; this module dispatches on the HP matrix type as the C# virtual members do.
use super::{
    hp_diagonal_matrix as diag, hp_svd_solver, hp_symmetric_matrix as sym, hp_triangular_matrix as tri, identity,
    units, vectorized, HpMat,
};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::{Matrix, MatrixType, PARALLEL_THRESHOLD};
use crate::vector::{hp_vector, HpData, Vector};

fn out_of_range() -> MathParserException {
    MathParserException::unexpected("Index was outside the bounds of the array.")
}

/// `GetLU(out indexes, out minPivot, out det)` (virtual): None for a singular matrix.
fn get_lu(m: &Matrix) -> CResult<Option<(HpMat, Vec<i32>, f64, f64)>> {
    match m.mtype() {
        MatrixType::Diagonal => Ok(diag::get_lu(m)?.map(|(lu, i, p, d)| (HpMat::from(&lu), i, p, d))),
        MatrixType::LowerTriangular | MatrixType::UpperTriangular => tri::get_lu(m),
        MatrixType::Symmetric => sym::get_lu(m),
        _ => base_get_lu(&HpMat::from(m)),
    }
}

/// `HpMatrix.GetLU`: Crout LU decomposition with partial pivoting.
fn base_get_lu(a: &HpMat) -> CResult<Option<(HpMat, Vec<i32>, f64, f64)>> {
    let n = a.row_count;
    if n != a.col_count {
        return Err(exceptions::MatrixNotSquare());
    }
    let mut lu = HpMat::empty(MatrixType::Full, n, n, a.units.clone());
    let mut vv = vec![0.0; n];
    let mut indexes: Vec<i32> = vec![0; n];
    let mut col_j = vec![0.0; n];
    let mut det = 1.0;
    for i in 0..n {
        indexes[i] = i as i32;
        let row = a.rows[i].copy();
        let size = row.size;
        let big = hp_vector::max_abs(&row);
        lu.rows[i] = row;
        if size == 0 {
            return Ok(None);
        }
        if big == 0.0 {
            return Ok(None);
        }
        vv[i] = 1.0 / big;
    }
    let mut min_pivot = f64::MAX;
    for j in 0..a.col_count {
        for (k, c) in col_j.iter_mut().enumerate() {
            *c = lu.get_value(k, j);
        }
        for i in 0..j {
            let size = lu.rows[i].size.min(i);
            let row = lu.rows[i].raw_all();
            let mut sum = col_j[i];
            let mut k0 = 0;
            while k0 < i && *row.get(k0).ok_or_else(out_of_range)? == 0.0 {
                k0 += 1;
            }
            if size > 0 {
                sum -= vectorized::dot_product(row, &col_j, k0, size);
            }
            col_j[i] = sum;
        }
        let mut big = 0.0;
        let mut imax = j;
        for i in j..n {
            let size = lu.rows[i].size.min(j);
            let row = lu.rows[i].raw_all();
            let mut sum = col_j[i];
            let mut k0 = 0;
            while k0 < j && *row.get(k0).ok_or_else(out_of_range)? == 0.0 {
                k0 += 1;
            }
            if size > 0 {
                sum -= vectorized::dot_product(row, &col_j, k0, size);
            }
            col_j[i] = sum;
            let dum = vv[i] * sum.abs();
            if dum > big {
                big = dum;
                imax = i;
            }
        }
        let d: f64 = big.abs();
        if j != imax {
            if d < min_pivot {
                min_pivot = d;
            }
            lu.rows.swap(j, imax);
            col_j.swap(j, imax);
            vv[imax] = vv[j];
            indexes.swap(j, imax);
            det = -det;
        }
        if j != n - 1 {
            let dum = 1.0 / col_j[j];
            vectorized::scale(&mut col_j[j + 1..n], dum);
        } else if d < min_pivot {
            min_pivot = d;
        }
        for (k, &c) in col_j.iter().enumerate() {
            lu.set_value(c, k, j)?;
        }
    }
    if min_pivot == 0.0 {
        return Ok(None);
    }
    Ok(Some((lu, indexes, min_pivot, det)))
}

/// `FwdAndBackSubst(LU, indexes, v, x)`
fn fwd_and_back_subst(lu: &HpMat, indexes: &[i32], v: &HpData, x: &mut [f64]) -> CResult<()> {
    let m = lu.row_count;
    let mut start: i64 = -1;
    for i in 0..m {
        let index = indexes[i] as usize;
        let mut sum = v.get_value(index);
        if start >= 0 {
            let row = &lu.rows[i];
            let size = row.size.min(i);
            sum -= vectorized::dot_product(row.raw_all(), x, start as usize, size);
        } else if sum != 0.0 {
            start = i as i64;
        }
        x[i] = sum;
    }
    for i in (0..m).rev() {
        let row = &lu.rows[i];
        let size = row.size.min(m);
        let sum = x[i] - vectorized::dot_product(row.raw_all(), x, i + 1, size);
        x[i] = sum / lu.get_value(i, i);
    }
    let _ = out_of_range;
    Ok(())
}

fn get_determinant(a: &HpMat, lu: &HpMat) -> RealValue {
    let n = a.row_count;
    let mut det = lu.get_value(0, 0);
    for i in 1..n {
        det *= lu.get_value(i, i);
    }
    RealValue::new(det, a.units.as_ref().and_then(|u| u.pow(n as f32)))
}

/// `MSolveForLU(LU, indexes, M, units)`
fn msolve_for_lu(lu: &HpMat, indexes: &[i32], b: &Matrix, units_a: &OptUnit) -> CResult<Matrix> {
    let m = lu.row_count;
    let n = b.col_count();
    let (u, d) = Unit::divide(&units(b), units_a, true);
    let mut result = Vec::with_capacity(n);
    for j in 0..n {
        let col = hp_vector::snap(&super::col(b, j as i64 + 1)?);
        let mut x = vec![0.0; m];
        fwd_and_back_subst(lu, indexes, &col, &mut x)?;
        if d != 1.0 {
            vectorized::scale(&mut x, d);
        }
        result.push(hp_vector::from_doubles(x, u.clone())?);
    }
    super::create_from_cols(&result, m)
}

/// `MSolveForLU(LU, indexes, units)`: the inverse.
fn msolve_for_lu_inverse(lu: &HpMat, indexes: &[i32], units: OptUnit) -> CResult<Matrix> {
    let m = lu.row_count;
    let id = identity(m);
    let mut result = Vec::with_capacity(m);
    for j in 0..m {
        let mut x = vec![0.0; m];
        fwd_and_back_subst(lu, indexes, &id.rows[j], &mut x)?;
        result.push(hp_vector::from_doubles(x, units.clone())?);
    }
    let im = id.into_matrix();
    for j in (0..m).rev() {
        super::set_col(&im, j, &result[j])?;
    }
    Ok(im)
}

/// `GetInverse(lu, indexes)`
fn get_inverse(a: &HpMat, lu: &HpMat, indexes: &[i32]) -> CResult<Matrix> {
    let n = a.row_count;
    let units = a.units.as_ref().and_then(|u| u.pow(-1.0));
    if n > PARALLEL_THRESHOLD {
        return msolve_for_lu_inverse(lu, indexes, units);
    }
    let mut vector = HpData::new(n, n, None);
    let mut x = vec![0.0; n];
    let mut mm = HpMat::empty(MatrixType::Full, n, n, units);
    for j in 0..n {
        vector.set_value(1.0, j);
        fwd_and_back_subst(lu, indexes, &vector, &mut x)?;
        for (i, &xi) in x.iter().enumerate() {
            mm.set_value(xi, i, j)?;
        }
        vector.set_value(0.0, j);
    }
    Ok(mm.into_matrix())
}

fn lu_or_singular(m: &Matrix) -> CResult<(HpMat, Vec<i32>, f64, f64)> {
    get_lu(m)?.ok_or_else(exceptions::MatrixSingular)
}

/// `Determinant()` (virtual)
pub fn determinant(m: &Matrix) -> CResult<RealValue> {
    match m.mtype() {
        MatrixType::Diagonal => return diag::determinant(m),
        MatrixType::LowerTriangular | MatrixType::UpperTriangular => return tri::determinant(m),
        MatrixType::Symmetric => return sym::determinant(m),
        _ => {}
    }
    let a = HpMat::from(m);
    if a.row_count != a.col_count {
        return Err(exceptions::MatrixNotSquare());
    }
    match get_lu(m)? {
        None => Ok(RealValue::ZERO),
        Some((lu, _, _, det)) => Ok(get_determinant(&a, &lu).mul_f(det)),
    }
}

/// `Invert()` (virtual)
pub fn invert(m: &Matrix) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Diagonal => return diag::invert(m),
        MatrixType::LowerTriangular | MatrixType::UpperTriangular => return tri::invert(m),
        MatrixType::Symmetric => return sym::invert(m),
        _ => {}
    }
    let a = HpMat::from(m);
    let (lu, indexes, min_pivot, _) = lu_or_singular(m)?;
    if min_pivot < 1e-15 {
        return Err(exceptions::MatrixCloseToSingular());
    }
    get_inverse(&a, &lu, &indexes)
}

/// `LSolve(HpVector vector)` (virtual)
pub fn lsolve(m: &Matrix, v: &Vector) -> CResult<Vector> {
    match m.mtype() {
        MatrixType::Diagonal => return diag::lsolve(m, v),
        MatrixType::LowerTriangular | MatrixType::UpperTriangular => return tri::lsolve(m, v),
        MatrixType::Symmetric => return sym::lsolve(m, v),
        _ => {}
    }
    let (lu, indexes, min_pivot, _) = lu_or_singular(m)?;
    if min_pivot < 1e-15 {
        return Err(exceptions::MatrixCloseToSingular());
    }
    let mut x = vec![0.0; m.row_count()];
    fwd_and_back_subst(&lu, &indexes, &hp_vector::snap(v), &mut x)?;
    hp_vector::from_doubles(x, units(m))
}

/// `MSolve(HpMatrix M)` (virtual)
pub fn msolve(m: &Matrix, b: &Matrix) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Diagonal => return diag::msolve(m, b),
        MatrixType::LowerTriangular | MatrixType::UpperTriangular => return tri::msolve(m, b),
        MatrixType::Symmetric => return sym::msolve(m, b),
        _ => {}
    }
    let (lu, indexes, min_pivot, _) = lu_or_singular(m)?;
    if min_pivot < 1e-15 {
        return Err(exceptions::MatrixCloseToSingular());
    }
    msolve_for_lu(&lu, &indexes, b, &units(m))
}

/// `LUDecomposition(out int[] indexes)` (virtual)
pub fn lu_decomposition(m: &Matrix) -> CResult<(Matrix, Vec<i32>)> {
    if m.mtype() == MatrixType::Diagonal {
        let (lu, indexes, _, _) = diag::get_lu(m)?.ok_or_else(exceptions::MatrixSingular)?;
        return Ok((lu, indexes));
    }
    let (lu, indexes, _, _) = lu_or_singular(m)?;
    Ok((lu.into_matrix(), indexes))
}

/// `GetQR(out Q, out R)`
fn get_qr(a: &HpMat) -> CResult<(HpMat, HpMat)> {
    let m = a.row_count;
    let n = a.col_count;
    let mut r = super::copy_mat(a);
    let mut q = identity(m);
    let mut v = vec![0.0; m];
    let mn = m.min(n);
    for k in 0..mn {
        let mut norm_x = 0.0;
        for i in k..m {
            let rr = r.get_value(i, k);
            norm_x += rr * rr;
        }
        norm_x = norm_x.sqrt();
        if norm_x == 0.0 {
            continue;
        }
        let rk = r.get_value(k, k);
        let alpha = if rk > 0.0 { -norm_x } else { norm_x };
        let rs = 2.0 * (0.5 * alpha * (alpha - rk)).sqrt();
        v[k] = (rk - alpha) / rs;
        for i in k + 1..m {
            v[i] = r.get_value(i, k) / rs;
        }
        for j in k..n {
            let mut dot = 0.0;
            for i in k..m {
                dot += v[i] * r.get_value(i, j);
            }
            let rr = 2.0 * dot;
            for i in k..m {
                let val = r.get_value(i, j) - rr * v[i];
                r.set_value(val, i, j)?;
            }
        }
        for i in 0..m {
            let size = q.rows[i].size;
            let dot = vectorized::dot_product(&v, q.rows[i].raw_all(), k, size);
            let rr = 2.0 * dot;
            for j in (k..m).rev() {
                let val = q.rows[i].get_value(j) - rr * v[j];
                q.rows[i].set_value(val, j);
            }
        }
    }
    Ok((q, r))
}

/// `QRDecomposition()`
pub fn qr_decomposition(m: &Matrix) -> CResult<Matrix> {
    let (q, r) = get_qr(&HpMat::from(m))?;
    let consistent = Unit::is_consistent_opt(&q.units, &r.units);
    let (q, r) = (q.into_matrix(), r.into_matrix());
    if consistent {
        return super::augment(&[q, r]);
    }
    Matrix::augment(&[q, r])
}

/// `GetSVD(out U, out sig, out V)`
fn get_svd(a: &HpMat) -> CResult<(Matrix, HpData, Matrix)> {
    if a.row_count < a.col_count {
        return Err(exceptions::MatrixNotHigh());
    }
    let (rc, cc) = (a.row_count, a.col_count);
    if cc == 0 {
        return Err(out_of_range());
    }
    let m = rc - 1;
    let n = cc - 1;
    let mut rv1 = vec![0.0; cc];
    let mut ud = super::raw_copy(a);
    let mut sig = HpData::new(cc, cc, a.units.clone());
    let mut vd = vec![vec![0.0; cc]; cc];
    let mut col_buffer = vec![0.0; rc];
    {
        let sig_raw = sig.values.as_mut().unwrap();
        let anorm = hp_svd_solver::bidiagonalize(&mut ud, sig_raw, &mut rv1, &mut col_buffer, m, n);
        hp_svd_solver::right_transform(&ud, &mut vd, &rv1, n);
        hp_svd_solver::left_transform(&mut ud, sig_raw, &mut col_buffer, m, n);
        hp_svd_solver::diagonalize(&mut ud, sig_raw, &mut vd, &mut rv1, m, n, anorm)?;
    }
    let u = super::from_jagged(ud, None)?;
    let v = super::from_jagged(vd, None)?;
    Ok((u, sig, v))
}

/// `SVDDecomposition()`
pub fn svd_decomposition(m: &Matrix) -> CResult<Matrix> {
    let (mut u, sig, mut v) = get_svd(&HpMat::from(m))?;
    // SvdSortDescending
    let mut sigv = super::hpd(sig);
    let indexes = sigv.hp_get_order_indexes(true, None)?;
    let is_sorted = indexes.iter().enumerate().all(|(i, &k)| k == i as i32);
    if !is_sorted {
        sigv = sigv.hp_sort(true)?;
        u = super::reorder_cols(&u, &indexes)?;
        v = super::reorder_rows(&v, &indexes)?;
    }
    let su = sigv.hp_units();
    let sig_m = super::hp_column_from_vector(&sigv)?;
    if Unit::is_consistent_opt(&units(&u), &su) && Unit::is_consistent_opt(&units(&v), &su) {
        return super::augment(&[u, sig_m, v]);
    }
    Matrix::augment(&[u, sig_m, v])
}

/// `L2Norm()`
pub fn l2_norm(m: &Matrix) -> CResult<RealValue> {
    let (_, sig, _) = get_svd(&HpMat::from(m))?;
    Ok(hp_vector::max_data(&sig))
}

/// `Cond2()`
pub fn cond2(m: &Matrix) -> CResult<RealValue> {
    let (_, sig, _) = get_svd(&HpMat::from(m))?;
    Ok(RealValue::div_op(
        &hp_vector::max_data(&sig),
        &hp_vector::min_data(&sig),
    ))
}

/// `Condition(norm)` (virtual)
fn condition(m: &Matrix, norm: &dyn Fn(&Matrix) -> CResult<RealValue>) -> CResult<RealValue> {
    match m.mtype() {
        MatrixType::Diagonal => return diag::condition(m, norm),
        MatrixType::Symmetric => return sym::condition(m, norm),
        _ => {}
    }
    let a = HpMat::from(m);
    if a.row_count != a.col_count {
        return Err(exceptions::MatrixNotSquare());
    }
    let (lu, indexes, _, _) = match get_lu(m)? {
        None => return Ok(RealValue::POSITIVE_INFINITY),
        Some(x) => x,
    };
    let inv = get_inverse(&a, &lu, &indexes)?;
    Ok(RealValue::mul_op(&norm(m)?, &norm(&inv)?))
}

/// `Cond1()`
pub fn cond1(m: &Matrix) -> CResult<RealValue> {
    condition(m, &|x| x.l1_norm())
}

/// `CondFrob()`
pub fn cond_frob(m: &Matrix) -> CResult<RealValue> {
    condition(m, &|x| x.frob_norm())
}

/// `CondInf()`
pub fn cond_inf(m: &Matrix) -> CResult<RealValue> {
    condition(m, &|x| x.inf_norm())
}

/// `double.BitIncrement(x)`
fn bit_increment(x: f64) -> f64 {
    if x.is_nan() || x == f64::INFINITY {
        return x;
    }
    if x == 0.0 {
        return f64::from_bits(1);
    }
    let bits = x.to_bits();
    if x > 0.0 {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    }
}

/// `Rank()`
pub fn rank(m: &Matrix) -> CResult<RealValue> {
    let (_, sig, _) = get_svd(&HpMat::from(m))?;
    let sig_max = hp_vector::max_data(&sig).d;
    let eps = bit_increment(sig_max) - sig_max;
    let mut rank = 0;
    for i in 0..sig.length {
        if sig.get_value(i).abs() > eps {
            rank += 1;
        }
    }
    Ok(RealValue::num(rank as f64))
}

fn remove_at(source: &[usize], index: usize) -> Vec<usize> {
    let mut t = Vec::with_capacity(source.len().saturating_sub(1));
    for (i, &s) in source.iter().enumerate() {
        if i != index {
            t.push(s);
        }
    }
    t
}

fn determinant_minor(a: &HpMat, rows: &[usize], cols: &[usize]) -> f64 {
    let n = rows.len();
    if n == 1 {
        return a.get_value(rows[0], cols[0]);
    }
    if n == 2 {
        let (r0, r1) = (rows[0], rows[1]);
        let aa = a.get_value(r0, cols[0]);
        let b = a.get_value(r0, cols[1]);
        let c = a.get_value(r1, cols[0]);
        let d = a.get_value(r1, cols[1]);
        return aa * d - b * c;
    }
    if n == 3 {
        let (r0, r1, r2) = (rows[0], rows[1], rows[2]);
        let aa = a.get_value(r0, cols[0]);
        let b = a.get_value(r0, cols[1]);
        let c = a.get_value(r0, cols[2]);
        let d = a.get_value(r1, cols[0]);
        let e = a.get_value(r1, cols[1]);
        let f = a.get_value(r1, cols[2]);
        let g = a.get_value(r2, cols[0]);
        let h = a.get_value(r2, cols[1]);
        let i = a.get_value(r2, cols[2]);
        return aa * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    }
    let mut det = 0.0;
    let sub_rows = &rows[1..];
    for j in 0..n {
        let aa = a.get_value(rows[0], cols[j]);
        if aa == 0.0 {
            continue;
        }
        let sub_cols = remove_at(cols, j);
        let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
        det += aa * determinant_minor(a, sub_rows, &sub_cols) * sign;
    }
    det
}

/// `AdjointByMinors()`
fn adjoint_by_minors(a: &HpMat) -> CResult<Matrix> {
    let n = a.row_count;
    let mut result = HpMat::empty(
        MatrixType::Full,
        n,
        n,
        a.units.as_ref().and_then(|u| u.pow(n as f32 - 1.0)),
    );
    let full: Vec<usize> = (0..n).collect();
    for i in 0..n {
        result.rows[i] = HpData::new(n, n, a.units.clone());
    }
    for i in 0..n {
        let sub_rows = remove_at(&full, i);
        for j in 0..n {
            let sub_cols = remove_at(&full, j);
            let sign = if (i + j) & 1 == 0 { 1.0 } else { -1.0 };
            let det = determinant_minor(a, &sub_rows, &sub_cols);
            result.set_value(det * sign, j, i)?;
        }
    }
    Ok(result.into_matrix())
}

/// `Adjoint()` (virtual)
pub fn adjoint(m: &Matrix) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Diagonal => return diag::adjoint(m),
        MatrixType::Symmetric => return sym::adjoint(m),
        _ => {}
    }
    let a = HpMat::from(m);
    let lu = get_lu(m)?;
    let min_pivot = lu.as_ref().map(|x| x.2).unwrap_or(0.0);
    if min_pivot.abs() < 1e-15 && a.row_count < 11 {
        return adjoint_by_minors(&a);
    }
    let (lu, indexes, _, det) = lu.ok_or_else(exceptions::MatrixSingular)?;
    let determinant = get_determinant(&a, &lu).mul_f(det);
    let inv = get_inverse(&a, &lu, &indexes)?;
    super::mul_mr(&inv, &determinant)
}

/// `Cofactor()`
pub fn cofactor(m: &Matrix) -> CResult<Matrix> {
    adjoint(m)?.transpose()
}

/// `CholeskyDecomposition()` (HpSymmetricMatrix and HpDiagonalMatrix)
pub fn cholesky_decomposition(m: &Matrix) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Symmetric => sym::cholesky_decomposition(m),
        MatrixType::Diagonal => diag::cholesky_decomposition(m),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}

/// `EigenValues(int count, double tol)`
pub fn eigen_values(m: &Matrix, count: i64, tol: f64) -> CResult<Vector> {
    match m.mtype() {
        MatrixType::Symmetric => sym::eigen_values(m, count, tol),
        MatrixType::Diagonal => diag::eigen_values(m, count),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}

/// `EigenVectors(int count, double tol)`
pub fn eigen_vectors(m: &Matrix, count: i64, tol: f64) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Symmetric => sym::eigen_vectors(m, count, tol),
        MatrixType::Diagonal => diag::eigen_vectors(m, count),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}

/// `Eigen(int count, double tol)`
pub fn eigen(m: &Matrix, count: i64, tol: f64) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Symmetric => sym::eigen(m, count, tol),
        MatrixType::Diagonal => diag::eigen(m, count),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}

/// `ClSolve(HpVector v)` (HpSymmetricMatrix; HpDiagonalMatrix uses LSolve)
pub fn cl_solve(m: &Matrix, v: &Vector) -> CResult<Vector> {
    match m.mtype() {
        MatrixType::Symmetric => sym::cl_solve(m, v),
        MatrixType::Diagonal => diag::lsolve(m, v),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}

/// `CmSolve(HpMatrix M)` (HpSymmetricMatrix; HpDiagonalMatrix uses MSolve)
pub fn cm_solve(m: &Matrix, b: &Matrix) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Symmetric => sym::cm_solve(m, b),
        MatrixType::Diagonal => diag::msolve(m, b),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}

/// `SlSolve(HpVector b, double tol)` (HpSymmetricMatrix; HpDiagonalMatrix uses LSolve)
pub fn sl_solve(m: &Matrix, b: &Vector, tol: f64) -> CResult<Vector> {
    match m.mtype() {
        MatrixType::Symmetric => sym::sl_solve(m, b, tol),
        MatrixType::Diagonal => diag::lsolve(m, b),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}

/// `SmSolve(HpMatrix B, double tol)` (HpSymmetricMatrix; HpDiagonalMatrix uses MSolve)
pub fn sm_solve(m: &Matrix, b: &Matrix, tol: f64) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Symmetric => sym::sm_solve(m, b, tol),
        MatrixType::Diagonal => diag::msolve(m, b),
        _ => Err(exceptions::MatrixMustBeSymmetric()),
    }
}
