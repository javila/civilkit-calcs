//! The names a Python block hands to the worksheet with `export(...)` and `export_data(...)`.
//!
//! A sheet spells `sigma_c` as `σ_c`, `phi` as `φ` and `Mstar` as `M_star`, and the quantity lines after a block use that
//! spelling. A Python block is otherwise copied as written, so the names it exports are rewritten here (and only those): the
//! keyword names of the call and, in the positional form `export("name", value)`, the name string. Everything else in the
//! block is left alone.

use crate::names::{is_name, is_name_char, is_name_start, rename};

/// Scans the lines of one Python block, remembering whether an export call is still open from an earlier line.
#[derive(Default)]
pub(crate) struct ExportScan {
    /// Bracket depth inside the open export call (0 when none is open; 1 is directly inside its parentheses).
    depth: usize,
    /// The last significant character seen inside the call (`(` or `,` mean the next word starts an argument).
    prev: char,
    /// Whether the next string literal is the first argument of the call.
    first_arg: bool,
}

impl ExportScan {
    pub(crate) fn line(&mut self, line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '#' {
                out.extend(&chars[i..]);
                break;
            }
            if c == '"' || c == '\'' {
                let mut j = i + 1;
                while j < chars.len() && chars[j] != c {
                    j += if chars[j] == '\\' { 2 } else { 1 };
                }
                let end = (j + 1).min(chars.len());
                let lit: String = chars[i..end].iter().collect();
                let inner: String = chars[(i + 1).min(end)..j.min(end)].iter().collect();
                if self.depth == 1 && self.first_arg && matches!(self.prev, '(') && is_name(&inner) && j < chars.len() {
                    out.push(c);
                    out.push_str(&rename(&inner));
                    out.push(c);
                } else {
                    out.push_str(&lit);
                }
                self.first_arg = false;
                if self.depth > 0 {
                    self.prev = 's';
                }
                i = end;
                continue;
            }
            if is_name_start(c) {
                let mut j = i;
                while j < chars.len() && is_name_char(chars[j]) {
                    j += 1;
                }
                let word: String = chars[i..j].iter().collect();
                let after = chars[j..].iter().find(|ch| !ch.is_whitespace()).copied();
                let eq_next = {
                    let k = chars[j..].iter().position(|ch| !ch.is_whitespace()).map(|p| j + p);
                    matches!(k, Some(k) if chars[k] == '=' && chars.get(k + 1) != Some(&'='))
                };
                if self.depth == 0 {
                    let before = out.trim_end().chars().last();
                    let free = !matches!(before, Some('.')) && !before.map_or(false, is_name_char);
                    if free && (word == "export" || word == "export_data") && after == Some('(') {
                        out.push_str(&word);
                        // consume up to and including the opening parenthesis
                        let k = chars[j..].iter().position(|ch| *ch == '(').map(|p| j + p).unwrap_or(j);
                        out.extend(&chars[j..=k]);
                        self.depth = 1;
                        self.prev = '(';
                        self.first_arg = true;
                        i = k + 1;
                        continue;
                    }
                    out.push_str(&word);
                } else {
                    if self.depth == 1 && matches!(self.prev, '(' | ',') && eq_next {
                        out.push_str(&rename(&word));
                    } else {
                        out.push_str(&word);
                    }
                    self.prev = 'w';
                    self.first_arg = false;
                }
                i = j;
                continue;
            }
            if self.depth > 0 {
                match c {
                    '(' | '[' | '{' => self.depth += 1,
                    ')' | ']' | '}' => self.depth -= 1,
                    _ => {}
                }
                if !c.is_whitespace() {
                    self.prev = c;
                    if c == ',' && self.depth == 1 {
                        self.first_arg = false;
                    }
                }
            }
            out.push(c);
            i += 1;
        }
        out
    }
}
