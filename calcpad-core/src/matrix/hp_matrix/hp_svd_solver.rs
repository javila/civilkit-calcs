//! Port of Matrix/HpMatrix/HpSvdSolver.cs: the one sided Golub-Kahan SVD of Numerical
//! Recipes on jagged arrays.
use super::hp_linear_solver::hypot;
use super::vectorized;
use crate::math_parser_exception::{CResult, MathParserException};

fn copy_sign(a: f64, b: f64) -> f64 {
    a.copysign(b)
}

fn cache_u_column(u: &[Vec<f64>], col: &mut [f64], j: usize, m: usize) {
    for k in j..=m {
        col[k] = u[k][j];
    }
}

fn restore_u_column(u: &mut [Vec<f64>], col: &[f64], j: usize, m: usize) {
    for k in j..=m {
        u[k][j] = col[k];
    }
}

/// `Bidiagonalize(U, sig, rv1, U_col_i, m, n, out anorm)`
pub fn bidiagonalize(
    u: &mut [Vec<f64>],
    sig: &mut [f64],
    rv1: &mut [f64],
    u_col_i: &mut [f64],
    m: usize,
    n: usize,
) -> f64 {
    let eps = 1.0 / f64::MAX;
    let mut scale = 0.0;
    let mut g = 0.0;
    let mut anorm = 0.0;
    for i in 0..=n {
        let l = i + 1;
        rv1[i] = scale * g;
        if i <= m {
            cache_u_column(u, u_col_i, i, m);
            scale = vectorized::sum_abs(u_col_i, i, m + 1);
            if scale.abs() > eps {
                let mut sum = vectorized::normalize_and_sum_sq(u_col_i, i, m + 1, scale);
                let mut f = u_col_i[i];
                g = -copy_sign(sum.sqrt(), f);
                let h = f * g - sum;
                u_col_i[i] = f - g;
                for j in l..=n {
                    sum = u_col_i[i] * u[i][j];
                    for k in l..=m {
                        sum += u_col_i[k] * u[k][j];
                    }
                    f = sum / h;
                    for k in i..=m {
                        u[k][j] += f * u_col_i[k];
                    }
                }
                for k in i..=m {
                    u_col_i[k] *= scale;
                }
            }
            restore_u_column(u, u_col_i, i, m);
        }
        sig[i] = scale * g;
        g = 0.0;
        scale = 0.0;
        if i <= m && i != n {
            scale = vectorized::sum_abs(&u[i], l, n + 1);
            if scale.abs() > eps {
                let sum = vectorized::normalize_and_sum_sq(&mut u[i], l, n + 1, scale);
                let f = u[i][l];
                g = -copy_sign(sum.sqrt(), f);
                u[i][l] = f - g;
                let h = 1.0 / (f * g - sum);
                for k in l..=n {
                    rv1[k] = u[i][k] * h;
                }
                for j in l..=m {
                    let s = vectorized::dot_product(&u[i], &u[j], l, n + 1);
                    for k in l..=n {
                        u[j][k] += s * rv1[k];
                    }
                }
                for k in l..=n {
                    u[i][k] *= scale;
                }
            }
        }
        let mut a = sig[i].abs();
        let rvi = rv1[i];
        if rvi != 0.0 {
            a += rvi.abs();
        }
        if anorm == 0.0 || a > anorm {
            anorm = a;
        }
    }
    anorm
}

/// `RightTransform(U, V, rv1, n)`
pub fn right_transform(u: &[Vec<f64>], v: &mut [Vec<f64>], rv1: &[f64], n: usize) {
    let mut l = n + 1;
    let mut g = 0.0;
    for i in (0..=n).rev() {
        if i < n {
            if g != 0.0 {
                g = 1.0 / g;
                let u_i = &u[i];
                for j in l..=n {
                    v[i][j] = (u_i[j] / u_i[l]) * g;
                }
                for j in l..=n {
                    let sum = vectorized::dot_product(u_i, &v[j], l, n + 1);
                    for k in l..=n {
                        let vik = v[i][k];
                        v[j][k] += sum * vik;
                    }
                }
            }
            for j in l..=n {
                v[i][j] = 0.0;
                v[j][i] = 0.0;
            }
        }
        v[i][i] = 1.0;
        g = rv1[i];
        l = i;
    }
}

/// `LeftTransform(U, sig, U_col_i, m, n)`
pub fn left_transform(u: &mut [Vec<f64>], sig: &[f64], u_col_i: &mut [f64], m: usize, n: usize) {
    for i in (0..=m.min(n)).rev() {
        let l = i + 1;
        let mut g = sig[i];
        let u_ii = u[i][i];
        for j in l..=n {
            u[i][j] = 0.0;
        }
        cache_u_column(u, u_col_i, i, m);
        if g != 0.0 {
            g = 1.0 / g;
            if l <= n {
                let u_li = u_col_i[l];
                for j in l..=n {
                    let mut sum = u_li * u[l][j];
                    for k in l + 1..=m {
                        sum += u_col_i[k] * u[k][j];
                    }
                    let f = (sum / u_ii) * g;
                    for k in i..=m {
                        u[k][j] += f * u_col_i[k];
                    }
                }
            }
            for j in i..=m {
                u_col_i[j] *= g;
            }
        } else {
            for j in i..=m {
                u_col_i[j] = 0.0;
            }
        }
        u_col_i[i] += 1.0;
        restore_u_column(u, u_col_i, i, m);
    }
}

/// `Diagonalize(U, sig, V, rv1, m, n, anorm)`
pub fn diagonalize(
    u: &mut [Vec<f64>],
    sig: &mut [f64],
    v: &mut [Vec<f64>],
    rv1: &mut [f64],
    m: usize,
    n: usize,
    anorm: f64,
) -> CResult<()> {
    let mut g;
    for k in (0..=n).rev() {
        for iter in 1..=30 {
            let mut flag = 1;
            let nm0 = 0usize;
            let mut l: i64 = k as i64;
            while l >= 0 {
                let lu = l as usize;
                if rv1[lu] == 0.0 || (rv1[lu].abs() + anorm) == anorm {
                    flag = 0;
                    break;
                }
                // C# tests sig[nm] with nm still 0 here
                if (sig[nm0].abs() + anorm) == anorm {
                    break;
                }
                l -= 1;
            }
            let nm = l - 1;
            let mut f;
            let mut h;
            let mut c;
            let mut s;
            if flag != 0 {
                c = 0.0;
                s = 1.0;
                let mut i = l;
                while i <= k as i64 {
                    let iu = i as usize;
                    f = s * rv1[iu];
                    rv1[iu] *= c;
                    if (f.abs() + anorm) == anorm {
                        break;
                    }
                    g = sig[iu];
                    h = hypot(f, g);
                    sig[iu] = h;
                    h = 1.0 / h;
                    c = g * h;
                    s = -f * h;
                    if nm < 0 {
                        return Err(MathParserException::unexpected(
                            "Index was outside the bounds of the array.",
                        ));
                    }
                    let nmu = nm as usize;
                    for row in u.iter_mut().take(m + 1) {
                        let uy = row[nmu];
                        let uz = row[iu];
                        row[nmu] = uy * c + uz * s;
                        row[iu] = uz * c - uy * s;
                    }
                    i += 1;
                }
            }
            let mut z = sig[k];
            if l == k as i64 {
                if z < 0.0 {
                    sig[k] = -z;
                    for x in v[k].iter_mut().take(n + 1) {
                        *x = -*x;
                    }
                }
                break;
            }
            if iter == 30 {
                return Err(MathParserException::new("no convergence in 30 SVD iterations."));
            }
            let lu = if l < 0 {
                return Err(MathParserException::unexpected(
                    "Index was outside the bounds of the array.",
                ));
            } else {
                l as usize
            };
            let mut x = sig[lu];
            let nm = k - 1;
            let mut y = sig[nm];
            g = rv1[nm];
            h = rv1[k];
            if g == 0.0 {
                f = ((y - z) * (y + z) - h * h) / (h * y * 2.0);
            } else {
                f = ((y - z) * (y + z) + (g - h) * (g + h)) / (h * y * 2.0);
            }
            g = hypot(f, 1.0);
            f = ((x - z) * (x + z) + h * ((y / (f + copy_sign(g, f))) - h)) / x;
            c = 1.0;
            s = 1.0;
            for j in lu..=nm {
                let i = j + 1;
                g = rv1[i];
                y = sig[i];
                h = s * g;
                g *= c;
                z = hypot(f, h);
                rv1[j] = z;
                c = f / z;
                s = h / z;
                f = x * c + g * s;
                g = g * c - x * s;
                h = y * s;
                y *= c;
                for jj in 0..=n {
                    let xv = v[j][jj];
                    let zv = v[i][jj];
                    v[j][jj] = xv * c + zv * s;
                    v[i][jj] = zv * c - xv * s;
                }
                z = hypot(f, h);
                sig[j] = z;
                if z != 0.0 {
                    z = 1.0 / z;
                    c = f * z;
                    s = h * z;
                }
                f = c * g + s * y;
                x = c * y - s * g;
                for row in u.iter_mut().take(m + 1) {
                    let yv = row[j];
                    let zv = row[i];
                    row[j] = yv * c + zv * s;
                    row[i] = zv * c - yv * s;
                }
            }
            rv1[lu] = 0.0;
            rv1[k] = f;
            sig[k] = x;
        }
    }
    Ok(())
}
