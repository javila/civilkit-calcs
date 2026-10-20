//! Port of Parsers/MathParser/MathParser.NumberParser.cs.
use crate::exceptions;
use crate::math_parser_exception::CResult;

const POW10: [f64; 36] = [
    0.0, 1.0, 1e-01, 1e-02, 1e-03, 1e-04, 1e-05, 1e-06, 1e-07, 1e-08, 1e-09, 1e-10, 1e-11, 1e-12,
    1e-13, 1e-14, 1e-15, 1e-16, 1e-17, 1e-18, 1e-19, 1e-20, 1e-21, 1e-22, 1e-23, 1e-24, 1e-25,
    1e-26, 1e-27, 1e-28, 1e-29, 1e-30, 1e-31, 1e-32, 1e-33, 1e-34,
];

fn mult(digit: i64, pos: i64) -> Option<i64> {
    if !(0..10).contains(&digit) || !(0..16).contains(&pos) {
        return None;
    }
    Some(digit * 10i64.pow(pos as u32))
}

pub fn parse(s: &str) -> CResult<f64> {
    parse_inner(s).ok_or_else(|| exceptions::ErrorParsingNumber(s))
}

fn parse_inner(s: &str) -> Option<f64> {
    let chars: Vec<char> = s.chars().collect();
    let mut max_digits = 16i64;
    let digits = chars.len() as i64;
    let mut decimal_position = digits;
    let mut is_leading_zeros = true;
    let mut leading_zeros = 0i64;
    for (i, &c) in chars.iter().enumerate() {
        if c == '.' {
            if decimal_position < digits {
                return None;
            }
            decimal_position = i as i64;
        } else if c != '0' {
            is_leading_zeros = false;
        }
        if is_leading_zeros {
            leading_zeros += 1;
        }
    }
    max_digits += leading_zeros;
    let mut n = digits;
    if n > max_digits {
        n = max_digits;
    }
    let mut n1 = n - 1;
    let mut k: i64 = 0;
    let dig = |i: i64| -> i64 { chars[i as usize] as i64 - '0' as i64 };
    if decimal_position > leading_zeros && decimal_position < n {
        for i in (decimal_position + 1)..n {
            k = k.checked_add(mult(dig(i), n1 - i)?)?;
        }
        n1 -= 1;
        for i in leading_zeros..decimal_position {
            k = k.checked_add(mult(dig(i), n1 - i)?)?;
        }
    } else {
        for i in leading_zeros..n {
            k = k.checked_add(mult(dig(i), n1 - i)?)?;
        }
    }
    if decimal_position >= n {
        let mut d = k as f64;
        for i in n..decimal_position {
            d = d * 10.0 + dig(i) as f64;
        }
        return Some(d);
    }
    n -= decimal_position;
    if n < 36 {
        return Some(k as f64 * POW10[n as usize]);
    }
    Some(k as f64 * 10f64.powf(-(n as f64)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numbers() {
        assert_eq!(parse("0.1").unwrap(), 0.1);
        assert_eq!(parse("1.5").unwrap(), 1.5);
        assert_eq!(parse("120").unwrap(), 120.0);
        assert_eq!(parse("3.14").unwrap(), 3.14);
        assert!(parse("1.2.3").is_err());
    }
}
