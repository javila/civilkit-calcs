//! The engine's dense linear algebra behind a single synchronous call, for the Python blocks of a worksheet.
//!
//! The Python side (`numpy.linalg`, `scipy.linalg` in the worksheet's light stand-ins) cannot run LU, Cholesky or an
//! eigen solver at a useful speed, and the engine already has them. Nothing is computed here that the engine does not
//! compute: this crate only turns a JSON request into engine `Matrix` values and the answer back into JSON.
//!
//! Request: `{"op": "...", "a": T, "b": T, ...}`. Answer: `{"ok": true, "out": [T, ..]}` or `{"ok": false, "error": "text"}`.
//! A tensor `T` is `{"shape": [..], "hex": "..."}`: the values in row-major order as little-endian 64-bit floats, written in
//! hexadecimal (a Python block packs and unpacks that at C speed; writing 57 600 floats as decimal text took seconds). A
//! tensor in a request may also be plain nested lists of numbers. A scalar has shape `[]`.

use calcpad_core::base_types::real_value::RealValue;
use calcpad_core::math_parser_exception::CResult;
use calcpad_core::matrix::{lower_triangular_matrix, symmetric_matrix, upper_triangular_matrix, Matrix};
use calcpad_core::vector::Vector;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

/// Changes whenever the request or answer format does. The worker compares it with the value it was built for, so a
/// cached module paired with other glue is refused instead of answering differently.
pub const ABI: &str = "calcpad-linalg-2";

#[wasm_bindgen]
pub fn abi() -> String {
    ABI.to_string()
}

/// One request, one answer (both JSON text).
#[wasm_bindgen]
pub fn linalg(request: &str) -> String {
    let answer = match serde_json::from_str::<Value>(request) {
        Err(e) => json!({"ok": false, "error": format!("unreadable request: {e}")}),
        Ok(req) => match run(&req) {
            Ok(out) => json!({"ok": true, "out": out}),
            Err(e) => json!({"ok": false, "error": e}),
        },
    };
    answer.to_string()
}

type R<T> = Result<T, String>;

fn eng<T>(r: CResult<T>) -> R<T> {
    r.map_err(|e| e.message)
}

fn unhex(s: &str) -> R<Vec<f64>> {
    let b = s.as_bytes();
    if b.len() % 16 != 0 {
        return Err("unreadable matrix data".to_string());
    }
    let nib = |c: u8| -> R<u8> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err("unreadable matrix data".to_string()),
        }
    };
    let mut out = Vec::with_capacity(b.len() / 16);
    for chunk in b.chunks(16) {
        let mut bytes = [0u8; 8];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = nib(chunk[2 * i])? << 4 | nib(chunk[2 * i + 1])?;
        }
        out.push(f64::from_le_bytes(bytes));
    }
    Ok(out)
}

fn hex(data: &[f64]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(data.len() * 16);
    for x in data {
        for byte in x.to_le_bytes() {
            s.push(DIGITS[(byte >> 4) as usize] as char);
            s.push(DIGITS[(byte & 15) as usize] as char);
        }
    }
    s
}

fn enc(shape: &[usize], data: &[f64]) -> Value {
    json!({"shape": shape, "hex": hex(data)})
}

fn nested(v: &Value, shape: &mut Vec<usize>, depth: usize, out: &mut Vec<f64>) -> R<()> {
    match v {
        Value::Array(items) => {
            if shape.len() == depth {
                shape.push(items.len());
            } else if shape[depth] != items.len() {
                return Err("rows of different lengths".to_string());
            }
            for x in items {
                nested(x, shape, depth + 1, out)?;
            }
            Ok(())
        }
        Value::Number(n) => {
            if shape.len() != depth {
                return Err("rows of different lengths".to_string());
            }
            out.push(n.as_f64().ok_or("a value is not a number")?);
            Ok(())
        }
        _ => Err("a value is not a number".to_string()),
    }
}

/// A request tensor: its shape and row-major values. Refuses values that are not finite, as numpy's linalg does for
/// the Python side's callers (a NaN would make every engine routine return garbage or loop).
fn tensor(v: &Value, what: &str) -> R<(Vec<usize>, Vec<f64>)> {
    let (shape, data) = if v.is_object() {
        let shape: Vec<usize> = v["shape"]
            .as_array()
            .ok_or_else(|| format!("{what} has no shape"))?
            .iter()
            .map(|x| x.as_u64().unwrap_or(0) as usize)
            .collect();
        let data = unhex(v["hex"].as_str().ok_or_else(|| format!("{what} has no data"))?)?;
        if shape.iter().product::<usize>() != data.len() {
            return Err(format!("{what}: shape and data disagree"));
        }
        (shape, data)
    } else {
        let mut shape = Vec::new();
        let mut data = Vec::new();
        nested(v, &mut shape, 0, &mut data).map_err(|e| format!("{what}: {e}"))?;
        (shape, data)
    };
    if data.iter().any(|x| !x.is_finite()) {
        return Err("Array must not contain infs or NaNs".to_string());
    }
    Ok((shape, data))
}

fn rows_of(v: &Value, what: &str) -> R<Vec<Vec<f64>>> {
    let (shape, data) = tensor(v, what)?;
    if shape.len() != 2 {
        return Err(format!("{what} must be a matrix"));
    }
    if shape[1] == 0 {
        return Ok(vec![Vec::new(); shape[0]]);
    }
    Ok(data.chunks(shape[1]).map(|r| r.to_vec()).collect())
}

/// `b` is a vector or a matrix (columns are right-hand sides). Returns the rows and whether it was a vector.
fn rhs_of(v: &Value) -> R<(Vec<Vec<f64>>, bool)> {
    let (shape, data) = tensor(v, "b")?;
    match shape.len() {
        1 => Ok((data.iter().map(|x| vec![*x]).collect(), true)),
        2 => Ok((rows_of(v, "b")?, false)),
        _ => Err("b must be a vector or a matrix".to_string()),
    }
}

fn square(a: &[Vec<f64>]) -> R<usize> {
    let n = a.len();
    if n == 0 || a[0].len() != n {
        return Err("Last 2 dimensions of the array must be square".to_string());
    }
    Ok(n)
}

fn full(a: &[Vec<f64>]) -> R<Matrix> {
    let m = eng(Matrix::new(a.len(), a.first().map_or(0, |r| r.len())))?;
    for (i, r) in a.iter().enumerate() {
        for (j, x) in r.iter().enumerate() {
            eng(m.set(i, j, RealValue::num(*x)))?;
        }
    }
    Ok(m)
}

/// A symmetric matrix read from one triangle of `a` (as LAPACK does: the other triangle is ignored).
fn symmetric(a: &[Vec<f64>], lower: bool) -> R<Matrix> {
    let n = square(a)?;
    let m = eng(symmetric_matrix::new_checked(n))?;
    for i in 0..n {
        for j in i..n {
            let x = if lower { a[j][i] } else { a[i][j] };
            eng(m.set(i, j, RealValue::num(x)))?;
        }
    }
    Ok(m)
}

/// A triangular matrix read from one triangle of `a`, transposed if `trans`.
fn triangular(a: &[Vec<f64>], lower: bool, trans: bool) -> R<Matrix> {
    let n = square(a)?;
    let as_lower = lower != trans;
    let m = eng(if as_lower {
        lower_triangular_matrix::new_checked(n)
    } else {
        upper_triangular_matrix::new_checked(n)
    })?;
    for i in 0..n {
        for j in 0..n {
            let keep = if as_lower { j <= i } else { j >= i };
            if keep {
                let x = if trans { a[j][i] } else { a[i][j] };
                eng(m.set(i, j, RealValue::num(x)))?;
            }
        }
    }
    Ok(m)
}

fn vec_of(rows: &[Vec<f64>], col: usize) -> R<Vector> {
    eng(Vector::from_values(
        rows.iter().map(|r| RealValue::num(r[col])).collect(),
    ))
}

fn grid(m: &Matrix) -> Vec<Vec<f64>> {
    (0..m.row_count())
        .map(|i| (0..m.col_count()).map(|j| m.get(i, j).d).collect())
        .collect()
}

fn mat_json(rows: &[Vec<f64>]) -> Value {
    let cols = rows.first().map_or(0, |r| r.len());
    let flat: Vec<f64> = rows.iter().flatten().copied().collect();
    enc(&[rows.len(), cols], &flat)
}

fn vec_json(v: &[f64]) -> Value {
    enc(&[v.len()], v)
}

fn num(x: f64) -> Value {
    enc(&[], &[x])
}

fn transposed(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let c = a.first().map_or(0, |r| r.len());
    (0..c).map(|j| a.iter().map(|r| r[j]).collect()).collect()
}

const MISMATCH: &str = "Input operand 1 has a mismatch in its core dimension (a and b have different sizes)";

/// Solves `m x = b` with the engine's `lsolve` (one column) or `msolve` (several). Rows of x.
fn solve_rows(m: &Matrix, b: &[Vec<f64>], is_vector: bool) -> R<Vec<Vec<f64>>> {
    if m.row_count() != b.len() {
        return Err(MISMATCH.to_string());
    }
    if is_vector {
        let x = eng(m.lsolve(&vec_of(b, 0)?))?;
        Ok((0..x.len()).map(|i| vec![x.get(i).d]).collect())
    } else {
        Ok(grid(&eng(m.msolve(&full(b)?))?))
    }
}

fn answer(x: Vec<Vec<f64>>, is_vector: bool) -> Value {
    if is_vector {
        vec_json(&x.iter().map(|r| r[0]).collect::<Vec<_>>())
    } else {
        mat_json(&x)
    }
}

fn run(req: &Value) -> R<Value> {
    let op = req["op"].as_str().ok_or("missing op")?;
    let lower = req["lower"].as_bool().unwrap_or(true);
    let a = rows_of(&req["a"], "a")?;
    match op {
        "solve" => {
            let n = square(&a)?;
            let (b, is_vec) = rhs_of(&req["b"])?;
            if b.len() != n {
                return Err(MISMATCH.to_string());
            }
            Ok(json!([answer(solve_rows(&full(&a)?, &b, is_vec)?, is_vec)]))
        }
        "tri_solve" => {
            let (b, is_vec) = rhs_of(&req["b"])?;
            let trans = req["trans"].as_bool().unwrap_or(false);
            Ok(json!([answer(
                solve_rows(&triangular(&a, lower, trans)?, &b, is_vec)?,
                is_vec
            )]))
        }
        "cho_solve" => {
            // a is the Cholesky factor c: A = c c^T (lower) or A = c^T c (upper). Two triangular solves.
            let (b, is_vec) = rhs_of(&req["b"])?;
            let y = solve_rows(&triangular(&a, lower, !lower)?, &b, is_vec)?;
            Ok(json!([answer(
                solve_rows(&triangular(&a, lower, lower)?, &y, is_vec)?,
                is_vec
            )]))
        }
        "matmul" => {
            // a (m by k) times b (k by n, or a vector of k): the engine's matrix product.
            let (b, is_vec) = rhs_of(&req["b"])?;
            if a.first().map_or(0, |r| r.len()) != b.len() {
                return Err(MISMATCH.to_string());
            }
            Ok(json!([answer(
                grid(&eng(Matrix::mul(&full(&a)?, &full(&b)?))?),
                is_vec
            )]))
        }
        "inv" => {
            square(&a)?;
            Ok(json!([mat_json(&grid(&eng(full(&a)?.invert())?))]))
        }
        "det" => {
            square(&a)?;
            Ok(json!([num(eng(full(&a)?.determinant())?.d)]))
        }
        "cholesky" => {
            let u = grid(&eng(symmetric(&a, lower)?.cholesky_decomposition())?);
            Ok(json!([mat_json(&if lower { transposed(&u) } else { u })]))
        }
        "eigh" => {
            let m = symmetric(&a, lower)?;
            let n = m.row_count() as i64;
            let w = eng(m.eigen_values(n, 1e-14))?;
            let w: Vec<f64> = (0..w.len()).map(|i| w.get(i).d).collect();
            if req["vectors"].as_bool().unwrap_or(false) {
                // The engine returns one eigenvector per row; numpy returns them as columns.
                let v = transposed(&grid(&eng(m.eigen_vectors(n, 1e-14))?));
                Ok(json!([vec_json(&w), mat_json(&v)]))
            } else {
                Ok(json!([vec_json(&w)]))
            }
        }
        "svd" => {
            // The engine decomposes tall matrices; a wide one goes through its transpose.
            let wide = a.len() < a.first().map_or(0, |r| r.len());
            let src = if wide { transposed(&a) } else { a.clone() };
            let n = src.first().map_or(0, |r| r.len());
            let d = grid(&eng(full(&src)?.svd_decomposition())?);
            let m = src.len();
            // Columns of d: u (n), singular values (1), v (n).
            let u: Vec<Vec<f64>> = d[..m].iter().map(|r| r[..n].to_vec()).collect();
            let s: Vec<f64> = d[..n].iter().map(|r| r[n]).collect();
            let v: Vec<Vec<f64>> = d[..n].iter().map(|r| r[n + 1..2 * n + 1].to_vec()).collect();
            // The engine's last block holds V^T (its rows are the right singular vectors), so for tall input
            // a = u diag(s) v. A wide input was decomposed as its transpose: a = (v^T)^T diag(s) u^T.
            if wide {
                Ok(json!([
                    mat_json(&transposed(&v)),
                    vec_json(&s),
                    mat_json(&transposed(&u))
                ]))
            } else {
                Ok(json!([mat_json(&u), vec_json(&s), mat_json(&v)]))
            }
        }
        "rank" => Ok(json!([num(eng(full(&a)?.rank())?.d)])),
        "cond" => {
            square(&a)?;
            Ok(json!([num(eng(full(&a)?.cond2())?.d)]))
        }
        other => Err(format!("unknown op {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer with every tensor in `out` turned back into plain numbers and nested lists.
    fn call(req: Value) -> Value {
        let mut ans: Value = serde_json::from_str(&linalg(&req.to_string())).unwrap();
        if let Some(out) = ans["out"].as_array().cloned() {
            ans["out"] = Value::Array(out.iter().map(plain).collect());
        }
        ans
    }

    fn plain(t: &Value) -> Value {
        let shape: Vec<usize> = t["shape"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_u64().unwrap() as usize)
            .collect();
        let data = unhex(t["hex"].as_str().unwrap()).unwrap();
        match shape.len() {
            0 => json!(data[0]),
            1 => json!(data),
            _ => json!(data.chunks(shape[1]).map(|r| r.to_vec()).collect::<Vec<_>>()),
        }
    }
    fn close(x: &Value, want: f64) -> bool {
        (x.as_f64().unwrap() - want).abs() < 1e-9
    }

    #[test]
    fn solves_a_hand_known_3x3() {
        // 2x + y - z = 8; -3x - y + 2z = -11; -2x + y + 2z = -3  ->  (2, 3, -1)
        let r = call(json!({"op":"solve","a":[[2,1,-1],[-3,-1,2],[-2,1,2]],"b":[8,-11,-3]}));
        assert_eq!(r["ok"], true, "{r}");
        let x = &r["out"][0];
        assert!(close(&x[0], 2.0) && close(&x[1], 3.0) && close(&x[2], -1.0), "{x}");
    }

    #[test]
    fn solves_for_several_right_hand_sides() {
        let r = call(json!({"op":"solve","a":[[2,0],[0,4]],"b":[[2,4],[8,12]]}));
        assert!(close(&r["out"][0][0][0], 1.0) && close(&r["out"][0][1][1], 3.0), "{r}");
    }

    #[test]
    fn singular_matrix_is_refused_with_the_engine_message() {
        let r = call(json!({"op":"solve","a":[[1,2],[2,4]],"b":[1,2]}));
        assert_eq!(r["ok"], false, "{r}");
        assert!(r["error"].as_str().unwrap().to_lowercase().contains("singular"), "{r}");
    }

    #[test]
    fn inverse_and_determinant() {
        let r = call(json!({"op":"inv","a":[[4,7],[2,6]]}));
        assert!(close(&r["out"][0][0][0], 0.6) && close(&r["out"][0][0][1], -0.7), "{r}");
        let d = call(json!({"op":"det","a":[[4,7],[2,6]]}));
        assert!(close(&d["out"][0], 10.0), "{d}");
    }

    #[test]
    fn cholesky_is_lower_for_lower_and_reconstructs() {
        let a = json!([[4, 12, -16], [12, 37, -43], [-16, -43, 98]]);
        let r = call(json!({"op":"cholesky","a":a,"lower":true}));
        let l = &r["out"][0];
        assert!(
            close(&l[0][0], 2.0) && close(&l[1][0], 6.0) && close(&l[1][1], 1.0) && close(&l[2][0], -8.0),
            "{r}"
        );
        assert!(
            close(&l[2][1], 5.0) && close(&l[2][2], 3.0) && close(&l[0][1], 0.0),
            "{r}"
        );
        let u = call(json!({"op":"cholesky","a":a,"lower":false}));
        assert!(close(&u["out"][0][0][1], 6.0) && close(&u["out"][0][1][0], 0.0), "{u}");
        let bad = call(json!({"op":"cholesky","a":[[1,2],[2,1]]}));
        assert_eq!(bad["ok"], false);
    }

    #[test]
    fn cho_solve_matches_solve() {
        let a = json!([[4, 12, -16], [12, 37, -43], [-16, -43, 98]]);
        let l = call(json!({"op":"cholesky","a":a,"lower":true}))["out"][0].clone();
        let x = call(json!({"op":"cho_solve","a":l,"b":[1,2,3],"lower":true}));
        let y = call(json!({"op":"solve","a":a,"b":[1,2,3]}));
        for i in 0..3 {
            assert!(close(&x["out"][0][i], y["out"][0][i].as_f64().unwrap()), "{x} {y}");
        }
    }

    #[test]
    fn cho_solve_with_an_upper_factor() {
        let a = json!([[4, 12, -16], [12, 37, -43], [-16, -43, 98]]);
        let u = call(json!({"op":"cholesky","a":a,"lower":false}))["out"][0].clone();
        let x = call(json!({"op":"cho_solve","a":u,"b":[1,2,3],"lower":false}));
        let y = call(json!({"op":"solve","a":a,"b":[1,2,3]}));
        for i in 0..3 {
            assert!(close(&x["out"][0][i], y["out"][0][i].as_f64().unwrap()), "{x} {y}");
        }
    }

    #[test]
    fn eigh_of_a_2x2_is_ascending() {
        let r = call(json!({"op":"eigh","a":[[2,1],[1,2]],"vectors":true}));
        assert!(close(&r["out"][0][0], 1.0) && close(&r["out"][0][1], 3.0), "{r}");
        // A v = w v for each column
        let v = &r["out"][1];
        for k in 0..2 {
            let w = r["out"][0][k].as_f64().unwrap();
            let (v0, v1) = (v[0][k].as_f64().unwrap(), v[1][k].as_f64().unwrap());
            assert!(
                (2.0 * v0 + v1 - w * v0).abs() < 1e-9 && (v0 + 2.0 * v1 - w * v1).abs() < 1e-9,
                "{r}"
            );
        }
    }

    #[test]
    fn svd_reconstructs_tall_and_wide() {
        for a in [
            json!([[3.0, 1.0], [1.0, 2.0], [0.5, -1.0]]),
            json!([[3.0, 1.0, 0.5], [1.0, 2.0, -1.0]]),
        ] {
            let r = call(json!({"op":"svd","a":a}));
            assert_eq!(r["ok"], true, "{r}");
            let (u, s, vt) = (&r["out"][0], &r["out"][1], &r["out"][2]);
            let k = s.as_array().unwrap().len();
            for (i, row) in a.as_array().unwrap().iter().enumerate() {
                for (j, want) in row.as_array().unwrap().iter().enumerate() {
                    let got: f64 = (0..k)
                        .map(|t| u[i][t].as_f64().unwrap() * s[t].as_f64().unwrap() * vt[t][j].as_f64().unwrap())
                        .sum();
                    assert!((got - want.as_f64().unwrap()).abs() < 1e-9, "{a} {r}");
                }
            }
        }
    }

    #[test]
    fn triangular_solve_reads_only_its_triangle() {
        // upper triangle [[2,1],[_,4]] ; the lower entry 99 must be ignored
        let r = call(json!({"op":"tri_solve","a":[[2,1],[99,4]],"b":[4,8],"lower":false}));
        assert!(close(&r["out"][0][1], 2.0) && close(&r["out"][0][0], 1.0), "{r}");
    }

    #[test]
    fn hex_tensors_round_trip_and_nan_is_refused() {
        let a = enc(&[2, 2], &[2.0, 1.0, 1.0, 3.0]);
        let r = call(json!({"op":"solve","a":a,"b":enc(&[2], &[3.0, 5.0])}));
        assert!(close(&r["out"][0][0], 0.8) && close(&r["out"][0][1], 1.4), "{r}");
        let bad = call(json!({"op":"inv","a":enc(&[2, 2], &[1.0, f64::NAN, 0.0, 1.0])}));
        assert_eq!(bad["ok"], false, "{bad}");
        assert!(bad["error"].as_str().unwrap().contains("NaN"), "{bad}");
    }

    #[test]
    fn matmul_of_a_matrix_with_a_matrix_and_a_vector() {
        let r = call(json!({"op":"matmul","a":[[1,2],[3,4],[5,6]],"b":[[1,0,2],[0,1,3]]}));
        assert_eq!(
            r["out"][0],
            json!([[1.0, 2.0, 8.0], [3.0, 4.0, 18.0], [5.0, 6.0, 28.0]]),
            "{r}"
        );
        let v = call(json!({"op":"matmul","a":[[1,2],[3,4]],"b":[1,1]}));
        assert_eq!(v["out"][0], json!([3.0, 7.0]), "{v}");
        assert_eq!(call(json!({"op":"matmul","a":[[1,2]],"b":[1,2,3]}))["ok"], false);
    }

    #[test]
    fn small_scale_matrices_are_not_called_singular() {
        let r = call(json!({"op":"solve","a":[[1e-20,0],[0,2e-20]],"b":[1e-20,4e-20]}));
        assert_eq!(r["ok"], true, "{r}");
        assert!(close(&r["out"][0][1], 2.0), "{r}");
    }
}
