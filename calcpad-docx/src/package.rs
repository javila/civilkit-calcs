//! The .docx package: parts, relationships and content types, written with calcpad-core's zip.
//! Styles and numbering are ports of StyleDefinitionsWriter.cs and `CreateNumberingPart`.
use calcpad_core::openxml::xml::escape;
use calcpad_core::openxml::zip::{self, Entry};

#[derive(Default)]
pub struct Package {
    /// (id, type, target, external)
    rels: Vec<(String, &'static str, String, bool)>,
    media: Vec<(String, Vec<u8>)>,
    next_id: usize,
    /// The numbering instances ordered lists use (D6).
    pub ordered_lists: Vec<i64>,
}

const REL_IMAGE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const REL_LINK: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
const REL_STYLES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const REL_NUMBERING: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering";

impl Package {
    fn new_id(&mut self) -> String {
        self.next_id += 1;
        format!("rId{}", self.next_id + 2)
    }

    pub fn add_hyperlink(&mut self, url: &str) -> String {
        let id = self.new_id();
        self.rels.push((id.clone(), REL_LINK, url.to_string(), true));
        id
    }

    /// Adds an image part; `ext` is its file extension (png, jpeg, gif, bmp, tiff, svg, ...).
    pub fn add_image(&mut self, ext: &str, data: Vec<u8>) -> String {
        let id = self.new_id();
        let name = format!("media/image{}.{}", self.media.len() + 1, ext);
        self.rels.push((id.clone(), REL_IMAGE, name.clone(), false));
        self.media.push((name, data));
        id
    }

    /// The finished document: `body` is the `<w:body>` element.
    pub fn finish(self, body: &str, ordered_lists: &[i64]) -> Vec<u8> {
        let mut document = String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><w:document \
             xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
             xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" \
             xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
             xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
             xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
             xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\" \
             xmlns:asvg=\"http://schemas.microsoft.com/office/drawing/2016/SVG/main\">",
        );
        document.push_str(body);
        document.push_str("</w:document>");

        let mut rels = String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
        );
        rels.push_str(&format!(
            "<Relationship Id=\"rId1\" Type=\"{}\" Target=\"numbering.xml\"/>",
            REL_NUMBERING
        ));
        rels.push_str(&format!(
            "<Relationship Id=\"rId2\" Type=\"{}\" Target=\"styles.xml\"/>",
            REL_STYLES
        ));
        for (id, ty, target, external) in &self.rels {
            rels.push_str(&format!(
                "<Relationship Id=\"{}\" Type=\"{}\" Target=\"{}\"{}/>",
                id,
                ty,
                escape(target, true),
                if *external { " TargetMode=\"External\"" } else { "" }
            ));
        }
        rels.push_str("</Relationships>");

        let mut types = String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
             <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
             <Default Extension=\"xml\" ContentType=\"application/xml\"/>",
        );
        let mut exts: Vec<&str> = self.media.iter().filter_map(|(n, _)| n.rsplit('.').next()).collect();
        exts.sort();
        exts.dedup();
        for e in exts {
            types.push_str(&format!(
                "<Default Extension=\"{}\" ContentType=\"{}\"/>",
                e,
                media_type(e)
            ));
        }
        types.push_str(
            "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
             <Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>\
             <Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>\
             </Types>",
        );

        let mut entries = vec![
            entry("[Content_Types].xml", types.into_bytes()),
            entry(
                "_rels/.rels",
                "<?xml version=\"1.0\" encoding=\"utf-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
                 <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>\
                 </Relationships>"
                    .as_bytes()
                    .to_vec(),
            ),
            entry("word/document.xml", document.into_bytes()),
            entry("word/_rels/document.xml.rels", rels.into_bytes()),
            entry("word/styles.xml", styles().into_bytes()),
            entry("word/numbering.xml", numbering(ordered_lists).into_bytes()),
        ];
        for (name, data) in self.media {
            entries.push(entry(&format!("word/{}", name), data));
        }
        zip::write(&entries)
    }
}

fn entry(name: &str, data: Vec<u8>) -> Entry {
    Entry {
        name: name.to_string(),
        data,
    }
}

fn media_type(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "gif" => "image/gif",
        "jpeg" | "jpg" => "image/jpeg",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "svg" => "image/svg+xml",
        "wmf" => "image/x-wmf",
        "emf" => "image/x-emf",
        "ico" => "image/x-icon",
        "pcx" => "image/x-pcx",
        _ => "application/octet-stream",
    }
}

/// `StyleDefinitionsWriter.CreateStyleDefinitionsPart`
fn styles() -> String {
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/><w:sz w:val=\"24\"/></w:rPr></w:rPrDefault></w:docDefaults>",
    );
    for (i, size) in ["48", "40", "36", "32", "28", "26"].iter().enumerate() {
        let id = format!("h{}", i + 1);
        s.push_str(&format!(
            "<w:style w:type=\"paragraph\" w:styleId=\"{0}\"><w:name w:val=\"{0}\"/><w:basedOn w:val=\"Normal\"/><w:next w:val=\"Normal\"/><w:rPr><w:b/><w:sz w:val=\"{1}\"/></w:rPr></w:style>",
            id, size
        ));
    }
    for (id, name, margin, align) in [
        ("bcpd", "Calcpad Bordered", ["34", "68", "34", "68"], "center"),
        ("cpd", "Calcpad", ["0", "0", "0", "0"], "top"),
    ] {
        s.push_str(&format!(
            "<w:style w:type=\"table\" w:styleId=\"{}\"><w:name w:val=\"{}\"/><w:tcPr><w:tcMar><w:top w:w=\"{}\" w:type=\"dxa\"/><w:left w:w=\"{}\" w:type=\"dxa\"/><w:bottom w:w=\"{}\" w:type=\"dxa\"/><w:right w:w=\"{}\" w:type=\"dxa\"/></w:tcMar><w:vAlign w:val=\"{}\"/></w:tcPr></w:style>",
            id, name, margin[0], margin[1], margin[2], margin[3], align
        ));
    }
    s.push_str("</w:styles>");
    s
}

/// `CreateNumberingPart`: bullets (instance 1) and decimal numbers (instance 2), plus one
/// instance per ordered list so each starts at 1.
fn numbering(ordered_lists: &[i64]) -> String {
    let level = |i: usize, fmt: &str, text: &str| {
        format!(
            "<w:lvl w:ilvl=\"{}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{}\"/><w:lvlText w:val=\"{}\"/><w:lvlJc w:val=\"left\"/></w:lvl>",
            i, fmt, text
        )
    };
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <w:abstractNum w:abstractNumId=\"0\"><w:multiLevelType w:val=\"multilevel\"/>",
    );
    for (i, b) in ["\u{2022}", "\u{25CB}", "\u{25AA}", "-"].iter().enumerate() {
        s.push_str(&level(i, "bullet", b));
    }
    s.push_str("</w:abstractNum><w:abstractNum w:abstractNumId=\"1\"><w:multiLevelType w:val=\"multilevel\"/>");
    s.push_str(&level(0, "decimal", "%1."));
    s.push_str("</w:abstractNum>");
    s.push_str("<w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num><w:num w:numId=\"2\"><w:abstractNumId w:val=\"1\"/></w:num>");
    for id in ordered_lists {
        s.push_str(&format!(
            "<w:num w:numId=\"{}\"><w:abstractNumId w:val=\"1\"/><w:lvlOverride w:ilvl=\"0\"><w:startOverride w:val=\"1\"/></w:lvlOverride></w:num>",
            id
        ));
    }
    s.push_str("</w:numbering>");
    s
}
