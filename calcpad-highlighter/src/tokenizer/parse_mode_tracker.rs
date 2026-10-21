//! ParseModeTracker.cs
use crate::linter::models::ParseMode;
use crate::text::is_white_space;

#[derive(Debug, Default)]
pub struct ParseModeTracker {
    stack: Vec<ParseMode>,
    mode: Option<ParseMode>,
}

impl ParseModeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> ParseMode {
        self.mode.unwrap_or(ParseMode::Cpd)
    }

    /// Applies a trimmed line; true when it was an opener or end directive.
    pub fn apply(&mut self, trimmed_line: &[char]) -> bool {
        if is_end_directive(trimmed_line, self.mode()) {
            self.mode = Some(self.stack.pop().unwrap_or(ParseMode::Cpd));
            return true;
        }
        if let Some(mode) = try_get_opener(trimmed_line) {
            self.stack.push(self.mode());
            self.mode = Some(mode);
            return true;
        }
        false
    }

    pub fn is_macro_directive(trimmed_line: &[char]) -> bool {
        starts_with_word(trimmed_line, "#def")
            || starts_with_word(trimmed_line, "#end def")
            || starts_with_word(trimmed_line, "#include")
    }

    pub fn is_directive(trimmed_line: &[char], mode: ParseMode) -> bool {
        try_get_opener(trimmed_line).is_some()
            || is_end_directive(trimmed_line, mode)
            || Self::is_macro_directive(trimmed_line)
    }
}

fn try_get_opener(s: &[char]) -> Option<ParseMode> {
    if starts_with_word(s, "#html") {
        Some(ParseMode::Html)
    } else if starts_with_word(s, "#cpd") {
        Some(ParseMode::Cpd)
    } else if starts_with_word(s, "#markdown") {
        Some(ParseMode::Markdown)
    } else {
        None
    }
}

fn is_end_directive(s: &[char], mode: ParseMode) -> bool {
    match mode {
        ParseMode::Html => starts_with_word(s, "#end html"),
        ParseMode::Markdown => starts_with_word(s, "#end markdown"),
        ParseMode::Cpd => {
            starts_with_word(s, "#end html") || starts_with_word(s, "#end cpd") || starts_with_word(s, "#end markdown")
        }
    }
}

/// `s.StartsWith(word, OrdinalIgnoreCase)` followed by the end of the line or whitespace.
fn starts_with_word(s: &[char], word: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    if s.len() < w.len() {
        return false;
    }
    let head: String = s[..w.len()].iter().collect();
    crate::text::eq_ignore_case(&head, word) && (s.len() == w.len() || is_white_space(s[w.len()]))
}
