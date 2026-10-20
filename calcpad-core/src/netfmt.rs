//! .NET compatible number and string formatting.
//!
//! Calcpad.Core relies on the .NET runtime for `string.Format`, `double.ToString(...)`
//! (standard and custom numeric format strings) and `decimal` rounding. Output must be
//! identical, so the subset that Calcpad uses is reimplemented here following the
//! behaviour of `System.Number.Formatting` in .NET (invariant culture).

/// `string.Format(template, args)` for templates that only use `{0}`..`{9}` placeholders.
pub fn fmt(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut num = String::new();
            let mut closed = false;
            for d in chars.by_ref() {
                if d == '}' {
                    closed = true;
                    break;
                }
                num.push(d);
            }
            if closed {
                if let Ok(i) = num.parse::<usize>() {
                    if let Some(a) = args.get(i) {
                        out.push_str(a);
                        continue;
                    }
                }
            }
            out.push('{');
            out.push_str(&num);
            if closed {
                out.push('}');
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Decimal digits of a finite, non-zero `|d|` and its scale, such that
/// `|d| = 0.d1d2d3... x 10^scale`. Trailing zeros are removed.
/// `sig` is the number of significant digits (correctly rounded, ties to even, like .NET Dragon4).
/// `sig == 0` returns the shortest round-trip digits.
pub fn digits(d: f64, sig: usize) -> (Vec<u8>, i32) {
    let a = d.abs();
    let s = if sig == 0 {
        format!("{:e}", a)
    } else {
        format!("{:.*e}", sig - 1, a)
    };
    let (mant, exp) = s.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let mut ds: Vec<u8> = mant.bytes().filter(|b| b.is_ascii_digit()).map(|b| b - b'0').collect();
    while ds.len() > 1 && *ds.last().unwrap() == 0 {
        ds.pop();
    }
    if ds.iter().all(|&x| x == 0) {
        return (Vec::new(), 0);
    }
    (ds, exp + 1)
}

/// .NET `RoundNumber` with `isCorrectlyRounded = false`: keep `pos` digits, round half up.
/// Returns new digits and scale. Empty digits means zero.
fn round_number(ds: &[u8], scale: i32, pos: i32) -> (Vec<u8>, i32) {
    if pos < 0 {
        return (Vec::new(), 0);
    }
    let pos = pos as usize;
    if pos >= ds.len() {
        return (ds.to_vec(), scale);
    }
    let mut out: Vec<u8> = ds[..pos].to_vec();
    let mut scale = scale;
    if ds[pos] >= 5 {
        let mut i = out.len();
        loop {
            if i == 0 {
                out.insert(0, 1);
                scale += 1;
                break;
            }
            i -= 1;
            if out[i] == 9 {
                out[i] = 0;
            } else {
                out[i] += 1;
                break;
            }
        }
    }
    while let Some(&0) = out.last() {
        out.pop();
    }
    if out.is_empty() {
        return (Vec::new(), 0);
    }
    (out, scale)
}

fn push_exponent(s: &mut String, exp: i32, min_digits: usize, positive_sign: bool) {
    if exp < 0 {
        s.push('-');
    } else if positive_sign {
        s.push('+');
    }
    let e = format!("{}", exp.unsigned_abs());
    for _ in e.len()..min_digits {
        s.push('0');
    }
    s.push_str(&e);
}

/// .NET "G<n>" / shortest "G" formatting of a double (invariant culture, 'E' exponent char).
/// `precision == 0` means the default (shortest round trip, 15 digit threshold).
pub fn format_general(d: f64, precision: usize) -> String {
    if d.is_nan() {
        return "NaN".to_string();
    }
    if d.is_infinite() {
        return if d > 0.0 {
            "\u{221E}".to_string()
        } else {
            "-\u{221E}".to_string()
        };
    }
    // .NET Core 3.0 and later print negative zero as "-0".
    let neg = d.is_sign_negative();
    let max_digits: i32 = if precision == 0 { 15 } else { precision as i32 };
    let (ds, scale) = if d == 0.0 {
        (Vec::new(), 0)
    } else {
        digits(d, precision)
    };
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    if ds.is_empty() {
        s.push('0');
        return s;
    }
    let mut dig_pos = scale;
    let mut scientific = false;
    if dig_pos > max_digits || dig_pos < -3 {
        dig_pos = 1;
        scientific = true;
    }
    let mut idx = 0usize;
    if dig_pos > 0 {
        while dig_pos > 0 {
            if idx < ds.len() {
                s.push((b'0' + ds[idx]) as char);
                idx += 1;
            } else {
                s.push('0');
            }
            dig_pos -= 1;
        }
    } else {
        s.push('0');
    }
    if idx < ds.len() || dig_pos < 0 {
        s.push('.');
        while dig_pos < 0 {
            s.push('0');
            dig_pos += 1;
        }
        while idx < ds.len() {
            s.push((b'0' + ds[idx]) as char);
            idx += 1;
        }
    }
    if scientific {
        s.push('E');
        push_exponent(&mut s, scale - 1, 2, true);
    }
    s
}

/// `double.ToString()` / `double.ToString(CultureInfo.InvariantCulture)`.
pub fn double_to_string(d: f64) -> String {
    format_general(d, 0)
}

/// `float.ToString()` (single precision shortest round trip, 7 digit threshold).
pub fn float_to_string(f: f32) -> String {
    if f.is_nan() {
        return "NaN".to_string();
    }
    if f == 0.0 {
        return if f.is_sign_negative() {
            "-0".to_string()
        } else {
            "0".to_string()
        };
    }
    let s = format!("{:e}", f.abs());
    let (mant, exp) = s.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let ds: Vec<u8> = mant.bytes().filter(|b| b.is_ascii_digit()).map(|b| b - b'0').collect();
    let scale = exp + 1;
    let mut out = String::new();
    if f < 0.0 {
        out.push('-');
    }
    let mut dig_pos = scale;
    let mut scientific = false;
    if dig_pos > 7 || dig_pos < -3 {
        dig_pos = 1;
        scientific = true;
    }
    let mut idx = 0;
    if dig_pos > 0 {
        while dig_pos > 0 {
            if idx < ds.len() {
                out.push((b'0' + ds[idx]) as char);
                idx += 1;
            } else {
                out.push('0');
            }
            dig_pos -= 1;
        }
    } else {
        out.push('0');
    }
    if idx < ds.len() || dig_pos < 0 {
        out.push('.');
        while dig_pos < 0 {
            out.push('0');
            dig_pos += 1;
        }
        while idx < ds.len() {
            out.push((b'0' + ds[idx]) as char);
            idx += 1;
        }
    }
    if scientific {
        out.push('E');
        push_exponent(&mut out, scale - 1, 2, true);
    }
    out
}

/// `Math.Round(decimal.Parse(d.ToString("G17")), i).ToString("G29")` for |d| in [1e-4, 1e16).
/// Returns None when the G17 representation would use an exponent.
pub fn g17_decimal_round(d: f64, i: i32) -> Option<String> {
    if d == 0.0 {
        return Some("0".to_string());
    }
    let (ds, scale) = digits(d, 17);
    if scale > 17 || scale < -3 {
        return None;
    }
    // Round half to even at `i` decimal places: keep scale + i digits.
    let keep = scale + i;
    let (rd, rs) = if keep < 0 {
        (Vec::new(), 0)
    } else if (keep as usize) >= ds.len() {
        (ds.clone(), scale)
    } else {
        let k = keep as usize;
        let mut out: Vec<u8> = ds[..k].to_vec();
        let rest = &ds[k..];
        let first = rest[0];
        let round_up = if first > 5 {
            true
        } else if first < 5 {
            false
        } else if rest[1..].iter().any(|&x| x != 0) {
            true
        } else {
            // tie: round to even
            match out.last() {
                Some(&l) => l % 2 == 1,
                None => false,
            }
        };
        let mut sc = scale;
        if round_up {
            let mut j = out.len();
            loop {
                if j == 0 {
                    out.insert(0, 1);
                    sc += 1;
                    break;
                }
                j -= 1;
                if out[j] == 9 {
                    out[j] = 0;
                } else {
                    out[j] += 1;
                    break;
                }
            }
        }
        while let Some(&0) = out.last() {
            out.pop();
        }
        if out.is_empty() {
            (Vec::new(), 0)
        } else {
            (out, sc)
        }
    };
    if rd.is_empty() {
        return Some("0".to_string());
    }
    let mut s = String::new();
    if d < 0.0 {
        s.push('-');
    }
    write_plain(&mut s, &rd, rs);
    Some(s)
}

/// Writes digits with decimal scale as a plain decimal number without exponent.
fn write_plain(s: &mut String, ds: &[u8], scale: i32) {
    if scale <= 0 {
        s.push_str("0.");
        for _ in 0..(-scale) {
            s.push('0');
        }
        for &x in ds {
            s.push((b'0' + x) as char);
        }
    } else {
        let sc = scale as usize;
        for k in 0..sc {
            if k < ds.len() {
                s.push((b'0' + ds[k]) as char);
            } else {
                s.push('0');
            }
        }
        if ds.len() > sc {
            s.push('.');
            for &x in &ds[sc..] {
                s.push((b'0' + x) as char);
            }
        }
    }
}

/// Parsed custom numeric format (`[0#]+(,[0#]+)?(\.[0#]+)?([eE][+-]?0+)?`).
struct CustomFormat {
    int_placeholders: Vec<char>,
    frac_placeholders: Vec<char>,
    grouping: bool,
    exp: Option<(char, char, usize)>, // (E/e, sign mode '+','-' or '\0', min digits)
}

fn parse_custom(format: &str) -> CustomFormat {
    let mut int_placeholders = Vec::new();
    let mut frac_placeholders = Vec::new();
    let mut grouping = false;
    let mut exp = None;
    let chars: Vec<char> = format.chars().collect();
    let mut i = 0;
    let mut in_frac = false;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '0' | '#' => {
                if in_frac {
                    frac_placeholders.push(c)
                } else {
                    int_placeholders.push(c)
                }
            }
            ',' => {
                if !in_frac {
                    grouping = true
                }
            }
            '.' => in_frac = true,
            'e' | 'E' => {
                let mut j = i + 1;
                let mut sign = '\0';
                if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                    sign = chars[j];
                    j += 1;
                }
                let mut n = 0;
                while j < chars.len() && chars[j] == '0' {
                    n += 1;
                    j += 1;
                }
                if n > 0 {
                    exp = Some((c, sign, n));
                    i = j;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    // A trailing grouping separator immediately before '.' or end is a scaling comma in .NET;
    // Calcpad's validation regex only allows ',' between placeholders.
    CustomFormat {
        int_placeholders,
        frac_placeholders,
        grouping,
        exp,
    }
}

fn group_digits(int_digits: &str) -> String {
    let bytes = int_digits.as_bytes();
    let n = bytes.len();
    let mut s = String::new();
    for (k, &b) in bytes.iter().enumerate() {
        if k > 0 && (n - k) % 3 == 0 {
            s.push(',');
        }
        s.push(b as char);
    }
    s
}

/// .NET custom numeric format of a double (15 significant digits, half up rounding).
pub fn format_custom(d: f64, format: &str) -> String {
    if d.is_nan() {
        return "NaN".to_string();
    }
    if d.is_infinite() {
        return if d > 0.0 {
            "\u{221E}".to_string()
        } else {
            "-\u{221E}".to_string()
        };
    }
    let cf = parse_custom(format);
    let neg = d.is_sign_negative();
    let (ds, mut scale) = if d == 0.0 { (Vec::new(), 0) } else { digits(d, 15) };
    let decimal_pos = cf.int_placeholders.len() as i32;
    let digit_count = decimal_pos + cf.frac_placeholders.len() as i32;
    let mut s = String::new();
    let (rd, exponent) = if let Some((_, _, _)) = cf.exp {
        if ds.is_empty() {
            (Vec::new(), 0)
        } else {
            let (r, sc) = round_number(&ds, scale, digit_count);
            scale = sc;
            let e = if r.is_empty() { 0 } else { scale - decimal_pos };
            (r, e)
        }
    } else if ds.is_empty() {
        (Vec::new(), 0)
    } else {
        let pos = scale + digit_count - decimal_pos;
        let (r, sc) = round_number(&ds, scale, pos);
        scale = sc;
        (r, 0)
    };
    // Effective scale of the number relative to the decimal point of the output.
    let eff_scale = if cf.exp.is_some() {
        if rd.is_empty() {
            0
        } else {
            decimal_pos
        }
    } else if rd.is_empty() {
        0
    } else {
        scale
    };
    // Integer digits.
    let mut int_part = String::new();
    if eff_scale > 0 {
        for k in 0..eff_scale as usize {
            if k < rd.len() {
                int_part.push((b'0' + rd[k]) as char);
            } else {
                int_part.push('0');
            }
        }
    }
    // Leading zero placeholders force digits.
    let first_zero = cf.int_placeholders.iter().position(|&c| c == '0');
    if let Some(fz) = first_zero {
        let min_len = cf.int_placeholders.len() - fz;
        while int_part.len() < min_len {
            int_part.insert(0, '0');
        }
    }
    // Fraction digits.
    let mut frac = String::new();
    for k in 0..cf.frac_placeholders.len() {
        let idx = eff_scale + k as i32;
        let digit = if idx >= 0 && (idx as usize) < rd.len() {
            rd[idx as usize]
        } else {
            0
        };
        frac.push((b'0' + digit) as char);
    }
    // Trim optional trailing digits ('#').
    let last_zero = cf.frac_placeholders.iter().rposition(|&c| c == '0');
    let min_frac = match last_zero {
        Some(p) => p + 1,
        None => 0,
    };
    while frac.len() > min_frac && frac.ends_with('0') {
        frac.pop();
    }
    let is_zero_output = rd.is_empty();
    if neg && !(is_zero_output && d == 0.0) {
        s.push('-');
    }
    if cf.grouping {
        s.push_str(&group_digits(&int_part));
    } else {
        s.push_str(&int_part);
    }
    if !frac.is_empty() {
        s.push('.');
        s.push_str(&frac);
    }
    if let Some((ec, sign, n)) = cf.exp {
        s.push(ec);
        push_exponent(&mut s, exponent, n, sign == '+');
    }
    if s.is_empty() || s == "-" {
        s.push('0');
    }
    s
}

/// .NET standard numeric format ('F','N','E','G','C') of a double, invariant culture.
pub fn format_standard(d: f64, kind: char, precision: Option<usize>) -> String {
    if d.is_nan() {
        return "NaN".to_string();
    }
    if d.is_infinite() {
        return if d > 0.0 {
            "\u{221E}".to_string()
        } else {
            "-\u{221E}".to_string()
        };
    }
    match kind.to_ascii_uppercase() {
        'F' | 'N' | 'C' => {
            let p = precision.unwrap_or(2);
            let s = format!("{:.*}", p, d.abs());
            let neg = d.is_sign_negative() && s.bytes().any(|b| b.is_ascii_digit() && b != b'0')
                || (d.is_sign_negative() && d != 0.0);
            let (ip, fp) = match s.split_once('.') {
                Some((a, b)) => (a.to_string(), Some(b.to_string())),
                None => (s.clone(), None),
            };
            let ip = if kind.to_ascii_uppercase() == 'F' {
                ip
            } else {
                group_digits(&ip)
            };
            let mut body = ip;
            if let Some(f) = fp {
                body.push('.');
                body.push_str(&f);
            }
            match kind.to_ascii_uppercase() {
                'C' => {
                    if neg {
                        format!("(\u{00A4}{})", body)
                    } else {
                        format!("\u{00A4}{}", body)
                    }
                }
                _ => {
                    if neg {
                        format!("-{}", body)
                    } else {
                        body
                    }
                }
            }
        }
        'E' => {
            let p = precision.unwrap_or(6);
            let mut s = String::new();
            if d.is_sign_negative() && d != 0.0 {
                s.push('-');
            }
            let (ds, scale) = if d == 0.0 { (Vec::new(), 1) } else { digits(d, p + 1) };
            let first = if ds.is_empty() { 0 } else { ds[0] };
            s.push((b'0' + first) as char);
            if p > 0 {
                s.push('.');
                for k in 1..=p {
                    let x = if k < ds.len() { ds[k] } else { 0 };
                    s.push((b'0' + x) as char);
                }
            }
            s.push(if kind == 'e' { 'e' } else { 'E' });
            let e = if ds.is_empty() { 0 } else { scale - 1 };
            push_exponent(&mut s, e, 3, true);
            s
        }
        'G' => format_general(d, precision.unwrap_or(0)),
        _ => format_general(d, 0),
    }
}

/// `long.ToString("D<n>")`.
pub fn format_long_d(v: i64, precision: Option<usize>) -> String {
    let p = precision.unwrap_or(0);
    let digits = v.unsigned_abs().to_string();
    let mut s = String::new();
    if v < 0 {
        s.push('-');
    }
    for _ in digits.len()..p {
        s.push('0');
    }
    s.push_str(&digits);
    s
}

/// `d.ToString(format, CultureInfo.InvariantCulture)` for the format strings that Calcpad accepts.
pub fn double_to_string_format(d: f64, format: &str) -> String {
    let mut chars = format.chars();
    let first = chars.next().unwrap_or('G');
    let rest: String = chars.collect();
    if first.is_ascii_alphabetic() && rest.chars().all(|c| c.is_ascii_digit()) {
        let precision = if rest.is_empty() {
            None
        } else {
            rest.parse::<usize>().ok()
        };
        return format_standard(d, first, precision);
    }
    format_custom(d, format)
}

/// Encodes like `System.Web.HttpUtility.HtmlEncode`.
pub fn html_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '\u{A0}'..='\u{FF}' => {
                out.push_str(&format!("&#{};", c as u32));
            }
            c if (c as u32) > 0xFFFF => out.push_str(&format!("&#{};", c as u32)),
            _ => out.push(c),
        }
    }
    out
}

/// .NET `Math.Round(x, MidpointRounding.AwayFromZero)`.
pub fn round_away(x: f64) -> f64 {
    x.round()
}

/// .NET `Math.Round(x, digits)` (banker's rounding) as used for doubles.
pub fn round_even_digits(x: f64, digits: i32) -> f64 {
    if !x.is_finite() {
        return x;
    }
    if digits == 0 {
        return x.round_ties_even();
    }
    let power10 = 10f64.powi(digits);
    let v = x * power10;
    let r = v.round_ties_even();
    // .NET: if (Math.Abs(value) < 1e16) { value *= power10; value = Round(value); value /= power10 }
    if x.abs() < 1e16 {
        r / power10
    } else {
        x
    }
}

/// `HttpUtility.HtmlAttributeEncode`: encodes `&`, `"`, `<` and `'`.
pub fn html_attribute_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// `double.TryParse(s, NumberStyles.Float, CultureInfo.InvariantCulture)` for plain
/// numbers: optional whitespace and sign, digits with a decimal point, an exponent.
pub fn parse_invariant_float(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty()
        || !t
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'))
    {
        return None;
    }
    t.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_format() {
        assert_eq!(double_to_string(1.5), "1.5");
        assert_eq!(double_to_string(1e15), "1E+15");
        assert_eq!(double_to_string(1e14), "100000000000000");
        assert_eq!(double_to_string(0.0001), "0.0001");
        assert_eq!(double_to_string(0.00001), "1E-05");
        assert_eq!(double_to_string(-2.5e-7), "-2.5E-07");
        assert_eq!(format_general(0.1, 17), "0.10000000000000001");
        assert_eq!(format_general(1.5, 17), "1.5");
    }

    #[test]
    fn decimal_round() {
        assert_eq!(g17_decimal_round(1.23456, 2).unwrap(), "1.23");
        assert_eq!(g17_decimal_round(2.5, 0).unwrap(), "2");
        assert_eq!(g17_decimal_round(7.0685834705770345, 2).unwrap(), "7.07");
        assert_eq!(g17_decimal_round(0.125, 2).unwrap(), "0.12");
        assert_eq!(g17_decimal_round(-0.001, 2).unwrap(), "0");
        assert_eq!(g17_decimal_round(1234.5, 0).unwrap(), "1234");
    }

    #[test]
    fn custom_exponent() {
        assert_eq!(format_custom(1.5e20, "#.##E+0"), "1.5E+20");
        assert_eq!(format_custom(1e16, "#.##E+0"), "1E+16");
        assert_eq!(format_custom(1.234e-7, "#.##E+0"), "1.23E-7");
        assert_eq!(format_custom(9.999e20, "#.##E+0"), "1E+21");
        assert_eq!(format_custom(-1.235e-9, "#.##E+0"), "-1.24E-9");
        assert_eq!(format_custom(1.23456, "0.00"), "1.23");
        assert_eq!(format_custom(1234567.891, "#,##0.0"), "1,234,567.9");
        assert_eq!(format_custom(0.5, "#.##"), ".5");
    }

    #[test]
    fn standard() {
        assert_eq!(format_standard(1.23456, 'F', Some(3)), "1.235");
        assert_eq!(format_standard(1234.5, 'N', Some(1)), "1,234.5");
        assert_eq!(format_standard(1234.5, 'E', Some(2)), "1.23E+003");
    }

    #[test]
    fn string_format() {
        assert_eq!(fmt("a {0} b {1}", &["x", "y"]), "a x b y");
    }
}
