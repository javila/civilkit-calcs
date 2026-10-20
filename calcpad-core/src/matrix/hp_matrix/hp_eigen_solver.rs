//! Port of Matrix/HpMatrix/HpEigenSolver.cs: Householder tridiagonalization with the
//! implicit QL algorithm (Numerical Recipes) and the Lanczos method with full
//! reorthogonalization. The rows of the symmetric input store the upper half from the
//! diagonal (`A[i][k]` is element (i, i + k)).
use super::hp_linear_solver::hypot;
use super::vectorized;
use crate::exceptions;
use crate::math_parser_exception::CResult;

/// `System.Random` with a seed (the .NET legacy subtractive generator that
/// `new Random(seed)` uses).
pub struct DotNetRandom {
    seed_array: [i32; 56],
    inext: usize,
    inextp: usize,
}

impl DotNetRandom {
    pub fn new(seed: i32) -> DotNetRandom {
        const MBIG: i32 = i32::MAX;
        const MSEED: i32 = 161803398;
        let mut seed_array = [0i32; 56];
        let subtraction = if seed == i32::MIN { i32::MAX } else { seed.abs() };
        let mut mj = MSEED - subtraction;
        seed_array[55] = mj;
        let mut mk = 1i32;
        let mut ii = 0usize;
        for _ in 1..55 {
            ii += 21;
            if ii >= 55 {
                ii -= 55;
            }
            seed_array[ii] = mk;
            mk = mj - mk;
            if mk < 0 {
                mk += MBIG;
            }
            mj = seed_array[ii];
        }
        for _ in 1..5 {
            for i in 1..56 {
                let mut n = i + 30;
                if n >= 55 {
                    n -= 55;
                }
                seed_array[i] = seed_array[i].wrapping_sub(seed_array[1 + n]);
                if seed_array[i] < 0 {
                    seed_array[i] += MBIG;
                }
            }
        }
        DotNetRandom {
            seed_array,
            inext: 0,
            inextp: 21,
        }
    }

    fn internal_sample(&mut self) -> i32 {
        let mut loc_inext = self.inext + 1;
        if loc_inext >= 56 {
            loc_inext = 1;
        }
        let mut loc_inextp = self.inextp + 1;
        if loc_inextp >= 56 {
            loc_inextp = 1;
        }
        let mut ret = self.seed_array[loc_inext].wrapping_sub(self.seed_array[loc_inextp]);
        if ret == i32::MAX {
            ret -= 1;
        }
        if ret < 0 {
            ret += i32::MAX;
        }
        self.seed_array[loc_inext] = ret;
        self.inext = loc_inext;
        self.inextp = loc_inextp;
        ret
    }

    /// `NextDouble()`
    pub fn next_double(&mut self) -> f64 {
        self.internal_sample() as f64 * (1.0 / i32::MAX as f64)
    }
}

fn create_random_normalized_vector(n: usize, rnd: &mut DotNetRandom) -> Vec<f64> {
    let mut v: Vec<f64> = (0..n).map(|_| rnd.next_double() - 0.5).collect();
    let norm = vectorized::norm(&v);
    for x in v.iter_mut() {
        *x /= norm;
    }
    v
}

fn symmetric_matrix_vector_product(a: &[Vec<f64>], x: &[f64], y: &mut [f64]) {
    for v in y.iter_mut() {
        *v = 0.0;
    }
    let n = a.len();
    for i in 0..n {
        let a_i = &a[i];
        let x_i = x[i];
        let mut sum = a_i[0] * x_i;
        for k in 1..a_i.len() {
            let j = i + k;
            if j < n {
                let val = a_i[k];
                sum += val * x[j];
                y[j] += val * x_i;
            }
        }
        y[i] += sum;
    }
}

/// `Lanczos(Matrix, eigenCount, reverse, tol, out eigvals)`: returns the eigenvectors
/// (n x eigenCount) and the eigenvalues.
pub fn lanczos(matrix: &[Vec<f64>], eigen_count: usize, reverse: bool, tol: f64) -> CResult<(Vec<Vec<f64>>, Vec<f64>)> {
    let n = matrix.len();
    let k = n.min(4 * eigen_count + 100);
    let mut w = vec![0.0; n];
    let mut alpha = vec![0.0; k];
    let mut beta = vec![0.0; k + 1];
    let mut v: Vec<Vec<f64>> = vec![Vec::new(); k + 2];
    v[0] = vec![0.0; n];
    v[1] = create_random_normalized_vector(n, &mut DotNetRandom::new(0));
    let mut previous: Option<Vec<f64>> = None;
    let initial_steps = eigen_count;
    let check_interval = (eigen_count / 8).max(1);
    let mut m = 0;
    for j in 1..=k {
        let j1 = j - 1;
        symmetric_matrix_vector_product(matrix, &v[j], &mut w);
        vectorized::multiply_add(&v[j1], -beta[j1], &mut w);
        alpha[j1] = vectorized::dot_product(&w, &v[j], 0, w.len());
        vectorized::multiply_add(&v[j], -alpha[j1], &mut w);
        for s in 1..=j {
            let dot = vectorized::dot_product(&w, &v[s], 0, n);
            vectorized::multiply_add(&v[s], -dot, &mut w);
        }
        beta[j] = vectorized::norm(&w);
        if beta[j] < 1e-15 {
            m = j;
            break;
        }
        let inv_beta = 1.0 / beta[j];
        v[j + 1] = w.iter().map(|x| x * inv_beta).collect();
        m = j;
        if j >= initial_steps
            && (j - initial_steps) % check_interval == 0
            && check_kaniel_paige_convergence(&alpha, &beta, &v, j, n, eigen_count, reverse, tol, &mut previous)?
        {
            m = j;
            break;
        }
    }
    let (vals, vecs) = eigen_solve_tridiagonal_system(&alpha, &beta, &v, m, n, eigen_count, reverse, true)?;
    Ok((vecs.unwrap_or_default(), vals))
}

#[allow(clippy::too_many_arguments)]
fn check_kaniel_paige_convergence(
    alpha: &[f64],
    beta: &[f64],
    v: &[Vec<f64>],
    current_dim: usize,
    n: usize,
    eigen_count: usize,
    reverse: bool,
    tol: f64,
    prev: &mut Option<Vec<f64>>,
) -> CResult<bool> {
    if current_dim < 2.max(eigen_count) {
        return Ok(false);
    }
    let (curr, _) = eigen_solve_tridiagonal_system(alpha, beta, v, current_dim, n, eigen_count, reverse, false)?;
    let mut has_converged = false;
    if let Some(p) = prev {
        if p.len() == curr.len() {
            has_converged = true;
            for i in 0..curr.len() {
                let err = (curr[i] - p[i]).abs() / (1.0 + curr[i].abs());
                if err > tol {
                    has_converged = false;
                    break;
                }
            }
            if has_converged {
                let beta_m = beta[current_dim].abs();
                let mut max_eig: f64 = 0.0;
                for c in &curr {
                    max_eig = max_eig.max(c.abs());
                }
                if beta_m > tol * max_eig {
                    has_converged = false;
                }
            }
        }
    }
    *prev = Some(curr);
    Ok(has_converged)
}

#[allow(clippy::too_many_arguments)]
fn eigen_solve_tridiagonal_system(
    alpha: &[f64],
    beta: &[f64],
    v: &[Vec<f64>],
    m: usize,
    n: usize,
    eigen_count: usize,
    reverse: bool,
    return_vecs: bool,
) -> CResult<(Vec<f64>, Option<Vec<Vec<f64>>>)> {
    let eigen_count = eigen_count.min(m);
    let mut d = alpha[..m].to_vec();
    let mut e = vec![0.0; m];
    if m >= 1 {
        e[1..m].copy_from_slice(&beta[1..m]);
    }
    let mut q: Vec<Vec<f64>> = (0..m)
        .map(|i| {
            let mut r = vec![0.0; m];
            r[i] = 1.0;
            r
        })
        .collect();
    implicit_ql(&mut d, &mut e, &mut q, true)?;
    let mut indexes: Vec<i32> = (0..m as i32).collect();
    let dd = d.clone();
    if reverse {
        crate::sort::sort_by::<i32, ()>(&mut indexes, None, &mut |a, b| {
            Ok(crate::vector::double_compare_to(dd[*b as usize], dd[*a as usize]))
        })?;
    } else {
        crate::sort::sort_by::<i32, ()>(&mut indexes, None, &mut |a, b| {
            Ok(crate::vector::double_compare_to(dd[*a as usize], dd[*b as usize]))
        })?;
    }
    let eigvals: Vec<f64> = (0..eigen_count).map(|i| d[indexes[i] as usize]).collect();
    if !return_vecs {
        return Ok((eigvals, None));
    }
    let mut eigvecs = vec![vec![0.0; eigen_count]; n];
    let mut sv = vec![0.0; n];
    for i in 0..eigen_count {
        let index = indexes[i] as usize;
        for x in sv.iter_mut() {
            *x = 0.0;
        }
        for (j, q_j) in q.iter().enumerate().take(m) {
            vectorized::multiply_add(&v[j + 1], q_j[index], &mut sv);
        }
        let norm = vectorized::norm(&sv);
        if norm != 0.0 {
            vectorized::scale(&mut sv, 1.0 / norm);
        }
        for j in 0..n {
            eigvecs[j][i] = sv[j];
        }
    }
    Ok((eigvals, Some(eigvecs)))
}

/// `Tridiagonalize(A, out d, out e, eigenvecs)`: returns (Q, d, e).
pub fn tridiagonalize(a: &[Vec<f64>], eigenvecs: bool) -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let n1 = a.len();
    let mut d = vec![0.0; n1];
    let mut e = vec![0.0; n1];
    let mut q = vec![vec![0.0; n1]; n1];
    let mut min = vec![0usize; n1];
    if n1 == 0 {
        return (q, d, e);
    }
    let n = n1 - 1;
    for i in 0..=n {
        let row = &a[i];
        for j in (0..row.len()).rev() {
            let k = i + j;
            if i < min[k] {
                min[k] = i;
            }
            q[k][i] = row[j];
        }
    }
    for i in (1..=n).rev() {
        let l = i - 1;
        let mut h = 0.0;
        let m = min[i];
        if l > 0 {
            let scale = vectorized::sum_abs(&q[i], m, i);
            if scale == 0.0 {
                e[i] = q[i][l];
            } else {
                h = vectorized::normalize_and_sum_sq(&mut q[i], m, i, scale);
                let mut f = q[i][l];
                let mut g = if f >= 0.0 { -h.sqrt() } else { h.sqrt() };
                e[i] = scale * g;
                h -= f * g;
                q[i][l] = f - g;
                for j in m..=l {
                    if eigenvecs {
                        let v = q[i][j] / h;
                        q[j][i] = v;
                    }
                    g = vectorized::dot_product(&q[i], &q[j], 0, j + 1);
                    for k in j + 1..=l {
                        g += q[k][j] * q[i][k];
                    }
                    e[j] = g / h;
                    if j == m {
                        f = e[j] * q[i][j];
                    } else {
                        f += e[j] * q[i][j];
                    }
                }
                let hh = f / (h + h);
                for j in m..=l {
                    f = q[i][j];
                    g = e[j] - hh * f;
                    e[j] = g;
                    for k in m..=j {
                        let v = f * e[k] + g * q[i][k];
                        q[j][k] -= v;
                    }
                }
            }
        } else {
            e[i] = q[i][l];
        }
        d[i] = h;
    }
    if eigenvecs {
        d[0] = 0.0;
    }
    e[0] = 0.0;
    if eigenvecs {
        for i in 0..=n {
            let m = min[i];
            if d[i] != 0.0 && i > 0 {
                let l = i - 1;
                for j in m..=l {
                    let mut g = q[i][m] * q[m][j];
                    for k in m + 1..=l {
                        g += q[i][k] * q[k][j];
                    }
                    for k in 0..=l {
                        let v = g * q[k][i];
                        q[k][j] -= v;
                    }
                }
            }
            d[i] = q[i][i];
            q[i][i] = 1.0;
            for j in 0..i {
                q[j][i] = 0.0;
                q[i][j] = 0.0;
            }
        }
    } else {
        for i in 0..=n {
            d[i] = q[i][i];
        }
    }
    (q, d, e)
}

/// `ImplicitQL(d, e, Q, eigenvecs)`
pub fn implicit_ql(d: &mut [f64], e: &mut [f64], q: &mut [Vec<f64>], eigenvecs: bool) -> CResult<()> {
    if d.is_empty() {
        return Ok(());
    }
    let n = d.len() - 1;
    for i in 0..n {
        e[i] = e[i + 1];
    }
    e[n] = 0.0;
    for l in 0..=n {
        let mut iter = 0;
        loop {
            let mut m = l;
            while m < n {
                let dd = d[m].abs() + d[m + 1].abs();
                if e[m].abs() <= 1e-15 * dd {
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
                let mut g = (d[l1] - d[l]) / (e[l] * 2.0);
                let mut r = hypot(g, 1.0);
                g = d[m] - d[l] + e[l] / (g + r.copysign(g));
                let mut s = 1.0;
                let mut c = s;
                let mut p = 0.0;
                let mut i = m as i64 - 1;
                while i >= l as i64 {
                    let iu = i as usize;
                    let i1 = iu + 1;
                    let mut f = s * e[iu];
                    let b = c * e[iu];
                    r = hypot(f, g);
                    e[i1] = r;
                    if r == 0.0 {
                        d[i1] -= p;
                        e[m] = 0.0;
                        break;
                    }
                    s = f / r;
                    c = g / r;
                    if p == 0.0 {
                        g = d[i1];
                    } else {
                        g = d[i1] - p;
                    }
                    r = (d[iu] - g) * s + c * b * 2.0;
                    p = s * r;
                    d[i1] = g + p;
                    g = c * r - b;
                    if eigenvecs {
                        for q_k in q.iter_mut().take(n + 1) {
                            f = q_k[i1];
                            q_k[i1] = s * q_k[iu] + c * f;
                            q_k[iu] = c * q_k[iu] - s * f;
                        }
                    }
                    i -= 1;
                }
                if r == 0.0 && i >= l as i64 {
                    // `continue` of the do-while: the condition m != l is evaluated
                    continue;
                }
                d[l] -= p;
                e[l] = g;
                e[m] = 0.0;
            }
            if m == l {
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::DotNetRandom;

    #[test]
    fn dotnet_random_seed_0() {
        // new Random(0).NextDouble() in .NET (legacy seeded algorithm)
        let mut r = DotNetRandom::new(0);
        let a = r.next_double();
        assert!((a - 0.7262432699679598).abs() < 1e-16, "{}", a);
    }
}
