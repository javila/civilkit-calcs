//! Helpers the test files share.
#![allow(dead_code)]

/// The page as it was before equations were shown as the author's: the computed form of every
/// region (a definition echoes its value, substituted steps shown), so numbers can be read from the
/// engine's output. Only the wrappers the display adds are removed: the number settings, the
/// author's equation (a `#noc` block or a MathML line) after a hidden computation, and the MathML
/// that carries an evaluation's left side around its `#val` result.
pub fn legacy(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut skip = vec![false; lines.len()];
    let is_eq = |l: &str| l.trim_start().starts_with("'<div class=\"imp-eq\"");
    for i in 0..lines.len() {
        let t = lines[i].trim();
        if t == "#nosub" || t.starts_with("#round") || t.starts_with("#format") || t.starts_with("#settings") {
            skip[i] = true;
        } else if t == "#val" && i >= 1 && is_eq(lines[i - 1]) {
            // `<div..><math..>HEAD` / `#val` / expression / `#end val` / `</math></div>`: the expression stays.
            skip[i - 1] = true;
            skip[i] = true;
            if let Some(end) = (i + 1..lines.len()).find(|&k| lines[k].trim() == "#end val") {
                skip[end] = true;
                if lines.get(end + 1).map(|l| l.trim()) == Some("'</math></div>") {
                    skip[end + 1] = true;
                }
            }
        } else if t == "#end hide" && lines.get(i + 1).is_some_and(|l| is_eq(l)) {
            // The author's equation, typeset, after a hidden computation: dropped like the #noc form.
            skip[i + 1] = true;
            if i >= 2 && lines[i - 2].trim() == "#hide" {
                skip[i - 2] = true;
                skip[i] = true;
            }
        } else if t == "#end hide" && lines.get(i + 1).map(|l| l.trim()) == Some("#noc") {
            // The author's equation shown after a hidden computation: dropped. When the computation
            // is one line it was hidden for this display only and is shown again (a definition then
            // echoes its value, as before).
            let close = (i + 2..lines.len())
                .find(|&k| lines[k].trim() == "#end noc")
                .unwrap_or(i + 1);
            for s in &mut skip[i + 1..=close] {
                *s = true;
            }
            if i >= 2 && lines[i - 2].trim() == "#hide" {
                skip[i - 2] = true;
                skip[i] = true;
            }
        }
    }
    let mut out = String::new();
    for (l, s) in lines.iter().zip(&skip) {
        if !s {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}
