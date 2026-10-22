//! Spike: import the simple subset of a foreign XML worksheet format (`.xmcd`) as Calcpad text.
//!
//! One engine: this crate evaluates nothing. It translates a worksheet into Calcpad worksheet
//! text for the existing engine (`calcpad-core`) and reports, per region kind, what converted
//! and, for everything that did not, a stable code, a reason and a region and line reference.
//! A region that cannot be converted is written to the output as a visible placeholder line (it
//! starts with [`GAP_MARKER`]); it is never dropped silently and nothing is invented.
//!
//! ```
//! let xml = r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30">
//!   <regions>
//!     <region region-id="1" left="0" top="0"><math><ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define></math></region>
//!     <region region-id="2" left="0" top="20"><math><ml:eval><ml:apply><ml:mult/><ml:id>x</ml:id><ml:real>3</ml:real></ml:apply></ml:eval></math></region>
//!   </regions>
//! </worksheet>"#;
//! let r = calcpad_import_xmcd::import_xmcd(xml);
//! assert!(r.calcpad.contains("x = 2"));
//! assert!(r.calcpad.contains("x*3"));
//! assert_eq!(r.report.regions_needing_attention(), 0);
//! ```
//!
//! # Front doors
//!
//! [`import_xmcd`] reads the plain-XML `.xmcd` file. The translator works on the format-neutral
//! [`ir::Sheet`], so another container of the same worksheet language only needs a reader that
//! produces a `Sheet` and then calls [`import_sheet`].

#![forbid(unsafe_code)]

pub mod ir;
pub mod report;
pub mod tables;
pub mod translate;
pub mod xmcd;

pub use report::{Code, ImportReport, Issue, KindStats, RankedGap, Severity};
pub use translate::GAP_MARKER;
pub use xmcd::Limits;

/// The result of an import: the Calcpad text and the report.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportResult {
    pub calcpad: String,
    pub report: ImportReport,
}

/// Imports an `.xmcd` worksheet with the default limits.
pub fn import_xmcd(xml: &str) -> ImportResult {
    import_xmcd_with(xml, Limits::default())
}

/// Imports an `.xmcd` worksheet with explicit limits.
pub fn import_xmcd_with(xml: &str, limits: Limits) -> ImportResult {
    match xmcd::read(xml, limits) {
        Ok(sheet) => import_sheet(sheet),
        Err(e) => {
            let (code, detail) = match e {
                xmcd::ReadError::TooLarge => (Code::InputTooLarge, None),
                xmcd::ReadError::NotAWorksheet => (Code::NotAWorksheet, None),
                // The parser's message can quote positions only, never input text, but keep it short.
                xmcd::ReadError::Xml(m) => (Code::XmlError, Some(m.chars().take(120).collect::<String>())),
            };
            let mut report = ImportReport::default();
            report.issues.push(Issue {
                code,
                severity: code.severity(),
                detail,
                region_id: None,
                line: None,
                reason: code.reason().to_string(),
                root: None,
            });
            ImportResult {
                calcpad: String::new(),
                report,
            }
        }
    }
}

/// Translates a sheet that a reader already produced.
pub fn import_sheet(sheet: ir::Sheet) -> ImportResult {
    let (calcpad, report) = translate::translate(sheet);
    ImportResult { calcpad, report }
}
