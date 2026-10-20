//! Pipe tables (Markdig Extensions/Tables/PipeTableParser.cs). Markdig detects the table
//! while parsing the inlines of a paragraph: every line must contain a column delimiter
//! `|` (outside code spans and not escaped) and the second line must be a header
//! separator such as `|---|:--:|`. Rows are then normalised to the widest row.
use super::blocks::{Align, TableData};

/// Splits a line at its unescaped `|` delimiters that are not inside code spans.
/// Returns the pieces and whether the line had any delimiter.
fn split_pipes(line: &str) -> (Vec<String>, usize) {
    let cs: Vec<char> = line.chars().collect();
    let mut pieces = Vec::new();
    let mut cur = String::new();
    let mut pipes = 0;
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '\\' && i + 1 < cs.len() {
            cur.push(c);
            cur.push(cs[i + 1]);
            i += 2;
            continue;
        }
        if c == '`' {
            let mut n = 0;
            while i + n < cs.len() && cs[i + n] == '`' {
                n += 1;
            }
            // find the matching closing run
            let mut j = i + n;
            let mut close = None;
            while j < cs.len() {
                if cs[j] == '`' {
                    let mut m = 0;
                    while j + m < cs.len() && cs[j + m] == '`' {
                        m += 1;
                    }
                    if m == n {
                        close = Some(j + m);
                        break;
                    }
                    j += m;
                } else {
                    j += 1;
                }
            }
            let end = close.unwrap_or(i + n);
            for &ch in &cs[i..end] {
                cur.push(ch);
            }
            i = end;
            continue;
        }
        if c == '|' {
            pipes += 1;
            pieces.push(std::mem::take(&mut cur));
            i += 1;
            continue;
        }
        cur.push(c);
        i += 1;
    }
    pieces.push(cur);
    (pieces, pipes)
}

/// The cells of a row: a leading pipe is dropped, and the part after the last pipe is a
/// cell only when it is not blank.
fn row_cells(line: &str) -> Vec<String> {
    let (mut pieces, pipes) = split_pipes(line);
    if pipes > 0 && pieces.first().is_some_and(|p| p.trim().is_empty()) {
        pieces.remove(0);
    }
    if pipes > 0 && pieces.len() > 1 && pieces.last().is_some_and(|p| p.trim().is_empty()) {
        pieces.pop();
    }
    pieces.into_iter().map(|p| p.trim().to_string()).collect()
}

fn parse_column_header(cell: &str) -> Option<Align> {
    let t = cell.trim();
    if t.is_empty() {
        return Some(Align::None);
    }
    let left = t.starts_with(':');
    let right = t.len() > 1 && t.ends_with(':');
    let inner = t.trim_start_matches(':').trim_end_matches(':');
    if inner.is_empty() || !inner.chars().all(|c| c == '-') {
        return None;
    }
    Some(match (left, right) {
        (true, true) => Align::Center,
        (true, false) => Align::Left,
        (false, true) => Align::Right,
        _ => Align::None,
    })
}

pub fn try_parse_table(content: &str) -> Option<TableData> {
    let lines: Vec<&str> = content.trim_end_matches('\n').split('\n').collect();
    if lines.len() < 2 {
        return None;
    }
    for l in &lines {
        if split_pipes(l).1 == 0 {
            return None;
        }
    }
    let sep = row_cells(lines[1]);
    let mut aligns = Vec::new();
    for c in &sep {
        aligns.push(parse_column_header(c)?);
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    rows.push(row_cells(lines[0]));
    for l in &lines[2..] {
        rows.push(row_cells(l));
    }
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    for r in rows.iter_mut() {
        while r.len() < width {
            r.push(String::new());
        }
    }
    Some(TableData {
        aligns,
        rows,
        cells: Vec::new(),
    })
}
