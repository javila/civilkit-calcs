//! CalcpadBuiltIns.cs: built-in language elements derived from the snippet registry, plus the
//! hard coded semantic groupings and the control block helpers.
use crate::snippets::{NameSet, SnippetRegistry};
use crate::text::{eq_ignore_case, lower_invariant, StrSet};
use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ControlBlockType {
    None,
    If,
    ElseIf,
    Else,
    EndIf,
    Repeat,
    For,
    While,
    Loop,
    Def,
    EndDef,
    Break,
    Continue,
}

pub fn functions() -> &'static NameSet {
    SnippetRegistry::get_function_names()
}
/// Keyword names with the `#` prefix (ignore case).
pub fn keywords() -> &'static NameSet {
    SnippetRegistry::get_keyword_names()
}
/// Command names with the `$` prefix (ignore case).
pub fn commands() -> &'static NameSet {
    SnippetRegistry::get_command_names()
}
pub fn units() -> &'static HashSet<String> {
    SnippetRegistry::get_unit_names()
}
pub fn common_constants() -> &'static HashSet<String> {
    SnippetRegistry::get_constant_names()
}
pub fn operators() -> &'static HashSet<char> {
    SnippetRegistry::get_operators()
}
pub fn control_block_keywords() -> &'static NameSet {
    SnippetRegistry::get_control_block_keyword_names()
}
pub fn end_keywords() -> &'static NameSet {
    SnippetRegistry::get_end_keyword_names()
}

fn trim_hash(s: &str) -> &str {
    s.trim_start_matches('#')
}

/// Valid hash keywords without the `#` prefix (ignore case).
pub fn valid_hash_keywords() -> &'static StrSet {
    static SET: OnceLock<StrSet> = OnceLock::new();
    SET.get_or_init(|| {
        let mut all = StrSet::new_ci();
        for set in [keywords(), control_block_keywords(), end_keywords()] {
            for k in set.iter() {
                let w = trim_hash(k);
                if !w.is_empty() {
                    all.insert(w);
                }
            }
        }
        all
    })
}

/// Control block keywords that start blocks (everything but `#else` and `#else if`).
pub fn control_block_starters() -> &'static StrSet {
    static SET: OnceLock<StrSet> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = StrSet::new_ci();
        for k in control_block_keywords().iter() {
            if !eq_ignore_case(k, "#else") && !eq_ignore_case(k, "#else if") {
                s.insert(k);
            }
        }
        s
    })
}

pub fn control_block_enders() -> &'static NameSet {
    end_keywords()
}

/// All commands except `$While`, `$Block` and `$Inline`.
pub fn commands_excluding_command_blocks() -> &'static StrSet {
    static SET: OnceLock<StrSet> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = StrSet::new_ci();
        for c in commands().iter() {
            if !is_command_block(c) {
                s.insert(c);
            }
        }
        s
    })
}

/// (Name, NameWithBrace) of `commands_excluding_command_blocks`.
pub fn commands_with_brace() -> &'static Vec<(String, String)> {
    static V: OnceLock<Vec<(String, String)>> = OnceLock::new();
    V.get_or_init(|| {
        commands_excluding_command_blocks()
            .iter()
            .map(|c| (c.clone(), format!("{c}{{")))
            .collect()
    })
}

pub fn is_comparison_operator(c: char) -> bool {
    matches!(c, '<' | '>' | '≤' | '≥' | '≡' | '≠')
}

const STANDALONE: [&str; 29] = [
    "#else",
    "#end if",
    "#end def",
    "#loop",
    "#break",
    "#continue",
    "#local",
    "#global",
    "#pause",
    "#input",
    "#end hide",
    "#end show",
    "#end pre",
    "#end post",
    "#end val",
    "#end equ",
    "#end noc",
    "#end nosub",
    "#end novar",
    "#end varsub",
    "#end html",
    "#end cpd",
    "#end markdown",
    "#split",
    "#wrap",
    "#phasor",
    "#complex",
    "#rad",
    "#deg",
];

/// Keywords that are standalone (ignore case). `#gra` is the 30th entry.
pub fn is_standalone_keyword(s: &str) -> bool {
    STANDALONE.iter().any(|k| eq_ignore_case(k, s)) || eq_ignore_case("#gra", s)
}

pub const SETTINGS_VARIABLES: [&str; 12] = [
    "PlotWidth",
    "PlotHeight",
    "PlotStep",
    "PlotSVG",
    "PlotAdaptive",
    "PlotSmooth",
    "PlotShadows",
    "PlotLightDir",
    "PlotPalette",
    "Precision",
    "Tol",
    "ReturnAngleUnits",
];

/// Case sensitive.
pub fn is_settings_variable(s: &str) -> bool {
    SETTINGS_VARIABLES.contains(&s)
}

pub fn is_multi_word_first_word(s: &str) -> bool {
    ["else", "end", "md"].iter().any(|k| eq_ignore_case(k, s))
}

pub fn is_command_block(s: &str) -> bool {
    ["$While", "$Block", "$Inline"].iter().any(|k| eq_ignore_case(k, s))
}

// Insertion order matters: the prefix match in `get_block_type` takes the first hit.
const KEYWORD_TO_TYPE: [(&str, ControlBlockType); 12] = [
    ("#if", ControlBlockType::If),
    ("#else if", ControlBlockType::ElseIf),
    ("#else", ControlBlockType::Else),
    ("#end if", ControlBlockType::EndIf),
    ("#repeat", ControlBlockType::Repeat),
    ("#for", ControlBlockType::For),
    ("#while", ControlBlockType::While),
    ("#loop", ControlBlockType::Loop),
    ("#def", ControlBlockType::Def),
    ("#end def", ControlBlockType::EndDef),
    ("#break", ControlBlockType::Break),
    ("#continue", ControlBlockType::Continue),
];

pub fn normalize_line(line: &str) -> String {
    lower_invariant(line.trim_matches(crate::text::is_white_space))
}

pub fn get_block_type(line: &str) -> ControlBlockType {
    let normalized = normalize_line(line);
    for (k, t) in KEYWORD_TO_TYPE.iter() {
        if eq_ignore_case(k, &normalized) {
            return *t;
        }
    }
    for (k, t) in KEYWORD_TO_TYPE.iter() {
        if normalized.starts_with(&format!("{k} ")) {
            return *t;
        }
    }
    ControlBlockType::None
}

pub fn is_block_starter(t: ControlBlockType) -> bool {
    use ControlBlockType::*;
    matches!(t, If | Repeat | For | While | Def)
}

pub fn is_loop_block_starter(t: ControlBlockType) -> bool {
    use ControlBlockType::*;
    matches!(t, Repeat | For | While)
}

pub fn get_keyword_string(t: ControlBlockType) -> &'static str {
    KEYWORD_TO_TYPE
        .iter()
        .find(|(_, x)| *x == t)
        .map(|(k, _)| *k)
        .unwrap_or("")
}

pub fn get_expected_ender_type(starter: ControlBlockType) -> ControlBlockType {
    use ControlBlockType::*;
    match starter {
        If => EndIf,
        Repeat | For | While => Loop,
        Def => EndDef,
        _ => None,
    }
}

pub fn get_expected_ender_keyword(starter: ControlBlockType) -> &'static str {
    get_keyword_string(get_expected_ender_type(starter))
}

pub fn matches_ender(starter: ControlBlockType, ender: ControlBlockType) -> bool {
    let expected = get_expected_ender_type(starter);
    expected != ControlBlockType::None && expected == ender
}

pub fn track_control_blocks(line: &str, stack: &mut Vec<(ControlBlockType, i32)>, line_number: i32) {
    let block_type = get_block_type(line);
    if is_block_starter(block_type) {
        stack.push((block_type, line_number));
    } else if matches!(
        block_type,
        ControlBlockType::EndIf | ControlBlockType::Loop | ControlBlockType::EndDef
    ) {
        if let Some(top) = stack.last() {
            if matches_ender(top.0, block_type) {
                stack.pop();
            }
        }
    }
}
