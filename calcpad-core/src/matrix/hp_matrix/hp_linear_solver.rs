//! Port of Matrix/HpMatrix/HpLinearSolver.cs: preconditioned conjugate gradients for
//! HP symmetric matrices, and `Hypot`.
use super::{vectorized, HpMat};
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::messages;

/// `SolvePCG(double[] b, double tol)`
pub(crate) fn solve_pcg(a: &HpMat, b: &[f64], tol: f64) -> CResult<Vec<f64>> {
    let n = a.row_count;
    let mut x = vec![0.0; n];
    let mut r = vec![0.0; n];
    let mut z = vec![0.0; n];
    let mut p = vec![0.0; n];
    let mut ap = vec![0.0; n];
    let mut diag = vec![0.0; n];
    for i in 0..n {
        let row = &a.rows[i];
        if row.size == 0 {
            return Err(exceptions::MatrixSingular());
        }
        diag[i] = 1.0 / row.raw_all()[0];
    }
    // b.AsSpan().CopyTo(r)
    if b.len() > n {
        return Err(MathParserException::unexpected(
            "Destination is too short. (Parameter 'destination')",
        ));
    }
    r[..b.len()].copy_from_slice(b);
    solve_pcg_with_arrays(a, &mut r, &mut x, &mut z, &mut p, &mut ap, &diag, tol)?;
    Ok(x)
}

/// `MSolvePCG(double[][] B, double tol)`: B holds the right hand sides as rows.
pub(crate) fn msolve_pcg(a: &HpMat, bt: &[Vec<f64>], tol: f64) -> CResult<Vec<Vec<f64>>> {
    let n = a.row_count;
    let mut diag = vec![0.0; n];
    for i in 0..n {
        let raw = a.rows[i].raw_all();
        let r0 = *raw
            .first()
            .ok_or_else(|| MathParserException::unexpected("Index was outside the bounds of the array."))?;
        diag[i] = 1.0 / r0;
    }
    let mut out = Vec::with_capacity(bt.len());
    for bj in bt {
        let mut x = vec![0.0; n];
        let mut r = vec![0.0; n];
        let mut z = vec![0.0; n];
        let mut p = vec![0.0; n];
        let mut ap = vec![0.0; n];
        let len = bj.len().min(n);
        r[..len].copy_from_slice(&bj[..len]);
        solve_pcg_with_arrays(a, &mut r, &mut x, &mut z, &mut p, &mut ap, &diag, tol)?;
        out.push(x);
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn solve_pcg_with_arrays(
    a: &HpMat,
    r: &mut [f64],
    x: &mut [f64],
    z: &mut [f64],
    p: &mut [f64],
    ap: &mut [f64],
    diag: &[f64],
    tol: f64,
) -> CResult<()> {
    let n = a.row_count;
    let max_iterations = (2 * n).min(1000);
    vectorized::multiply(diag, r, z);
    p.copy_from_slice(z);
    let mut rz_old = vectorized::dot_product(r, z, 0, n);
    let tol_sq = tol * tol;
    let mut iter = 0;
    for v in x.iter_mut().take(n) {
        *v = 0.0;
    }
    while iter < max_iterations {
        let rnorm_sq = vectorized::sum_sq(r, 0, n);
        if rnorm_sq < tol_sq {
            break;
        }
        symmetric_matrix_vector_product(a, p, ap);
        let p_ap = vectorized::dot_product(p, ap, 0, n);
        if p_ap.abs() < 1e-15 {
            return Err(MathParserException::new(messages::TheMatrixIsIllConditioned));
        }
        let alpha = rz_old / p_ap;
        vectorized::multiply_add(p, alpha, x);
        vectorized::multiply_add(ap, -alpha, r);
        vectorized::multiply(diag, r, z);
        let rznew = vectorized::dot_product(r, z, 0, n);
        let beta = rznew / rz_old;
        vectorized::scale(p, beta);
        vectorized::add(z, p);
        rz_old = rznew;
        iter += 1;
    }
    if iter == max_iterations {
        return Err(MathParserException::new(
            messages::ThePCGSolverFailedToConvergeIn_0_Iterations.replace("{0}", &max_iterations.to_string()),
        ));
    }
    Ok(())
}

fn symmetric_matrix_vector_product(a: &HpMat, x: &[f64], y: &mut [f64]) {
    for v in y.iter_mut() {
        *v = 0.0;
    }
    let m = a.row_count;
    let n = x.len();
    for i in 0..m {
        let row = &a.rows[i];
        let raw = row.raw_all();
        let x_i = x[i];
        let mut sum = raw[0] * x_i;
        for k in 1..row.size {
            let j = i + k;
            if j < n {
                let val = raw[k];
                sum += val * x[j];
                y[j] += val * x_i;
            }
        }
        y[i] += sum;
    }
}

/// `Hypot(double a, double b)`
#[inline]
pub fn hypot(a: f64, b: f64) -> f64 {
    let a1 = a.abs();
    let b1 = b.abs();
    if crate::vector::double_compare_to(a1, b1) > 0 {
        a1 * (1.0 + (b1 / a1) * (b1 / a1)).sqrt()
    } else if b1 == 0.0 {
        0.0
    } else {
        b1 * (1.0 + (a1 / b1) * (a1 / b1)).sqrt()
    }
}
