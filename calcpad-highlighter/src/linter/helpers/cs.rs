//! Small helpers reproducing .NET string/span semantics on `&[char]` (ordinal comparisons).
use crate::text::is_white_space;

fn fold(c: char) -> char {
    let mut up = c.to_uppercase();
    match (up.next(), up.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

/// `span.StartsWith(prefix, OrdinalIgnoreCase)`.
pub fn starts_with_ic(s: &[char], prefix: &str) -> bool {
    let mut it = s.iter();
    prefix.chars().all(|p| it.next().is_some_and(|c| fold(*c) == fold(p)))
}

/// `span.Equals(other, OrdinalIgnoreCase)`.
pub fn equals_ic(s: &[char], other: &str) -> bool {
    s.len() == other.chars().count() && starts_with_ic(s, other)
}

/// `span.EndsWith(suffix, OrdinalIgnoreCase)`.
pub fn ends_with_ic(s: &[char], suffix: &str) -> bool {
    let n = suffix.chars().count();
    s.len() >= n && starts_with_ic(&s[s.len() - n..], suffix)
}

/// `span.IndexOf(needle, OrdinalIgnoreCase)`.
pub fn index_of_ic(hay: &[char], needle: &str, from: usize) -> Option<usize> {
    let n = needle.chars().count();
    if from > hay.len() || hay.len() - from < n {
        return None;
    }
    (from..=hay.len() - n).find(|&i| starts_with_ic(&hay[i..], needle))
}

/// `span.IndexOf(needle, OrdinalIgnoreCase)` for a char needle slice.
pub fn index_of_chars_ic(hay: &[char], needle: &[char]) -> Option<usize> {
    let s: String = needle.iter().collect();
    index_of_ic(hay, &s, 0)
}

/// `span.LastIndexOf(needle)` (ordinal).
pub fn last_index_of(hay: &[char], needle: &str) -> Option<usize> {
    let nd: Vec<char> = needle.chars().collect();
    if nd.is_empty() {
        return Some(hay.len());
    }
    if hay.len() < nd.len() {
        return None;
    }
    (0..=hay.len() - nd.len())
        .rev()
        .find(|&i| hay[i..i + nd.len()] == nd[..])
}

/// `string.IsNullOrWhiteSpace`.
pub fn is_blank(s: &str) -> bool {
    s.chars().all(is_white_space)
}

/// `string.Trim()`.
pub fn trim_str(s: &str) -> &str {
    s.trim_matches(is_white_space)
}

/// `string.TrimStart()`.
pub fn trim_start_str(s: &str) -> &str {
    s.trim_start_matches(is_white_space)
}

/// `string.Length` is a UTF-16 length in C#; the port counts Unicode scalars everywhere
/// (columns are converted to UTF-16 by the JSON layer).
pub fn len(s: &str) -> i32 {
    s.chars().count() as i32
}

pub fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}
