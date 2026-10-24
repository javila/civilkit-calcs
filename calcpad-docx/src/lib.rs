//! Word (.docx) export of a calculated worksheet: a port of Calcpad.OpenXml's Word path
//! (OpenXmlWriter, TableBuilder, StyleDefinitionsWriter, ImageWriter/ImageProcessor and
//! CssParser), writing the package with calcpad-core's own zip and XML code. The equations are
//! the Office Math that calcpad-core's XmlWriter makes (`ExpressionParser::capture_open_xml`).
pub mod css;
pub mod fonts;
pub mod html;
pub mod image;
pub mod mathml;
pub mod package;
pub mod writer;
pub mod xlsx;
pub mod xlsx_chart;
pub mod xlsx_report;

pub use xlsx_report::html_to_xlsx;

pub use image::ImageResolver;

/// `OpenXmlWriter.Convert`: the worksheet's HTML (a whole page or its body) and the equations
/// captured with it, as a .docx file. `resolver` supplies images that are not data: URLs.
pub fn html_to_docx(html: &str, expressions: &[String], resolver: Option<&ImageResolver>) -> Vec<u8> {
    let html = mathml::convert_page(html);
    let doc = html::parse(&html);
    let mut w = writer::Writer::new(expressions, resolver);
    let body = w.body(&doc);
    let ordered = std::mem::take(&mut w.package.ordered_lists);
    let package = std::mem::take(&mut w.package);
    package.finish(&body, &ordered)
}
