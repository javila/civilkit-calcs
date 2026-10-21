//! Port of Calcpad.Core/Parsers/ExpressionParser/ExpressionParser.ReadWriteOptions.cs,
//! .DataExchange.cs, .Portable.cs and the `#read`/`#write`/`#append` keywords: matrices to and
//! from CSV/text files and Excel workbooks, through `DataFiles`.
use super::{keyword_length, ExpressionParser, Keyword};
use crate::data_files::DataFiles;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::openxml::excel_data;
use crate::output::html_writer::HtmlWriter;
use crate::output::OutputWriter;
use crate::validator;

const DATA_URI: &str = "data:";
const DATA_URI_ERROR_LENGTH: usize = 32;
const CSV_MIME: &str = "text/csv";
const XLSX_MIME: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
const XLSM_MIME: &str = "application/vnd.ms-excel.sheet.macroEnabled.12";
pub const MAX_EMBEDDED_DATA_SIZE: usize = 10 * 1024 * 1024;

fn starts_with_ignore_case(s: &[char], prefix: &str) -> bool {
    let p: Vec<char> = prefix.chars().collect();
    s.len() >= p.len()
        && s[..p.len()]
            .iter()
            .zip(&p)
            .all(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
}

fn mime_extension(mime: &str) -> CResult<&'static str> {
    let m: Vec<char> = mime.chars().collect();
    if starts_with_ignore_case(&m, CSV_MIME) {
        Ok("csv")
    } else if starts_with_ignore_case(&m, XLSX_MIME) {
        Ok("xlsx")
    } else if starts_with_ignore_case(&m, XLSM_MIME) {
        Ok("xlsm")
    } else {
        Err(exceptions::FileFormatNotSupported(mime))
    }
}

/// The MIME type a file is embedded under, from its extension.
pub fn extension_mime(ext: &str) -> &'static str {
    if ext.eq_ignore_ascii_case("xlsx") {
        XLSX_MIME
    } else if ext.eq_ignore_ascii_case("xlsm") {
        XLSM_MIME
    } else {
        CSV_MIME
    }
}

fn out_of_range() -> MathParserException {
    MathParserException::unexpected("Index was outside the bounds of the array.".to_string())
}

/// `ReadWriteOptions`: what a `#read`/`#write`/`#append` line says.
#[derive(Debug, Default)]
pub(crate) struct ReadWriteOptions {
    pub name: String,
    pub path: String,
    pub ext: String,
    pub sheet: String,
    pub start: String,
    pub end: String,
    /// The base64 payload of a "data:" source, which is read instead of a file.
    pub data: String,
    pub full_path: Option<String>,
    pub ty: char,
    pub separator: char,
    pub append: bool,
    pub is_excel: bool,
    pub is_hp: bool,
}

/// `TextSpan` over a char array.
struct Span<'a> {
    s: &'a [char],
    start: usize,
    end: usize,
}

impl Span<'_> {
    fn reset(&mut self, i: usize) {
        self.start = i;
        self.end = i;
    }
    fn cut(&self) -> CResult<String> {
        if self.start > self.end || self.end > self.s.len() {
            return Err(MathParserException::unexpected(
                "Specified argument was out of the range of valid values.".to_string(),
            ));
        }
        Ok(self.s[self.start..self.end].iter().collect())
    }
}

impl ReadWriteOptions {
    /// `new ReadWriteOptions(s, command, sourceDir)`: command 0 reads, 1 writes, 2 appends.
    pub fn new(line: &str, command: i32, source_dir: Option<&str>, files: &dyn DataFiles) -> CResult<ReadWriteOptions> {
        let s: Vec<char> = line.chars().collect();
        let len = s.len();
        let mut o = ReadWriteOptions {
            ty: if command > 0 { 'N' } else { 'R' },
            separator: ',',
            ..Default::default()
        };
        let mut ts = Span {
            s: &s,
            start: 0,
            end: 0,
        };
        let mut i0 = 0usize;
        let search = |i0: &mut usize, c: char| {
            while *i0 < len {
                let k = *i0;
                *i0 += 1;
                if s[k] == c {
                    break;
                }
            }
        };
        search(&mut i0, ' ');
        if i0 == len {
            return Ok(o);
        }
        ts.reset(i0);
        search(&mut i0, ' ');
        ts.end = if i0 == len { i0 } else { i0 - 1 };
        o.name = ts.cut()?;
        if !validator::is_variable(&o.name) {
            return Err(exceptions::InvalidVariableName(&o.name));
        }
        if i0 == len {
            return Ok(o);
        }
        ts.reset(i0);
        search(&mut i0, ' ');
        ts.end = if i0 == len { i0 } else { i0 - 1 };
        let keyword = ts.cut()?;
        if !keyword.eq_ignore_ascii_case(if command == 0 { "from" } else { "to" }) {
            return Err(exceptions::InvalidSyntax(&keyword));
        }
        if i0 == len {
            return Err(exceptions::MissingFileName());
        }
        ts.reset(i0);
        let i1: usize;
        let mut data_ext = "";
        if starts_with_ignore_case(&s[i0..], DATA_URI) {
            let end = s[i0..].iter().position(|&c| c == ' ' || c == '@' || c == '!');
            i1 = match end {
                None => len,
                Some(e) => i0 + e,
            };
            let uri = &s[i0..i1];
            let comma = uri.iter().position(|&c| c == ',');
            match comma {
                Some(comma) if command <= 0 => {
                    let mime: String = uri[DATA_URI.len()..comma].iter().collect();
                    data_ext = mime_extension(&mime)?;
                    o.data = uri[comma + 1..].iter().collect();
                }
                _ => {
                    let shown: String = uri[..uri.len().min(DATA_URI_ERROR_LENGTH)].iter().collect();
                    return Err(exceptions::InvalidSyntax(&shown));
                }
            }
            i0 = i1 - 1;
        } else {
            // `while (i1 >= 0) { if (s[--i1] == '.') break; }`
            let mut k = len as isize;
            loop {
                if k < 0 {
                    break;
                }
                k -= 1;
                if k < 0 {
                    return Err(out_of_range());
                }
                if s[k as usize] == '.' {
                    break;
                }
            }
            ts.end = k as usize;
            o.path = ts.cut()?;
            i1 = k as usize + 1;
        }
        ts.reset(i1);
        let (mut has_sheet, mut has_start, mut has_end) = (false, false, false);
        let mut i = i1;
        while i < len {
            let c = s[i];
            match c {
                '@' => {
                    has_sheet = true;
                    o.ext = ts.cut()?;
                    ts.reset(i + 1);
                }
                '!' => {
                    has_start = true;
                    if has_sheet {
                        o.sheet = ts.cut()?;
                    } else {
                        o.ext = ts.cut()?;
                    }
                    ts.reset(i + 1);
                }
                ':' => {
                    has_end = true;
                    o.start = ts.cut()?;
                    ts.reset(i + 1);
                }
                ' ' => {}
                _ => ts.end += 1,
            }
            i0 = i;
            if c == ' ' {
                break;
            }
            i += 1;
        }
        if !o.data.is_empty() {
            o.ext = data_ext.to_string();
        }
        let is_xls = |ext: &str| ext.len() >= 3 && ext[..3].eq_ignore_ascii_case("xls");
        o.is_excel = is_xls(&o.ext);
        if !o.is_excel {
            has_start = has_sheet;
        }
        if has_end {
            o.end = ts.cut()?;
        } else if has_start {
            o.start = ts.cut()?;
        } else if has_sheet {
            o.sheet = ts.cut()?;
        } else if o.ext.is_empty() {
            o.ext = ts.cut()?;
            o.is_excel = is_xls(&o.ext);
        }
        if o.data.is_empty() {
            let raw_path = format!("{}.{}", o.path, o.ext);
            let path = files.expand_environment_variables(&raw_path);
            o.full_path = Some(files.full_path(&path, source_dir));
        }
        o.append = command == 2;
        i0 += 1;
        for _ in 0..2 {
            if i0 >= len {
                return Ok(o);
            }
            ts.reset(i0);
            search(&mut i0, '=');
            ts.end = i0 - 1;
            let keyword = ts.cut()?;
            ts.reset(i0);
            search(&mut i0, ' ');
            ts.end = if i0 == len { i0 } else { i0 - 1 };
            let option: Vec<char> = ts.cut()?.chars().collect();
            if keyword.is_empty() {
                continue;
            }
            if option.is_empty() {
                return Err(exceptions::InvalidSyntax(&keyword));
            }
            let option_s: String = option.iter().collect();
            if keyword.eq_ignore_ascii_case("type") {
                if option.len() == 4 {
                    if option_s.to_ascii_lowercase().ends_with("_hp") {
                        o.is_hp = true;
                    } else {
                        return Err(exceptions::InvalidSyntax(&option_s));
                    }
                } else {
                    o.is_hp = false;
                    if option.len() != 1 {
                        return Err(exceptions::InvalidSyntax(&option_s));
                    }
                }
                o.ty = option[0];
            } else if keyword.eq_ignore_ascii_case("sep") {
                if option.len() != 3 || option[0] != '\'' || option[2] != '\'' {
                    return Err(exceptions::InvalidSyntax(&option_s));
                }
                o.separator = option[1];
            } else {
                return Err(exceptions::InvalidSyntax(&keyword));
            }
        }
        Ok(o)
    }

    fn file_name(&self) -> String {
        format!("{}.{}", self.path, self.ext)
    }
}

// ----- DataExchange --------------------------------------------------------------------------

/// `StreamReader` decoding: a BOM picks UTF-8 or UTF-16, otherwise UTF-8.
fn decode_text(bytes: &[u8]) -> String {
    if let Some(b) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(b).to_string();
    }
    let utf16 = |b: &[u8], le: bool| {
        let units: Vec<u16> = b
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&c| {
                if le {
                    u16::from_le_bytes(c)
                } else {
                    u16::from_be_bytes(c)
                }
            })
            .collect();
        String::from_utf16_lossy(&units)
    };
    if let Some(b) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return utf16(b, true);
    }
    if let Some(b) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return utf16(b, false);
    }
    String::from_utf8_lossy(bytes).to_string()
}

/// `TextReader.ReadLine` over the whole text.
fn read_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let b = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\r' || b[i] == b'\n' {
            lines.push(&text[start..i]);
            if b[i] == b'\r' && i + 1 < b.len() && b[i + 1] == b'\n' {
                i += 1;
            }
            start = i + 1;
        }
        i += 1;
    }
    if start < b.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// `ParseBound`: "R1C2", "C2R1", "R3" or "C4".
fn parse_bound(s: &str) -> CResult<(i32, i32)> {
    if s.is_empty() {
        return Ok((0, 0));
    }
    let c: Vec<char> = s.chars().collect();
    let len = c.len();
    let mut ir = c.iter().position(|&x| x == 'R').map_or(0, |p| p + 1);
    let mut ic = c.iter().position(|&x| x == 'C').map_or(0, |p| p + 1);
    if ir == 0 {
        ir = len + 1;
    }
    if ic == 0 {
        ic = len + 1;
    }
    if ir != 1 && ic != 1 {
        return Err(exceptions::InvalidSyntax(s));
    }
    let slice = |a: usize, b: usize| -> CResult<String> {
        if a > b || b > len {
            return Err(MathParserException::unexpected(
                "Specified argument was out of the range of valid values.".to_string(),
            ));
        }
        Ok(c[a..b].iter().collect())
    };
    let (sr, sc) = if ir < ic {
        (
            slice(ir, ic - 1)?,
            if ic < len { slice(ic, len)? } else { "0".to_string() },
        )
    } else {
        (
            if ir < len { slice(ir, len)? } else { "0".to_string() },
            slice(ic, ir - 1)?,
        )
    };
    let parse = |t: &str| -> Option<i32> {
        let t = t.trim();
        if t.is_empty() {
            None
        } else {
            t.parse::<i32>().ok()
        }
    };
    match (parse(&sc), parse(&sr)) {
        (Some(col), Some(row)) => Ok((row, col)),
        _ => Err(exceptions::InvalidSyntax(s)),
    }
}

/// `ParseBounds`: (start, end) as (row, col), each ordered.
fn parse_bounds(start: &str, end: &str) -> CResult<((i32, i32), (i32, i32))> {
    let mut a = parse_bound(start)?;
    let mut b = parse_bound(end)?;
    if a.0 > b.0 && b.0 > 0 {
        std::mem::swap(&mut a.0, &mut b.0);
    }
    if a.1 > b.1 && b.1 > 0 {
        std::mem::swap(&mut a.1, &mut b.1);
    }
    Ok((a, b))
}

/// `string.Split(separator, StringSplitOptions.TrimEntries)`
fn split_trim(line: &str, sep: char) -> Vec<String> {
    line.split(sep).map(|v| v.trim().to_string()).collect()
}

fn format_csv_data(o: &ReadWriteOptions, lines: &[&str], start: (i32, i32), end: (i32, i32)) -> Vec<Vec<String>> {
    let j0 = (start.1 - 1).max(0) as usize;
    lines
        .iter()
        .map(|line| {
            let values = split_trim(line, o.separator);
            if start.1 == 0 && end.1 == 0 {
                return values;
            }
            let mut m = values.len() as i64 - j0 as i64;
            if end.1 > 0 {
                m = m.min(end.1 as i64 - j0 as i64);
            }
            if m > 0 {
                values[j0..j0 + m as usize].to_vec()
            } else {
                Vec::new()
            }
        })
        .collect()
}

fn read_csv_text(o: &ReadWriteOptions, text: &str) -> CResult<Vec<Vec<String>>> {
    let (start, end) = parse_bounds(&o.start, &o.end)?;
    let mut lines = Vec::new();
    for (k, line) in read_lines(text).into_iter().enumerate() {
        let i = k as i32 + 1;
        if i >= start.0 {
            if end.0 > 0 && i > end.0 {
                break;
            }
            lines.push(line);
        }
    }
    Ok(format_csv_data(o, &lines, start, end))
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        out.push(BASE64[(n >> 18) as usize & 63] as char);
        out.push(BASE64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            BASE64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            BASE64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// `Convert.TryFromBase64Chars` (whitespace is ignored).
pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let chars: Vec<u8> = s
        .bytes()
        .filter(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .collect();
    if chars.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(chars.len() / 4 * 3);
    for (q, quad) in chars.chunks(4).enumerate() {
        let last = q == chars.len() / 4 - 1;
        let mut n = 0u32;
        let mut pad = 0;
        for (k, &c) in quad.iter().enumerate() {
            let v = match c {
                b'A'..=b'Z' => c - b'A',
                b'a'..=b'z' => c - b'a' + 26,
                b'0'..=b'9' => c - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' if last && k >= 2 => {
                    pad += 1;
                    0
                }
                _ => return None,
            };
            if pad > 0 && c != b'=' {
                return None;
            }
            n = n << 6 | v as u32;
        }
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    Some(out)
}

/// `DataExchange.Read`
pub(crate) fn read(o: &ReadWriteOptions, files: &dyn DataFiles) -> CResult<Option<Vec<Vec<String>>>> {
    if !o.data.is_empty() {
        let bytes = base64_decode(&o.data).ok_or_else(|| exceptions::InvalidSyntax(DATA_URI))?;
        if o.is_excel {
            return excel_data::read(&bytes, &o.sheet, &o.start, &o.end).map_err(MathParserException::new);
        }
        return read_csv_text(o, &String::from_utf8_lossy(&bytes))
            .map(Some)
            .map_err(|e| MathParserException::new(e.message));
    }
    let file_name = o.file_name();
    if file_name == "." {
        return Err(exceptions::MissingFileName());
    }
    let full_path = o.full_path.clone().unwrap_or_default();
    let Some(bytes) = files.read(&full_path) else {
        return Err(exceptions::FileNotFound(&file_name));
    };
    if o.is_excel {
        if !excel_data::is_excel_file(&o.ext) {
            return Err(exceptions::FileFormatNotSupported(&o.ext));
        }
        return excel_data::read(&bytes, &o.sheet, &o.start, &o.end).map_err(MathParserException::new);
    }
    read_csv_text(o, &decode_text(&bytes))
        .map(Some)
        .map_err(|e| MathParserException::new(e.message))
}

/// `DataExchange.Write`: the target is checked whether or not the write runs.
pub(crate) fn write(
    o: &ReadWriteOptions,
    data: &[Vec<String>],
    allow_write: bool,
    files: &dyn DataFiles,
) -> CResult<()> {
    if o.file_name() == "." {
        return Err(exceptions::MissingFileName());
    }
    let full_path = o.full_path.clone().unwrap_or_default();
    let dir = files.directory_name(&full_path).unwrap_or_default();
    if !files.dir_exists(&dir) {
        return Err(exceptions::PathNotFound(&dir));
    }
    if o.is_excel && !o.ext.eq_ignore_ascii_case("xlsx") && !o.ext.eq_ignore_ascii_case("xlsm") {
        return Err(exceptions::FileFormatNotSupported(&o.ext));
    }
    if !allow_write {
        return Ok(());
    }
    let r = if o.is_excel {
        let existing = if o.append { files.read(&full_path) } else { None };
        excel_data::write(existing.as_deref(), &o.sheet, &o.start, &o.end, data, o.append)
            .and_then(|bytes| files.write(&full_path, &bytes))
    } else {
        write_csv(o, data).and_then(|text| {
            if o.append {
                files.append(&full_path, text.as_bytes())
            } else {
                files.write(&full_path, text.as_bytes())
            }
        })
    };
    r.map_err(MathParserException::new)
}

fn write_csv(o: &ReadWriteOptions, data: &[Vec<String>]) -> Result<String, String> {
    let (start, end) = parse_bounds(&o.start, &o.end).map_err(|e| e.message)?;
    let i0 = (start.0 - 1).max(0) as usize;
    let mut n = data.len();
    if end.0 > 0 {
        n = n.min(end.0 as usize);
    }
    let j0 = (start.1 - 1).max(0) as usize;
    let mut out = String::new();
    for row in data.iter().take(n).skip(i0) {
        let mut m = row.len() as i64 - j0 as i64;
        if end.1 > 0 {
            m = m.min(end.1 as i64 - j0 as i64);
        }
        if m > 0 {
            out.push_str(&row[j0..j0 + m as usize].join(&o.separator.to_string()));
        }
        out.push('\n');
    }
    Ok(out)
}

// ----- keywords --------------------------------------------------------------------------------

impl ExpressionParser {
    fn source_dir(&self) -> Option<String> {
        match &self.source_file_path {
            Some(p) if !p.is_empty() => self.data_files.directory_name(p),
            _ => None,
        }
    }

    /// `ParseKeywordRead`
    pub(super) fn parse_keyword_read(&mut self, s: &str) -> CResult<()> {
        if self.calculate {
            if self.condition.is_satisfied() {
                let files = self.data_files.clone();
                let dir = self.source_dir();
                let o = ReadWriteOptions::new(s, 0, dir.as_deref(), files.as_ref())?;
                if o.name.is_empty() {
                    return Ok(());
                }
                let data = read(&o, files.as_ref())?;
                if let Some(data) = data {
                    if o.ty == 'V' {
                        self.p().set_vector(&o.name, &data, o.is_hp)?;
                    } else {
                        self.p().set_matrix(&o.name, &data, o.ty, o.is_hp)?;
                    }
                }
                if self.is_visible {
                    self.report_data_exchange_result(&o, "read from", true);
                }
            }
        } else if self.is_visible {
            let id = self.html_id();
            let rest: String = s.chars().skip(keyword_length(Keyword::Read)).collect();
            self.sb
                .push_str(&format!("<p><span{} class=\"cond\">#read</span> {}</p>", id, rest));
        }
        Ok(())
    }

    /// `ParseKeywordWrite`
    pub(super) fn parse_keyword_write(&mut self, s: &str, keyword: Keyword) -> CResult<()> {
        if self.calculate {
            if self.condition.is_satisfied() {
                let files = self.data_files.clone();
                let dir = self.source_dir();
                let command = if keyword == Keyword::Write { 1 } else { 2 };
                let o = ReadWriteOptions::new(s, command, dir.as_deref(), files.as_ref())?;
                if o.name.is_empty() {
                    return Ok(());
                }
                let m = self.p().get_matrix(&o.name, o.ty)?;
                write(&o, &m, self.allow_data_write, files.as_ref())?;
                if self.is_visible {
                    let verb = if keyword == Keyword::Write {
                        "written to"
                    } else {
                        "appended to"
                    };
                    let done = self.allow_data_write;
                    self.report_data_exchange_result(&o, verb, done);
                }
            }
        } else if self.is_visible {
            let id = self.html_id();
            let kl = keyword_length(keyword);
            let head: String = s.chars().take(kl).collect();
            let rest: String = s.chars().skip(kl).collect();
            self.sb
                .push_str(&format!("<p><span{} class=\"cond\">{}</span> {}</p>", id, head, rest));
        }
        Ok(())
    }

    fn report_data_exchange_result(&mut self, o: &ReadWriteOptions, command: &str, done: bool) {
        let id = self.html_id();
        let var = HtmlWriter::new(Some(&self.math_settings.borrow()), false).format_variable(&o.name, "", true);
        let mut sb = format!("<p{}>Matrix <span class=\"eq\">{}</span>", id, var);
        sb.push_str(&if done {
            format!(" was successfully {} ", command)
        } else {
            format!(" will be {} ", command)
        });
        if o.data.is_empty() {
            let name = o.file_name();
            if done {
                let full = o.full_path.clone().unwrap_or_default().replace('\\', "/");
                sb.push_str(&format!("<a href=\"file:///{}\">{}</a>", full, name));
            } else {
                sb.push_str(&name);
            }
        } else {
            sb.push_str("embedded data");
        }
        if o.is_excel {
            if !o.sheet.is_empty() {
                sb.push_str(&format!("@{}", o.sheet));
            }
            if !o.start.is_empty() {
                sb.push_str(&format!("!{}", o.start));
            }
            if !o.end.is_empty() {
                sb.push_str(&format!(":{}", o.end));
            }
        } else {
            if !o.start.is_empty() {
                sb.push_str(&format!("@{}", o.start));
            }
            if !o.end.is_empty() {
                sb.push_str(&format!(":{}", o.end));
            }
            sb.push_str(&format!(" <small>SEP</small>='{}'", o.separator));
        }
        sb.push_str(&format!(" <small>TYPE</small>={}</p>", o.ty));
        self.sb.push_str(&sb);
    }

    /// `ExpressionParser.EmbedReadDirective`: the `#read` line with its file carried inline as
    /// a data URI, and the file's size; None when the line reads no file.
    pub fn embed_read_directive(
        line: &str,
        source_file_path: Option<&str>,
        files: &dyn DataFiles,
    ) -> CResult<Option<(String, usize)>> {
        let s = line.trim();
        let source_dir = match source_file_path {
            Some(p) if !p.is_empty() => files.directory_name(p),
            _ => None,
        };
        let o = ReadWriteOptions::new(s, 0, source_dir.as_deref(), files)?;
        if o.name.is_empty() || !o.data.is_empty() {
            return Ok(None);
        }
        let Some((start, length)) = ExpressionParser::try_get_data_path(s) else {
            return Ok(None);
        };
        let full_path = o.full_path.clone().unwrap_or_default();
        if let Some(bytes) = files.read(&full_path) {
            if bytes.len() > MAX_EMBEDDED_DATA_SIZE {
                return Err(exceptions::EmbeddedDataSizeLimit());
            }
        }
        read(&o, files)?;
        let bytes = files
            .read(&full_path)
            .ok_or_else(|| exceptions::FileNotFound(&o.file_name()))?;
        let uri = format!(
            "{}{};base64,{}",
            DATA_URI,
            extension_mime(&o.ext),
            base64_encode(&bytes)
        );
        let chars: Vec<char> = s.chars().collect();
        let head: String = chars[..start].iter().collect();
        let tail: String = chars[start + length..].iter().collect();
        Ok(Some((format!("{}{}{}", head, uri, tail), bytes.len())))
    }

    /// `ExpressionParser.TryGetDataPath`: (start, length) in chars of the file a
    /// `#read`/`#write`/`#append` line names.
    pub fn try_get_data_path(line: &str) -> Option<(usize, usize)> {
        let l: Vec<char> = line.chars().collect();
        let connector = if starts_with_ignore_case(&l, "#read") {
            "from"
        } else if starts_with_ignore_case(&l, "#write") || starts_with_ignore_case(&l, "#append") {
            "to"
        } else {
            return None;
        };
        let len = l.len();
        let mut i = 0;
        for token in 0..3 {
            let from = i;
            while i < len {
                let k = i;
                i += 1;
                if l[k] == ' ' {
                    break;
                }
            }
            if i == len {
                return None;
            }
            if token == 2 {
                let word: String = l[from..i - 1].iter().collect();
                if !word.eq_ignore_ascii_case(connector) {
                    return None;
                }
            }
        }
        if starts_with_ignore_case(&l[i..], DATA_URI) {
            return None;
        }
        let dot = l.iter().rposition(|&c| c == '.')?;
        if dot < i {
            return None;
        }
        let end = (dot + 1..len)
            .find(|&j| matches!(l[j], '@' | '!' | ':' | ' '))
            .unwrap_or(len);
        let length = end - i;
        if length > 0 {
            Some((i, length))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_files::NoFiles;

    #[test]
    fn options_and_bounds() {
        let o = ReadWriteOptions::new("#read M from a.csv@R2C2:R3C3 type=L sep=';'", 0, Some("/w"), &NoFiles).unwrap();
        assert_eq!((o.name.as_str(), o.path.as_str(), o.ext.as_str()), ("M", "a", "csv"));
        assert_eq!(
            (o.start.as_str(), o.end.as_str(), o.ty, o.separator),
            ("R2C2", "R3C3", 'L', ';')
        );
        assert_eq!(o.full_path.as_deref(), Some("/w/a.csv"));
        // The scanner stops at the first space, so a sheet name cannot contain one (as upstream).
        let o = ReadWriteOptions::new("#read M from b.xlsx@Sheet 1!A1:B2", 0, None, &NoFiles);
        assert!(o.is_err_and(|e| e.message.contains("Invalid syntax")));
        let o = ReadWriteOptions::new("#read M from b.xlsx@S!A1:B2 type=R_hp", 0, None, &NoFiles).unwrap();
        assert_eq!(
            (o.sheet.as_str(), o.start.as_str(), o.end.as_str(), o.is_hp),
            ("S", "A1", "B2", true)
        );
        assert_eq!(parse_bounds("R3C4", "R1C2").unwrap(), ((1, 2), (3, 4)));
        assert_eq!(base64_decode(&base64_encode(b"1,2\n3")).unwrap(), b"1,2\n3");
        assert_eq!(read_lines("a\r\nb\rc\n"), vec!["a", "b", "c"]);
    }
}
