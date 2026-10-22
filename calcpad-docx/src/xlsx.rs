//! A small writer for the Excel workbooks of calcpad-rs's report export: cells with styles
//! (bold, size, colour, fill, borders, wrapping), merged ranges, column widths, row heights,
//! internal hyperlinks and pictures. Strings are written inline. The package parts are written
//! with calcpad-core's zip and XML escaping.
use std::collections::HashMap;

use calcpad_core::openxml::xml::escape;
use calcpad_core::openxml::zip::{self, Entry};

/// A cell's formatting. `Default` is Excel's normal cell.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// Point size; 0 is the default (11).
    pub size: u32,
    /// `RRGGBB`, empty for the default colour.
    pub color: &'static str,
    /// `RRGGBB` background, empty for none.
    pub fill: &'static str,
    pub border: bool,
    pub wrap: bool,
    /// "left", "center", "right" or empty.
    pub align: &'static str,
    pub top: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Empty,
    Text(String),
    Number(f64),
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub value: Value,
    pub style: usize,
}

pub struct Picture {
    /// Zero-based column and row of the top left corner.
    pub col: u32,
    pub row: u32,
    /// Size in pixels as shown.
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
    /// "png", "jpeg" or "gif".
    pub ext: &'static str,
    pub name: String,
}

#[derive(Default)]
pub struct Sheet {
    pub name: String,
    /// Row (1-based) to column (1-based) to cell.
    pub cells: std::collections::BTreeMap<u32, std::collections::BTreeMap<u32, Cell>>,
    pub heights: HashMap<u32, f64>,
    pub merges: Vec<String>,
    /// (first column, last column, width in characters), 1-based.
    pub widths: Vec<(u32, u32, f64)>,
    /// (cell reference, target sheet name).
    pub links: Vec<(String, String)>,
    pub pictures: Vec<Picture>,
    /// Native charts, placed like pictures.
    pub charts: Vec<crate::xlsx_chart::Chart>,
    /// Rows above this one stay in view (0: none).
    pub freeze_rows: u32,
}

pub fn col_name(mut n: u32) -> String {
    let mut s = Vec::new();
    while n > 0 {
        let r = (n - 1) % 26;
        s.push((b'A' + r as u8) as char);
        n = (n - 1) / 26;
    }
    s.iter().rev().collect()
}

pub fn cell_ref(col: u32, row: u32) -> String {
    format!("{}{}", col_name(col), row)
}

impl Sheet {
    pub fn new(name: &str) -> Sheet {
        Sheet {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn set(&mut self, row: u32, col: u32, value: Value, style: usize) {
        self.cells.entry(row).or_default().insert(col, Cell { value, style });
    }

    pub fn merge(&mut self, row: u32, col1: u32, col2: u32) {
        if col2 > col1 {
            self.merges
                .push(format!("{}:{}", cell_ref(col1, row), cell_ref(col2, row)));
        }
    }

    pub fn last_row(&self) -> u32 {
        self.cells.keys().next_back().copied().unwrap_or(0)
    }
}

#[derive(Default)]
pub struct Workbook {
    pub sheets: Vec<Sheet>,
    styles: Vec<Style>,
}

impl Workbook {
    pub fn new() -> Workbook {
        Workbook::default()
    }

    /// The index of the style (0 is the normal cell).
    pub fn style(&mut self, s: Style) -> usize {
        if s == Style::default() {
            return 0;
        }
        if let Some(i) = self.styles.iter().position(|x| *x == s) {
            return i + 1;
        }
        self.styles.push(s);
        self.styles.len()
    }

    fn styles_xml(&self) -> String {
        // Fonts, fills and borders are deduplicated; index 0 of each is the default.
        let mut fonts: Vec<(bool, bool, bool, u32, &str)> = vec![(false, false, false, 0, "")];
        let mut fills: Vec<&str> = vec!["", "gray125"];
        let mut xfs = String::from("<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>");
        for s in &self.styles {
            let font = (s.bold, s.italic, s.underline, s.size, s.color);
            let fi = fonts.iter().position(|f| *f == font).unwrap_or_else(|| {
                fonts.push(font);
                fonts.len() - 1
            });
            let fl = if s.fill.is_empty() {
                0
            } else {
                fills.iter().position(|f| *f == s.fill).unwrap_or_else(|| {
                    fills.push(s.fill);
                    fills.len() - 1
                })
            };
            let align = if s.wrap || !s.align.is_empty() || s.top {
                format!(
                    "<alignment{}{}{}/>",
                    if s.align.is_empty() {
                        String::new()
                    } else {
                        format!(" horizontal=\"{}\"", s.align)
                    },
                    if s.top { " vertical=\"top\"" } else { "" },
                    if s.wrap { " wrapText=\"1\"" } else { "" }
                )
            } else {
                String::new()
            };
            xfs.push_str(&format!(
                "<xf numFmtId=\"0\" fontId=\"{}\" fillId=\"{}\" borderId=\"{}\" xfId=\"0\" applyFont=\"1\" applyFill=\"1\" applyBorder=\"1\" applyAlignment=\"1\">{}</xf>",
                fi,
                fl,
                s.border as u32,
                align
            ));
        }
        let mut f = String::new();
        for (b, i, u, size, color) in &fonts {
            f.push_str("<font>");
            if *b {
                f.push_str("<b/>");
            }
            if *i {
                f.push_str("<i/>");
            }
            if *u {
                f.push_str("<u/>");
            }
            f.push_str(&format!("<sz val=\"{}\"/>", if *size == 0 { 11 } else { *size }));
            if !color.is_empty() {
                f.push_str(&format!("<color rgb=\"FF{}\"/>", color));
            }
            f.push_str("<name val=\"Calibri\"/><family val=\"2\"/></font>");
        }
        let mut fl = String::new();
        for (n, c) in fills.iter().enumerate() {
            if n == 0 {
                fl.push_str("<fill><patternFill patternType=\"none\"/></fill>");
            } else if n == 1 {
                fl.push_str("<fill><patternFill patternType=\"gray125\"/></fill>");
            } else {
                fl.push_str(&format!(
                    "<fill><patternFill patternType=\"solid\"><fgColor rgb=\"FF{}\"/><bgColor indexed=\"64\"/></patternFill></fill>",
                    c
                ));
            }
        }
        let thin = "<border><left style=\"thin\"><color rgb=\"FFBFBFBF\"/></left><right style=\"thin\"><color rgb=\"FFBFBFBF\"/></right><top style=\"thin\"><color rgb=\"FFBFBFBF\"/></top><bottom style=\"thin\"><color rgb=\"FFBFBFBF\"/></bottom><diagonal/></border>";
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><fonts count=\"{}\">{}</fonts><fills count=\"{}\">{}</fills><borders count=\"2\"><border><left/><right/><top/><bottom/><diagonal/></border>{}</borders><cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs><cellXfs count=\"{}\">{}</cellXfs><cellStyles count=\"1\"><cellStyle name=\"Normal\" xfId=\"0\" builtinId=\"0\"/></cellStyles></styleSheet>",
            fonts.len(),
            f,
            fills.len(),
            fl,
            thin,
            self.styles.len() + 1,
            xfs
        )
    }

    fn sheet_xml(sheet: &Sheet, drawing_rid: Option<&str>) -> String {
        let mut x = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">",
        );
        // Printing: fit the width of the sheet on a landscape page, as long as it needs.
        x.push_str("<sheetPr><pageSetUpPr fitToPage=\"1\"/></sheetPr>");
        if sheet.freeze_rows > 0 {
            x.push_str(&format!(
                "<sheetViews><sheetView workbookViewId=\"0\"><pane ySplit=\"{}\" topLeftCell=\"A{}\" activePane=\"bottomLeft\" state=\"frozen\"/></sheetView></sheetViews>",
                sheet.freeze_rows,
                sheet.freeze_rows + 1
            ));
        }
        x.push_str("<sheetFormatPr defaultRowHeight=\"15\"/>");
        if !sheet.widths.is_empty() {
            x.push_str("<cols>");
            for (a, b, w) in &sheet.widths {
                x.push_str(&format!(
                    "<col min=\"{}\" max=\"{}\" width=\"{:.2}\" customWidth=\"1\"/>",
                    a, b, w
                ));
            }
            x.push_str("</cols>");
        }
        x.push_str("<sheetData>");
        let mut rows: Vec<u32> = sheet.cells.keys().copied().collect();
        for r in sheet.heights.keys() {
            if !sheet.cells.contains_key(r) {
                rows.push(*r);
            }
        }
        rows.sort_unstable();
        rows.dedup();
        for r in rows {
            let ht = sheet
                .heights
                .get(&r)
                .map(|h| format!(" ht=\"{:.2}\" customHeight=\"1\"", h))
                .unwrap_or_default();
            x.push_str(&format!("<row r=\"{}\"{}>", r, ht));
            if let Some(cells) = sheet.cells.get(&r) {
                for (c, cell) in cells {
                    let s = if cell.style > 0 {
                        format!(" s=\"{}\"", cell.style)
                    } else {
                        String::new()
                    };
                    let reference = cell_ref(*c, r);
                    match &cell.value {
                        Value::Empty => x.push_str(&format!("<c r=\"{}\"{}/>", reference, s)),
                        Value::Number(n) => {
                            x.push_str(&format!("<c r=\"{}\"{}><v>{}</v></c>", reference, s, number(*n)))
                        }
                        Value::Text(t) => x.push_str(&format!(
                            "<c r=\"{}\"{} t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                            reference,
                            s,
                            escape(&clean(t), false)
                        )),
                    }
                }
            }
            x.push_str("</row>");
        }
        x.push_str("</sheetData>");
        if !sheet.merges.is_empty() {
            x.push_str(&format!("<mergeCells count=\"{}\">", sheet.merges.len()));
            for m in &sheet.merges {
                x.push_str(&format!("<mergeCell ref=\"{}\"/>", m));
            }
            x.push_str("</mergeCells>");
        }
        if !sheet.links.is_empty() {
            x.push_str("<hyperlinks>");
            for (r, target) in &sheet.links {
                x.push_str(&format!(
                    "<hyperlink ref=\"{}\" location=\"{}\" display=\"{}\"/>",
                    r,
                    escape(&format!("'{}'!A1", target.replace('\'', "''")), true),
                    escape(target, true)
                ));
            }
            x.push_str("</hyperlinks>");
        }
        x.push_str(
            "<pageMargins left=\"0.7\" right=\"0.7\" top=\"0.75\" bottom=\"0.75\" header=\"0.3\" footer=\"0.3\"/>",
        );
        if let Some(rid) = drawing_rid {
            x.push_str(&format!("<drawing r:id=\"{}\"/>", rid));
        }
        x.push_str("</worksheet>");
        x
    }

    /// The package.
    pub fn finish(&self) -> Vec<u8> {
        let mut entries: Vec<Entry> = Vec::new();
        let mut content_types = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/><Default Extension=\"gif\" ContentType=\"image/gif\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>",
        );
        let mut workbook = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets>",
        );
        let mut wb_rels = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
        );
        let mut drawing_no = 0;
        let mut image_no = 0;
        let mut chart_no = 0;
        for (i, sheet) in self.sheets.iter().enumerate() {
            let n = i + 1;
            workbook.push_str(&format!(
                "<sheet name=\"{}\" sheetId=\"{}\" r:id=\"rId{}\"/>",
                escape(&sheet.name, true),
                n,
                n
            ));
            wb_rels.push_str(&format!(
                "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{}.xml\"/>",
                n, n
            ));
            content_types.push_str(&format!(
                "<Override PartName=\"/xl/worksheets/sheet{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>",
                n
            ));
            let mut drawing_rid = None;
            if !sheet.pictures.is_empty() || !sheet.charts.is_empty() {
                drawing_no += 1;
                drawing_rid = Some("rId1");
                entries.push(Entry {
                    name: format!("xl/worksheets/_rels/sheet{}.xml.rels", n),
                    data: format!(
                        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing\" Target=\"../drawings/drawing{}.xml\"/></Relationships>",
                        drawing_no
                    )
                    .into_bytes(),
                });
                content_types.push_str(&format!(
                    "<Override PartName=\"/xl/drawings/drawing{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.drawing+xml\"/>",
                    drawing_no
                ));
                let mut drawing = String::from(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<xdr:wsDr xmlns:xdr=\"http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">",
                );
                let mut rels = String::from(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
                );
                for (k, p) in sheet.pictures.iter().enumerate() {
                    image_no += 1;
                    entries.push(Entry {
                        name: format!("xl/media/image{}.{}", image_no, p.ext),
                        data: p.png.clone(),
                    });
                    rels.push_str(&format!(
                        "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"../media/image{}.{}\"/>",
                        k + 1,
                        image_no,
                        p.ext
                    ));
                    let (cx, cy) = (p.width as u64 * 9525, p.height as u64 * 9525);
                    drawing.push_str(&format!(
                        "<xdr:oneCellAnchor><xdr:from><xdr:col>{}</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>{}</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:from><xdr:ext cx=\"{}\" cy=\"{}\"/><xdr:pic><xdr:nvPicPr><xdr:cNvPr id=\"{}\" name=\"{}\" descr=\"{}\"/><xdr:cNvPicPr><a:picLocks noChangeAspect=\"1\"/></xdr:cNvPicPr></xdr:nvPicPr><xdr:blipFill><a:blip r:embed=\"rId{}\"/><a:stretch><a:fillRect/></a:stretch></xdr:blipFill><xdr:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></xdr:spPr></xdr:pic><xdr:clientData/></xdr:oneCellAnchor>",
                        p.col,
                        p.row,
                        cx,
                        cy,
                        k + 2,
                        escape(&p.name, true),
                        escape(&p.name, true),
                        k + 1,
                        cx,
                        cy
                    ));
                }
                for (k, c) in sheet.charts.iter().enumerate() {
                    chart_no += 1;
                    let rid = sheet.pictures.len() + k + 1;
                    entries.push(Entry {
                        name: format!("xl/charts/chart{}.xml", chart_no),
                        data: crate::xlsx_chart::chart_xml(c).into_bytes(),
                    });
                    content_types.push_str(&format!(
                        "<Override PartName=\"/xl/charts/chart{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.drawingml.chart+xml\"/>",
                        chart_no
                    ));
                    rels.push_str(&format!(
                        "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart\" Target=\"../charts/chart{}.xml\"/>",
                        rid, chart_no
                    ));
                    drawing.push_str(&crate::xlsx_chart::anchor_xml(c, rid, rid + 1));
                }
                drawing.push_str("</xdr:wsDr>");
                rels.push_str("</Relationships>");
                entries.push(Entry {
                    name: format!("xl/drawings/drawing{}.xml", drawing_no),
                    data: drawing.into_bytes(),
                });
                entries.push(Entry {
                    name: format!("xl/drawings/_rels/drawing{}.xml.rels", drawing_no),
                    data: rels.into_bytes(),
                });
            }
            entries.push(Entry {
                name: format!("xl/worksheets/sheet{}.xml", n),
                data: Self::sheet_xml(sheet, drawing_rid).into_bytes(),
            });
        }
        workbook.push_str("</sheets></workbook>");
        wb_rels.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/></Relationships>",
            self.sheets.len() + 1
        ));
        content_types.push_str("</Types>");
        let mut all = vec![
            Entry {
                name: "[Content_Types].xml".into(),
                data: content_types.into_bytes(),
            },
            Entry {
                name: "_rels/.rels".into(),
                data: b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>".to_vec(),
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
                name: "xl/styles.xml".into(),
                data: self.styles_xml().into_bytes(),
            },
        ];
        all.extend(entries);
        zip::write(&all)
    }
}

fn number(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{:?}", n)
    }
}

/// Control characters other than tab and line breaks are not allowed in XML.
fn clean(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r'))
        .collect()
}
