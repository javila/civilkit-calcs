//! CalcpadCharacterHelpers.cs: identifier character classification.
use crate::text::{is_digit, is_letter, is_letter_or_digit};

pub fn is_greek_letter(c: char) -> bool {
    ('\u{03B1}'..='\u{03C9}').contains(&c) || ('\u{0391}'..='\u{03A9}').contains(&c)
}

/// ° ø Ø ∡ ℧
pub fn is_special_math_char(c: char) -> bool {
    matches!(c, '°' | 'ø' | 'Ø' | '∡' | '℧')
}

pub fn is_subscript_digit(c: char) -> bool {
    ('₀'..='₉').contains(&c)
}

pub fn is_superscript_digit(c: char) -> bool {
    ('⁰'..='⁹').contains(&c)
}

pub fn is_identifier_start_char(c: char) -> bool {
    is_letter(c) || is_greek_letter(c) || is_special_math_char(c)
}

pub fn is_identifier_start_char_with_underscore(c: char) -> bool {
    is_letter(c) || c == '_' || is_greek_letter(c)
}

pub fn is_identifier_char(c: char) -> bool {
    is_letter_or_digit(c)
        || c == '_'
        || is_greek_letter(c)
        || is_subscript_digit(c)
        || is_superscript_digit(c)
        || is_combining_mark(c)
}

/// Acute, macron, dot above and diaeresis combining marks.
pub fn is_combining_mark(c: char) -> bool {
    matches!(c, '\u{0301}' | '\u{0304}' | '\u{0307}' | '\u{0308}')
}

pub fn is_identifier_char_with_dollar(c: char) -> bool {
    is_identifier_char(c) || c == '$'
}

pub fn is_letter_for_tokenizer(c: char) -> bool {
    is_letter(c) || c == '_' || is_greek_letter(c) || is_special_math_char(c)
}

pub fn is_unit_start(c: char) -> bool {
    matches!(c, '°' | 'µ' | 'μ' | '‰' | '‱')
}

pub fn extract_identifier(text: &str) -> String {
    text.chars().take_while(|c| is_identifier_char(*c)).collect()
}

/// `Validator.IsMacroLetter`: ASCII letters, underscore, and digits after the first position.
pub fn is_macro_letter(c: char, position: usize) -> bool {
    c.is_ascii_alphabetic() || c == '_' || (is_digit(c) && position > 0)
}
