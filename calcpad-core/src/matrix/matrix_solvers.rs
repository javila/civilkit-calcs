//! The numerical members of Matrix/Matrix.cs and its subclasses: LU, QR and SVD
//! decompositions, solving, inverse, determinant, condition numbers, rank, adjoint,
//! cofactor, Cholesky and LDLT, and the eigenvalue solvers of the symmetric and diagonal
//! matrices.
//!
//! The C# virtual members are dispatched here on the matrix type: the base implementation
//! (Matrix.cs) serves full and column matrices, the overrides of DiagonalMatrix.cs,
//! LowerTriangularMatrix.cs, UpperTriangularMatrix.cs and SymmetricMatrix.cs the others.
//! `Parallel.For` loops run sequentially; they write disjoint results, so the order does
//! not change the values.
use super::{
    diagonal_matrix, hp_matrix, lower_triangular_matrix, symmetric_matrix, upper_triangular_matrix, Matrix, MatrixType,
    PARALLEL_THRESHOLD,
};
use crate::base_types::real_value::RealValue;
use crate::calculator::real_calculator;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::vector::{self, column_vector, Vector};

// ----- RealValue helpers (the C# operators) ----------------------------------------------------

#[inline]
fn mul(a: &RealValue, b: &RealValue) -> RealValue {
    RealValue::mul_op(a, b)
}

#[inline]
fn div(a: &RealValue, b: &RealValue) -> RealValue {
    RealValue::div_op(a, b)
}

#[inline]
fn add(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
    RealValue::add(a, b)
}

#[inline]
fn sub(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
    RealValue::sub(a, b)
}

#[inline]
fn abs(a: &RealValue) -> RealValue {
    real_calculator::abs(a)
}

#[inline]
fn compare(a: &RealValue, b: &RealValue) -> CResult<i32> {
    vector::real_compare_to(a, b)
}

/// `RealValue.Sqrt(RealValue)`
fn sqrt(value: &RealValue) -> RealValue {
    if let Some(u) = &value.units {
        if u.is_dimensionless() {
            return RealValue::num((value.d * u.get_dimensionless_factor()).sqrt());
        }
        return RealValue::new(value.d.sqrt(), u.pow(0.5));
    }
    RealValue::num(value.d.sqrt())
}

/// `Sqr(in RealValue x)`
fn sqr(x: &RealValue) -> RealValue {
    mul(x, x)
}

/// `CopySign(in RealValue a, in RealValue b)`
fn copy_sign(a: &RealValue, b: &RealValue) -> RealValue {
    if b.d >= 0.0 {
        abs(a)
    } else {
        abs(a).neg()
    }
}

/// `Hypot(in RealValue a, in RealValue b)`
fn hypot(a: &RealValue, b: &RealValue) -> CResult<RealValue> {
    let a1 = abs(a);
    let b1 = abs(b);
    if compare(&a1, &b1)? > 0 {
        Ok(mul(&a1, &sqrt(&add(&RealValue::ONE, &sqr(&div(&b1, &a1)))?)))
    } else if b1.d == 0.0 {
        Ok(b1)
    } else {
        Ok(mul(&b1, &sqrt(&add(&RealValue::ONE, &sqr(&div(&a1, &b1)))?)))
    }
}

fn not_symmetric() -> MathParserException {
    exceptions::MatrixMustBeSymmetric()
}

/// An index outside a .NET array.
fn outside() -> MathParserException {
    vector::index_was_outside()
}

/// `M[i, j] = value` with the bounds check of the .NET row array.
fn set_checked(m: &Matrix, i: usize, j: usize, value: RealValue) -> CResult<()> {
    if i >= m.rows_len() || j >= m.col_count() {
        return Err(outside());
    }
    m.set(i, j, value)
}

/// The result of `GetLU(out int[] indexes, out double minPivot, out double det)`.
struct Lu {
    lu: Option<Matrix>,
    indexes: Vec<i32>,
    min_pivot: f64,
    det: f64,
}

impl Matrix {
    // ----- LU decomposition ------------------------------------------------------------------

    /// `GetLU(...)` (protected virtual)
    fn get_lu(&self) -> CResult<Lu> {
        let n = self.row_count();
        match self.mtype() {
            MatrixType::Diagonal => {
                let diag = self.row_ref(0);
                let mut lu = Lu {
                    lu: None,
                    indexes: vec![0; n],
                    min_pivot: f64::MAX,
                    det: 1.0,
                };
                for i in 0..n {
                    if diag.get(i).d == 0.0 {
                        return Ok(lu);
                    }
                    lu.indexes[i] = i as i32;
                }
                lu.lu = Some(diagonal_matrix::raw_copy(self));
                Ok(lu)
            }
            MatrixType::LowerTriangular | MatrixType::UpperTriangular => {
                let lower = self.mtype() == MatrixType::LowerTriangular;
                let mut lu = Lu {
                    lu: None,
                    indexes: vec![0; n],
                    min_pivot: f64::MAX,
                    det: 1.0,
                };
                for i in 0..n {
                    let d = self.row_ref(i).get(if lower { i } else { 0 });
                    if d.d == 0.0 {
                        return Ok(lu);
                    }
                    lu.indexes[i] = i as i32;
                }
                lu.lu = Some(if lower {
                    lower_triangular_matrix::raw_copy(self)
                } else {
                    upper_triangular_matrix::raw_copy(self)
                });
                Ok(lu)
            }
            MatrixType::Symmetric => {
                let mut lu = Lu {
                    lu: None,
                    indexes: (0..n as i32).collect(),
                    min_pivot: f64::MAX,
                    det: 1.0,
                };
                if let Some(u) = self.get_ldlt()? {
                    let m = symmetric_matrix::new(n);
                    for i in 0..n {
                        m.set_row_ref(i, u.row_ref(i));
                    }
                    lu.lu = Some(m);
                }
                Ok(lu)
            }
            MatrixType::Full | MatrixType::Column => self.base_get_lu(),
        }
    }

    /// `Matrix.GetLU(...)`: Crout's LU decomposition with partial pivoting.
    fn base_get_lu(&self) -> CResult<Lu> {
        let n = self.row_count();
        if n != self.col_count() {
            return Err(exceptions::MatrixNotSquare());
        }
        let lu_m = Matrix::full(n, n);
        let mut vv = vec![RealValue::ZERO; n];
        let mut indexes: Vec<i32> = vec![0; n];
        let mut col_j = vec![RealValue::ZERO; n];
        let mut det = 1.0;
        for i in 0..n {
            indexes[i] = i as i32;
            lu_m.set_row_ref(i, self.row_ref(i).copy());
        }
        let mut min_pivot = 0.0;
        for i in 0..n {
            let row = lu_m.row_ref(i);
            let size = row.size();
            if size == 0 {
                return Ok(Lu {
                    lu: None,
                    indexes,
                    min_pivot,
                    det,
                });
            }
            let mut big = abs(&row.get(0));
            for j in 1..size {
                let temp = abs(&row.get(j));
                if compare(&temp, &big)? > 0 {
                    big = temp;
                }
            }
            if big.d == 0.0 {
                return Ok(Lu {
                    lu: None,
                    indexes,
                    min_pivot,
                    det,
                });
            }
            vv[i] = div(&RealValue::ONE, &big);
        }
        min_pivot = f64::MAX;
        let cols = self.col_count();
        for j in 0..cols {
            for (i, c) in col_j.iter_mut().enumerate() {
                *c = lu_m.get(i, j);
            }
            for i in 0..j {
                let row = lu_m.row_ref(i);
                let size = row.size().min(i);
                let mut sum = col_j[i].clone();
                let mut k0 = 0;
                while k0 < size && row.get(k0).d == 0.0 {
                    k0 += 1;
                }
                for k in k0..size {
                    sum = sub(&sum, &mul(&row.get(k), &col_j[k]))?;
                }
                col_j[i] = sum;
            }
            let mut big = RealValue::ZERO;
            let mut imax = j;
            for i in j..n {
                let row = lu_m.row_ref(i);
                let size = row.size().min(j);
                let mut sum = col_j[i].clone();
                let mut k0 = 0;
                while k0 < size && row.get(k0).d == 0.0 {
                    k0 += 1;
                }
                for k in k0..size {
                    sum = sub(&sum, &mul(&row.get(k), &col_j[k]))?;
                }
                let dum = mul(&vv[i], &abs(&sum));
                col_j[i] = sum;
                if compare(&dum, &big)? > 0 {
                    big = dum;
                    imax = i;
                }
            }
            let d = big.d.abs();
            if j != imax {
                if d < min_pivot {
                    min_pivot = d;
                }
                let rj = lu_m.row_ref(j);
                let ri = lu_m.row_ref(imax);
                lu_m.set_row_ref(j, ri);
                lu_m.set_row_ref(imax, rj);
                col_j.swap(j, imax);
                vv[imax] = vv[j].clone();
                indexes.swap(j, imax);
                det = -det;
            }
            if j != n - 1 {
                let dum = div(&RealValue::ONE, &col_j[j]);
                for c in col_j.iter_mut().skip(j + 1) {
                    *c = mul(c, &dum);
                }
            } else if d < min_pivot {
                min_pivot = d;
            }
            for (i, c) in col_j.iter().enumerate() {
                lu_m.set(i, j, c.clone())?;
            }
        }
        Ok(Lu {
            lu: if min_pivot == 0.0 { None } else { Some(lu_m) },
            indexes,
            min_pivot,
            det,
        })
    }

    /// `LUDecomposition(out int[] indexes)` (virtual)
    pub fn lu_decomposition(&self) -> CResult<(Matrix, Vec<i32>)> {
        if self.is_hp() {
            return hp_matrix::numerics::lu_decomposition(self);
        }
        let lu = self.get_lu()?;
        match lu.lu {
            Some(m) => Ok((m, lu.indexes)),
            None => Err(exceptions::MatrixSingular()),
        }
    }

    /// `FwdAndBackSubst(Matrix LU, int[] indexes, Vector v, RealValue[] x)`
    fn fwd_and_back_subst(lu: &Matrix, indexes: &[i32], v: &Vector, x: &mut [RealValue]) -> CResult<()> {
        let m = lu.row_count();
        let mut start: i64 = -1;
        for i in 0..m {
            let index = indexes[i] as usize;
            let mut sum = v.try_get(index)?;
            if start >= 0 {
                let row = lu.row_ref(i);
                let size = row.size().min(i);
                for j in start as usize..size {
                    sum = sub(&sum, &mul(&row.get(j), &x[j]))?;
                }
            } else if !sum.equals(&RealValue::ZERO) {
                start = i as i64;
            }
            x[i] = sum;
        }
        for i in (0..m).rev() {
            let mut sum = x[i].clone();
            let row = lu.row_ref(i);
            let n = row.size().min(m);
            for j in i + 1..n {
                sum = sub(&sum, &mul(&row.get(j), &x[j]))?;
            }
            x[i] = div(&sum, &lu.get(i, i));
        }
        Ok(())
    }

    /// `MSolveForLU(Matrix LU, int[] indexes, Matrix M)`
    fn msolve_for_lu(lu: &Matrix, indexes: &[i32], mm: &Matrix) -> CResult<Matrix> {
        let m = lu.row_count();
        let n = mm.col_count();
        let mut result = Vec::with_capacity(n);
        for j in 0..n {
            let mut x = vec![RealValue::ZERO; m];
            Matrix::fwd_and_back_subst(lu, indexes, &column_vector::new(mm, j), &mut x)?;
            result.push(Vector::from_values(x)?);
        }
        Matrix::create_from_cols(&result, m)
    }

    /// `MSolveForLU(Matrix LU, int[] indexes)`: the inverse, column by column.
    fn msolve_for_lu_identity(lu: &Matrix, indexes: &[i32]) -> CResult<Matrix> {
        let m = lu.row_count();
        let identity = Matrix::identity(m)?;
        let mut result = Vec::with_capacity(m);
        for j in 0..m {
            let mut x = vec![RealValue::ZERO; m];
            Matrix::fwd_and_back_subst(lu, indexes, &identity.row_ref(j), &mut x)?;
            result.push(Vector::from_values(x)?);
        }
        for j in (0..m).rev() {
            identity.set_col(j, &result[j])?;
        }
        Ok(identity)
    }

    /// `GetInverse(Matrix lu, int[] indexes)`
    fn get_inverse(&self, lu: &Matrix, indexes: &[i32]) -> CResult<Matrix> {
        let n = self.row_count();
        if n > PARALLEL_THRESHOLD {
            return Matrix::msolve_for_lu_identity(lu, indexes);
        }
        let vector = Vector::dense(n);
        let mut x = vec![RealValue::ZERO; n];
        let m = Matrix::new(n, n)?;
        for j in 0..n {
            vector.set(j, RealValue::ONE)?;
            Matrix::fwd_and_back_subst(lu, indexes, &vector, &mut x)?;
            for (i, xi) in x.iter().enumerate() {
                m.set(i, j, xi.clone())?;
            }
            vector.set(j, RealValue::ZERO)?;
        }
        Ok(m)
    }

    /// `GetDeterminant(Matrix lu)`
    fn get_determinant(&self, lu: &Matrix) -> CResult<RealValue> {
        if lu.rows_len() == 0 {
            return Err(outside());
        }
        let mut det = lu.row_ref(0).get(0);
        for i in 1..self.row_count() {
            det = mul(&det, &lu.row_ref(i).get(i));
        }
        Ok(det)
    }

    // ----- determinant, inverse and solving ----------------------------------------------------

    /// `Determinant()` (virtual)
    pub fn determinant(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::determinant(self);
        }
        let n = self.row_count();
        match self.mtype() {
            MatrixType::Diagonal => self.row_ref(0).product(),
            MatrixType::LowerTriangular | MatrixType::UpperTriangular => {
                if self.rows_len() == 0 {
                    return Err(outside());
                }
                let lower = self.mtype() == MatrixType::LowerTriangular;
                let mut det = self.row_ref(0).get(0);
                for i in 1..n {
                    det = mul(&det, &self.row_ref(i).get(if lower { i } else { 0 }));
                }
                Ok(det)
            }
            MatrixType::Symmetric => match self.get_ldlt()? {
                None => Ok(RealValue::ZERO),
                Some(u) => symmetric_determinant(&u, n),
            },
            MatrixType::Full | MatrixType::Column => {
                if n != self.col_count() {
                    return Err(exceptions::MatrixNotSquare());
                }
                let lu = self.get_lu()?;
                match &lu.lu {
                    None => Ok(RealValue::ZERO),
                    Some(m) => Ok(self.get_determinant(m)?.mul_f(lu.det)),
                }
            }
        }
    }

    /// `Invert()` (virtual)
    pub fn invert(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::invert(self);
        }
        let n = self.row_count();
        match self.mtype() {
            MatrixType::Diagonal => {
                self.diagonal_check_singular()?;
                let m = diagonal_matrix::new(n);
                m.set_row_ref(0, Vector::div_rv(&RealValue::ONE, &self.row_ref(0))?);
                Ok(m)
            }
            MatrixType::LowerTriangular => {
                let l = lower_triangular_matrix::new(n);
                for i in 0..n {
                    let row = self.row_ref(i);
                    let v = row.get(i);
                    if v.d == 0.0 {
                        return Err(exceptions::MatrixSingular());
                    }
                    let v = div(&RealValue::ONE, &v);
                    l.set(i, i, v.clone())?;
                    for j in 0..i {
                        let mut sum = mul(&row.get(j), &l.get(j, j));
                        for k in j + 1..i {
                            sum = add(&sum, &mul(&row.get(k), &l.get(k, j)))?;
                        }
                        l.set(i, j, mul(&sum.neg(), &v))?;
                    }
                }
                Ok(l)
            }
            MatrixType::UpperTriangular => {
                let u = upper_triangular_matrix::new(n);
                if n == 0 {
                    return Ok(u);
                }
                let nn = n - 1;
                for i in (0..=nn).rev() {
                    let row = self.row_ref(i);
                    let v = row.get(0);
                    if v.d == 0.0 {
                        return Err(exceptions::MatrixSingular());
                    }
                    let v = div(&RealValue::ONE, &v);
                    u.set(i, i, v.clone())?;
                    for j in (i + 1..=nn).rev() {
                        let mut sum = mul(&row.get(nn - i), &u.get(nn, j));
                        for k in (i + 1..nn).rev() {
                            sum = add(&sum, &mul(&row.get(k - i), &u.get(k, j)))?;
                        }
                        u.set(i, j, mul(&sum.neg(), &v))?;
                    }
                }
                Ok(u)
            }
            MatrixType::Symmetric => {
                let u = self.get_ldlt()?.ok_or_else(exceptions::MatrixSingular)?;
                self.symmetric_inverse(&u)
            }
            MatrixType::Full | MatrixType::Column => {
                let lu = self.get_lu()?;
                let m = lu.lu.as_ref().ok_or_else(exceptions::MatrixSingular)?;
                if lu.min_pivot < 1e-15 {
                    return Err(exceptions::MatrixCloseToSingular());
                }
                self.get_inverse(m, &lu.indexes)
            }
        }
    }

    /// `CheckSingular()` (DiagonalMatrix)
    fn diagonal_check_singular(&self) -> CResult<()> {
        let diag = self.row_ref(0);
        for i in (0..self.row_count()).rev() {
            if diag.get(i).d == 0.0 {
                return Err(exceptions::MatrixSingular());
            }
        }
        Ok(())
    }

    /// `LSolve` of the triangular matrices
    fn triangular_lsolve(&self, v: &Vector) -> CResult<Vector> {
        let n = self.row_count();
        let x = Vector::dense(n);
        if self.mtype() == MatrixType::LowerTriangular {
            for i in 0..n {
                let mut sum = v.try_get(i)?;
                let row = self.row_ref(i);
                for j in 0..i {
                    sum = sub(&sum, &mul(&row.get(j), &x.get(j)))?;
                }
                let ri = row.get(i);
                if ri.d == 0.0 {
                    return Err(exceptions::MatrixSingular());
                }
                x.set(i, div(&sum, &ri))?;
            }
        } else {
            for i in (0..n).rev() {
                let mut sum = v.try_get(i)?;
                let row = self.row_ref(i);
                for j in (i + 1..n).rev() {
                    sum = sub(&sum, &mul(&row.get(j - i), &x.get(j)))?;
                }
                x.set(i, div(&sum, &row.get(0)))?;
            }
        }
        Ok(x)
    }

    /// `LSolve(Vector v)` (virtual)
    pub fn lsolve(&self, v: &Vector) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::numerics::lsolve(self, v);
        }
        match self.mtype() {
            MatrixType::Diagonal => {
                self.diagonal_check_singular()?;
                Vector::div(v, &self.row_ref(0))
            }
            MatrixType::LowerTriangular | MatrixType::UpperTriangular => self.triangular_lsolve(v),
            MatrixType::Symmetric => {
                let u = self.get_ldlt()?.ok_or_else(exceptions::MatrixSingular)?;
                let mut x = vec![RealValue::ZERO; self.row_count()];
                symmetric_fwd_and_back_subst(&u, v, &mut x, false)?;
                Vector::from_values(x)
            }
            MatrixType::Full | MatrixType::Column => {
                let lu = self.get_lu()?;
                let m = lu.lu.as_ref().ok_or_else(exceptions::MatrixSingular)?;
                if lu.min_pivot < 1e-15 {
                    return Err(exceptions::MatrixCloseToSingular());
                }
                let mut x = vec![RealValue::ZERO; self.row_count()];
                Matrix::fwd_and_back_subst(m, &lu.indexes, v, &mut x)?;
                Vector::from_values(x)
            }
        }
    }

    /// `MSolve(Matrix M)` (virtual)
    pub fn msolve(&self, mm: &Matrix) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::msolve(self, mm);
        }
        let m = self.row_count();
        let n = mm.col_count();
        match self.mtype() {
            MatrixType::Diagonal => {
                self.diagonal_check_singular()?;
                let diag = self.row_ref(0);
                let mut v = Vec::with_capacity(n);
                for j in 0..n {
                    v.push(Vector::div(&mm.col(j as i64 + 1)?, &diag)?);
                }
                Matrix::create_from_cols(&v, m)
            }
            MatrixType::LowerTriangular | MatrixType::UpperTriangular => {
                let mut v = Vec::with_capacity(n);
                for j in 0..n {
                    v.push(self.triangular_lsolve(&mm.col(j as i64 + 1)?)?);
                }
                Matrix::create_from_cols(&v, m)
            }
            MatrixType::Symmetric => {
                let u = self.get_ldlt()?.ok_or_else(exceptions::MatrixSingular)?;
                symmetric_msolve_for_u(&u, mm, false)
            }
            MatrixType::Full | MatrixType::Column => {
                let lu = self.get_lu()?;
                let lm = lu.lu.as_ref().ok_or_else(exceptions::MatrixSingular)?;
                if lu.min_pivot < 1e-15 {
                    return Err(exceptions::MatrixCloseToSingular());
                }
                Matrix::msolve_for_lu(lm, &lu.indexes, mm)
            }
        }
    }

    /// `ClSolve(Vector v)` (SymmetricMatrix; MatrixCalculator uses LSolve for a diagonal
    /// matrix)
    pub fn cl_solve(&self, v: &Vector) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::numerics::cl_solve(self, v);
        }
        match self.mtype() {
            MatrixType::Symmetric => {
                let u = self.get_cholesky()?.ok_or_else(exceptions::MatrixNotPositiveDefinite)?;
                let mut x = vec![RealValue::ZERO; self.row_count()];
                symmetric_fwd_and_back_subst(&u, v, &mut x, true)?;
                Vector::from_values(x)
            }
            MatrixType::Diagonal => self.lsolve(v),
            _ => Err(not_symmetric()),
        }
    }

    /// `CmSolve(Matrix M)` (SymmetricMatrix; MatrixCalculator uses MSolve for a diagonal
    /// matrix)
    pub fn cm_solve(&self, mm: &Matrix) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::cm_solve(self, mm);
        }
        match self.mtype() {
            MatrixType::Symmetric => {
                let u = self.get_cholesky()?.ok_or_else(exceptions::MatrixNotPositiveDefinite)?;
                symmetric_msolve_for_u(&u, mm, true)
            }
            MatrixType::Diagonal => self.msolve(mm),
            _ => Err(not_symmetric()),
        }
    }

    // ----- QR and SVD --------------------------------------------------------------------------

    /// `GetQR(out Matrix Q, out Matrix R)`: Householder reflections.
    fn get_qr(&self) -> CResult<(Matrix, Matrix)> {
        let m = self.row_count();
        let n = self.col_count();
        let r_m = self.copy()?;
        let q = Matrix::identity(m)?;
        let mut v = vec![RealValue::ZERO; m];
        let mn = m.min(n);
        for k in 0..mn {
            let mut r = r_m.get(k, k);
            let mut norm_x = mul(&r, &r);
            for i in k + 1..m {
                r = r_m.get(i, k);
                norm_x = add(&norm_x, &mul(&r, &r))?;
            }
            norm_x = sqrt(&norm_x);
            if norm_x.d == 0.0 {
                continue;
            }
            r = r_m.get(k, k);
            let alpha = if r.d > 0.0 { norm_x.neg() } else { norm_x };
            let rs = sqrt(&mul(&alpha, &sub(&alpha, &r)?).mul_f(0.5)).mul_f(2.0);
            v[k] = div(&sub(&r, &alpha)?, &rs);
            for i in k + 1..m {
                v[i] = div(&r_m.get(i, k), &rs);
            }
            for j in k..n {
                let mut dot = mul(&v[k], &r_m.get(k, j));
                for i in k + 1..m {
                    dot = add(&dot, &mul(&v[i], &r_m.get(i, j)))?;
                }
                r = dot.mul_f(2.0);
                for i in k..m {
                    r_m.set(i, j, sub(&r_m.get(i, j), &mul(&r, &v[i]))?)?;
                }
            }
            for i in 0..m {
                let q_i = q.row_ref(i);
                let size = q_i.size();
                let mut dot = mul(&q_i.get(k), &v[k]);
                for j in k + 1..size {
                    dot = add(&dot, &mul(&q_i.get(j), &v[j]))?;
                }
                r = dot.mul_f(2.0);
                for j in (k..m).rev() {
                    q_i.set(j, sub(&q_i.get(j), &mul(&r, &v[j]))?)?;
                }
            }
        }
        Ok((q, r_m))
    }

    /// `QRDecomposition()` (virtual)
    pub fn qr_decomposition(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::qr_decomposition(self);
        }
        let (q, r) = self.get_qr()?;
        Matrix::augment(&[q, r])
    }

    /// `GetSVD(out Matrix U, out Vector sig, out Matrix V)`
    fn get_svd(&self) -> CResult<(Matrix, Vector, Matrix)> {
        let rows = self.row_count();
        let cols = self.col_count();
        if rows < cols {
            return Err(exceptions::MatrixNotHigh());
        }
        if cols == 0 {
            return Err(outside());
        }
        let m = rows - 1;
        let n = cols - 1;
        let rv1 = Vector::dense(cols);
        let u = self.copy()?;
        let sig = Vector::dense(cols);
        let v = Matrix::new(cols, cols)?;
        let anorm = svd_bidiagonalize(&u, &sig, &rv1, m, n)?;
        svd_right_transform(&u, &v, &rv1, n)?;
        svd_left_transform(&u, &sig, m, n)?;
        svd_diagonalize(&u, &sig, &v, &rv1, m, n, &anorm)?;
        Ok((u, sig, v))
    }

    /// `SVDDecomposition()` (virtual)
    pub fn svd_decomposition(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::svd_decomposition(self);
        }
        let (mut u, mut sig, mut v) = self.get_svd()?;
        let indexes = sig.get_order_indexes(true, None)?;
        let is_sorted = indexes.iter().enumerate().all(|(i, &k)| k == i as i32);
        if !is_sorted {
            sig = sig.sort(true)?;
            u = u.reorder_cols(&indexes)?;
            v = v.reorder_rows(&indexes)?;
        }
        Matrix::augment(&[u, Matrix::from_vector(&sig)?, v])
    }

    // ----- norms, condition and rank -------------------------------------------------------------

    /// `L2Norm()` (virtual)
    pub fn l2_norm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::l2_norm(self);
        }
        let (_, sig, _) = self.get_svd()?;
        sig.max()
    }

    /// `Condition(Func<Matrix, RealValue> norm)` (protected virtual)
    fn condition(&self, norm: &dyn Fn(&Matrix) -> CResult<RealValue>) -> CResult<RealValue> {
        match self.mtype() {
            MatrixType::Diagonal => {
                let det = self.determinant()?;
                if det.d == 0.0 {
                    return Ok(RealValue::POSITIVE_INFINITY);
                }
                let m = self.invert()?;
                Ok(mul(&norm(self)?, &norm(&m)?))
            }
            MatrixType::Symmetric => match self.get_ldlt()? {
                None => Ok(RealValue::POSITIVE_INFINITY),
                Some(u) => {
                    let m = self.symmetric_inverse(&u)?;
                    Ok(mul(&norm(self)?, &norm(&m)?))
                }
            },
            _ => {
                if self.row_count() != self.col_count() {
                    return Err(exceptions::MatrixNotSquare());
                }
                let lu = self.get_lu()?;
                match &lu.lu {
                    None => Ok(RealValue::POSITIVE_INFINITY),
                    Some(lm) => {
                        let m = self.get_inverse(lm, &lu.indexes)?;
                        Ok(mul(&norm(self)?, &norm(&m)?))
                    }
                }
            }
        }
    }

    /// `Cond1()` (virtual)
    pub fn cond1(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond1(self);
        }
        self.condition(&|m| m.l1_norm())
    }

    /// `Cond2()` (virtual)
    pub fn cond2(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond2(self);
        }
        let (_, sig, _) = self.get_svd()?;
        Ok(div(&sig.max()?, &sig.min()?))
    }

    /// `CondFrob()` (virtual)
    pub fn cond_frob(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond_frob(self);
        }
        self.condition(&|m| m.frob_norm())
    }

    /// `CondInf()` (virtual)
    pub fn cond_inf(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond_inf(self);
        }
        self.condition(&|m| m.inf_norm())
    }

    /// `Rank()` (virtual)
    pub fn rank(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::rank(self);
        }
        let (_, sig, _) = self.get_svd()?;
        let sig_max = sig.max()?.d;
        let eps = sig_max.next_up() - sig_max;
        let mut rank = 0;
        for i in 0..sig.len() {
            if sig.get(i).d.abs() > eps {
                rank += 1;
            }
        }
        Ok(RealValue::num(rank as f64))
    }

    // ----- adjoint and cofactor ------------------------------------------------------------------

    /// `Adjoint()` (virtual)
    pub fn adjoint(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::adjoint(self);
        }
        match self.mtype() {
            MatrixType::Diagonal => {
                let det = self.determinant()?;
                if det.d == 0.0 {
                    return Err(exceptions::MatrixSingular());
                }
                let m = self.invert()?;
                Matrix::mul_mr(&m, &det)
            }
            MatrixType::Symmetric => {
                let u = self.get_ldlt()?.ok_or_else(exceptions::MatrixSingular)?;
                let det = symmetric_determinant(&u, self.row_count())?;
                let m = self.symmetric_inverse(&u)?;
                Matrix::mul_mr(&m, &det)
            }
            _ => {
                let lu = self.get_lu()?;
                if lu.min_pivot.abs() < 1e-15 && self.row_count() < 11 {
                    return self.adjoint_by_minors();
                }
                let lm = lu.lu.as_ref().ok_or_else(exceptions::MatrixSingular)?;
                let determinant = self.get_determinant(lm)?.mul_f(lu.det);
                let m = self.get_inverse(lm, &lu.indexes)?;
                Matrix::mul_mr(&m, &determinant)
            }
        }
    }

    /// `AdjointByMinors()`
    fn adjoint_by_minors(&self) -> CResult<Matrix> {
        let n = self.row_count();
        let result = Matrix::new(n, n)?;
        let full: Vec<usize> = (0..n).collect();
        for i in 0..n {
            result.set_row_ref(i, crate::vector::large_vector::new_ls(n, n));
        }
        for i in 0..n {
            let sub_rows = remove_at(&full, i);
            for j in 0..n {
                let sub_cols = remove_at(&full, j);
                let sign = if (i + j) & 1 == 0 { 1.0 } else { -1.0 };
                let det = self.minor_determinant(&sub_rows, &sub_cols)?;
                result.set(j, i, det.mul_f(sign))?;
            }
        }
        Ok(result)
    }

    /// `Determinant(Span<int> rows, Span<int> cols)`: Laplace expansion.
    fn minor_determinant(&self, rows: &[usize], cols: &[usize]) -> CResult<RealValue> {
        let n = rows.len();
        if n == 0 {
            return Err(outside());
        }
        if n == 1 {
            return Ok(self.get(rows[0], cols[0]));
        }
        let at = |i: usize, j: usize| self.get(rows[i], cols[j]);
        if n == 2 {
            return sub(&mul(&at(0, 0), &at(1, 1)), &mul(&at(0, 1), &at(1, 0)));
        }
        if n == 3 {
            let (a, b, c) = (at(0, 0), at(0, 1), at(0, 2));
            let (d, e, f) = (at(1, 0), at(1, 1), at(1, 2));
            let (g, h, i) = (at(2, 0), at(2, 1), at(2, 2));
            let t1 = mul(&a, &sub(&mul(&e, &i), &mul(&f, &h))?);
            let t2 = mul(&b, &sub(&mul(&d, &i), &mul(&f, &g))?);
            let t3 = mul(&c, &sub(&mul(&d, &h), &mul(&e, &g))?);
            return add(&sub(&t1, &t2)?, &t3);
        }
        let mut det = RealValue::ZERO;
        let sub_rows = &rows[1..];
        for j in 0..n {
            let a = self.get(rows[0], cols[j]);
            if a.d == 0.0 {
                continue;
            }
            let sub_cols = remove_at(cols, j);
            let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
            det = add(
                &det,
                &mul(&a, &self.minor_determinant(sub_rows, &sub_cols)?).mul_f(sign),
            )?;
        }
        Ok(det)
    }

    /// `Cofactor()` (virtual)
    pub fn cofactor(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::cofactor(self);
        }
        self.adjoint()?.transpose()
    }

    // ----- symmetric and diagonal members --------------------------------------------------------

    /// `GetCholesky()` (SymmetricMatrix)
    fn get_cholesky(&self) -> CResult<Option<Matrix>> {
        let n = self.row_count();
        let u = upper_triangular_matrix::new(n);
        let mut maxsize: i64 = 1;
        for i in 0..n {
            let row = self.row_ref(i);
            maxsize -= 1;
            if row.size() as i64 > maxsize {
                maxsize = row.size() as i64;
            }
            let mut sum = row.get(0);
            for k in 0..i {
                let u_ki = u.get(k, i);
                sum = sub(&sum, &mul(&u_ki, &u_ki))?;
            }
            if sum.d <= 0.0 {
                return Ok(None);
            }
            let u_i = u.row_ref(i);
            let mut nn = (n - i) as i64;
            if nn > maxsize {
                nn = maxsize;
            }
            let u_ii = RealValue::new(sum.d.sqrt(), sum.units.as_ref().and_then(|u| u.pow(0.5)));
            for j in 1..nn.max(1) as usize {
                sum = row.get(j);
                for k in 0..i {
                    let u_k = u.row_ref(k);
                    let ik = i - k;
                    sum = sub(&sum, &mul(&u_k.get(ik), &u_k.get(ik + j)))?;
                }
                u_i.set(j, div(&sum, &u_ii))?;
            }
            u_i.set(0, u_ii)?;
        }
        Ok(Some(u))
    }

    /// `GetLDLT()` (SymmetricMatrix): L^T is stored in U and D in its main diagonal.
    fn get_ldlt(&self) -> CResult<Option<Matrix>> {
        let n = self.row_count();
        let u = upper_triangular_matrix::new(n);
        let mut maxsize: i64 = 0;
        for i in 0..n {
            let row = self.row_ref(i);
            maxsize -= 1;
            if row.size() as i64 > maxsize {
                maxsize = row.size() as i64;
            }
            let mut sum = row.get(0);
            for k in 0..i {
                let u_ki = u.get(k, i);
                sum = sub(&sum, &mul(&mul(&u_ki, &u_ki), &u.get(k, k)))?;
            }
            if sum.d == 0.0 {
                return Ok(None);
            }
            let u_i = u.row_ref(i);
            let mut nn = (n - i) as i64;
            if nn > maxsize {
                nn = maxsize;
            }
            let u_ii = sum.clone();
            for j in 1..nn.max(1) as usize {
                sum = row.get(j);
                for k in 0..i {
                    let u_k = u.row_ref(k);
                    let ik = i - k;
                    sum = sub(&sum, &mul(&mul(&u_k.get(ik), &u_k.get(ik + j)), &u.get(k, k)))?;
                }
                u_i.set(j, div(&sum, &u_ii))?;
            }
            u_i.set(0, u_ii)?;
        }
        Ok(Some(u))
    }

    /// `GetInverse(UpperTriangularMatrix U)` (SymmetricMatrix)
    fn symmetric_inverse(&self, u: &Matrix) -> CResult<Matrix> {
        let n = self.row_count();
        if n > PARALLEL_THRESHOLD {
            return symmetric_msolve_for_u_identity(u);
        }
        let vector = Vector::dense(n);
        let mut x = vec![RealValue::ZERO; n];
        let m = symmetric_matrix::new(n);
        for j in 0..n {
            vector.set(j, RealValue::ONE)?;
            symmetric_fwd_and_back_subst(u, &vector, &mut x, false)?;
            for (i, xi) in x.iter().enumerate().skip(j) {
                m.set(i, j, xi.clone())?;
            }
            vector.set(j, RealValue::ZERO)?;
        }
        Ok(m)
    }

    /// `CholeskyDecomposition()` (SymmetricMatrix and DiagonalMatrix)
    pub fn cholesky_decomposition(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::cholesky_decomposition(self);
        }
        match self.mtype() {
            MatrixType::Symmetric => self.get_cholesky()?.ok_or_else(exceptions::MatrixNotPositiveDefinite),
            MatrixType::Diagonal => {
                let n = self.row_count();
                let u = upper_triangular_matrix::new(n);
                let row = self.row_ref(0);
                for i in 0..n {
                    u.set(i, i, row.get(i))?;
                }
                Ok(u)
            }
            _ => Err(not_symmetric()),
        }
    }

    /// `EigenValues(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen_values(&self, count: i64, tol: f64) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::numerics::eigen_values(self, count, tol);
        }
        let mut count = count;
        let reverse = self.check_count(&mut count);
        match self.mtype() {
            MatrixType::Diagonal => self.row_ref(0).sort(reverse)?.first(count as usize),
            MatrixType::Symmetric => {
                let (mut d, mut e, mut q) = self.tridiagonalize(false)?;
                implicit_ql(&mut d, &mut e, &mut q, false)?;
                sort_eigen_values(d, count, reverse)
            }
            _ => Err(not_symmetric()),
        }
    }

    /// `EigenVectors(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen_vectors(&self, count: i64, tol: f64) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::eigen_vectors(self, count, tol);
        }
        let mut count = count;
        let reverse = self.check_count(&mut count);
        match self.mtype() {
            MatrixType::Diagonal => {
                let n = self.row_count();
                let indexes = self.row_ref(0).get_order_indexes(false, None)?;
                let m = Matrix::new(count as usize, n)?;
                for i in 0..count as usize {
                    let k = *indexes.get(i).ok_or_else(outside)? as usize;
                    set_checked(&m, k, i, RealValue::ONE)?;
                }
                Ok(m)
            }
            MatrixType::Symmetric => {
                let (mut d, mut e, mut q) = self.tridiagonalize(true)?;
                implicit_ql(&mut d, &mut e, &mut q, true)?;
                sort_eigen_vectors(d, &q, count, reverse, false)
            }
            _ => Err(not_symmetric()),
        }
    }

    /// `Eigen(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen(&self, count: i64, tol: f64) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::eigen(self, count, tol);
        }
        let mut count = count;
        let reverse = self.check_count(&mut count);
        match self.mtype() {
            MatrixType::Diagonal => {
                let n = self.row_count();
                let diag = self.row_ref(0);
                let indexes = diag.get_order_indexes(reverse, None)?;
                let m = Matrix::new(count as usize, n + 1)?;
                for i in 0..count as usize {
                    let k = *indexes.get(i).ok_or_else(outside)? as usize;
                    set_checked(&m, k, 0, diag.get(i))?;
                    set_checked(&m, i, k + 1, RealValue::ONE)?;
                }
                Ok(m)
            }
            MatrixType::Symmetric => {
                let (mut d, mut e, mut q) = self.tridiagonalize(true)?;
                implicit_ql(&mut d, &mut e, &mut q, true)?;
                sort_eigen_vectors(d, &q, count, reverse, true)
            }
            _ => Err(not_symmetric()),
        }
    }

    /// `Tridiagonalize(out RealValue[] d, out RealValue[] e, bool eigenvecs)`: Householder
    /// reduction of a symmetric matrix (Numerical Recipes, 3rd edition).
    #[allow(clippy::type_complexity)]
    fn tridiagonalize(&self, eigenvecs: bool) -> CResult<(Vec<RealValue>, Vec<RealValue>, Vec<Vec<RealValue>>)> {
        let len = self.row_count();
        let mut d = vec![RealValue::ZERO; len];
        let mut e = vec![RealValue::ZERO; len];
        let mut q = vec![vec![RealValue::ZERO; len]; len];
        if len == 0 {
            return Ok((d, e, q));
        }
        let n = len - 1;
        let mut min = vec![0usize; len];
        for i in 0..=n {
            let row = self.row_ref(i);
            let size = row.size();
            for j in (0..size).rev() {
                let k = i + j;
                if i < min[k] {
                    min[k] = i;
                }
                q[k][i] = row.get(j);
            }
        }
        for i in (1..=n).rev() {
            let l = i - 1;
            let mut h = RealValue::ZERO;
            let m = min[i];
            if l > 0 {
                let mut scale = abs(&q[i][m]);
                for k in m + 1..=l {
                    scale = add(&scale, &abs(&q[i][k]))?;
                }
                if scale.d == 0.0 {
                    e[i] = q[i][l].clone();
                } else {
                    q[i][m] = div(&q[i][m], &scale);
                    h = sqr(&q[i][m]);
                    for k in m + 1..=l {
                        q[i][k] = div(&q[i][k], &scale);
                        h = add(&h, &sqr(&q[i][k]))?;
                    }
                    let mut f = q[i][l].clone();
                    let mut g = if f.d >= 0.0 { sqrt(&h).neg() } else { sqrt(&h) };
                    e[i] = mul(&scale, &g);
                    h = sub(&h, &mul(&f, &g))?;
                    q[i][l] = sub(&f, &g)?;
                    for j in m..=l {
                        if eigenvecs {
                            q[j][i] = div(&q[i][j], &h);
                        }
                        g = mul(&q[j][0], &q[i][0]);
                        for k in 1..=j {
                            g = add(&g, &mul(&q[j][k], &q[i][k]))?;
                        }
                        for k in j + 1..=l {
                            g = add(&g, &mul(&q[k][j], &q[i][k]))?;
                        }
                        e[j] = div(&g, &h);
                        if j == m {
                            f = mul(&e[j], &q[i][j]);
                        } else {
                            f = add(&f, &mul(&e[j], &q[i][j]))?;
                        }
                    }
                    let hh = div(&f, &add(&h, &h)?);
                    for j in m..=l {
                        f = q[i][j].clone();
                        g = sub(&e[j], &mul(&hh, &f))?;
                        e[j] = g.clone();
                        for k in m..=j {
                            let t = add(&mul(&f, &e[k]), &mul(&g, &q[i][k]))?;
                            q[j][k] = sub(&q[j][k], &t)?;
                        }
                    }
                }
            } else {
                e[i] = q[i][l].clone();
            }
            d[i] = h;
        }
        if eigenvecs {
            d[0] = RealValue::ZERO;
        }
        e[0] = RealValue::ZERO;
        if eigenvecs {
            for i in 0..=n {
                let m = min[i];
                if d[i].d != 0.0 && i > 0 {
                    let l = i - 1;
                    for j in m..=l {
                        let mut g = mul(&q[i][m], &q[m][j]);
                        for k in m + 1..=l {
                            g = add(&g, &mul(&q[i][k], &q[k][j]))?;
                        }
                        for k in 0..=l {
                            let t = mul(&g, &q[k][i]);
                            q[k][j] = sub(&q[k][j], &t)?;
                        }
                    }
                }
                d[i] = q[i][i].clone();
                q[i][i] = RealValue::ONE;
                for j in 0..i {
                    q[j][i] = RealValue::ZERO;
                    q[i][j] = RealValue::ZERO;
                }
            }
        } else {
            for i in 0..=n {
                d[i] = q[i][i].clone();
            }
        }
        Ok((d, e, q))
    }
}

/// `RemoveAt(ReadOnlySpan<int> source, int index, Span<int> target)`
fn remove_at(source: &[usize], index: usize) -> Vec<usize> {
    let mut v = Vec::with_capacity(source.len().saturating_sub(1));
    v.extend_from_slice(&source[..index]);
    if index + 1 < source.len() {
        v.extend_from_slice(&source[index + 1..]);
    }
    v
}

/// `GetDeterminant(UpperTriangularMatrix U)` (SymmetricMatrix)
fn symmetric_determinant(u: &Matrix, n: usize) -> CResult<RealValue> {
    if n == 0 {
        return Err(outside());
    }
    let mut det = u.get(0, 0);
    for i in 1..n {
        det = mul(&det, &u.get(i, i));
    }
    Ok(det)
}

/// `FwdAndBackSubst(UpperTriangularMatrix U, Vector v, RealValue[] x, bool Cholesky)`
/// (SymmetricMatrix)
fn symmetric_fwd_and_back_subst(u: &Matrix, v: &Vector, x: &mut [RealValue], cholesky: bool) -> CResult<()> {
    let m = u.row_count();
    let mut start: i64 = -1;
    for i in 0..m {
        let mut sum = v.try_get(i)?;
        if start >= 0 {
            for j in start as usize..i {
                sum = sub(&sum, &mul(&u.get(j, i), &x[j]))?;
            }
        } else if !sum.equals(&RealValue::ZERO) {
            start = i as i64;
        }
        x[i] = if cholesky { div(&sum, &u.get(i, i)) } else { sum };
    }
    if !cholesky {
        for (i, xi) in x.iter_mut().enumerate().take(m) {
            *xi = div(xi, &u.get(i, i));
        }
    }
    for i in (0..m).rev() {
        let mut sum = x[i].clone();
        let row = u.row_ref(i);
        let n = (m - i).min(row.size());
        for j in 1..n {
            sum = sub(&sum, &mul(&row.get(j), &x[i + j]))?;
        }
        x[i] = if cholesky { div(&sum, &row.get(0)) } else { sum };
    }
    Ok(())
}

/// `MSolveForU(UpperTriangularMatrix U, Matrix M, bool Cholesky)` (SymmetricMatrix)
fn symmetric_msolve_for_u(u: &Matrix, mm: &Matrix, cholesky: bool) -> CResult<Matrix> {
    let m = u.row_count();
    let n = mm.col_count();
    let mut result = Vec::with_capacity(n);
    for j in 0..n {
        let mut x = vec![RealValue::ZERO; m];
        symmetric_fwd_and_back_subst(u, &column_vector::new(mm, j), &mut x, cholesky)?;
        result.push(Vector::from_values(x)?);
    }
    Matrix::create_from_cols(&result, m)
}

/// `MSolveForU(UpperTriangularMatrix U)` (SymmetricMatrix): used for the inverse of large
/// matrices. As in C#, the right hand sides are the stored rows of a symmetric identity
/// matrix.
fn symmetric_msolve_for_u_identity(u: &Matrix) -> CResult<Matrix> {
    let m = u.row_count();
    let identity = symmetric_matrix::new(m);
    for i in 0..m {
        identity.set(i, i, RealValue::ONE)?;
    }
    let mut result = Vec::with_capacity(m);
    for j in 0..m {
        let mut x = vec![RealValue::ZERO; m];
        symmetric_fwd_and_back_subst(u, &identity.row_ref(j), &mut x, false)?;
        result.push(Vector::from_values(x)?);
    }
    for j in (0..m).rev() {
        let m_j = identity.row_ref(j);
        let col = &result[j];
        for i in (j..m).rev() {
            m_j.set(i - j, col.get(i - j))?;
        }
    }
    Ok(identity)
}

/// `ImplicitQL(RealValue[] d, RealValue[] e, RealValue[][] Q, bool eigenvecs)`: the QL
/// algorithm with implicit shifts (Numerical Recipes, 3rd edition).
fn implicit_ql(d: &mut [RealValue], e: &mut [RealValue], q: &mut [Vec<RealValue>], eigenvecs: bool) -> CResult<()> {
    if d.is_empty() {
        return Ok(());
    }
    let n = d.len() - 1;
    for i in 0..n {
        e[i] = e[i + 1].clone();
    }
    e[n] = RealValue::ZERO;
    for l in 0..=n {
        let mut iter = 0;
        loop {
            let mut m = l;
            while m < n {
                let dd = add(&abs(&d[m]), &abs(&d[m + 1]))?;
                if dd.equals(&add(&abs(&e[m]), &dd)?) {
                    break;
                }
                m += 1;
            }
            if m != l {
                iter += 1;
                if iter == 30 {
                    return Err(exceptions::QLAlgorithmFailed());
                }
                let l1 = l + 1;
                let mut g = div(&sub(&d[l1], &d[l])?, &e[l].mul_f(2.0));
                let mut r = hypot(&g, &RealValue::ONE)?;
                g = add(&sub(&d[m], &d[l])?, &div(&e[l], &add(&g, &copy_sign(&r, &g))?))?;
                let mut s = RealValue::ONE;
                let mut c = s.clone();
                let mut p = RealValue::ZERO;
                let mut i = m as i64 - 1;
                while i >= l as i64 {
                    let iu = i as usize;
                    let i1 = iu + 1;
                    let mut f = mul(&s, &e[iu]);
                    let b = mul(&c, &e[iu]);
                    r = hypot(&f, &g)?;
                    e[i1] = r.clone();
                    if r.d == 0.0 {
                        d[i1] = sub(&d[i1], &p)?;
                        e[m] = RealValue::ZERO;
                        break;
                    }
                    s = div(&f, &r);
                    c = div(&g, &r);
                    g = if p.d == 0.0 { d[i1].clone() } else { sub(&d[i1], &p)? };
                    r = add(&mul(&sub(&d[iu], &g)?, &s), &mul(&c, &b).mul_f(2.0))?;
                    p = mul(&s, &r);
                    d[i1] = add(&g, &p)?;
                    g = sub(&mul(&c, &r), &b)?;
                    if eigenvecs {
                        for q_k in q.iter_mut().take(n + 1) {
                            f = q_k[i1].clone();
                            q_k[i1] = add(&mul(&s, &q_k[iu]), &mul(&c, &f))?;
                            q_k[iu] = sub(&mul(&c, &q_k[iu]), &mul(&s, &f))?;
                        }
                    }
                    i -= 1;
                }
                if r.d == 0.0 && i >= l as i64 {
                    continue;
                }
                d[l] = sub(&d[l], &p)?;
                e[l] = g;
                e[m] = RealValue::ZERO;
            }
            if m == l {
                break;
            }
        }
    }
    Ok(())
}

/// `SortEigenValues(RealValue[] d, int count, bool reverse)`
fn sort_eigen_values(mut d: Vec<RealValue>, count: i64, reverse: bool) -> CResult<Vector> {
    if reverse {
        crate::sort::sort_by::<RealValue, ()>(&mut d, None, &mut |x, y| Ok(-vector::real_compare_to(x, y)?))?;
    } else {
        crate::sort::sort_default::<RealValue, ()>(&mut d, None, &mut vector::real_compare_to)?;
    }
    let count = count as usize;
    if count > d.len() {
        return Err(MathParserException::unexpected(
            "Specified argument was out of the range of valid values.",
        ));
    }
    d.truncate(count);
    Vector::from_values(d)
}

/// `SortEigenVectors(RealValue[] d, RealValue[][] Q, int count, bool reverse, bool combine)`
fn sort_eigen_vectors(
    mut d: Vec<RealValue>,
    q: &[Vec<RealValue>],
    count: i64,
    reverse: bool,
    combine: bool,
) -> CResult<Matrix> {
    let len = d.len();
    let count = (count as usize).min(len);
    let mut indexes: Vec<i32> = (0..len as i32).collect();
    crate::sort::sort_default(&mut d, Some(&mut indexes), &mut vector::real_compare_to)?;
    let j0 = if combine { 1 } else { 0 };
    let n = q.len();
    let m = Matrix::new(count, n + j0)?;
    for i in 0..count {
        let index = if reverse { indexes[len - 1 - i] } else { indexes[i] } as usize;
        let row = m.row_ref(i);
        for j in (0..n).rev() {
            row.set(j + j0, q[j][index].clone())?;
        }
        if combine {
            // C# writes d[i] (not d[index]) here.
            row.set(0, d[i].clone())?;
        }
    }
    Ok(m)
}

// ----- SVD steps (Numerical Recipes svdcmp) ---------------------------------------------------

/// `SvdBidiagonalize(...)`: Householder reduction to bidiagonal form; returns `anorm`.
fn svd_bidiagonalize(u: &Matrix, sig: &Vector, rv1: &Vector, m: usize, n: usize) -> CResult<RealValue> {
    let eps = 1.0 / f64::MAX;
    let mut scale = RealValue::ZERO;
    let mut g = RealValue::ZERO;
    let mut anorm = RealValue::ZERO;
    let mut u_col_i = vec![RealValue::ZERO; m + 1];
    for i in 0..=n {
        let l = i + 1;
        rv1.set(i, mul(&scale, &g))?;
        if i <= m {
            for k in i..=m {
                u_col_i[k] = u.get(k, i);
            }
            scale = abs(&u_col_i[i]);
            for k in l..=m {
                scale = add(&scale, &abs(&u_col_i[k]))?;
            }
            if scale.d.abs() > eps {
                let mut u_ii = div(&u_col_i[i], &scale);
                let mut sum = mul(&u_ii, &u_ii);
                u_col_i[i] = u_ii.clone();
                for k in l..=m {
                    u_col_i[k] = div(&u_col_i[k], &scale);
                    sum = add(&sum, &sqr(&u_col_i[k]))?;
                }
                let mut f = u_ii.clone();
                g = copy_sign(&sqrt(&sum), &f).neg();
                let h = sub(&mul(&f, &g), &sum)?;
                u_ii = sub(&f, &g)?;
                u_col_i[i] = u_ii.clone();
                for j in l..=n {
                    sum = mul(&u_ii, &u.get(i, j));
                    for k in l..=m {
                        sum = add(&sum, &mul(&u_col_i[k], &u.get(k, j)))?;
                    }
                    f = div(&sum, &h);
                    for k in i..=m {
                        u.set(k, j, add(&u.get(k, j), &mul(&f, &u_col_i[k]))?)?;
                    }
                }
                for c in u_col_i.iter_mut().take(m + 1).skip(i) {
                    *c = mul(c, &scale);
                }
            }
            for k in i..=m {
                u.set(k, i, u_col_i[k].clone())?;
            }
        }
        sig.set(i, mul(&scale, &g))?;
        g = RealValue::ZERO;
        scale = RealValue::ZERO;
        if i <= m && i != n {
            let u_i = u.row_ref(i);
            scale = abs(&u_i.get(l));
            for k in l + 1..=n {
                scale = add(&scale, &abs(&u_i.get(k)))?;
            }
            if scale.d.abs() > eps {
                let mut u_il = div(&u_i.get(l), &scale);
                u_i.set(l, u_il.clone())?;
                let mut sum = mul(&u_il, &u_il);
                for k in l + 1..=n {
                    u_i.set(k, div(&u_i.get(k), &scale))?;
                    sum = add(&sum, &sqr(&u_i.get(k)))?;
                }
                let f = u_il.clone();
                g = copy_sign(&sqrt(&sum), &f).neg();
                let h = div(&RealValue::ONE, &sub(&mul(&f, &g), &sum)?);
                u_il = sub(&f, &g)?;
                u_i.set(l, u_il.clone())?;
                for k in l..=n {
                    rv1.set(k, mul(&u_i.get(k), &h))?;
                }
                for j in l..=m {
                    let u_j = u.row_ref(j);
                    let mut sum = mul(&u_j.get(l), &u_il);
                    for k in l + 1..=n {
                        sum = add(&sum, &mul(&u_j.get(k), &u_i.get(k)))?;
                    }
                    for k in l..=n {
                        u_j.set(k, add(&u_j.get(k), &mul(&sum, &rv1.get(k)))?)?;
                    }
                }
                for k in l..=n {
                    u_i.set(k, mul(&u_i.get(k), &scale))?;
                }
            }
        }
        let mut a = abs(&sig.get(i));
        let rvi = rv1.get(i);
        if rvi.d != 0.0 {
            a = add(&a, &abs(&rvi))?;
        }
        if anorm.d == 0.0 || compare(&a, &anorm)? > 0 {
            anorm = a;
        }
    }
    Ok(anorm)
}

/// `SvdRightTransform(...)`: accumulation of the right hand transformations.
fn svd_right_transform(u: &Matrix, v: &Matrix, rv1: &Vector, n: usize) -> CResult<()> {
    let mut l = n + 1;
    let mut g = RealValue::ZERO;
    for i in (0..=n).rev() {
        let v_i = v.row_ref(i);
        if i < n {
            if g.d != 0.0 {
                g = div(&RealValue::ONE, &g);
                let u_i = u.row_ref(i);
                for j in l..=n {
                    v_i.set(j, mul(&div(&u_i.get(j), &u_i.get(l)), &g))?;
                }
                let u_il = u_i.get(l);
                for j in l..=n {
                    let v_j = v.row_ref(j);
                    let mut sum = mul(&u_il, &v_j.get(l));
                    for k in l + 1..=n {
                        sum = add(&sum, &mul(&u_i.get(k), &v_j.get(k)))?;
                    }
                    for k in l..=n {
                        v_j.set(k, add(&v_j.get(k), &mul(&sum, &v_i.get(k)))?)?;
                    }
                }
            }
            for j in l..=n {
                v.set(j, i, RealValue::ZERO)?;
                v_i.set(j, RealValue::ZERO)?;
            }
        }
        v_i.set(i, RealValue::ONE)?;
        g = rv1.get(i);
        l = i;
    }
    Ok(())
}

/// `SvdLeftTransform(...)`: accumulation of the left hand transformations.
fn svd_left_transform(u: &Matrix, sig: &Vector, m: usize, n: usize) -> CResult<()> {
    let mut u_col_i = vec![RealValue::ZERO; m + 1];
    for i in (0..=m.min(n)).rev() {
        let l = i + 1;
        let mut g = sig.get(i);
        let u_i = u.row_ref(i);
        let u_ii = u_i.get(i);
        for j in l..=n {
            u_i.set(j, RealValue::ZERO)?;
        }
        for k in i..=m {
            u_col_i[k] = u.get(k, i);
        }
        if g.d != 0.0 {
            g = div(&RealValue::ONE, &g);
            if l <= n {
                let u_li = u_col_i[l].clone();
                for j in l..=n {
                    let mut sum = mul(&u_li, &u.get(l, j));
                    for k in l + 1..=m {
                        sum = add(&sum, &mul(&u_col_i[k], &u.get(k, j)))?;
                    }
                    let f = mul(&div(&sum, &u_ii), &g);
                    for k in i..=m {
                        u.set(k, j, add(&u.get(k, j), &mul(&f, &u_col_i[k]))?)?;
                    }
                }
            }
            for c in u_col_i.iter_mut().take(m + 1).skip(i) {
                *c = mul(c, &g);
            }
        } else {
            for c in u_col_i.iter_mut().take(m + 1).skip(i) {
                *c = RealValue::ZERO;
            }
        }
        u_col_i[i] = add(&u_col_i[i], &RealValue::ONE)?;
        for k in i..=m {
            u.set(k, i, u_col_i[k].clone())?;
        }
    }
    Ok(())
}

/// `SvdDiagonalize(...)`: diagonalization of the bidiagonal form.
fn svd_diagonalize(
    u: &Matrix,
    sig: &Vector,
    v: &Matrix,
    rv1: &Vector,
    m: usize,
    n: usize,
    anorm: &RealValue,
) -> CResult<()> {
    let negligible = |x: &RealValue| -> CResult<bool> { Ok(compare(&add(&abs(x), anorm)?, anorm)? == 0) };
    for k in (0..=n).rev() {
        for iter in 1..=30 {
            let mut flag = true;
            // C# tests sig[nm] with nm = 0 inside this loop.
            let mut l: i64 = k as i64;
            while l >= 0 {
                let lu = l as usize;
                let r = rv1.get(lu);
                if r.d == 0.0 || negligible(&r)? {
                    flag = false;
                    break;
                }
                if negligible(&sig.get(0))? {
                    break;
                }
                l -= 1;
            }
            let nm = l - 1;
            if flag {
                if nm < 0 {
                    return Err(outside());
                }
                let nm = nm as usize;
                let mut c = RealValue::ZERO;
                let mut s = RealValue::ONE;
                for i in l as usize..=k {
                    let f = mul(&s, &rv1.get(i));
                    rv1.set(i, mul(&c, &rv1.get(i)))?;
                    if negligible(&f)? {
                        break;
                    }
                    let g = sig.get(i);
                    let mut h = hypot(&f, &g)?;
                    sig.set(i, h.clone())?;
                    h = div(&RealValue::ONE, &h);
                    c = mul(&g, &h);
                    s = mul(&f.neg(), &h);
                    for j in 0..=m {
                        let u_j = u.row_ref(j);
                        let uy = u_j.get(nm);
                        let uz = u_j.get(i);
                        u_j.set(nm, add(&mul(&uy, &c), &mul(&uz, &s))?)?;
                        u_j.set(i, sub(&mul(&uz, &c), &mul(&uy, &s))?)?;
                    }
                }
            }
            let z = sig.get(k);
            if l == k as i64 {
                if z.d < 0.0 {
                    sig.set(k, z.neg())?;
                    let v_k = v.row_ref(k);
                    for j in 0..=n {
                        v_k.set(j, v_k.get(j).neg())?;
                    }
                }
                break;
            }
            if iter == 30 {
                return Err(MathParserException::new("no convergence in 30 svdcmp iterations"));
            }
            if l < 0 || k == 0 {
                return Err(outside());
            }
            let l = l as usize;
            let mut x = sig.get(l);
            let nm = k - 1;
            let mut y = sig.get(nm);
            let mut g = rv1.get(nm);
            let mut h = rv1.get(k);
            let two_hy = mul(&h, &y).mul_f(2.0);
            let mut f = if g.d == 0.0 {
                div(&sub(&mul(&sub(&y, &z)?, &add(&y, &z)?), &mul(&h, &h))?, &two_hy)
            } else {
                div(
                    &add(&mul(&sub(&y, &z)?, &add(&y, &z)?), &mul(&sub(&g, &h)?, &add(&g, &h)?))?,
                    &two_hy,
                )
            };
            g = hypot(&f, &RealValue::ONE)?;
            f = div(
                &add(
                    &mul(&sub(&x, &z)?, &add(&x, &z)?),
                    &mul(&h, &sub(&div(&y, &add(&f, &copy_sign(&g, &f))?), &h)?),
                )?,
                &x,
            );
            let mut c = RealValue::ONE;
            let mut s = RealValue::ONE;
            for j in l..=nm {
                let i = j + 1;
                g = rv1.get(i);
                y = sig.get(i);
                h = mul(&s, &g);
                g = mul(&c, &g);
                let mut z = hypot(&f, &h)?;
                rv1.set(j, z.clone())?;
                c = div(&f, &z);
                s = div(&h, &z);
                f = add(&mul(&x, &c), &mul(&g, &s))?;
                g = sub(&mul(&g, &c), &mul(&x, &s))?;
                h = mul(&y, &s);
                y = mul(&y, &c);
                let v_j = v.row_ref(j);
                let v_i = v.row_ref(i);
                for jj in 0..=n {
                    x = v_j.get(jj);
                    z = v_i.get(jj);
                    v_j.set(jj, add(&mul(&x, &c), &mul(&z, &s))?)?;
                    v_i.set(jj, sub(&mul(&z, &c), &mul(&x, &s))?)?;
                }
                z = hypot(&f, &h)?;
                sig.set(j, z.clone())?;
                if z.d != 0.0 {
                    z = div(&RealValue::ONE, &z);
                    c = mul(&f, &z);
                    s = mul(&h, &z);
                }
                f = add(&mul(&c, &g), &mul(&s, &y))?;
                x = sub(&mul(&c, &y), &mul(&s, &g))?;
                for jj in 0..=m {
                    let u_jj = u.row_ref(jj);
                    y = u_jj.get(j);
                    z = u_jj.get(i);
                    u_jj.set(j, add(&mul(&y, &c), &mul(&z, &s))?)?;
                    u_jj.set(i, sub(&mul(&z, &c), &mul(&y, &s))?)?;
                }
            }
            rv1.set(l, RealValue::ZERO)?;
            rv1.set(k, f)?;
            sig.set(k, x)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_types::unit::{OptUnit, Unit};

    fn units(s: &str) -> OptUnit {
        Some(Unit::get(s, false).unwrap())
    }

    fn fill(m: &Matrix, a: &[&[f64]], u: &OptUnit) -> Matrix {
        for (i, row) in a.iter().enumerate() {
            for (j, &x) in row.iter().enumerate() {
                if x != 0.0 && m.set(i, j, RealValue::new(x, u.clone())).is_err() {
                    // entries outside a triangular or diagonal pattern are ignored
                }
            }
        }
        m.clone()
    }

    fn full(a: &[&[f64]]) -> Matrix {
        fill(&Matrix::new(a.len(), a[0].len()).unwrap(), a, &None)
    }

    fn sym(a: &[&[f64]]) -> Matrix {
        fill(&symmetric_matrix::new(a.len()), a, &None)
    }

    /// The matrix as plain numbers (unit factors are not applied).
    fn nums(m: &Matrix) -> Vec<Vec<f64>> {
        (0..m.row_count())
            .map(|i| (0..m.col_count()).map(|j| m.get(i, j).d).collect())
            .collect()
    }

    fn product(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let (n, p) = (b.len(), b[0].len());
        a.iter()
            .map(|r| (0..p).map(|j| (0..n).map(|k| r[k] * b[k][j]).sum()).collect())
            .collect()
    }

    fn transposed(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
        (0..a[0].len()).map(|j| a.iter().map(|r| r[j]).collect()).collect()
    }

    fn assert_close(a: &[Vec<f64>], b: &[Vec<f64>], tol: f64) {
        assert_eq!(a.len(), b.len());
        for (ra, rb) in a.iter().zip(b) {
            assert_eq!(ra.len(), rb.len());
            for (x, y) in ra.iter().zip(rb) {
                assert!((x - y).abs() <= tol, "{:?} != {:?}", a, b);
            }
        }
    }

    fn identity(n: usize) -> Vec<Vec<f64>> {
        (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect()
    }

    fn columns(m: &[Vec<f64>], from: usize, to: usize) -> Vec<Vec<f64>> {
        m.iter().map(|r| r[from..to].to_vec()).collect()
    }

    const A3: [&[f64]; 3] = [&[2.0, 1.0, 1.0], &[4.0, -6.0, 0.0], &[-2.0, 7.0, 2.0]];
    const S3: [&[f64]; 3] = [&[4.0, 12.0, -16.0], &[12.0, 37.0, -43.0], &[-16.0, -43.0, 98.0]];

    #[test]
    fn lu_reconstructs_the_permuted_matrix() {
        let a = full(&A3);
        let (lu, idx) = a.lu_decomposition().unwrap();
        let lu = nums(&lu);
        let n = 3;
        let l: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            1.0
                        } else if j < i {
                            lu[i][j]
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .collect();
        let u: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..n).map(|j| if j >= i { lu[i][j] } else { 0.0 }).collect())
            .collect();
        let pa: Vec<Vec<f64>> = idx.iter().map(|&k| nums(&a)[k as usize].clone()).collect();
        assert_close(&product(&l, &u), &pa, 1e-12);
    }

    #[test]
    fn full_matrix_determinant_inverse_and_solve() {
        let a = full(&A3);
        // det = 2(-12) - 1(8) + 1(28 - 12) = -16
        assert!((a.determinant().unwrap().d + 16.0).abs() < 1e-12);
        let inv = a.invert().unwrap();
        assert_close(&product(&nums(&a), &nums(&inv)), &identity(3), 1e-12);
        let b = Vector::from_values(vec![RealValue::num(5.0), RealValue::num(-2.0), RealValue::num(9.0)]).unwrap();
        let x = a.lsolve(&b).unwrap();
        let xs: Vec<Vec<f64>> = (0..3).map(|i| vec![x.get(i).d]).collect();
        assert_close(&product(&nums(&a), &xs), &[vec![5.0], vec![-2.0], vec![9.0]], 1e-12);
        let m = a.msolve(&full(&[&[1.0, 0.0], &[0.0, 1.0], &[1.0, 1.0]])).unwrap();
        assert_close(
            &product(&nums(&a), &nums(&m)),
            &[vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]],
            1e-12,
        );
    }

    #[test]
    fn singular_matrix_is_reported() {
        let a = full(&[&[1.0, 2.0], &[2.0, 4.0]]);
        assert_eq!(a.determinant().unwrap().d, 0.0);
        assert!(a.invert().is_err());
        assert_eq!(a.rank().unwrap().d, 1.0);
        // the adjoint of a singular 2x2 matrix comes from the minors
        assert_close(&nums(&a.adjoint().unwrap()), &[vec![4.0, -2.0], vec![-2.0, 1.0]], 0.0);
    }

    #[test]
    fn adjoint_and_cofactor_of_a_regular_matrix() {
        let a = full(&[&[3.0, 1.0], &[2.0, 5.0]]);
        assert_close(&nums(&a.adjoint().unwrap()), &[vec![5.0, -1.0], vec![-2.0, 3.0]], 1e-12);
        assert_close(
            &nums(&a.cofactor().unwrap()),
            &[vec![5.0, -2.0], vec![-1.0, 3.0]],
            1e-12,
        );
    }

    #[test]
    fn qr_is_orthogonal_times_upper_triangular() {
        let a = full(&[
            &[12.0, -51.0, 4.0],
            &[6.0, 167.0, -68.0],
            &[-4.0, 24.0, -41.0],
            &[1.0, 2.0, 3.0],
        ]);
        let qr = nums(&a.qr_decomposition().unwrap());
        let q = columns(&qr, 0, 4);
        let r = columns(&qr, 4, 7);
        assert_close(&product(&transposed(&q), &q), &identity(4), 1e-12);
        assert_close(&product(&q, &r), &nums(&a), 1e-10);
        for (i, row) in r.iter().enumerate() {
            for x in row.iter().take(i) {
                assert!(x.abs() < 1e-10);
            }
        }
    }

    #[test]
    fn svd_reconstructs_the_matrix() {
        let a = full(&[&[1.0, 2.0], &[3.0, 4.0], &[5.0, 6.0]]);
        let usv = nums(&a.svd_decomposition().unwrap());
        let u = columns(&usv, 0, 2);
        let s: Vec<f64> = usv.iter().take(2).map(|r| r[2]).collect();
        let v: Vec<Vec<f64>> = usv.iter().take(2).map(|r| r[3..5].to_vec()).collect();
        assert!(s[0] >= s[1]);
        let us: Vec<Vec<f64>> = u.iter().map(|r| vec![r[0] * s[0], r[1] * s[1]]).collect();
        // C# stores V transposed, so the last block of the result is V^T.
        assert_close(&product(&us, &v), &nums(&a), 1e-12);
        let l2 = a.l2_norm().unwrap().d;
        assert!((l2 - s[0]).abs() < 1e-12);
        let c2 = a.cond2().unwrap().d;
        assert!((c2 - s[0] / s[1]).abs() < 1e-9);
    }

    #[test]
    fn condition_numbers_of_the_identity_are_one() {
        let i3 = full(&[&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0], &[0.0, 0.0, 1.0]]);
        assert!((i3.cond1().unwrap().d - 1.0).abs() < 1e-14);
        assert!((i3.cond_inf().unwrap().d - 1.0).abs() < 1e-14);
        assert!((i3.cond_frob().unwrap().d - 3.0).abs() < 1e-14);
        assert!((i3.cond2().unwrap().d - 1.0).abs() < 1e-14);
    }

    #[test]
    fn symmetric_matrix_members() {
        let s = sym(&S3);
        let a = nums(&s);
        // Cholesky U^T U = A
        let u = nums(&s.cholesky_decomposition().unwrap());
        assert_close(&product(&transposed(&u), &u), &a, 1e-12);
        // det = (2 * 1 * 3)^2
        assert!((s.determinant().unwrap().d - 36.0).abs() < 1e-9);
        assert_close(&product(&a, &nums(&s.invert().unwrap())), &identity(3), 1e-9);
        let b = Vector::from_values(vec![RealValue::num(1.0), RealValue::num(2.0), RealValue::num(3.0)]).unwrap();
        for x in [s.lsolve(&b).unwrap(), s.cl_solve(&b).unwrap()] {
            let xs: Vec<Vec<f64>> = (0..3).map(|i| vec![x.get(i).d]).collect();
            assert_close(&product(&a, &xs), &[vec![1.0], vec![2.0], vec![3.0]], 1e-9);
        }
        let mm = full(&[&[1.0], &[0.0], &[2.0]]);
        let x = nums(&s.cm_solve(&mm).unwrap());
        assert_close(&product(&a, &x), &nums(&mm), 1e-9);
        assert!(full(&A3).cholesky_decomposition().is_err());
    }

    #[test]
    fn symmetric_eigen_pairs() {
        let s = sym(&[&[2.0, -1.0, 0.0], &[-1.0, 2.0, -1.0], &[0.0, -1.0, 2.0]]);
        let a = nums(&s);
        let vals = s.eigen_values(0, 0.0).unwrap();
        let expected = [2.0 - 2f64.sqrt(), 2.0, 2.0 + 2f64.sqrt()];
        for (i, e) in expected.iter().enumerate() {
            assert!((vals.get(i).d - e).abs() < 1e-12);
        }
        let e = nums(&s.eigen(0, 0.0).unwrap());
        for row in &e {
            let lambda = row[0];
            let v: Vec<Vec<f64>> = row[1..].iter().map(|&x| vec![x]).collect();
            let av = product(&a, &v);
            let lv: Vec<Vec<f64>> = v.iter().map(|r| vec![r[0] * lambda]).collect();
            assert_close(&av, &lv, 1e-12);
            let norm: f64 = v.iter().map(|r| r[0] * r[0]).sum();
            assert!((norm - 1.0).abs() < 1e-12);
        }
        let largest = s.eigen_values(-1, 0.0).unwrap();
        assert!((largest.get(0).d - expected[2]).abs() < 1e-12);
        assert_eq!(s.eigen_vectors(2, 0.0).unwrap().row_count(), 2);
    }

    #[test]
    fn diagonal_matrix_members() {
        let d = fill(
            &diagonal_matrix::new(3),
            &[&[2.0, 0.0, 0.0], &[0.0, 4.0, 0.0], &[0.0, 0.0, 5.0]],
            &None,
        );
        assert_eq!(d.determinant().unwrap().d, 40.0);
        assert_close(
            &nums(&d.invert().unwrap()),
            &[vec![0.5, 0.0, 0.0], vec![0.0, 0.25, 0.0], vec![0.0, 0.0, 0.2]],
            0.0,
        );
        let b = Vector::from_values(vec![RealValue::num(2.0), RealValue::num(2.0), RealValue::num(10.0)]).unwrap();
        let x = d.lsolve(&b).unwrap();
        assert_eq!((x.get(0).d, x.get(1).d, x.get(2).d), (1.0, 0.5, 2.0));
        let vals = d.eigen_values(-3, 0.0).unwrap();
        assert_eq!((vals.get(0).d, vals.get(1).d, vals.get(2).d), (5.0, 4.0, 2.0));
        assert_close(
            &nums(&d.adjoint().unwrap()),
            &[vec![20.0, 0.0, 0.0], vec![0.0, 10.0, 0.0], vec![0.0, 0.0, 8.0]],
            1e-12,
        );
        let z = fill(&diagonal_matrix::new(2), &[&[1.0, 0.0], &[0.0, 0.0]], &None);
        assert!(z.invert().is_err());
    }

    #[test]
    fn triangular_matrix_members() {
        let l = fill(
            &lower_triangular_matrix::new(3),
            &[&[2.0, 0.0, 0.0], &[3.0, 1.0, 0.0], &[1.0, -1.0, 4.0]],
            &None,
        );
        let u = fill(
            &upper_triangular_matrix::new(3),
            &[&[2.0, 3.0, 1.0], &[0.0, 1.0, -1.0], &[0.0, 0.0, 4.0]],
            &None,
        );
        for m in [&l, &u] {
            assert_eq!(m.determinant().unwrap().d, 8.0);
            assert_close(&product(&nums(m), &nums(&m.invert().unwrap())), &identity(3), 1e-12);
            let b = Vector::from_values(vec![RealValue::num(1.0), RealValue::num(2.0), RealValue::num(3.0)]).unwrap();
            let x = m.lsolve(&b).unwrap();
            let xs: Vec<Vec<f64>> = (0..3).map(|i| vec![x.get(i).d]).collect();
            assert_close(&product(&nums(m), &xs), &[vec![1.0], vec![2.0], vec![3.0]], 1e-12);
        }
    }

    #[test]
    fn determinant_and_solve_with_units() {
        let u = units("m");
        let a = fill(&Matrix::new(2, 2).unwrap(), &[&[3.0, 1.0], &[2.0, 5.0]], &u);
        let det = a.determinant().unwrap();
        assert!((det.d - 13.0).abs() < 1e-12);
        let m2 = Unit::multiply(&u, &u, false).0;
        assert!(Unit::is_consistent_opt(&det.units, &m2));
        let s = fill(&symmetric_matrix::new(3), &S3, &units("kN"));
        let c = s.cholesky_decomposition().unwrap();
        assert_close(&product(&transposed(&nums(&c)), &nums(&c)), &nums(&s), 1e-12);
        assert!(c.get(0, 0).units.is_some());
        let b = Vector::from_values(vec![RealValue::new(4.0, u.clone()), RealValue::new(7.0, u.clone())]).unwrap();
        let x = a.lsolve(&b).unwrap();
        assert!((3.0 * x.get(0).d + x.get(1).d - 4.0).abs() < 1e-12);
        assert!((2.0 * x.get(0).d + 5.0 * x.get(1).d - 7.0).abs() < 1e-12);
    }
}
