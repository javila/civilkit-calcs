//! Character and string helpers that reproduce the .NET semantics the C# highlighter relies on.
//!
//! Lines are scanned as `&[char]` (Unicode scalar values). C# strings are UTF-16, so a
//! character outside the BMP is one position here and two in C#; the JSON layer converts
//! columns back to UTF-16 (see `crate::utf16`).

/// `char.IsLetter`: categories Lu, Ll, Lt, Lm, Lo. Rust's `is_alphabetic` also covers Nl
/// (letter numbers such as Roman numerals) and a few marks, which are removed here.
pub fn is_letter(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphabetic();
    }
    c.is_alphabetic() && !is_letter_number(c) && !is_other_alphabetic_mark(c)
}

fn is_letter_number(c: char) -> bool {
    matches!(c as u32,
        0x16EE..=0x16F0 | 0x2160..=0x2182 | 0x2185..=0x2188 | 0x3007 | 0x3021..=0x3029 | 0x3038..=0x303A
        | 0xA6E6..=0xA6EF | 0x10140..=0x10174 | 0x10341 | 0x1034A | 0x103D1..=0x103D5 | 0x12400..=0x1246E)
}

fn is_other_alphabetic_mark(c: char) -> bool {
    // Combining marks that carry the Alphabetic property but are Mn/Mc in .NET.
    matches!(c as u32, 0x0345 | 0x05B0..=0x05BD | 0x05BF | 0x05C1..=0x05C2 | 0x05C4..=0x05C5 | 0x05C7
        | 0x0610..=0x061A | 0x064B..=0x0657 | 0x0659..=0x065F | 0x0670 | 0x06D6..=0x06DC | 0x06E1..=0x06E4
        | 0x06E7..=0x06E8 | 0x06ED | 0x0900..=0x0903 | 0x093A..=0x093B | 0x093E..=0x094C | 0x094E..=0x094F
        | 0x0955..=0x0957 | 0x0962..=0x0963 | 0x24B6..=0x24E9)
}

/// `char.IsDigit` (category Nd).
pub fn is_digit(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_digit();
    }
    matches!(c as u32,
        0x0660..=0x0669 | 0x06F0..=0x06F9 | 0x07C0..=0x07C9 | 0x0966..=0x096F | 0x09E6..=0x09EF
        | 0x0A66..=0x0A6F | 0x0AE6..=0x0AEF | 0x0B66..=0x0B6F | 0x0BE6..=0x0BEF | 0x0C66..=0x0C6F
        | 0x0CE6..=0x0CEF | 0x0D66..=0x0D6F | 0x0E50..=0x0E59 | 0x0ED0..=0x0ED9 | 0x0F20..=0x0F29
        | 0x1040..=0x1049 | 0x17E0..=0x17E9 | 0x1810..=0x1819 | 0xFF10..=0xFF19 | 0x1D7CE..=0x1D7FF)
}

/// `char.IsLetterOrDigit`.
pub fn is_letter_or_digit(c: char) -> bool {
    is_letter(c) || is_digit(c)
}

/// `char.IsWhiteSpace`.
pub fn is_white_space(c: char) -> bool {
    c.is_whitespace()
}

/// `string.Substring(start, len)` on a char slice, clamped like a checked slice.
pub fn sub(s: &[char], start: usize, len: usize) -> String {
    s[start..start + len].iter().collect()
}

/// `s[start..end]` as a string.
pub fn range(s: &[char], start: usize, end: usize) -> String {
    s[start..end].iter().collect()
}

pub fn to_string(s: &[char]) -> String {
    s.iter().collect()
}

pub fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

/// `span.Trim()` (whitespace by `char.IsWhiteSpace`).
pub fn trim(s: &[char]) -> &[char] {
    trim_end(trim_start(s))
}

pub fn trim_start(s: &[char]) -> &[char] {
    let n = s.iter().take_while(|c| is_white_space(**c)).count();
    &s[n..]
}

pub fn trim_end(s: &[char]) -> &[char] {
    let mut e = s.len();
    while e > 0 && is_white_space(s[e - 1]) {
        e -= 1;
    }
    &s[..e]
}

pub fn is_white_space_slice(s: &[char]) -> bool {
    s.iter().all(|c| is_white_space(*c))
}

/// Index of the first occurrence of `needle` in `hay` starting at `from` (ordinal), `None` if absent.
pub fn index_of(hay: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() {
        return if from <= hay.len() { Some(from) } else { None };
    }
    if hay.len() < needle.len() || from > hay.len() - needle.len() {
        return None;
    }
    (from..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

pub fn index_of_str(hay: &[char], needle: &str, from: usize) -> Option<usize> {
    index_of(hay, &chars(needle), from)
}

pub fn index_of_char(hay: &[char], c: char, from: usize) -> Option<usize> {
    if from >= hay.len() {
        return None;
    }
    hay[from..].iter().position(|x| *x == c).map(|p| p + from)
}

pub fn last_index_of_char(hay: &[char], c: char) -> Option<usize> {
    hay.iter().rposition(|x| *x == c)
}

pub fn starts_with_str(s: &[char], p: &str) -> bool {
    let mut it = s.iter();
    p.chars().all(|pc| it.next() == Some(&pc))
}

/// Ordinal ignore-case comparison of two strings (OrdinalIgnoreCase upper-cases invariantly).
pub fn eq_ignore_case(a: &str, b: &str) -> bool {
    if a.len() == b.len() && a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }
    upper_invariant(a) == upper_invariant(b)
}

/// The key under which OrdinalIgnoreCase collections compare strings.
pub fn upper_invariant(s: &str) -> String {
    if s.is_ascii() {
        s.to_ascii_uppercase()
    } else {
        s.chars().map(|c| c.to_uppercase().next().unwrap_or(c)).collect()
    }
}

/// `string.ToLowerInvariant()`.
pub fn lower_invariant(s: &str) -> String {
    s.to_lowercase()
}

/// `Dictionary<string, V>` with insertion-order enumeration (what .NET gives when nothing is
/// removed). `ci` selects `StringComparer.OrdinalIgnoreCase`; otherwise `Ordinal`.
#[derive(Clone, Debug)]
pub struct StrMap<V> {
    ci: bool,
    index: std::collections::HashMap<String, usize>,
    entries: Vec<(String, V)>,
}

impl<V> Default for StrMap<V> {
    fn default() -> Self {
        StrMap {
            ci: false,
            index: Default::default(),
            entries: Vec::new(),
        }
    }
}

impl<V> StrMap<V> {
    /// Ordinal (case-sensitive) comparison.
    pub fn new() -> Self {
        Self::default()
    }
    /// OrdinalIgnoreCase comparison.
    pub fn new_ci() -> Self {
        StrMap {
            ci: true,
            index: Default::default(),
            entries: Vec::new(),
        }
    }
    fn key(&self, k: &str) -> String {
        if self.ci {
            upper_invariant(k)
        } else {
            k.to_string()
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn contains_key(&self, k: &str) -> bool {
        self.index.contains_key(&self.key(k))
    }
    pub fn get(&self, k: &str) -> Option<&V> {
        self.index.get(&self.key(k)).map(|i| &self.entries[*i].1)
    }
    pub fn get_mut(&mut self, k: &str) -> Option<&mut V> {
        let key = self.key(k);
        match self.index.get(&key) {
            Some(i) => Some(&mut self.entries[*i].1),
            None => None,
        }
    }
    /// `dict[key] = value`: replaces the value, keeps the original key spelling and position.
    pub fn insert(&mut self, k: &str, v: V) {
        let key = self.key(k);
        match self.index.get(&key) {
            Some(i) => self.entries[*i].1 = v,
            None => {
                self.index.insert(key, self.entries.len());
                self.entries.push((k.to_string(), v));
            }
        }
    }
    /// `dict.TryAdd`: true if added.
    pub fn try_add(&mut self, k: &str, v: V) -> bool {
        if self.contains_key(k) {
            false
        } else {
            self.insert(k, v);
            true
        }
    }
    pub fn remove(&mut self, k: &str) -> Option<V> {
        let key = self.key(k);
        let i = self.index.remove(&key)?;
        let (_, v) = self.entries.remove(i);
        for x in self.index.values_mut() {
            if *x > i {
                *x -= 1;
            }
        }
        Some(v)
    }
    /// The value for `k`, inserting `make()` first when absent.
    pub fn get_or_insert_with(&mut self, k: &str, make: impl FnOnce() -> V) -> &mut V {
        if !self.contains_key(k) {
            self.insert(k, make());
        }
        self.get_mut(k).unwrap()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut V)> {
        self.entries.iter_mut().map(|(k, v)| (&*k, v))
    }
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.iter().map(|(k, _)| k)
    }
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, v)| v)
    }
    pub fn clear(&mut self) {
        self.index.clear();
        self.entries.clear();
    }
}

/// `HashSet<string>` with insertion-order enumeration.
#[derive(Clone, Debug)]
pub struct StrSet {
    map: StrMap<()>,
}

impl Default for StrSet {
    fn default() -> Self {
        StrSet { map: StrMap::new() }
    }
}

impl StrSet {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_ci() -> Self {
        StrSet { map: StrMap::new_ci() }
    }
    /// True if the item was not yet present.
    pub fn insert(&mut self, s: &str) -> bool {
        self.map.try_add(s, ())
    }
    pub fn contains(&self, s: &str) -> bool {
        self.map.contains_key(s)
    }
    pub fn remove(&mut self, s: &str) -> bool {
        self.map.remove(s).is_some()
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }
    pub fn clear(&mut self) {
        self.map.clear()
    }
}
