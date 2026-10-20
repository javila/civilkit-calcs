//! Port of Matrix/HpMatrix/Vectorized.cs and the `System.Numerics.Vector<double>`
//! reductions that the HP types use.
//!
//! C# processes blocks of `Vector<double>.Count` = 4 doubles (x64 with AVX2, the machines
//! that produced the reference output). Element-wise operations give the same results as
//! a scalar loop, so they are written as scalar loops. Reductions are reproduced exactly:
//!
//! * `Vector.Sum(v)` and `Vector.Dot(a, b)` on a 256-bit vector of doubles are lowered by
//!   the .NET JIT to one `vhaddpd` (pairs within each 128-bit half) followed by adding the
//!   upper and the lower half: `(v0 + v1) + (v2 + v3)`. Source: dotnet/runtime
//!   release/8.0, `src/coreclr/jit/gentree.cpp` `Compiler::gtNewSimdSumNode` and
//!   `src/coreclr/jit/lowerxarch.cpp` `Lowering::LowerHWIntrinsicDot` (TYP_DOUBLE, simd32:
//!   multiply, AVX HorizontalAdd, then upper + lower).
//! * Lane-wise accumulators (`vb += va[i]`) keep four partial sums, one per lane.
//! * `Vector.Min`/`Vector.Max` select `x < y ? x : y` / `x > y ? x : y` per lane.
//!
//! A block path is taken only where C# takes it (usually when the length is above 15).

/// `Vector<double>.Count`
pub const VEC: usize = 4;

/// `Vector.Sum(v)` for 4 lanes.
#[inline]
pub fn vsum(v: [f64; 4]) -> f64 {
    (v[0] + v[1]) + (v[2] + v[3])
}

/// `Vector.Dot(a, b)` of the 4 doubles starting at `a[ia]` and `b[ib]`.
#[inline]
pub fn vdot(a: &[f64], ia: usize, b: &[f64], ib: usize) -> f64 {
    let p0 = a[ia] * b[ib];
    let p1 = a[ia + 1] * b[ib + 1];
    let p2 = a[ia + 2] * b[ib + 2];
    let p3 = a[ia + 3] * b[ib + 3];
    (p0 + p1) + (p2 + p3)
}

/// `Vector.Min(x, y)` per lane
#[inline]
pub fn vmin(x: f64, y: f64) -> f64 {
    if x < y {
        x
    } else {
        y
    }
}

/// `Vector.Max(x, y)` per lane
#[inline]
pub fn vmax(x: f64, y: f64) -> f64 {
    if x > y {
        x
    } else {
        y
    }
}

/// `Scale(Span<double> x, double d)`
pub fn scale(x: &mut [f64], d: f64) {
    for v in x.iter_mut() {
        *v *= d;
    }
}

/// `Add(x, y)`: y += x over the common length
pub fn add(x: &[f64], y: &mut [f64]) {
    let len = x.len().min(y.len());
    for i in 0..len {
        y[i] += x[i];
    }
}

/// `Multiply(x, y, z)`: z = x * y over the common length of x and y
pub fn multiply(x: &[f64], y: &[f64], z: &mut [f64]) {
    let len = x.len().min(y.len());
    for i in 0..len {
        z[i] = x[i] * y[i];
    }
}

/// `SumAbs(sv, l, m)`: the sum of |sv[l..m]|
pub fn sum_abs(sv: &[f64], l: usize, m: usize) -> f64 {
    if m <= l {
        return 0.0;
    }
    let len = m - l;
    let sa = &sv[l..m];
    let mut sum = 0.0;
    let mut nv = 0;
    if len > 15 {
        nv = len / VEC;
        let mut vb = [0.0; 4];
        for i in 0..nv {
            for k in 0..4 {
                vb[k] += sa[i * VEC + k].abs();
            }
        }
        sum = vsum(vb);
        nv *= VEC;
    }
    for v in &sa[nv..] {
        sum += v.abs();
    }
    sum
}

/// `NormalizeAndSumSq(sv, l, m, scale)`: divides sv[l..m] by `scale` and returns the sum of
/// the squares.
pub fn normalize_and_sum_sq(sv: &mut [f64], l: usize, m: usize, scale: f64) -> f64 {
    if m <= l {
        return 0.0;
    }
    let len = m - l;
    let sa = &mut sv[l..m];
    let mut sum = 0.0;
    let mut nv = 0;
    if len > 15 {
        nv = len / VEC;
        let mut vb = [0.0; 4];
        for i in 0..nv {
            for k in 0..4 {
                let v = sa[i * VEC + k] / scale;
                sa[i * VEC + k] = v;
                vb[k] += v * v;
            }
        }
        sum += vsum(vb);
        nv *= VEC;
    }
    for v in sa[nv..].iter_mut() {
        let v_j = *v / scale;
        *v = v_j;
        sum += v_j * v_j;
    }
    sum
}

/// `DotProduct(a, b, l, m)`: the dot product of a[l..m] and b[l..m]
pub fn dot_product(a: &[f64], b: &[f64], l: usize, m: usize) -> f64 {
    if m <= l {
        return 0.0;
    }
    let len = m - l;
    let mut sum = 0.0;
    let mut nv = 0;
    if len > 15 {
        nv = len / VEC;
        for i in 0..nv {
            sum += vdot(a, l + i * VEC, b, l + i * VEC);
        }
        nv *= VEC;
    }
    for i in nv..len {
        sum += a[l + i] * b[l + i];
    }
    sum
}

/// `SumSq(a, l, m)`
pub fn sum_sq(a: &[f64], l: usize, m: usize) -> f64 {
    if m <= l {
        return 0.0;
    }
    let len = m - l;
    let mut sumsq = 0.0;
    let mut nv = 0;
    if len > 15 {
        nv = len / VEC;
        for i in 0..nv {
            sumsq += vdot(a, l + i * VEC, a, l + i * VEC);
        }
        nv *= VEC;
    }
    for i in nv..len {
        let b = a[l + i];
        sumsq += b * b;
    }
    sumsq
}

/// `Norm(x)`
pub fn norm(x: &[f64]) -> f64 {
    sum_sq(x, 0, x.len()).sqrt()
}

/// `MultiplyAdd(x, d, y)`: y += d * x over the common length
pub fn multiply_add(x: &[f64], d: f64, y: &mut [f64]) {
    if d == 0.0 {
        return;
    }
    if d == 1.0 {
        add(x, y);
        return;
    }
    let len = x.len().min(y.len());
    for i in 0..len {
        y[i] += d * x[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reductions_use_the_simd_order() {
        // 16 elements take the block path: four blocks of four
        let a: Vec<f64> = (0..20).map(|i| 1.0 / (i as f64 + 1.0)).collect();
        let b: Vec<f64> = (0..20).map(|i| (i as f64).sin()).collect();
        let mut expected = 0.0;
        for blk in 0..5 {
            let p: Vec<f64> = (0..4).map(|k| a[blk * 4 + k] * b[blk * 4 + k]).collect();
            expected += (p[0] + p[1]) + (p[2] + p[3]);
        }
        assert_eq!(dot_product(&a, &b, 0, 20).to_bits(), expected.to_bits());
        // 15 elements are summed sequentially
        let mut seq = 0.0;
        for i in 0..15 {
            seq += a[i] * b[i];
        }
        assert_eq!(dot_product(&a, &b, 0, 15).to_bits(), seq.to_bits());
    }
}
