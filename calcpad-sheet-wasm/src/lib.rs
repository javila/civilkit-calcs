//! WebAssembly binding of the sheet syntax compiler (civilkit-calcs-sheet). One function does
//! the work: `compile` answers a JSON string, so the boundary stays a single call.

use calcpad_sheet::{compile_with, format as format_sheet, is_sheet as is_sheet_text, LineKind, Options, Severity};
use wasm_bindgen::prelude::*;

fn quote(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// The compiled worksheet as JSON: `{ text, map, diagnostics, latex, defs, kinds }`. Lines are 0 based.
/// With `plain_tex` formulas are written as italic TeX source instead of placeholders.
pub fn compile_json(src: &str, plain_tex: bool) -> String {
    let c = compile_with(src, &Options { plain_tex });
    let mut o = String::from("{\"text\":");
    quote(&c.text, &mut o);
    o.push_str(",\"map\":[");
    o.push_str(&c.map.iter().map(usize::to_string).collect::<Vec<_>>().join(","));
    o.push_str("],\"diagnostics\":[");
    for (i, d) in c.diagnostics.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str(&format!(
            "{{\"line\":{},\"severity\":\"{}\",\"message\":",
            d.line,
            if d.severity == Severity::Error {
                "error"
            } else {
                "warning"
            }
        ));
        quote(&d.message, &mut o);
        o.push('}');
    }
    o.push_str(&format!("],\"latex\":{},\"defs\":[", c.uses_latex));
    for (i, d) in c.defs.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str("{\"name\":");
        quote(&d.name, &mut o);
        o.push_str(",\"calcpad\":");
        quote(&d.calcpad, &mut o);
        o.push_str(&format!(",\"line\":{}}}", d.line));
    }
    o.push_str("],\"kinds\":\"");
    // One letter per sheet line: b blank, p prose, q quantity, k block, r raw, g plot, c condition.
    for k in &c.kinds {
        o.push(match k {
            LineKind::Blank => 'b',
            LineKind::Prose => 'p',
            LineKind::Quantity => 'q',
            LineKind::Block => 'k',
            LineKind::Raw => 'r',
            LineKind::Plot => 'g',
            LineKind::Condition => 'c',
        });
    }
    o.push_str("\"}");
    o
}

#[wasm_bindgen]
pub fn compile(src: &str, plain_tex: bool) -> String {
    compile_json(src, plain_tex)
}

/// The sheet, prettified (docs/sheet-syntax.md).
#[wasm_bindgen]
pub fn format(src: &str) -> String {
    format_sheet(src)
}

#[wasm_bindgen]
pub fn is_sheet(src: &str) -> bool {
    is_sheet_text(src)
}

#[cfg(test)]
mod tests {
    #[test]
    fn answers_json() {
        let j = super::compile_json("<!--sheet-->\na: 1 m", false);
        assert!(j.starts_with("{\"text\":\"a = 1 m\\n\""), "{j}");
        assert!(j.contains("\"map\":[1]"));
    }
}
