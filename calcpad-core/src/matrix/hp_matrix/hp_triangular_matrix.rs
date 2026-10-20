//! Port of Matrix/HpMatrix/HpLowerTriangularMatrix.cs and HpUpperTriangularMatrix.cs: the
//! transposes and the numerical members. (Constructors, element access and Resize are in
//! `mod.rs`.)
use super::{hpd, new_lower, new_upper, units, HpMat};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::Unit;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::{Matrix, MatrixType};
use crate::vector::{hp_vector, HpData, Vector};

fn out_of_range() -> MathParserException {
    MathParserException::unexpected("Index was outside the bounds of the array.")
}

/// `HpLowerTriangularMatrix.Transpose()`
pub fn lower_transpose(m: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let n = a.row_count;
    let u = new_upper(n, a.units.clone())?;
    for i in (0..n).rev() {
        // RawCol(i)
        let len = a.col_count - i;
        let col: Vec<f64> = (0..len).map(|k| a.rows[i + k].get_value(i)).collect();
        u.set_row_ref(i, hp_vector::from_doubles(col, a.units.clone())?);
    }
    Ok(u)
}

/// `HpUpperTriangularMatrix.Transpose()`
pub fn upper_transpose(m: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let n = a.row_count;
    let l = new_lower(n, a.units.clone())?;
    for i in (0..n).rev() {
        // RawCol(i)
        let mut v = HpData {
            values: None,
            size: 0,
            capacity: 0,
            length: i + 1,
            units: a.units.clone(),
        };
        for k in (0..=i).rev() {
            v.set_value(a.rows[k].get_value(i - k), k);
        }
        l.set_row_ref(i, hpd(v));
    }
    Ok(l)
}

/// `RawCopy()` of both triangular types: C# returns an HpLowerTriangularMatrix.
pub(crate) fn raw_copy(m: &Matrix) -> HpMat {
    let a = HpMat::from(m);
    let n = a.rows.len();
    let mut c = HpMat::empty(MatrixType::LowerTriangular, n, n, a.units.clone());
    for i in (0..n).rev() {
        c.rows[i] = a.rows[i].copy();
    }
    c
}

/// `Determinant()` of both types. The upper triangular version reads
/// `_hpRows[i].GetValue(i)` too, as C# does.
pub fn determinant(m: &Matrix) -> CResult<RealValue> {
    let a = HpMat::from(m);
    let n = a.row_count;
    if a.rows.is_empty() {
        return Err(out_of_range());
    }
    let mut det = a.rows[0].get_value(0);
    for i in 1..n {
        det *= a.rows[i].get_value(i);
    }
    Ok(RealValue::new(det, a.units.as_ref().and_then(|u| u.pow(n as f32))))
}

/// `GetLU` of both types
pub(crate) fn get_lu(m: &Matrix) -> CResult<Option<(HpMat, Vec<i32>, f64, f64)>> {
    let a = HpMat::from(m);
    let n = a.row_count;
    let mut indexes = vec![0; n];
    for (i, x) in indexes.iter_mut().enumerate() {
        let v = if a.mtype == MatrixType::LowerTriangular {
            a.rows[i].get_value(i)
        } else {
            a.rows[i].get(0).d
        };
        if v == 0.0 {
            return Ok(None);
        }
        *x = i as i32;
    }
    Ok(Some((raw_copy(m), indexes, f64::MAX, 1.0)))
}

/// `Invert()` of both types
pub fn invert(m: &Matrix) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let n = a.row_count;
    if a.mtype == MatrixType::LowerTriangular {
        let mut l = HpMat::empty(
            MatrixType::LowerTriangular,
            n,
            n,
            a.units.as_ref().and_then(|u| u.pow(-1.0)),
        );
        for i in 0..n {
            let row = a.rows[i].raw_all();
            let mut v = *row.get(i).ok_or_else(out_of_range)?;
            if v == 0.0 {
                return Err(exceptions::MatrixSingular());
            }
            v = 1.0 / v;
            l.set_value(v, i, i)?;
            for j in 0..i {
                let mut sum = row[j] * l.get_value(j, j);
                for k in j + 1..i {
                    sum += row[k] * l.get_value(k, j);
                }
                l.set_value(-sum * v, i, j)?;
            }
        }
        return Ok(l.into_matrix());
    }
    let mut u = HpMat::empty(MatrixType::UpperTriangular, n, n, a.units.clone());
    if n == 0 {
        return Ok(u.into_matrix());
    }
    let nn = n - 1;
    for i in (0..=nn).rev() {
        let row = a.rows[i].raw_all();
        let mut v = *row.first().ok_or_else(out_of_range)?;
        if v == 0.0 {
            return Err(exceptions::MatrixSingular());
        }
        v = 1.0 / v;
        u.set_value(v, i, i)?;
        for j in (i + 1..=nn).rev() {
            let mut sum = *row.get(nn - i).ok_or_else(out_of_range)? * u.get_value(nn, j);
            for k in (i + 1..nn).rev() {
                sum += row[k - i] * u.get_value(k, j);
            }
            u.set_value(-sum * v, i, j)?;
        }
    }
    Ok(u.into_matrix())
}

/// `LSolve(HpVector vector)` of both types (the upper version ignores the unit factor,
/// as C# does)
pub fn lsolve(m: &Matrix, vector: &Vector) -> CResult<Vector> {
    let a = HpMat::from(m);
    let v = hp_vector::snap(vector);
    let n = a.row_count;
    let (u, d) = Unit::divide(&v.units, &a.units, true);
    let mut x = HpData::new(n, n, u);
    let values = v.raw_all();
    {
        let xv = x.values.as_mut().unwrap();
        if a.mtype == MatrixType::LowerTriangular {
            for i in 0..n {
                let mut sum = *values.get(i).ok_or_else(out_of_range)?;
                let row = a.rows[i].raw_all();
                let k = i.min(row.len());
                for j in 0..k {
                    sum -= row[j] * xv[j];
                }
                let ri = *row.get(i).ok_or_else(out_of_range)?;
                if ri == 0.0 {
                    return Err(exceptions::MatrixSingular());
                }
                xv[i] = sum * d / ri;
            }
        } else {
            for i in (0..n).rev() {
                let mut sum = *values.get(i).ok_or_else(out_of_range)?;
                let row = a.rows[i].raw_all();
                let kk = (a.col_count as i64).min(i as i64 + row.len() as i64) - 1;
                let mut j = kk;
                while j > i as i64 {
                    sum -= row[j as usize - i] * xv[j as usize];
                    j -= 1;
                }
                xv[i] = sum / *row.first().ok_or_else(out_of_range)?;
            }
        }
    }
    Ok(hpd(x))
}

/// `MSolve(HpMatrix M)` of both types
pub fn msolve(m: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let rc = m.row_count();
    let n = b.col_count();
    let mut result = Vec::with_capacity(n);
    for j in 0..n {
        result.push(lsolve(m, &super::col(b, j as i64 + 1)?)?);
    }
    super::create_from_cols(&result, rc)
}

#[allow(dead_code)]
fn _units(m: &Matrix) -> crate::base_types::unit::OptUnit {
    units(m)
}
