//! Case-insensitive (ordinal) string-keyed collections, matching .NET
//! `StringComparer.OrdinalIgnoreCase` for the identifier-like names used here.

use std::collections::HashMap;

/// Folds a string the way ordinal-ignore-case comparison does: per character simple
/// upper-casing (characters whose upper case is not a single character are kept).
pub(crate) fn fold(s: &str) -> String {
    s.chars().map(fold_char).collect()
}

fn fold_char(c: char) -> char {
    let mut up = c.to_uppercase();
    match (up.next(), up.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

/// Ordinal-ignore-case string equality.
pub(crate) fn eq_ignore_case(a: &str, b: &str) -> bool {
    a.chars().map(fold_char).eq(b.chars().map(fold_char))
}

/// Ordinal-ignore-case `starts_with`.
pub(crate) fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    let mut it = s.chars().map(fold_char);
    prefix.chars().map(fold_char).all(|p| it.next() == Some(p))
}

/// A map with case-insensitive string keys. The first inserted spelling of a key is kept.
#[derive(Debug, Clone)]
pub struct FoldedMap<V> {
    map: HashMap<String, (String, V)>,
}

impl<V> Default for FoldedMap<V> {
    fn default() -> Self {
        FoldedMap { map: HashMap::new() }
    }
}

impl<V> FoldedMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.map.get(&fold(key)).map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(&fold(key))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterates the keys in their original (first inserted) spelling, in no particular order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.values().map(|(k, _)| k.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.map.values().map(|(k, v)| (k.as_str(), v))
    }

    /// Mutable access to the value for `key`, inserting `default()` when absent.
    pub(crate) fn entry_or_insert_with(&mut self, key: &str, default: impl FnOnce() -> V) -> &mut V {
        &mut self
            .map
            .entry(fold(key))
            .or_insert_with(|| (key.to_string(), default()))
            .1
    }

    pub(crate) fn insert_if_absent(&mut self, key: &str, value: V) {
        self.map.entry(fold(key)).or_insert_with(|| (key.to_string(), value));
    }
}

/// A set of strings compared ignoring case.
#[derive(Debug, Clone, Default)]
pub struct NameSet {
    inner: FoldedMap<()>,
}

impl NameSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterates the names in their original spelling, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.keys()
    }

    pub(crate) fn insert(&mut self, name: &str) {
        self.inner.insert_if_absent(name, ());
    }
}
