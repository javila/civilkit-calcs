//! Port of Matrix/HpMatrix/HpFFT.cs: the radix 2 FFT of Numerical Recipes.

/// `Transform(re, im, isign)`: replaces re, im by the discrete Fourier transform (isign = 1)
/// or by n times the inverse transform (isign = -1). n must be a power of 2.
#[allow(clippy::assign_op_pattern)] // keeps the operand order of the C# recurrence
pub fn transform(re: &mut [f64], im: &mut [f64], isign: i32) {
    let n = re.len().min(im.len());
    let mut j = 0usize;
    for i in 0..n {
        if j > i {
            re.swap(i, j);
            im.swap(i, j);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }
    let mut mmax = 1usize;
    while n > mmax {
        let istep = mmax << 1;
        let theta = isign as f64 * (std::f64::consts::PI / mmax as f64);
        let mut wtemp = (0.5 * theta).sin();
        let wpr = -2.0 * wtemp * wtemp;
        let wpi = theta.sin();
        let mut wr = 1.0;
        let mut wi = 0.0;
        for m in 0..mmax {
            let mut i = m;
            while i < n {
                let j = i + mmax;
                let tempr = wr * re[j] - wi * im[j];
                let tempi = wr * im[j] + wi * re[j];
                re[j] = re[i] - tempr;
                im[j] = im[i] - tempi;
                re[i] += tempr;
                im[i] += tempi;
                i += istep;
            }
            wtemp = wr;
            wr = wtemp * wpr - wi * wpi + wr;
            wi = wi * wpr + wtemp * wpi + wi;
        }
        mmax = istep;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn inverse_round_trip() {
        let mut re: Vec<f64> = (0..8).map(|i| (i as f64 * 0.7).sin()).collect();
        let mut im = vec![0.0; 8];
        let orig = re.clone();
        super::transform(&mut re, &mut im, 1);
        super::transform(&mut re, &mut im, -1);
        for i in 0..8 {
            assert!((re[i] / 8.0 - orig[i]).abs() < 1e-14);
            assert!((im[i] / 8.0).abs() < 1e-14);
        }
    }
}
