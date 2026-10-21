//! Parsing/: low-level utilities. `LineEnumerator`, `SplitEnumerator` and `TextSpan` are
//! allocation tricks over spans in C#; here they are `split_lines` and slices.
pub mod char_classifier;

/// `LineEnumerator`: splits on `\r\n`, `\r` and `\n`; a trailing line ending yields a final
/// empty line, and an empty input yields one empty line.
pub fn split_lines(text: &[char]) -> Vec<&[char]> {
    let mut lines = Vec::new();
    let mut rest = text;
    loop {
        if rest.is_empty() {
            lines.push(rest);
            break;
        }
        match rest.iter().position(|c| *c == '\r' || *c == '\n') {
            None => {
                lines.push(rest);
                break;
            }
            Some(i) => {
                lines.push(&rest[..i]);
                if i < rest.len() - 1 && rest[i] == '\r' && rest[i + 1] == '\n' {
                    rest = &rest[i + 2..];
                } else {
                    rest = &rest[i + 1..];
                }
            }
        }
    }
    lines
}

/// `string.Split(delimiter)` as `SplitEnumerator` yields it: no segments for an empty span and
/// no trailing empty segment after a final delimiter is NOT dropped (`a;` gives `a` and an empty
/// remainder is not produced because the enumerator stops on an empty span).
pub fn split_enumerator(span: &[char], delimiter: char) -> Vec<&[char]> {
    let mut out = Vec::new();
    let mut rest = span;
    while !rest.is_empty() {
        match rest.iter().position(|c| *c == delimiter) {
            None => {
                out.push(rest);
                break;
            }
            Some(i) => {
                out.push(&rest[..i]);
                rest = &rest[i + 1..];
            }
        }
    }
    out
}
