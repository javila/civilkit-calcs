//! CharClassifier.cs
use crate::linter::helpers::character_helpers::{
    is_greek_letter, is_special_math_char, is_subscript_digit, is_superscript_digit,
};
use crate::text::{is_letter, is_white_space};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharClass {
    Other,
    Whitespace,
    Digit,
    Letter,
    Operator,
    Bracket,
    Delimiter,
    Dot,
    Dollar,
    Hash,
    Quote,
    Question,
    Comma,
    Newline,
}

fn ascii_class(c: char) -> CharClass {
    use CharClass::*;
    match c {
        ' ' | '\t' => Whitespace,
        '\r' | '\n' => Newline,
        '0'..='9' => Digit,
        'a'..='z' | 'A'..='Z' | '_' => Letter,
        '!' | '^' | '/' | '\\' | '*' | '-' | '+' | '<' | '>' | '=' => Operator,
        '(' | ')' | '[' | ']' | '{' | '}' => Bracket,
        ';' | '|' | '&' | '@' | ':' => Delimiter,
        '.' => Dot,
        '$' => Dollar,
        '#' => Hash,
        '\'' | '"' => Quote,
        '?' => Question,
        ',' => Comma,
        _ => Other,
    }
}

pub fn classify(c: char) -> CharClass {
    if (c as u32) < 128 {
        return ascii_class(c);
    }
    if matches!(c, '÷' | '⦼' | '≡' | '≠' | '≤' | '≥' | '∧' | '∨' | '⊕' | '∠' | '←' | '·') {
        CharClass::Operator
    } else if is_greek_letter(c)
        || is_special_math_char(c)
        || is_subscript_digit(c)
        || is_superscript_digit(c)
        || is_letter(c)
    {
        CharClass::Letter
    } else if is_white_space(c) {
        CharClass::Whitespace
    } else {
        CharClass::Other
    }
}

pub fn is_operator(c: char) -> bool {
    classify(c) == CharClass::Operator
}

pub fn is_bracket(c: char) -> bool {
    (c as u32) < 128 && ascii_class(c) == CharClass::Bracket
}

pub fn is_delimiter(c: char) -> bool {
    (c as u32) < 128 && ascii_class(c) == CharClass::Delimiter
}

/// `;|&@:({[` continue a line implicitly when they end it.
pub fn is_line_extension(c: char) -> bool {
    is_delimiter(c) || c == '(' || c == '{' || c == '['
}
