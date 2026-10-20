//! Checks of the HP vectors and matrices: the results are compared with the non-HP
//! Vector/Matrix implementations on the same data (as the upstream HP comparison tests do)
//! and the numerical members are checked by reconstruction.
use super::*;
use crate::base_types::value::BinOp;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// A deterministic pseudo random sequence in [-5, 5).
fn data(n: usize, seed: u64) -> Vec<f64> {
    let mut s = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 11) as f64 / (1u64 << 53) as f64) * 10.0 - 5.0
        })
        .collect()
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    (a - b).abs() <= tol * (1.0 + a.abs().max(b.abs()))
}

fn assert_vec_close(hp: &Vector, v: &Vector, tol: f64) {
    assert_eq!(hp.len(), v.len(), "length");
    for i in 0..v.len() {
        let (x, y) = (hp.get(i).d, v.get(i).d);
        assert!(close(x, y, tol), "element {}: {} != {}", i, x, y);
    }
}

fn assert_mat_close(hp: &Matrix, m: &Matrix, tol: f64) {
    assert_eq!(hp.row_count(), m.row_count(), "rows");
    assert_eq!(hp.col_count(), m.col_count(), "cols");
    for i in 0..m.row_count() {
        for j in 0..m.col_count() {
            let (x, y) = (hp.get(i, j).d, m.get(i, j).d);
            assert!(close(x, y, tol), "element ({}, {}): {} != {}", i, j, x, y);
        }
    }
}

fn dense(v: &[f64]) -> Vector {
    Vector::from_values(v.iter().map(|&x| RealValue::num(x)).collect()).unwrap()
}

fn hp_vec(v: &[f64]) -> Vector {
    hp_vector::from_doubles(v.to_vec(), None).unwrap()
}

fn full(rows: usize, cols: usize, v: &[f64]) -> Matrix {
    let m = Matrix::new(rows, cols).unwrap();
    for i in 0..rows {
        for j in 0..cols {
            m.set(i, j, RealValue::num(v[i * cols + j])).unwrap();
        }
    }
    m
}

fn hp_full(rows: usize, cols: usize, v: &[f64]) -> Matrix {
    from_jagged((0..rows).map(|i| v[i * cols..(i + 1) * cols].to_vec()).collect(), None).unwrap()
}

/// A * B with plain loops
fn naive_product(a: &Matrix, b: &Matrix) -> Vec<f64> {
    let (m, n, p) = (a.row_count(), a.col_count(), b.col_count());
    let mut c = vec![0.0; m * p];
    for i in 0..m {
        for j in 0..p {
            let mut s = 0.0;
            for k in 0..n {
                s += a.get(i, k).d * b.get(k, j).d;
            }
            c[i * p + j] = s;
        }
    }
    c
}

#[test]
fn vector_operators_match_the_non_hp_vectors() {
    for &n in &[5usize, 16, 23, 64] {
        let a = data(n, 1);
        let mut b = data(n, 2);
        b[n / 2] = 0.5; // keep the divisors away from zero
        let (ha, hb, da, db) = (hp_vec(&a), hp_vec(&b), dense(&a), dense(&b));
        let r = RealValue::num(1.75);
        for op in [
            BinOp::Add,
            BinOp::Sub,
            BinOp::Mul,
            BinOp::Div,
            BinOp::Lt,
            BinOp::Gt,
            BinOp::Le,
            BinOp::Ge,
            BinOp::Eq,
            BinOp::Ne,
            BinOp::And,
            BinOp::Or,
            BinOp::Xor,
        ] {
            let hv = hp_vector::ops::op_vv(op, &ha, &hb).unwrap();
            let dv = crate::base_types::value::vector_op_vv(op, &da, &db).unwrap();
            assert_vec_close(&hv, &dv, 1e-14);
            let hv = hp_vector::ops::op_vr(op, &ha, &r).unwrap();
            let dv = crate::base_types::value::vector_op_vr(op, &da, &r).unwrap();
            assert_vec_close(&hv, &dv, 1e-14);
            let hv = hp_vector::ops::op_rv(op, &r, &hb).unwrap();
            let dv = crate::base_types::value::vector_op_rv(op, &r, &db).unwrap();
            assert_vec_close(&hv, &dv, 1e-14);
        }
        // The SIMD remainder can differ from fmod in the last bits.
        let hv = hp_vector::ops::op_vv(BinOp::Rem, &ha, &hb).unwrap();
        let dv = crate::base_types::value::vector_op_vv(BinOp::Rem, &da, &db).unwrap();
        assert_vec_close(&hv, &dv, 1e-11);
    }
}

#[test]
fn vector_aggregates_match_the_non_hp_vectors() {
    for &n in &[3usize, 15, 16, 17, 100] {
        let a = data(n, 3);
        let (h, d) = (hp_vec(&a), dense(&a));
        let pairs: Vec<(f64, f64)> = vec![
            (h.sum().unwrap().d, d.sum().unwrap().d),
            (h.sum_sq().unwrap().d, d.sum_sq().unwrap().d),
            (h.norm().unwrap().d, d.norm().unwrap().d),
            (h.l1_norm().unwrap().d, d.l1_norm().unwrap().d),
            (h.inf_norm().unwrap().d, d.inf_norm().unwrap().d),
            (h.lp_norm(3).unwrap().d, d.lp_norm(3).unwrap().d),
            (h.min().unwrap().d, d.min().unwrap().d),
            (h.max().unwrap().d, d.max().unwrap().d),
            (h.average().unwrap().d, d.average().unwrap().d),
            (h.product().unwrap().d, d.product().unwrap().d),
            (h.srss().unwrap().d, d.srss().unwrap().d),
        ];
        for (k, (x, y)) in pairs.iter().enumerate() {
            assert!(close(*x, *y, 1e-13), "n = {}, aggregate {}: {} != {}", n, k, x, y);
        }
        assert_vec_close(&h.sort(false).unwrap(), &d.sort(false).unwrap(), 0.0);
        assert_vec_close(&h.sort(true).unwrap(), &d.sort(true).unwrap(), 0.0);
        assert_vec_close(&h.order(false).unwrap(), &d.order(false).unwrap(), 0.0);
        assert_vec_close(&h.reverse().unwrap(), &d.reverse().unwrap(), 0.0);
        assert_vec_close(
            &h.slice(2, n as i64 - 1).unwrap(),
            &d.slice(2, n as i64 - 1).unwrap(),
            0.0,
        );
        assert_vec_close(&h.first(2).unwrap(), &d.first(2).unwrap(), 0.0);
        assert_vec_close(&h.last(2).unwrap(), &d.last(2).unwrap(), 0.0);
        let dot_h = hp_vector::dot_product(&h, &h).unwrap().d;
        let dot_d = Vector::dot_product(&d, &d).unwrap().d;
        assert!(close(dot_h, dot_d, 1e-13));
    }
}

#[test]
fn simd_sum_order_is_reproduced() {
    // 16 elements: four lane accumulators, then (l0 + l1) + (l2 + l3)
    let a: Vec<f64> = (0..16).map(|i| 1.0 + (i as f64) * 1e-16 * 3.0).collect();
    let h = hp_vec(&a);
    let mut lanes = [0.0; 4];
    lanes.copy_from_slice(&a[..4]);
    for blk in 1..4 {
        for k in 0..4 {
            lanes[k] += a[blk * 4 + k];
        }
    }
    let expected = (lanes[0] + lanes[1]) + (lanes[2] + lanes[3]);
    assert_eq!(h.sum().unwrap().d.to_bits(), expected.to_bits());
}

#[test]
fn matrix_operators_match_the_non_hp_matrices() {
    let (m, n) = (7, 5);
    let a = data(m * n, 4);
    let b = data(m * n, 5);
    let (ha, hb, da, db) = (hp_full(m, n, &a), hp_full(m, n, &b), full(m, n, &a), full(m, n, &b));
    let r = RealValue::num(-2.5);
    for op in [
        BinOp::Add,
        BinOp::Sub,
        BinOp::Div,
        BinOp::Lt,
        BinOp::Ge,
        BinOp::Eq,
        BinOp::And,
        BinOp::Xor,
    ] {
        let hm = ops::op_mm(op, &ha, &hb).unwrap();
        let dm = crate::base_types::value::matrix_op_mm(op, &da, &db).unwrap();
        assert_mat_close(&hm, &dm, 1e-14);
        let hm = ops::op_mr(op, &ha, &r).unwrap();
        let dm = crate::base_types::value::matrix_op_mr(op, &da, &r).unwrap();
        assert_mat_close(&hm, &dm, 1e-14);
        let hm = ops::op_rm(op, &r, &hb).unwrap();
        let dm = crate::base_types::value::matrix_op_rm(op, &r, &db).unwrap();
        assert_mat_close(&hm, &dm, 1e-14);
    }
    assert_mat_close(&ha.transpose().unwrap(), &da.transpose().unwrap(), 0.0);
    assert!(close(ha.sum().unwrap().d, da.sum().unwrap().d, 1e-13));
    assert!(close(ha.sum_sq().unwrap().d, da.sum_sq().unwrap().d, 1e-13));
    assert!(close(ha.l1_norm().unwrap().d, da.l1_norm().unwrap().d, 1e-13));
    assert!(close(ha.inf_norm().unwrap().d, da.inf_norm().unwrap().d, 1e-13));
    assert!(close(ha.min().unwrap().d, da.min().unwrap().d, 0.0));
    assert!(close(ha.max().unwrap().d, da.max().unwrap().d, 0.0));
    assert_mat_close(
        &ha.submatrix(2, 4, 1, 3).unwrap(),
        &da.submatrix(2, 4, 1, 3).unwrap(),
        0.0,
    );
    let had = hadamard(&ha, &hb).unwrap();
    assert_mat_close(&had, &Matrix::hadamard(&da, &db).unwrap(), 1e-14);
    let fro = frobenius(&ha, &hb).unwrap().d;
    assert!(close(fro, Matrix::frobenius(&da, &db).unwrap().d, 1e-13));
}

#[test]
fn products_match_the_naive_product() {
    // 7 x 5 times 5 x 3 (general), 12 x 12 (Winograd below the kernel size),
    // 70 x 70 (padded to 128: recursion with the 64 x 64 FMA kernel)
    for &(m, n, p) in &[(7usize, 5usize, 3usize), (12, 12, 12), (70, 70, 70)] {
        let a = hp_full(m, n, &data(m * n, 6));
        let b = hp_full(n, p, &data(n * p, 7));
        let c = mul(&a, &b).unwrap();
        let expected = naive_product(&a, &b);
        for i in 0..m {
            for j in 0..p {
                assert!(
                    close(c.get(i, j).d, expected[i * p + j], 1e-11),
                    "({}, {}) of {}x{}",
                    i,
                    j,
                    m,
                    p
                );
            }
        }
        let da = full(m, n, &data(m * n, 6));
        let db = full(n, p, &data(n * p, 7));
        assert_mat_close(&c, &Matrix::mul(&da, &db).unwrap(), 1e-11);
    }
}

/// A well conditioned matrix: random plus a large diagonal.
fn test_matrix(n: usize, seed: u64) -> Vec<f64> {
    let mut a = data(n * n, seed);
    for i in 0..n {
        a[i * n + i] += 3.0 * n as f64;
    }
    a
}

fn matvec(a: &Matrix, x: &Vector) -> Vec<f64> {
    (0..a.row_count())
        .map(|i| (0..a.col_count()).map(|j| a.get(i, j).d * x.get(j).d).sum())
        .collect()
}

#[test]
fn lu_solve_and_inverse_reconstruct() {
    for &n in &[4usize, 30] {
        let a = hp_full(n, n, &test_matrix(n, 8));
        let b = hp_vec(&data(n, 9));
        let x = numerics::lsolve(&a, &b).unwrap();
        let ax = matvec(&a, &x);
        for i in 0..n {
            assert!(close(ax[i], b.get(i).d, 1e-12), "A x = b at {}", i);
        }
        let inv = numerics::invert(&a).unwrap();
        let p = mul(&a, &inv).unwrap();
        for i in 0..n {
            for j in 0..n {
                let e = if i == j { 1.0 } else { 0.0 };
                assert!(close(p.get(i, j).d, e, 1e-12), "A inv(A) at ({}, {})", i, j);
            }
        }
        // det(A) * det(inv(A)) = 1
        let d1 = numerics::determinant(&a).unwrap().d;
        let d2 = numerics::determinant(&inv).unwrap().d;
        assert!(close(d1 * d2, 1.0, 1e-10));
        let bm = hp_full(n, 2, &data(2 * n, 10));
        let xm = numerics::msolve(&a, &bm).unwrap();
        let axm = mul(&a, &xm).unwrap();
        assert_mat_close(&axm, &bm, 1e-11);
    }
}

fn symmetric(n: usize, seed: u64) -> Matrix {
    let v = test_matrix(n, seed);
    let m = new_symmetric(n, None).unwrap();
    for i in 0..n {
        for j in i..n {
            set_value(&m, (v[i * n + j] + v[j * n + i]) / 2.0, i, j).unwrap();
        }
    }
    m
}

#[test]
fn symmetric_solvers_reconstruct() {
    for &n in &[5usize, 40] {
        let a = symmetric(n, 11);
        let b = hp_vec(&data(n, 12));
        for (x, tol) in [
            (numerics::lsolve(&a, &b).unwrap(), 1e-11),
            (numerics::cl_solve(&a, &b).unwrap(), 1e-11),
            // PCG stops when |r| < tol; C# rejects |p.Ap| < 1e-15 (absolute), which a
            // tighter tolerance reaches on this matrix
            (numerics::sl_solve(&a, &b, 1e-6).unwrap(), 1e-6),
        ] {
            let ax = matvec(&a, &x);
            for i in 0..n {
                assert!(close(ax[i], b.get(i).d, tol), "A x = b at {}", i);
            }
        }
        // Cholesky: U^T U = A
        let u = numerics::cholesky_decomposition(&a).unwrap();
        for i in 0..n {
            for j in 0..n {
                let s: f64 = (0..n).map(|k| u.get(k, i).d * u.get(k, j).d).sum();
                assert!(close(s, a.get(i, j).d, 1e-12), "U^T U at ({}, {})", i, j);
            }
        }
        let inv = numerics::invert(&a).unwrap();
        let p = mul(&a, &inv).unwrap();
        for i in 0..n {
            for j in 0..n {
                let e = if i == j { 1.0 } else { 0.0 };
                assert!(close(p.get(i, j).d, e, 1e-12));
            }
        }
        // Eigen pairs: A v = lambda v
        let vals = numerics::eigen_values(&a, 0, 1e-12).unwrap();
        let vecs = numerics::eigen_vectors(&a, 0, 1e-12).unwrap();
        for k in 0..n {
            let lambda = vals.get(k).d;
            let v: Vector = hp_vec(&(0..n).map(|j| vecs.get(k, j).d).collect::<Vec<_>>());
            let av = matvec(&a, &v);
            for i in 0..n {
                assert!(close(av[i], lambda * v.get(i).d, 1e-10), "eigen pair {} at {}", k, i);
            }
        }
    }
}

#[test]
fn svd_and_qr_reconstruct() {
    let (m, n) = (8, 5);
    let a = hp_full(m, n, &data(m * n, 13));
    let usv = numerics::svd_decomposition(&a).unwrap();
    // [U | sig | V]
    assert_eq!(usv.col_count(), n + 1 + n);
    for i in 0..m {
        for j in 0..n {
            let s: f64 = (0..n)
                .map(|k| usv.get(i, k).d * usv.get(k, n).d * usv.get(k, n + 1 + j).d)
                .sum();
            assert!(close(s, a.get(i, j).d, 1e-12), "U S V^T at ({}, {})", i, j);
        }
    }
    let qr = numerics::qr_decomposition(&a).unwrap();
    // [Q | R], Q is m x m
    for i in 0..m {
        for j in 0..n {
            let s: f64 = (0..m).map(|k| qr.get(i, k).d * qr.get(k, m + j).d).sum();
            assert!(close(s, a.get(i, j).d, 1e-12), "Q R at ({}, {})", i, j);
        }
    }
    let r = numerics::rank(&a).unwrap().d;
    assert_eq!(r, n as f64);
}

#[test]
fn fft_inverse_round_trip() {
    let v = data(8, 14);
    let m = hp_full(1, 8, &v);
    let f = fft(&m, false).unwrap();
    let g = fft(&f, true).unwrap();
    for j in 0..8 {
        assert!(close(g.get(0, j).d, v[j], 1e-14));
        assert!(close(g.get(1, j).d, 0.0, 1e-14));
    }
}

#[test]
fn triangular_and_diagonal_solvers_reconstruct() {
    let n = 6;
    let v = test_matrix(n, 15);
    let l = new_lower(n, None).unwrap();
    let u = new_upper(n, None).unwrap();
    let d = new_diagonal(n, None).unwrap();
    for i in 0..n {
        for j in 0..=i {
            set_value(&l, v[i * n + j], i, j).unwrap();
            set_value(&u, v[j * n + i], j, i).unwrap();
        }
        set_value(&d, v[i * n + i], i, i).unwrap();
    }
    let b = hp_vec(&data(n, 16));
    for a in [&l, &u, &d] {
        let x = numerics::lsolve(a, &b).unwrap();
        let ax = matvec(a, &x);
        for i in 0..n {
            assert!(close(ax[i], b.get(i).d, 1e-12), "{:?} A x = b at {}", a.mtype(), i);
        }
        let inv = numerics::invert(a).unwrap();
        let p = mul(a, &inv).unwrap();
        for i in 0..n {
            for j in 0..n {
                let e = if i == j { 1.0 } else { 0.0 };
                assert!(
                    close(p.get(i, j).d, e, 1e-12),
                    "{:?} A inv(A) at ({}, {})",
                    a.mtype(),
                    i,
                    j
                );
            }
        }
    }
}

#[test]
fn lanczos_eigen_pairs_and_pcg_matrix_solve() {
    let n = 210; // above 200 with few eigenvalues: Lanczos
    let a = symmetric(n, 17);
    let count = 3;
    let vals = numerics::eigen_values(&a, count, 1e-10).unwrap();
    let vecs = numerics::eigen_vectors(&a, count, 1e-10).unwrap();
    assert_eq!(vals.len(), count as usize);
    for k in 0..count as usize {
        let lambda = vals.get(k).d;
        let v = hp_vec(&(0..n).map(|j| vecs.get(k, j).d).collect::<Vec<_>>());
        let av = matvec(&a, &v);
        for i in 0..n {
            assert!(close(av[i], lambda * v.get(i).d, 1e-6), "Lanczos pair {} at {}", k, i);
        }
    }
    // the direct path gives the same smallest eigenvalues
    let a_small = symmetric(30, 18);
    let direct = numerics::eigen_values(&a_small, 4, 1e-12).unwrap();
    let all = numerics::eigen_values(&a_small, 0, 1e-12).unwrap();
    for k in 0..4 {
        assert_eq!(direct.get(k).d.to_bits(), all.get(k).d.to_bits());
    }
    let b = hp_full(30, 3, &data(90, 19));
    let x = numerics::sm_solve(&a_small, &b, 1e-6).unwrap();
    assert_mat_close(&mul(&a_small, &x).unwrap(), &b, 1e-6);
    let x = numerics::cm_solve(&a_small, &b).unwrap();
    assert_mat_close(&mul(&a_small, &x).unwrap(), &b, 1e-11);
}

#[test]
fn conversion_from_non_hp_matrices() {
    let v = data(16, 20);
    let d = full(4, 4, &v);
    let h = from_matrix(&d).unwrap();
    assert!(h.is_hp());
    assert_mat_close(&h, &d, 0.0);
    let s = crate::matrix::symmetric_matrix::new(3);
    for i in 0..3 {
        for j in i..3 {
            s.set(i, j, RealValue::num((i + 2 * j) as f64)).unwrap();
        }
    }
    let hs = from_matrix(&s).unwrap();
    assert_eq!(hs.mtype(), MatrixType::Symmetric);
    assert_mat_close(&hs, &s, 0.0);
}
