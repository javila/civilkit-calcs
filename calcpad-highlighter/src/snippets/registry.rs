//! `Snippets/SnippetRegistry.cs`: aggregates every snippet category and provides the cached
//! lookups used by the linter's built-in name sets and function signatures.
//!
//! Lookups are computed once and cached (`OnceLock`). Case-insensitive sets and maps
//! ([`NameSet`], [`FoldedMap`]) correspond to the C# `StringComparer.OrdinalIgnoreCase` ones.

use super::calcpad_type::CalcpadType;
use super::data::{self, data};
use super::folded::{eq_ignore_case, starts_with_ignore_case, FoldedMap, NameSet};
use super::models::SnippetItem;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

/// Namespace for the static `SnippetRegistry` accessors.
pub struct SnippetRegistry;

impl SnippetRegistry {
    /// All snippets from every category, in registry order (constants, operators, functions,
    /// vector functions, matrix functions, keywords, commands, settings, units, format
    /// strings, HTML, CSS, Markdown, symbols). The C# frozen set has no defined order;
    /// insertion order is used here.
    pub fn all_snippets() -> &'static [&'static SnippetItem] {
        static ALL: OnceLock<Vec<&'static SnippetItem>> = OnceLock::new();
        ALL.get_or_init(|| data().all().collect())
    }

    /// Snippets grouped by their top-level category (the part before the first `/`),
    /// preserving registry order within each group.
    pub fn by_category() -> &'static HashMap<String, Vec<&'static SnippetItem>> {
        static BY_CAT: OnceLock<HashMap<String, Vec<&'static SnippetItem>>> = OnceLock::new();
        BY_CAT.get_or_init(|| {
            let mut map: HashMap<String, Vec<&'static SnippetItem>> = HashMap::new();
            for s in Self::all_snippets() {
                map.entry(top_level_category(&s.category).to_string())
                    .or_default()
                    .push(s);
            }
            map
        })
    }

    /// All snippets for serialization. Excludes `internal_only` snippets, which exist for the
    /// linter's built-in sets only.
    pub fn get_all_snippets_array() -> Vec<&'static SnippetItem> {
        Self::all_snippets()
            .iter()
            .copied()
            .filter(|s| !s.internal_only)
            .collect()
    }

    /// Snippets whose category starts with `category_prefix` (ordinal, ignoring case).
    /// Excludes `internal_only` snippets.
    pub fn get_snippets_by_category(category_prefix: &str) -> Vec<&'static SnippetItem> {
        Self::all_snippets()
            .iter()
            .copied()
            .filter(|s| !s.internal_only && starts_with_ignore_case(&s.category, category_prefix))
            .collect()
    }

    /// All function names (function, vector function and matrix function files).
    pub fn get_function_names() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| {
            let mut names = NameSet::new();
            for s in all_function_items() {
                add_nonempty(&mut names, extract_function_name(&s.insert));
            }
            names
        })
    }

    /// Keyword names with the `#` prefix (snippets with keyword type "Keyword").
    pub fn get_keyword_names() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| keyword_names_of_type("Keyword"))
    }

    /// Command names with the `$` prefix.
    pub fn get_command_names() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| {
            let mut names = NameSet::new();
            for s in data::command_snippets() {
                add_nonempty(&mut names, extract_command_name(&s.insert));
            }
            names
        })
    }

    /// Unit names (case-sensitive).
    pub fn get_unit_names() -> &'static HashSet<String> {
        static SET: OnceLock<HashSet<String>> = OnceLock::new();
        SET.get_or_init(|| {
            data::unit_snippets()
                .iter()
                .map(|s| s.insert.trim())
                .filter(|n| !n.is_empty())
                .map(str::to_string)
                .collect()
        })
    }

    /// Constant names (case-sensitive): snippets with keyword type "Constant".
    pub fn get_constant_names() -> &'static HashSet<String> {
        static SET: OnceLock<HashSet<String>> = OnceLock::new();
        SET.get_or_init(|| {
            data::constant_snippets()
                .iter()
                .filter(|s| has_keyword_type(s, "Constant"))
                .map(|s| s.insert.trim())
                .filter(|n| !n.is_empty())
                .map(str::to_string)
                .collect()
        })
    }

    /// Setting names (special backend variables such as `PlotHeight`).
    pub fn get_setting_names() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| {
            let mut names = NameSet::new();
            for s in data::setting_snippets() {
                add_nonempty(&mut names, extract_setting_name(&s.insert));
            }
            names
        })
    }

    /// Control block keyword names (`#if`, `#repeat`, `#for`, `#while`, `#def`, `#else`, ...).
    pub fn get_control_block_keyword_names() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| keyword_names_of_type("ControlBlockKeyword"))
    }

    /// End keyword names (`#end if`, `#end def`, `#loop`, ...).
    pub fn get_end_keyword_names() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| keyword_names_of_type("EndKeyword"))
    }

    /// Operator characters: single-character inserts of the operator snippets.
    pub fn get_operators() -> &'static HashSet<char> {
        static SET: OnceLock<HashSet<char>> = OnceLock::new();
        SET.get_or_init(|| {
            let mut ops = HashSet::new();
            for s in data::operator_snippets() {
                let mut chars = s.insert.trim().chars();
                // C# `trimmed.Length == 1` counts UTF-16 units, so astral characters never match.
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    if c.len_utf16() == 1 {
                        ops.insert(c);
                    }
                }
            }
            ops
        })
    }

    /// Function snippets by name. Where overloads exist, the most permissive signature is
    /// chosen for linting: `accepts_any_count` first, then lowest `min_params`, then highest
    /// `max_params` (-1 meaning unlimited); ties keep the earliest item.
    pub fn get_function_snippets_by_name() -> &'static FoldedMap<&'static SnippetItem> {
        static MAP: OnceLock<FoldedMap<&'static SnippetItem>> = OnceLock::new();
        MAP.get_or_init(|| {
            let mut map = FoldedMap::new();
            for (name, overloads) in Self::get_function_overloads().iter() {
                map.insert_if_absent(name, select_most_permissive_signature(overloads));
            }
            map
        })
    }

    /// Every function signature grouped by name (case-insensitive), in source order.
    pub fn get_function_overloads() -> &'static FoldedMap<Vec<&'static SnippetItem>> {
        static MAP: OnceLock<FoldedMap<Vec<&'static SnippetItem>>> = OnceLock::new();
        MAP.get_or_init(|| {
            let mut map: FoldedMap<Vec<&'static SnippetItem>> = FoldedMap::new();
            for s in all_function_items() {
                let name = extract_function_name(&s.insert);
                if !name.is_empty() {
                    map.entry_or_insert_with(name, Vec::new).push(s);
                }
            }
            map
        })
    }

    /// Overloads of a function by name; empty when the function is not found.
    pub fn get_overloads_for_function(function_name: &str) -> &'static [&'static SnippetItem] {
        Self::get_function_overloads()
            .get(function_name)
            .map_or(&[], Vec::as_slice)
    }

    /// Names of functions returning vectors (vector function file first, then functions).
    pub fn get_vector_returning_functions() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| returning_functions(data::vector_function_snippets(), CalcpadType::Vector))
    }

    /// Names of functions returning matrices (matrix function file first, then functions).
    pub fn get_matrix_returning_functions() -> &'static NameSet {
        static SET: OnceLock<NameSet> = OnceLock::new();
        SET.get_or_init(|| returning_functions(data::matrix_function_snippets(), CalcpadType::Matrix))
    }
}

fn top_level_category(category: &str) -> &str {
    category.find('/').map_or(category, |i| &category[..i])
}

fn all_function_items() -> impl Iterator<Item = &'static SnippetItem> {
    data::function_snippets()
        .iter()
        .chain(data::vector_function_snippets())
        .chain(data::matrix_function_snippets())
}

fn add_nonempty(set: &mut NameSet, name: &str) {
    if !name.is_empty() {
        set.insert(name);
    }
}

fn has_keyword_type(s: &SnippetItem, keyword_type: &str) -> bool {
    s.keyword_type
        .as_deref()
        .is_some_and(|k| eq_ignore_case(k, keyword_type))
}

fn keyword_names_of_type(keyword_type: &str) -> NameSet {
    let mut names = NameSet::new();
    for s in data::keyword_snippets() {
        if has_keyword_type(s, keyword_type) {
            add_nonempty(&mut names, &extract_keyword_name(&s.insert));
        }
    }
    names
}

/// Names from `primary` whose return type is `ty`, then from the plain function file.
fn returning_functions(primary: &'static [SnippetItem], ty: CalcpadType) -> NameSet {
    let mut names = NameSet::new();
    for s in primary.iter().chain(data::function_snippets()) {
        if s.return_type == Some(ty) {
            add_nonempty(&mut names, extract_function_name(&s.insert));
        }
    }
    names
}

fn select_most_permissive_signature(overloads: &[&'static SnippetItem]) -> &'static SnippetItem {
    // (accepts_any_count desc, min_params asc, max_params desc with -1 as unlimited); first wins ties.
    let key = |s: &SnippetItem| {
        let max = if s.max_params() == -1 { i32::MAX } else { s.max_params() };
        (!s.accepts_any_count, s.min_params(), std::cmp::Reverse(max))
    };
    let mut best = overloads[0];
    for &s in &overloads[1..] {
        if key(s) < key(best) {
            best = s;
        }
    }
    best
}

/// Function name from insert text, e.g. `sin(§)` gives `sin`. Empty when there is no `(`
/// after the first character.
pub(crate) fn extract_function_name(insert: &str) -> &str {
    match insert.find('(') {
        Some(i) if i > 0 => insert[..i].trim(),
        _ => "",
    }
}

fn is_word_end(c: char) -> bool {
    matches!(c, ' ' | '\n' | '\t' | '§')
}

/// Keyword name from insert text, e.g. `#if §` gives `#if`. Handles the multi-word keywords
/// `#end x`, `#else x` and `#md x`, whose second word is lower-cased.
pub(crate) fn extract_keyword_name(insert: &str) -> String {
    if !insert.starts_with('#') {
        return String::new();
    }
    let first_end = match insert.find(is_word_end) {
        Some(i) if i > 0 => i,
        _ => return insert.trim().to_string(),
    };
    let first_word = insert[..first_end].trim();
    let first_lower = first_word.trim_start_matches('#').to_lowercase();
    if first_lower == "end" || first_lower == "else" || first_lower == "md" {
        let remaining = insert[first_end..].trim_start();
        if !remaining.is_empty() && !remaining.starts_with('§') && !remaining.starts_with('\n') {
            match remaining.find(is_word_end) {
                Some(second_end) if second_end > 0 => {
                    let second = remaining[..second_end].trim().to_lowercase();
                    return format!("{first_word} {second}");
                }
                _ => {
                    let second = remaining.trim().to_lowercase();
                    if !second.is_empty() {
                        return format!("{first_word} {second}");
                    }
                }
            }
        }
    }
    first_word.to_string()
}

/// Command name from insert text, e.g. `$Root{§ @ § = § : §}` gives `$Root`. Empty unless the
/// text starts with `$`.
pub(crate) fn extract_command_name(insert: &str) -> &str {
    if !insert.starts_with('$') {
        return "";
    }
    match insert.find(['{', ' ', '=']) {
        Some(i) if i > 0 => insert[..i].trim(),
        _ => insert.trim(),
    }
}

/// Setting name from insert text, e.g. `PlotHeight = §` gives `PlotHeight`.
pub(crate) fn extract_setting_name(insert: &str) -> &str {
    match insert.find('=') {
        Some(i) if i > 0 => insert[..i].trim(),
        _ => insert.trim(),
    }
}
