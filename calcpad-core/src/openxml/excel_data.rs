//! Port of Calcpad.OpenXml/ExcelData.cs: reads a cell range out of an .xlsx/.xlsm workbook
//! and writes one into a new or existing workbook. The DocumentFormat.OpenXml SDK is replaced
//! by the zip and XML helpers next to this file; only the parts this code touches are read.
use std::collections::HashMap;

use super::xml::{self, local, Element, Node};
use super::zip::{self, Entry};

const DEFAULT_SHEET: &str = "Sheet1";
const NS_MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const NS_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const REL_WORKSHEET: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet";
const REL_OFFICE_DOC: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const CT_WORKSHEET: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml";
const NULL_REFERENCE: &str = "Object reference not set to an instance of an object.";

pub fn is_excel_file(ext: &str) -> bool {
    ext.eq_ignore_ascii_case("xlsx") || ext.eq_ignore_ascii_case("xlsm")
}

/// `CellRef`
#[derive(Clone, Copy, Debug, Default)]
struct CellRef {
    row: i32,
    col: i32,
}

fn col_row_sep_index(s: &str) -> usize {
    s.char_indices()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn col_number(name: &str) -> i32 {
    let mut sum: i32 = 0;
    for c in name.chars() {
        sum = sum.wrapping_mul(26);
        sum = sum.wrapping_add(c.to_ascii_uppercase() as i32 - 'A' as i32 + 1);
    }
    sum
}

fn col_name(mut n: i32) -> String {
    let mut name = String::new();
    while n > 0 {
        let m = (n - 1) % 26;
        name.insert(0, (b'A' + m as u8) as char);
        n = (n - m) / 26;
    }
    name
}

impl CellRef {
    fn parse(s: &str) -> Result<CellRef, String> {
        let mut r = CellRef::default();
        if !s.is_empty() {
            let index = col_row_sep_index(s);
            if index < s.len() {
                // int.Parse
                r.row = s[index..]
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| format!("The input string '{}' was not in a correct format.", &s[index..]))?;
            }
            if index > 0 {
                r.col = col_number(&s[..index]);
            }
        }
        Ok(r)
    }

    fn col_from_ref(s: Option<&str>) -> i32 {
        match s {
            None | Some("") => 0,
            Some(s) => col_number(&s[..col_row_sep_index(s)]),
        }
    }

    fn to_ref(self) -> String {
        format!("{}{}", col_name(self.col), self.row)
    }
}

struct CellRange {
    start: CellRef,
    end: CellRef,
}

impl CellRange {
    fn new(start: &str, end: &str) -> Result<CellRange, String> {
        Ok(CellRange {
            start: CellRef::parse(start)?,
            end: CellRef::parse(end)?,
        })
    }

    fn row_count(&self) -> i32 {
        self.end.row - self.start.row + 1
    }

    fn col_count(&self) -> i32 {
        self.end.col - self.start.col + 1
    }

    fn normalize(&mut self, min_row: i32, min_col: i32, max_row: i32, max_col: i32) -> Result<(), String> {
        // Math.Clamp throws when min > max.
        let clamp = |v: i32, lo: i32, hi: i32| -> Result<i32, String> {
            if lo > hi {
                return Err(format!("'{}' cannot be greater than {}.", lo, hi));
            }
            Ok(v.clamp(lo, hi))
        };
        self.start.row = clamp(self.start.row, min_row, max_row)?;
        self.start.col = clamp(self.start.col, min_col, max_col)?;
        self.end.row = if self.end.row == 0 {
            max_row
        } else {
            clamp(self.end.row, min_row, max_row)?
        };
        self.end.col = if self.end.col == 0 {
            max_col
        } else {
            clamp(self.end.col, min_col, max_col)?
        };
        if self.start.row > self.end.row {
            std::mem::swap(&mut self.start.row, &mut self.end.row);
        }
        if self.start.col > self.end.col {
            std::mem::swap(&mut self.start.col, &mut self.end.col);
        }
        Ok(())
    }
}

/// The package: its parts by name (without a leading slash).
struct Package {
    parts: Vec<Entry>,
}

fn resolve(base_part: &str, target: &str) -> String {
    if let Some(abs) = target.strip_prefix('/') {
        return abs.to_string();
    }
    let mut parts: Vec<&str> = base_part.split('/').collect();
    parts.pop();
    for seg in target.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

fn rels_name(part: &str) -> String {
    match part.rfind('/') {
        Some(i) => format!("{}/_rels/{}.rels", &part[..i], &part[i + 1..]),
        None => format!("_rels/{}.rels", part),
    }
}

impl Package {
    fn open(bytes: &[u8]) -> Result<Package, String> {
        Ok(Package {
            parts: zip::read(bytes)?,
        })
    }

    fn get(&self, name: &str) -> Option<&[u8]> {
        self.parts
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .map(|e| e.data.as_slice())
    }

    fn xml(&self, name: &str) -> Result<Element, String> {
        let data = self.get(name).ok_or_else(|| NULL_REFERENCE.to_string())?;
        xml::parse(&String::from_utf8_lossy(data))
    }

    fn put(&mut self, name: &str, data: Vec<u8>) {
        match self.parts.iter_mut().find(|e| e.name.eq_ignore_ascii_case(name)) {
            Some(e) => e.data = data,
            None => self.parts.push(Entry {
                name: name.to_string(),
                data,
            }),
        }
    }

    /// Relationships of a part: (id, type, resolved target).
    fn rels(&self, part: &str) -> Vec<(String, String, String)> {
        let Ok(r) = self.xml(&rels_name(part)) else {
            return Vec::new();
        };
        r.elements()
            .filter(|e| local(&e.name) == "Relationship")
            .map(|e| {
                (
                    e.attr("Id").unwrap_or_default().to_string(),
                    e.attr("Type").unwrap_or_default().to_string(),
                    resolve(part, e.attr("Target").unwrap_or_default()),
                )
            })
            .collect()
    }

    fn workbook_part(&self) -> Result<String, String> {
        self.rels("")
            .into_iter()
            .find(|(_, t, _)| t == REL_OFFICE_DOC || t.ends_with("/officeDocument"))
            .map(|(_, _, target)| target)
            .ok_or_else(|| "The Excel workbook is missing or not initialized.".to_string())
    }
}

/// `ExcelData.ReadFromMemory` (and `Read`, given the file's bytes). None where C# returns null.
pub fn read(
    bytes: &[u8],
    sheet_name: &str,
    range_start: &str,
    range_end: &str,
) -> Result<Option<Vec<Vec<String>>>, String> {
    let pkg = Package::open(bytes)?;
    let wb_name = pkg.workbook_part()?;
    let workbook = pkg.xml(&wb_name)?;
    let mut sheets = Vec::new();
    workbook.descendants("sheet", &mut sheets);
    let sheet = if !sheet_name.is_empty() {
        sheets
            .iter()
            .find(|s| s.attr("name") == Some(sheet_name))
            .ok_or_else(|| format!("Worksheet \"{}\" not found.", sheet_name))?
    } else {
        sheets
            .first()
            .ok_or("This Excel workbook doesn not contain worksheets.")?
    };
    let id = sheet.attr("r:id").unwrap_or_default();
    let wb_rels = pkg.rels(&wb_name);
    let ws_name = wb_rels
        .iter()
        .find(|(rid, _, _)| rid == id)
        .map(|(_, _, t)| t.clone())
        .ok_or_else(|| format!("The specified package part (id '{}') does not exist.", id))?;
    let worksheet = pkg.xml(&ws_name)?;
    let shared: Vec<Element> = match wb_rels.iter().find(|(_, t, _)| t.ends_with("/sharedStrings")) {
        Some((_, _, target)) => match pkg.xml(target) {
            Ok(sst) => sst.elements().cloned().collect(),
            Err(_) => Vec::new(),
        },
        None => Vec::new(),
    };
    let mut range = CellRange::new(range_start, range_end)?;
    let mut rows = Vec::new();
    worksheet.descendants("row", &mut rows);
    let max_row: i32 = match rows.last() {
        None => return Err(NULL_REFERENCE.into()),
        Some(r) => r.attr("r").and_then(|v| v.parse::<u32>().ok()).ok_or(NULL_REFERENCE)? as i32,
    };
    let mut cells = Vec::new();
    worksheet.descendants("c", &mut cells);
    let max_col = cells
        .iter()
        .map(|c| CellRef::col_from_ref(c.attr("r")))
        .max()
        .ok_or("Sequence contains no elements")?;
    range.normalize(1, 1, max_row, max_col)?;
    let (row_count, col_count) = (range.row_count(), range.col_count());
    if row_count < 0 || col_count < 0 {
        return Ok(None);
    }
    let mut by_ref: HashMap<&str, &Element> = HashMap::new();
    for c in &cells {
        if let Some(r) = c.attr("r") {
            by_ref.entry(r).or_insert(c);
        }
    }
    let mut data = Vec::with_capacity(row_count as usize);
    let mut current = range.start;
    for _ in 0..row_count {
        let mut row = Vec::with_capacity(col_count as usize);
        current.col = range.start.col;
        for _ in 0..col_count {
            row.push(cell_value(&by_ref, &shared, &current.to_ref())?);
            current.col += 1;
        }
        data.push(row);
        current.row += 1;
    }
    Ok(Some(data))
}

fn cell_value(by_ref: &HashMap<&str, &Element>, shared: &[Element], cell_ref: &str) -> Result<String, String> {
    let Some(cell) = by_ref.get(cell_ref) else {
        return Ok(String::new());
    };
    let value = cell.inner_text();
    if cell.attr("t") == Some("s") {
        let i: usize = value
            .trim()
            .parse()
            .map_err(|_| format!("The input string '{}' was not in a correct format.", value))?;
        return shared.get(i).map(|si| si.inner_text()).ok_or_else(|| {
            "Index was out of range. Must be non-negative and less than the size of the collection.".into()
        });
    } else if cell.child("f").is_some() {
        if let Some(v) = cell.child("v").map(|v| v.inner_text()) {
            if !v.is_empty() {
                return Ok(v);
            }
        }
    }
    Ok(value)
}

fn new_workbook(sheet_name: &str) -> Package {
    let name = if sheet_name.trim().is_empty() {
        DEFAULT_SHEET
    } else {
        sheet_name
    };
    let content_types = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"{}\"/></Types>",
        CT_WORKSHEET
    );
    let rels = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Type=\"{}\" Target=\"/xl/workbook.xml\" Id=\"R1\"/></Relationships>",
        REL_OFFICE_DOC
    );
    let workbook = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><x:workbook xmlns:x=\"{}\"><x:sheets><x:sheet xmlns:r=\"{}\" r:id=\"R2\" sheetId=\"1\" name=\"{}\"/></x:sheets></x:workbook>",
        NS_MAIN,
        NS_REL,
        xml::escape(name, true)
    );
    let wb_rels = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Type=\"{}\" Target=\"/xl/worksheets/sheet1.xml\" Id=\"R2\"/></Relationships>",
        REL_WORKSHEET
    );
    let sheet = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><x:worksheet xmlns:x=\"{}\"><x:sheetData/></x:worksheet>",
        NS_MAIN
    );
    Package {
        parts: vec![
            Entry {
                name: "[Content_Types].xml".into(),
                data: content_types.into_bytes(),
            },
            Entry {
                name: "_rels/.rels".into(),
                data: rels.into_bytes(),
            },
            Entry {
                name: "xl/workbook.xml".into(),
                data: workbook.into_bytes(),
            },
            Entry {
                name: "xl/_rels/workbook.xml.rels".into(),
                data: wb_rels.into_bytes(),
            },
            Entry {
                name: "xl/worksheets/sheet1.xml".into(),
                data: sheet.into_bytes(),
            },
        ],
    }
}

/// `InsertWorksheet`: a new worksheet part, its relationship and its entry in the workbook.
fn insert_worksheet(
    pkg: &mut Package,
    wb_name: &str,
    workbook: &mut Element,
    sheet_name: &str,
) -> Result<String, String> {
    let mut n = 1;
    let ws_name = loop {
        let candidate = format!("xl/worksheets/sheet{}.xml", n);
        if pkg.get(&candidate).is_none() {
            break candidate;
        }
        n += 1;
    };
    let prefix = workbook
        .name
        .split_once(':')
        .map(|(p, _)| format!("{}:", p))
        .unwrap_or_default();
    pkg.put(
        &ws_name,
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><x:worksheet xmlns:x=\"{}\"><x:sheetData/></x:worksheet>",
            NS_MAIN
        )
        .into_bytes(),
    );
    let rels_part = rels_name(wb_name);
    let mut rels = pkg.xml(&rels_part).unwrap_or_else(|_| {
        let mut r = Element::new("Relationships");
        r.set_attr("xmlns", "http://schemas.openxmlformats.org/package/2006/relationships");
        r
    });
    let mut k = 1;
    let rid = loop {
        let id = format!("R{}", k);
        if !rels.elements().any(|e| e.attr("Id") == Some(&id)) {
            break id;
        }
        k += 1;
    };
    let mut rel = Element::new("Relationship");
    rel.set_attr("Type", REL_WORKSHEET);
    rel.set_attr("Target", &format!("/{}", ws_name));
    rel.set_attr("Id", &rid);
    rels.children.push(Node::Element(rel));
    pkg.put(&rels_part, xml::to_document(&rels).into_bytes());
    if let Ok(mut ct) = pkg.xml("[Content_Types].xml") {
        let mut o = Element::new("Override");
        o.set_attr("PartName", &format!("/{}", ws_name));
        o.set_attr("ContentType", CT_WORKSHEET);
        ct.children.push(Node::Element(o));
        pkg.put("[Content_Types].xml", xml::to_document(&ct).into_bytes());
    }
    if workbook.child("sheets").is_none() {
        workbook
            .children
            .push(Node::Element(Element::new(&format!("{}sheets", prefix))));
    }
    let sheets = workbook.child_mut("sheets").unwrap();
    let sheet_id = sheets
        .elements()
        .filter_map(|s| s.attr("sheetId").and_then(|v| v.parse::<u32>().ok()))
        .max()
        .unwrap_or(0)
        + 1;
    let mut sheet = Element::new(&format!("{}sheet", prefix));
    sheet.set_attr("xmlns:r", NS_REL);
    sheet.set_attr("r:id", &rid);
    sheet.set_attr("sheetId", &sheet_id.to_string());
    sheet.set_attr("name", sheet_name);
    sheets.children.push(Node::Element(sheet));
    Ok(rid)
}

/// `double.TryParse(value, CultureInfo.CurrentCulture)` with the invariant culture: Float
/// and AllowThousands.
fn try_parse_number(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty()
        || !t
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E' | ','))
    {
        return None;
    }
    let mantissa_end = t.find(['e', 'E']).unwrap_or(t.len());
    if t[mantissa_end..].contains(',') || t.starts_with(',') {
        return None;
    }
    t.replace(',', "").parse::<f64>().ok()
}

fn insert_value(row: &mut Element, prefix: &str, cell_ref: &str, value: &str) {
    let cell = insert_cell(row, prefix, cell_ref);
    cell.children.clear();
    match try_parse_number(value) {
        Some(d) => {
            cell.set_attr("t", "n");
            let mut v = Element::new(&format!("{}v", prefix));
            v.children.push(Node::Text(crate::netfmt::double_to_string(d)));
            cell.children.push(Node::Element(v));
        }
        None => {
            cell.set_attr("t", "inlineStr");
            let mut is = Element::new(&format!("{}is", prefix));
            let mut t = Element::new(&format!("{}t", prefix));
            t.children.push(Node::Text(value.to_string()));
            is.children.push(Node::Element(t));
            cell.children.push(Node::Element(is));
        }
    }
}

fn insert_cell<'a>(row: &'a mut Element, prefix: &str, cell_ref: &str) -> &'a mut Element {
    let pos = row
        .children
        .iter()
        .position(|n| matches!(n, Node::Element(c) if local(&c.name) == "c" && c.attr("r") == Some(cell_ref)));
    let idx = match pos {
        Some(i) => i,
        None => {
            let mut cell = Element::new(&format!("{}c", prefix));
            cell.set_attr("r", cell_ref);
            // By column number: as text "AA1" would sort before "B1".
            let key = CellRef::col_from_ref(Some(cell_ref));
            let before = row.children.iter().position(|n| {
                matches!(n, Node::Element(c) if local(&c.name) == "c"
                    && c.attr("r").is_some_and(|r| CellRef::col_from_ref(Some(r)) > key))
            });
            match before {
                Some(i) => {
                    row.children.insert(i, Node::Element(cell));
                    i
                }
                None => {
                    row.children.push(Node::Element(cell));
                    row.children.len() - 1
                }
            }
        }
    };
    match &mut row.children[idx] {
        Node::Element(e) => e,
        _ => unreachable!(),
    }
}

/// `ExcelData.Write`: `existing` is the current file, if any. Returns the new file.
pub fn write(
    existing: Option<&[u8]>,
    sheet_name: &str,
    range_start: &str,
    range_end: &str,
    data: &[Vec<String>],
    append: bool,
) -> Result<Vec<u8>, String> {
    let mut pkg = match existing {
        Some(bytes) if append => Package::open(bytes)?,
        _ => new_workbook(sheet_name),
    };
    let wb_name = pkg.workbook_part()?;
    let mut workbook = pkg.xml(&wb_name)?;
    let (sheet_rid, sheet_name) = {
        let mut sheets = Vec::new();
        workbook.descendants("sheet", &mut sheets);
        if sheet_name.trim().is_empty() {
            (
                sheets.first().and_then(|s| s.attr("r:id")).map(str::to_string),
                DEFAULT_SHEET.to_string(),
            )
        } else {
            (
                sheets
                    .iter()
                    .find(|s| s.attr("name") == Some(sheet_name))
                    .and_then(|s| s.attr("r:id"))
                    .map(str::to_string),
                sheet_name.to_string(),
            )
        }
    };
    let rid = match sheet_rid {
        Some(r) => r,
        None => {
            let r = insert_worksheet(&mut pkg, &wb_name, &mut workbook, &sheet_name)?;
            pkg.put(&wb_name, xml::to_document(&workbook).into_bytes());
            r
        }
    };
    let ws_name = pkg
        .rels(&wb_name)
        .into_iter()
        .find(|(id, _, _)| *id == rid)
        .map(|(_, _, t)| t)
        .ok_or_else(|| format!("The specified package part (id '{}') does not exist.", rid))?;
    let mut worksheet = pkg.xml(&ws_name)?;
    let prefix = worksheet
        .name
        .split_once(':')
        .map(|(p, _)| format!("{}:", p))
        .unwrap_or_default();
    let mut range = CellRange::new(range_start, range_end)?;
    let max_row = range.start.row.max(1) + data.len() as i32 - 1;
    let max_col = range.start.col.max(1)
        + data
            .iter()
            .map(|r| r.len() as i32)
            .max()
            .ok_or("Sequence contains no elements")?
        - 1;
    range.normalize(1, 1, max_row, max_col)?;
    let (row_count, col_count) = (range.row_count(), range.col_count());
    if worksheet.child("sheetData").is_none() {
        worksheet
            .children
            .push(Node::Element(Element::new(&format!("{}sheetData", prefix))));
    }
    let sheet_data = worksheet.child_mut("sheetData").unwrap();
    let mut current = range.start;
    for i in 0..row_count.max(0) as usize {
        let r = current.row.to_string();
        let pos = sheet_data
            .children
            .iter()
            .position(|n| matches!(n, Node::Element(e) if local(&e.name) == "row" && e.attr("r") == Some(r.as_str())));
        let idx = match pos {
            Some(p) => p,
            None => {
                let mut row = Element::new(&format!("{}row", prefix));
                row.set_attr("r", &r);
                sheet_data.children.push(Node::Element(row));
                sheet_data.children.len() - 1
            }
        };
        let Node::Element(row) = &mut sheet_data.children[idx] else {
            unreachable!()
        };
        let row_data = &data[i];
        let len = row_data.len().min(col_count.max(0) as usize);
        current.col = range.start.col;
        for value in row_data.iter().take(len) {
            insert_value(row, &prefix, &current.to_ref(), value);
            current.col += 1;
        }
        current.row += 1;
    }
    pkg.put(&ws_name, xml::to_document(&worksheet).into_bytes());
    Ok(zip::write(&pkg.parts))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&[&str]]) -> Vec<Vec<String>> {
        v.iter().map(|r| r.iter().map(|c| c.to_string()).collect()).collect()
    }

    #[test]
    fn cells_past_column_z_stay_in_column_order() {
        let row: Vec<String> = (1..=30).map(|i| i.to_string()).collect();
        let book = write(None, "Sheet1", "A1", "AD1", std::slice::from_ref(&row), false).unwrap();
        // Writing over the row again inserts nothing out of place.
        let book = write(Some(&book), "Sheet1", "Z1", "AB1", &s(&[&["x", "y", "z"]]), true).unwrap();
        let parts = zip::read(&book).unwrap();
        let sheet = parts.iter().find(|p| p.name.ends_with("sheet1.xml")).unwrap();
        let text = String::from_utf8(sheet.data.clone()).unwrap();
        let refs: Vec<i32> = text
            .match_indices(" r=\"")
            .filter_map(|(i, _)| {
                let v = &text[i + 4..];
                let r = &v[..v.find('"')?];
                (!r.chars().all(|c| c.is_ascii_digit())).then(|| CellRef::col_from_ref(Some(r)))
            })
            .collect();
        assert_eq!(refs, (1..=30).collect::<Vec<i32>>(), "{text}");
        let back = read(&book, "Sheet1", "Y1", "AC1").unwrap().unwrap();
        assert_eq!(back, s(&[&["25", "x", "y", "z", "29"]]));
    }

    #[test]
    fn writes_and_reads_back() {
        let book = write(None, "Sheet1", "A1", "B2", &s(&[&["1", "2"], &["3", "4m"]]), false).unwrap();
        assert_eq!(
            read(&book, "Sheet1", "A1", "B2").unwrap().unwrap(),
            s(&[&["1", "2"], &["3", "4m"]])
        );
        // Append into a second sheet of the same book. With no range, a read starts at A1
        // (the start clamps to row and column 1) and ends at the last used cell.
        let book = write(Some(&book), "Other", "B2", "", &s(&[&["5"]]), true).unwrap();
        assert_eq!(
            read(&book, "Other", "", "").unwrap().unwrap(),
            s(&[&["", ""], &["", "5"]])
        );
        assert_eq!(read(&book, "Other", "B2", "").unwrap().unwrap(), s(&[&["5"]]));
        assert_eq!(read(&book, "", "A2", "").unwrap().unwrap(), s(&[&["3", "4m"]]));
        assert!(read(&book, "Missing", "", "").is_err());
        assert_eq!(col_name(28), "AB");
        assert_eq!(col_number("AB"), 28);
    }
}
