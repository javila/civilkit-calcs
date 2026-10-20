//! Port of Matrix/HpMatrix/HpMatmulWinograd.cs: Strassen-Winograd multiplication of square
//! HP matrices padded to a power of two, with a 64 x 64 FMA kernel (AVX2 and AVX-512 give
//! the same fused results) and a plain multiply-add kernel below 64.
use super::{units, vectorized, HpMat};
use crate::base_types::unit::Unit;
use crate::math_parser_exception::CResult;
use crate::matrix::{Matrix, MatrixType};
use crate::vector::HpData;

const KERNEL_SIZE: usize = 64;

/// A view of a square block of a flat row major array.
#[derive(Clone, Copy)]
struct View {
    offset: usize,
    n: usize,
    stride: usize,
}

impl View {
    fn slice(self, r: usize, c: usize, n: usize) -> View {
        View {
            offset: self.offset + r * self.stride + c,
            n,
            stride: self.stride,
        }
    }
    #[inline]
    fn at(self, i: usize, j: usize) -> usize {
        self.offset + i * self.stride + j
    }
}

fn next_power_of_two(n: usize) -> usize {
    let mut p = 1;
    while p < n {
        p *= 2;
    }
    p
}

/// `HpMatmulWinograd.Multiply(A, B)`
pub fn multiply(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let aa = HpMat::from(a);
    let bb = HpMat::from(b);
    let n = aa.row_count;
    let m = next_power_of_two(n);
    let l = m * m;
    let mut af = vec![0.0; l];
    let mut bf = vec![0.0; l];
    let mut cf = vec![0.0; l];
    copy_to_flat(&aa, &mut af, n, m);
    copy_to_flat(&bb, &mut bf, n, m);
    let v = View {
        offset: 0,
        n: m,
        stride: m,
    };
    winograd_recursive(&af, v, &bf, v, &mut cf, v, 0);
    let (unit, d) = Unit::multiply(&units(a), &units(b), true);
    vectorized::scale(&mut cf, d);
    // HpMatrix.FromFlatArray(CFlat, unit, n, m)
    let mut c = HpMat::empty(MatrixType::Full, n, n, unit.clone());
    for i in 0..n {
        let mut row = HpData::new(n, n, unit.clone());
        row.values.as_mut().unwrap().copy_from_slice(&cf[i * m..i * m + n]);
        c.rows[i] = row;
    }
    Ok(c.into_matrix())
}

fn copy_to_flat(src: &HpMat, dst: &mut [f64], n: usize, stride: usize) {
    let mut ii = 0;
    if src.mtype == MatrixType::Full || src.mtype == MatrixType::LowerTriangular {
        for i in 0..n {
            let row = &src.rows[i];
            dst[ii..ii + row.size].copy_from_slice(&row.raw_all()[..row.size]);
            ii += stride;
        }
    } else {
        for i in 0..n {
            for j in 0..n {
                dst[ii + j] = src.get_value(i, j);
            }
            ii += stride;
        }
    }
}

fn op2(a: &[f64], va: View, b: &[f64], vb: View, c: &mut [f64], vc: View, f: fn(f64, f64) -> f64) {
    for i in 0..va.n {
        for j in 0..va.n {
            c[vc.at(i, j)] = f(a[va.at(i, j)], b[vb.at(i, j)]);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn winograd_recursive(a: &[f64], va: View, b: &[f64], vb: View, c: &mut [f64], vc: View, level: usize) {
    let n = va.n;
    if n == KERNEL_SIZE {
        fma_kernel(a, va, b, vb, c, vc);
        return;
    }
    if n < KERNEL_SIZE {
        multiply_simd(a, va, b, vb, c, vc);
        return;
    }
    let half = n / 2;
    let hl = half * half;
    let t = View {
        offset: 0,
        n: half,
        stride: half,
    };
    let (a11, a12, a21, a22) = (
        va.slice(0, 0, half),
        va.slice(0, half, half),
        va.slice(half, 0, half),
        va.slice(half, half, half),
    );
    let (b11, b12, b21, b22) = (
        vb.slice(0, 0, half),
        vb.slice(0, half, half),
        vb.slice(half, 0, half),
        vb.slice(half, half, half),
    );
    let (c11, c12, c21, c22) = (
        vc.slice(0, 0, half),
        vc.slice(0, half, half),
        vc.slice(half, 0, half),
        vc.slice(half, half, half),
    );
    let mut a22m = vec![0.0; hl];
    let mut b22m = vec![0.0; hl];
    // A22Mod = A12 - A21 + A22, B22Mod = B12 - B21 + B22
    for i in 0..half {
        for j in 0..half {
            a22m[t.at(i, j)] = a[a12.at(i, j)] - a[a21.at(i, j)] + a[a22.at(i, j)];
            b22m[t.at(i, j)] = b[b12.at(i, j)] - b[b21.at(i, j)] + b[b22.at(i, j)];
        }
    }
    let next = level + 1;
    let mut t1 = vec![0.0; hl];
    let mut t2 = vec![0.0; hl];
    let mut t3 = vec![0.0; hl];
    let mut t4 = vec![0.0; hl];
    let mut t5 = vec![0.0; hl];
    let mut t6 = vec![0.0; hl];
    let add = |x: f64, y: f64| x + y;
    let sub = |x: f64, y: f64| x - y;
    op2(a, a21, &a22m, t, &mut t1, t, add);
    op2(&a22m, t, a, a12, &mut t2, t, sub);
    op2(&a22m, t, a, a11, &mut t3, t, sub);
    op2(&b22m, t, b, b11, &mut t4, t, sub);
    op2(b, b21, &b22m, t, &mut t5, t, add);
    op2(&b22m, t, b, b12, &mut t6, t, sub);
    let mut m1 = vec![0.0; hl];
    let mut m2 = vec![0.0; hl];
    let mut m3 = vec![0.0; hl];
    let mut m4 = vec![0.0; hl];
    let mut m5 = vec![0.0; hl];
    let mut m6 = vec![0.0; hl];
    let mut m7 = vec![0.0; hl];
    winograd_recursive(a, a11, b, b11, &mut m1, t, next);
    winograd_recursive(a, a12, b, b21, &mut m2, t, next);
    winograd_recursive(a, a21, &t4, t, &mut m3, t, next);
    winograd_recursive(&a22m, t, &b22m, t, &mut m4, t, next);
    winograd_recursive(&t1, t, &t5, t, &mut m5, t, next);
    winograd_recursive(&t2, t, &t6, t, &mut m6, t, next);
    winograd_recursive(&t3, t, b, b12, &mut m7, t, next);
    for i in 0..half {
        for j in 0..half {
            let k = t.at(i, j);
            // C11 = M1 + M2
            c[c11.at(i, j)] = m1[k] + m2[k];
            // C22 = M5 + M6 - M2 - M4
            let c22v = m5[k] + m6[k] - m2[k] - m4[k];
            c[c22.at(i, j)] = c22v;
            // C12 = M5 - M7; C12 -= C22
            c[c12.at(i, j)] = (m5[k] - m7[k]) - c22v;
            // C21 = M3 + M6; C21 = C22 - C21
            c[c21.at(i, j)] = c22v - (m3[k] + m6[k]);
        }
    }
}

/// `MultiplySimd`: C[i][j] = sum over k of A[i][k] * B[k][j], accumulated in order of k.
fn multiply_simd(a: &[f64], va: View, b: &[f64], vb: View, c: &mut [f64], vc: View) {
    let n = va.n;
    for i in 0..n {
        for j in 0..n {
            c[vc.at(i, j)] = 0.0;
        }
        for k in 0..n {
            let a_ik = a[va.at(i, k)];
            for j in 0..n {
                c[vc.at(i, j)] += a_ik * b[vb.at(k, j)];
            }
        }
    }
}

/// `MultiplyFmaKernel_64x64`: fused multiply-add accumulation from zero over k.
fn fma_kernel(a: &[f64], va: View, b: &[f64], vb: View, c: &mut [f64], vc: View) {
    let n = KERNEL_SIZE;
    let mut acc = [0.0f64; KERNEL_SIZE];
    for i in 0..n {
        for x in acc.iter_mut() {
            *x = 0.0;
        }
        for k in 0..n {
            let a_ik = a[va.at(i, k)];
            for (j, x) in acc.iter_mut().enumerate() {
                *x = a_ik.mul_add(b[vb.at(k, j)], *x);
            }
        }
        for (j, x) in acc.iter().enumerate() {
            c[vc.at(i, j)] = *x;
        }
    }
}
