//! Port of Matrix/HpMatrix/HpColumnMatrix.cs: the products with a column matrix.
//! (The constructors, element access, Resize, Transpose and the norms are in `mod.rs`.)
use super::{hp_column_from_vector, row_ref, units, vectorized, HpMat};
use crate::base_types::unit::Unit;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::{Matrix, MatrixType, PARALLEL_THRESHOLD};
use crate::vector::hp_vector;

fn out_of_range() -> MathParserException {
    MathParserException::unexpected("Index was outside the bounds of the array.")
}

/// `operator *(HpMatrix a, HpColumnMatrix b)`
pub fn mul(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    if a.mtype() == MatrixType::Column {
        if a.row_count() != b.row_count() {
            return Err(exceptions::MatrixDimensions());
        }
        return hp_column_from_vector(&hp_vector::mul(&a.row_ref(0), &b.row_ref(0))?);
    }
    let m = a.row_count();
    let (u, d) = Unit::multiply(&units(a), &units(b), true);
    let br = hp_vector::snap(&b.row_ref(0));
    let mut cr = vec![0.0; m];
    let _ = PARALLEL_THRESHOLD;
    for (i, c) in cr.iter_mut().enumerate() {
        let r = hp_vector::snap(&row_ref(a, i));
        *c = hp_vector::dot_product_data(&r, &br).d * d;
    }
    hp_column_from_vector(&hp_vector::from_doubles(cr, u)?)
}

/// `operator *(HpUpperTriangularMatrix a, HpColumnMatrix b)`
pub fn mul_upper(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let m = a.row_count();
    let br = hp_vector::snap(&b.row_ref(0));
    let vb = br.raw_all();
    let n = br.size as i64;
    let (u, d) = Unit::multiply(&units(a), &units(b), true);
    let aa = HpMat::from(a);
    let mut vc = vec![0.0; m];
    for (i, c) in vc.iter_mut().enumerate() {
        let ar = &aa.rows[i];
        let size = (n - i as i64).min(ar.size as i64);
        // vb.AsSpan(i) throws when i is past the end of the array
        if i > vb.len() {
            return Err(MathParserException::unexpected(
                "Specified argument was out of the range of valid values.",
            ));
        }
        *c = if size > 0 {
            vectorized::dot_product(ar.raw_all(), &vb[i..], 0, size as usize) * d
        } else {
            0.0
        };
    }
    hp_column_from_vector(&hp_vector::from_doubles(vc, u)?)
}

/// `operator *(HpSymmetricMatrix a, HpColumnMatrix b)`
pub fn mul_symmetric(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let m = a.row_count();
    let bc = hp_vector::snap(&b.row_ref(0));
    let vb = bc.raw_all();
    let n = bc.size;
    let (u, d) = Unit::multiply(&units(a), &units(b), true);
    let _ = d;
    let aa = HpMat::from(a);
    let mut vc = vec![0.0; m];
    for i in 0..m {
        let row = &aa.rows[i];
        let va = row.raw_all();
        let b_i = *vb.get(i).ok_or_else(out_of_range)?;
        let mut sum = *va.first().ok_or_else(out_of_range)? * b_i;
        for k in 1..row.size {
            let j = i + k;
            if j < n {
                let val = va[k];
                sum += val * vb[j];
                vc[j] += val * b_i;
            }
        }
        vc[i] += sum;
    }
    hp_column_from_vector(&hp_vector::from_doubles(vc, u)?)
}
