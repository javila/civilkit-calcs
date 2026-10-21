//! The port scans text by Unicode scalar value; C# strings, Monaco and the JSON wire use UTF-16
//! code units. These helpers convert positions at the JSON boundary.

/// UTF-16 offset of the scalar-value index `col` in `line` (clamped to the line length).
pub fn to_utf16(line: &[char], col: i32) -> i32 {
    let n = (col.max(0) as usize).min(line.len());
    let mut u = line[..n].iter().map(|c| c.len_utf16()).sum::<usize>() as i32;
    if col as usize > line.len() && col > 0 {
        u += col - line.len() as i32;
    }
    u
}

/// Scalar-value index of the UTF-16 offset `col16` in `line`.
pub fn from_utf16(line: &[char], col16: i32) -> i32 {
    let mut units = 0;
    for (i, c) in line.iter().enumerate() {
        if units >= col16 {
            return i as i32;
        }
        units += c.len_utf16() as i32;
    }
    line.len() as i32 + (col16 - units).max(0)
}

/// The lines of `content` as scalar-value vectors (same split as the tokenizer).
pub fn lines_of(content: &str) -> Vec<Vec<char>> {
    let chars: Vec<char> = content.chars().collect();
    crate::parsing::split_lines(&chars)
        .into_iter()
        .map(|l| l.to_vec())
        .collect()
}

/// (column, length) in UTF-16 units for a span on `line_no`.
pub fn span_to_utf16(lines: &[Vec<char>], line_no: i32, col: i32, len: i32) -> (i32, i32) {
    match usize::try_from(line_no).ok().and_then(|i| lines.get(i)) {
        Some(l) => {
            let a = to_utf16(l, col);
            let b = to_utf16(l, col + len);
            (a, b - a)
        }
        None => (col, len),
    }
}
