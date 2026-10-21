//! JSON shape of the web server's `GET /snippets` (`CalcpadController.GetSnippets`).
//!
//! The response is `{"count":N,"snippets":[...]}` with camelCase property names, null for
//! absent optional fields, and `System.Text.Json` default string escaping (non-ASCII and the
//! HTML-sensitive characters are written as `\uXXXX`). `internal_only` snippets are left out.

use super::models::{SnippetItem, SnippetParameter};
use super::registry::SnippetRegistry;
use std::fmt::Write;

/// Serializes the snippets as the `/snippets` endpoint does. A `None` or whitespace-only
/// `category` returns every snippet, otherwise those whose category starts with it
/// (ignoring case).
pub fn snippets_json(category: Option<&str>) -> String {
    let snippets = match category {
        Some(c) if !c.trim().is_empty() => SnippetRegistry::get_snippets_by_category(c),
        _ => SnippetRegistry::get_all_snippets_array(),
    };
    let mut out = String::with_capacity(snippets.len() * 256);
    let _ = write!(out, "{{\"count\":{},\"snippets\":[", snippets.len());
    for (i, s) in snippets.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_item(&mut out, s);
    }
    out.push_str("]}");
    out
}

fn write_item(out: &mut String, s: &SnippetItem) {
    out.push_str("{\"insert\":");
    write_str(out, &s.insert);
    out.push_str(",\"description\":");
    write_str(out, &s.description);
    out.push_str(",\"documentation\":");
    write_opt(out, s.documentation.as_deref());
    out.push_str(",\"example\":");
    write_opt(out, s.example.as_deref());
    out.push_str(",\"label\":");
    write_opt(out, s.label.as_deref());
    out.push_str(",\"category\":");
    write_str(out, &s.category);
    out.push_str(",\"quickType\":");
    write_opt(out, s.quick_type.as_deref());
    out.push_str(",\"keywordType\":");
    write_opt(out, s.keyword_type.as_deref());
    out.push_str(",\"returnType\":");
    write_opt(out, s.return_type.map(|t| t.name()));
    out.push_str(",\"returnTypeDescription\":");
    write_opt(out, s.return_type_description.as_deref());
    let _ = write!(
        out,
        ",\"isElementWise\":{},\"acceptsAnyCount\":{},\"parameters\":",
        s.is_element_wise, s.accepts_any_count
    );
    match &s.parameters {
        None => out.push_str("null"),
        Some(ps) => {
            out.push('[');
            for (i, p) in ps.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_parameter(out, p);
            }
            out.push(']');
        }
    }
    out.push('}');
}

fn write_parameter(out: &mut String, p: &SnippetParameter) {
    out.push_str("{\"name\":");
    write_str(out, &p.name);
    out.push_str(",\"description\":");
    write_opt(out, p.description.as_deref());
    out.push_str(",\"type\":");
    write_str(out, p.param_type.name());
    out.push_str(",\"typeDescription\":");
    write_opt(out, p.type_description.as_deref());
    let _ = write!(
        out,
        ",\"isOptional\":{},\"isVariadic\":{}}}",
        p.is_optional, p.is_variadic
    );
}

fn write_opt(out: &mut String, s: Option<&str>) {
    match s {
        Some(s) => write_str(out, s),
        None => out.push_str("null"),
    }
}

/// Writes a JSON string the way `System.Text.Json`'s default encoder does.
fn write_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            ' '..='~' if !matches!(c, '"' | '&' | '\'' | '+' | '<' | '>' | '`') => out.push(c),
            _ => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{unit:04X}");
                }
            }
        }
    }
    out.push('"');
}
