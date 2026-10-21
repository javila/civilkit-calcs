//! Port of Calcpad.Cli/CalcpadReader.cs: reads worksheets from disk, resolves `#include`
//! files and renders the macro error listing.
use std::collections::VecDeque;
use std::path::{Component, Path, PathBuf};

use calcpad_core::extension_methods::{enumerate_lines, replace_c_style_operators};
use calcpad_core::parsers::macro_parser::{get_fields, has_input_fields, set_line_input_fields, FileProvider};
use calcpad_core::validator::is_keyword;
use calcpad_core::{CResult, MathParserException};

fn read_all_text(path: &str) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    let s = String::from_utf8_lossy(&bytes).to_string();
    Ok(s.strip_prefix('\u{FEFF}').map(|x| x.to_string()).unwrap_or(s))
}

/// `CalcpadReader.Read(fileName)`
pub fn read(file_name: &str) -> Result<String, String> {
    let text = read_all_text(file_name).map_err(|e| e.to_string())?;
    let mut output_lines: Vec<String> = Vec::new();
    let mut has_form = false;
    for line in enumerate_lines(&text) {
        let s: String;
        if let Some(n) = line.find('\u{000B}') {
            has_form = true;
            if n == 0 {
                set_input_fields_from_file(&line[1..], &mut output_lines);
                break;
            } else {
                set_input_fields_from_file(&line[n + 1..], &mut output_lines);
                s = line[..n].to_string();
            }
        } else {
            s = replace_c_style_operators(line.trim_start_matches('\t'));
            if !has_form {
                has_form = has_input_fields(&s);
            }
        }
        output_lines.push(s);
    }
    Ok(output_lines.join("\n"))
}

fn set_input_fields_from_file(fields_text: &str, lines: &mut [String]) {
    let mut values: VecDeque<String> = VecDeque::new();
    if fields_text.is_empty() {
        return;
    }
    for s in fields_text.split('\t') {
        values.push_back(s.to_string());
    }
    if fields_text.ends_with('\t') {
        values.pop_back();
    }
    for line in lines.iter_mut() {
        let mut sb = String::new();
        if set_line_input_fields(line, &mut sb, Some(&mut values), false) {
            *line = sb;
        }
        if values.is_empty() {
            return;
        }
    }
}

fn get_module_name(s: &str) -> Option<String> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    if n < 9 {
        return None;
    }
    let mut n: isize = chars
        .iter()
        .position(|&c| c == '\'' || c == '"')
        .map(|x| x as isize)
        .unwrap_or(-1);
    let n1: isize = chars.iter().rposition(|&c| c == '#').map(|x| x as isize).unwrap_or(-1);
    if n < 9 || n1 > 0 && n1 < n {
        n = n1;
    }
    if n < 9 {
        n = chars.len() as isize;
    }
    Some(chars[8..n as usize].iter().collect::<String>().trim().to_string())
}

/// `CalcpadReader.Include(fileName, fields)`
pub fn include(file_name: &str, fields: Option<VecDeque<String>>) -> CResult<String> {
    let s = read_all_text(file_name).map_err(|e| MathParserException::unexpected(e.to_string()))?;
    let j = s.find('\u{000B}');
    let has_form = j.is_some_and(|j| j > 0);
    let body = if has_form { &s[..j.unwrap()] } else { &s[..] };
    let sf = if has_form { &s[j.unwrap() + 1..] } else { "" };
    let mut fields = fields;
    let mut get_fields_q = get_fields_queue(sf, fields.as_mut());
    let mut is_local = false;
    let mut get_lines: Vec<String> = Vec::new();
    let lines = enumerate_lines(body);
    for line in lines {
        if is_keyword(line, "#local") {
            is_local = true;
        } else if is_keyword(line, "#global") {
            is_local = false;
        } else if !is_local {
            if is_keyword(line, "#include") {
                let name = get_module_name(line).unwrap_or_default();
                let nested = if fields.is_none() { None } else { Some(VecDeque::new()) };
                get_lines.push(include(&name, nested)?);
            } else {
                get_lines.push(line.to_string());
            }
        }
    }
    if has_form && get_lines.last().is_some_and(|l| l.trim().is_empty()) {
        get_lines.pop();
    }
    for line in get_lines.iter_mut() {
        if let Some(q) = get_fields_q.as_mut() {
            if !q.is_empty() {
                let mut sb = String::new();
                if set_line_input_fields(line.trim_end(), &mut sb, Some(q), false) {
                    *line = sb;
                }
            }
        }
    }
    Ok(get_lines.join("\n"))
}

fn get_fields_queue(s: &str, fields: Option<&mut VecDeque<String>>) -> Option<VecDeque<String>> {
    let fields = fields?;
    if !fields.is_empty() {
        if !s.is_empty() {
            let mut gf = get_fields(s, '\t');
            if fields.len() < gf.len() {
                for _ in 0..fields.len() {
                    gf.pop_front();
                }
                while let Some(x) = gf.pop_front() {
                    fields.push_back(x);
                }
            }
        }
        return Some(fields.clone());
    }
    if !s.is_empty() {
        return Some(get_fields(s, '\t'));
    }
    None
}

/// `CalcpadReader.CodeToHtml`: the listing shown when macros fail.
pub fn code_to_html(code: &str) -> String {
    const SPACES: &str = "&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;";
    let mut errors: VecDeque<usize> = VecDeque::new();
    let mut sb = String::from("<pre class=\"code\">\n");
    let mut line_number = 0usize;
    for line in enumerate_lines(code) {
        line_number += 1;
        let (line_text, source_line) = match line.find('\u{000B}') {
            Some(i) => (&line[..i], line[i + 1..].to_string()),
            None => (line, line_number.to_string()),
        };
        let line_num_text = line_number.to_string();
        let n = line_num_text.len();
        let pad = &SPACES[(6 * n).min(SPACES.len())..];
        sb.push_str(&format!(
            "<p class=\"line-text\" id=\"line-{}><span title=\"Source line {}\">{}{}</span>&emsp;\u{2502}&emsp;",
            line_number, source_line, pad, line_number
        ));
        if line.starts_with("#Error") {
            errors.push_back(line_number);
            let lt: String = line_text.chars().skip(1).collect();
            sb.push_str(&format!("<span class=\"err\">{}</span>", lt));
        } else {
            sb.push_str(line_text);
        }
        sb.push_str("</p>");
    }
    sb.push_str("</pre>");
    if !errors.is_empty() && line_number > 30 {
        sb.push_str(&format!(
            "<div class=\"errorHeader\">Found <b>{}</b> errors in modules and macros:\n",
            errors.len()
        ));
        let mut count = 0;
        while !errors.is_empty() {
            count += 1;
            if count >= 20 {
                break;
            }
            let line = errors.pop_front().unwrap();
            sb.push_str(&format!(
                " <span class=\"roundBox\" data-line=\"{}\">{}</span>",
                line, line
            ));
        }
        if !errors.is_empty() {
            sb.push_str(" ...");
        }
        sb.push_str("</div>");
        sb.push_str("<style>body {padding-top:0.5em;} p {margin:0; line-height:1.15em;}</style>\n");
    } else {
        sb.push_str("<style>p {margin:0; line-height:1.15em;}</style>\n");
    }
    sb
}

/// Disk access for `#include`.
pub struct DiskFiles;

fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

impl FileProvider for DiskFiles {
    fn full_path(&self, path: &str, base_dir: Option<&str>) -> String {
        let p = Path::new(path);
        let joined = if p.is_absolute() {
            p.to_path_buf()
        } else {
            match base_dir {
                Some(b) => Path::new(b).join(p),
                None => std::env::current_dir().unwrap_or_default().join(p),
            }
        };
        normalize(&joined).to_string_lossy().to_string()
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).is_file()
    }

    fn directory_name(&self, path: &str) -> Option<String> {
        Path::new(path).parent().map(|p| p.to_string_lossy().to_string())
    }

    fn include(&self, path: &str, fields: Option<VecDeque<String>>) -> CResult<String> {
        include(path, fields)
    }

    fn expand_environment_variables(&self, s: &str) -> String {
        let mut out = String::new();
        let mut rest = s;
        while let Some(i) = rest.find('%') {
            let after = &rest[i + 1..];
            if let Some(j) = after.find('%') {
                let name = &after[..j];
                match std::env::var(name) {
                    Ok(v) if !name.is_empty() => {
                        out.push_str(&rest[..i]);
                        out.push_str(&v);
                        rest = &after[j + 1..];
                        continue;
                    }
                    _ => {
                        out.push_str(&rest[..i + 1]);
                        rest = after;
                        continue;
                    }
                }
            }
            break;
        }
        out.push_str(rest);
        out
    }
}
