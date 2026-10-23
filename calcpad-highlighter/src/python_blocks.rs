//! `#python` ... `#end python` blocks (docs/python-blocks.md).
//!
//! The web app runs the block's Python before the engine and replaces the block by its results, so for the editor the
//! block is not Calcpad: its lines must not be tokenized, linted or prettified as Calcpad. [`mask`] gives the text the
//! Calcpad passes work on: same number of lines, every line of a block blank, except that each name the block exports
//! (`export(M_max=...)`, `export("M_max", ...)`) becomes a `name = 0` line where it is exported, so the lines after the
//! block can use the name. The grammar is kept in step with web/calcpad-web/src/wasm/python-blocks.ts.

/// One block: the line of `#python` and the line of `#end python` (`None` when the block is left open).
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub start: usize,
    pub end: Option<usize>,
}

/// The text the Calcpad passes see, and where the Python is.
#[derive(Debug, Clone)]
pub struct Masked {
    pub text: String,
    /// Per line: it belongs to a block (the delimiters and the code).
    pub python: Vec<bool>,
    /// Per line: it is a `#python` or `#end python` delimiter.
    pub delimiter: Vec<bool>,
    pub blocks: Vec<Block>,
    /// The exported names, with the line they are masked onto.
    pub exports: Vec<(String, usize)>,
}

fn is_open(line: &str) -> bool {
    line.trim().eq_ignore_ascii_case("#python")
}

fn is_close(line: &str) -> bool {
    let mut w = line.split_whitespace();
    matches!((w.next(), w.next(), w.next()), (Some(a), Some(b), None)
        if a.eq_ignore_ascii_case("#end") && b.eq_ignore_ascii_case("python"))
}

pub fn find(lines: &[&str]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if is_open(lines[i]) {
            let end = (i + 1..lines.len()).find(|&j| is_close(lines[j]));
            blocks.push(Block { start: i, end });
            i = end.unwrap_or(lines.len());
        }
        i += 1;
    }
    blocks
}

/// The names `export(...)` calls in `body` give, each with the line (0-based, in the worksheet) of its argument.
pub fn exports(body: &[&str], first_line: usize) -> Vec<(String, usize)> {
    let mut chars: Vec<(char, usize)> = Vec::new();
    for (k, l) in body.iter().enumerate() {
        for c in l.chars() {
            chars.push((c, first_line + k));
        }
        chars.push(('\n', first_line + k));
    }
    let n = chars.len();
    let ch = |i: usize| if i < n { chars[i].0 } else { '\0' };
    let is_id = |c: char| c.is_alphanumeric() || c == '_';
    // Past a string or comment starting at `i`; `i` is unchanged when none starts there.
    let skip = |mut i: usize| -> usize {
        match ch(i) {
            '#' => {
                while i < n && ch(i) != '\n' {
                    i += 1;
                }
                i
            }
            q @ ('"' | '\'') => {
                let triple = ch(i + 1) == q && ch(i + 2) == q;
                i += if triple { 3 } else { 1 };
                while i < n {
                    if ch(i) == '\\' {
                        i += 2;
                        continue;
                    }
                    if triple && ch(i) == q && ch(i + 1) == q && ch(i + 2) == q {
                        return i + 3;
                    }
                    if !triple && (ch(i) == q || ch(i) == '\n') {
                        return i + 1;
                    }
                    i += 1;
                }
                i
            }
            _ => i,
        }
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let j = skip(i);
        if j != i {
            i = j;
            continue;
        }
        let c = ch(i);
        if !(c.is_alphabetic() || c == '_') {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && is_id(ch(i)) {
            i += 1;
        }
        let ident: String = chars[start..i].iter().map(|x| x.0).collect();
        let before = if start > 0 { ch(start - 1) } else { ' ' };
        if ident != "export" || before == '.' {
            continue;
        }
        let mut k = i;
        while ch(k) == ' ' || ch(k) == '\t' {
            k += 1;
        }
        if ch(k) != '(' {
            continue;
        }
        // The arguments: split at the commas of depth 1.
        let mut depth = 1;
        let mut p = k + 1;
        let mut args: Vec<(usize, usize)> = Vec::new();
        let mut a0 = p;
        while p < n && depth > 0 {
            let q = skip(p);
            if q != p {
                p = q;
                continue;
            }
            match ch(p) {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        args.push((a0, p));
                    }
                }
                ',' if depth == 1 => {
                    args.push((a0, p));
                    a0 = p + 1;
                }
                _ => {}
            }
            p += 1;
        }
        for (idx, (a, b)) in args.iter().enumerate() {
            let mut s = *a;
            while s < *b && ch(s).is_whitespace() {
                s += 1;
            }
            if s >= *b {
                continue;
            }
            let line = chars[s].1;
            let mut e = s;
            while e < *b && is_id(ch(e)) {
                e += 1;
            }
            let name: String = chars[s..e].iter().map(|x| x.0).collect();
            let mut t = e;
            while t < *b && (ch(t) == ' ' || ch(t) == '\t') {
                t += 1;
            }
            if !name.is_empty() && ch(t) == '=' && ch(t + 1) != '=' && !ch(s).is_ascii_digit() {
                out.push((name, line));
            } else if idx == 0 && args.len() >= 2 && (ch(s) == '"' || ch(s) == '\'') {
                let q = ch(s);
                let lit: String = chars[s + 1..*b].iter().map(|x| x.0).take_while(|&c| c != q).collect();
                if !lit.is_empty() && lit.chars().all(is_id) {
                    out.push((lit, line));
                }
            }
        }
    }
    out.retain(|(name, _)| !name.starts_with('_'));
    out
}

/// `None` when the content holds no block, so the common case costs one scan and changes nothing.
pub fn mask(content: &str) -> Option<Masked> {
    let lines: Vec<&str> = content.split('\n').collect();
    let blocks = find(&lines);
    if blocks.is_empty() {
        return None;
    }
    let mut python = vec![false; lines.len()];
    let mut delimiter = vec![false; lines.len()];
    let mut text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let mut found = Vec::new();
    for b in &blocks {
        let last = b.end.unwrap_or(lines.len() - 1);
        for i in b.start..=last {
            python[i] = true;
            text[i] = String::new();
        }
        delimiter[b.start] = true;
        if let Some(e) = b.end {
            delimiter[e] = true;
        }
        let body_end = b.end.unwrap_or(lines.len());
        let mut taken = vec![false; last + 1];
        for (name, line) in exports(&lines[b.start + 1..body_end], b.start + 1) {
            // One definition per line: the next free line of the block (then an earlier one) when the call's own line is taken.
            if let Some(at) = (line..=last).chain(b.start..line).find(|&i| !taken[i]) {
                taken[at] = true;
                text[at] = format!("{name} = 0");
                found.push((name, at));
            }
        }
    }
    Some(Masked {
        text: text.join("\n"),
        python,
        delimiter,
        blocks,
        exports: found,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(src: &str) -> Vec<(String, usize)> {
        let lines: Vec<&str> = src.split('\n').collect();
        exports(&lines, 0)
    }

    #[test]
    fn blocks_are_found_by_their_delimiter_lines() {
        let l = [
            "a = 1",
            "  #Python ",
            "x = 1",
            "#end   python",
            "b = 2",
            "#python",
            "y = 2",
        ];
        assert_eq!(
            find(&l),
            vec![Block { start: 1, end: Some(3) }, Block { start: 5, end: None }]
        );
        assert!(find(&["#python x", "#end python"]).is_empty());
    }

    #[test]
    fn export_names_come_from_keywords_and_the_positional_form() {
        assert_eq!(
            names("export(a=1, b = [1, 2], c=f(d=3))\nexport('m', 4)\nx.export(z=1)\n# export(q=1)\ns = 'export(r=1)'"),
            vec![("a".into(), 0), ("b".into(), 0), ("c".into(), 0), ("m".into(), 1)]
        );
        assert_eq!(
            names("export(\n    a=1,\n    b=2,\n)"),
            vec![("a".into(), 1), ("b".into(), 2)]
        );
        assert!(names("export(a==1)").is_empty());
        assert!(names("export(_hidden=1)").is_empty());
    }

    #[test]
    fn a_masked_block_keeps_the_line_count_and_defines_the_exports() {
        let src = "a = 1\n#python\nimport numpy as np\nexport(M=1, V=2)\n#end python\nb = M + V";
        let m = mask(src).unwrap();
        assert_eq!(m.text.split('\n').count(), 6);
        let lines: Vec<&str> = m.text.split('\n').collect();
        assert_eq!(lines, ["a = 1", "", "", "M = 0", "V = 0", "b = M + V"]);
        assert_eq!(m.python, [false, true, true, true, true, false]);
        assert_eq!(m.delimiter, [false, true, false, false, true, false]);
    }

    #[test]
    fn many_names_on_one_line_spread_over_the_block() {
        let src = "#python\nexport(a=1, b=2, c=3)\n#end python";
        let m = mask(src).unwrap();
        let names: Vec<&str> = m.exports.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names.len(), 3, "{:?}", m.exports);
        assert_eq!(m.text.split('\n').count(), 3);
    }

    #[test]
    fn no_block_means_no_mask() {
        assert!(mask("a = 1\nb = 2").is_none());
    }
}
