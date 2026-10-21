//! PdfSettingsDto.cs: the `pdf` payload of a metadata comment.
use crate::core_dto::directive_dto::*;
use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

pub const PDF_SETTING_KEY_NAMES: [&str; 12] = [
    "Format",
    "Orientation",
    "MarginTop",
    "MarginRight",
    "MarginBottom",
    "MarginLeft",
    "ShowPageNumbers",
    "ShowDate",
    "DocumentTitle",
    "DateTimeFormat",
    "HeaderFooterLines",
    "LineColor",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PdfSettingKey {
    Format,
    Orientation,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    ShowPageNumbers,
    ShowDate,
    DocumentTitle,
    DateTimeFormat,
    HeaderFooterLines,
    LineColor,
}

/// Paper names the PDF generator maps to a paper format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdfPaperFormat {
    Letter,
    Legal,
    Tabloid,
    Ledger,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
}

impl PdfPaperFormat {
    /// `Enum.GetNames<PdfPaperFormat>()`
    pub const NAMES: [&'static str; 11] = [
        "Letter", "Legal", "Tabloid", "Ledger", "A0", "A1", "A2", "A3", "A4", "A5", "A6",
    ];
}

/// Canonical values a null `PdfSettingsDto` field resolves to.
pub struct PdfSettingsDefaults;

impl PdfSettingsDefaults {
    pub const FORMAT: &'static str = "Letter";
    pub const ORIENTATION: &'static str = "portrait";
    pub const MARGIN_TOP: &'static str = "0.75in";
    pub const MARGIN_RIGHT: &'static str = "0.5in";
    pub const MARGIN_BOTTOM: &'static str = "0.75in";
    pub const MARGIN_LEFT: &'static str = "0.5in";
    pub const SHOW_PAGE_NUMBERS: bool = true;
    pub const SHOW_DATE: bool = true;
    pub const HEADER_FOOTER_LINES: &'static str = "both";
    pub const LINE_COLOR: &'static str = "#b3b3b3";
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PdfSettingsDto {
    pub format: Option<String>,
    pub orientation: Option<String>,
    pub margin_top: Option<String>,
    pub margin_right: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_left: Option<String>,
    pub show_page_numbers: Option<bool>,
    pub show_date: Option<bool>,
    pub document_title: Option<String>,
    pub date_time_format: Option<String>,
    pub header_footer_lines: Option<String>,
    pub line_color: Option<String>,
}

const ORIENTATIONS: [&str; 2] = ["portrait", "landscape"];
pub const HEADER_FOOTER_LINE_MODES: [&str; 4] = ["both", "header", "footer", "none"];

/// `HexColor`. .NET `$` also matches before a final "\n", written out here as `\n?\z`.
pub fn hex_color() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)^#[0-9a-f]{6}\n?\z").expect("valid pattern"))
}

/// A CSS length as the headless browser accepts it for a page margin (a bare number is rejected).
fn css_length() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)^\d*\.?\d+(?:cm|mm|in|pt|pc|px)\n?\z").expect("valid pattern"))
}

impl PdfSettingsDto {
    /// `KnownKeys.Contains(name)`.
    pub fn is_known_key(name: &str) -> bool {
        is_known_key(&PDF_SETTING_KEY_NAMES, name)
    }

    pub fn parse(json: &str) -> Result<Option<PdfSettingsDto>, JsonError> {
        match root_object(json)? {
            None => Ok(None),
            Some(obj) => Self::parse_value(&Value::Object(obj)).map(Some),
        }
    }

    /// Deserialises an already parsed JSON value (the C# code re-parses the element's raw text).
    pub fn parse_value(v: &Value) -> Result<PdfSettingsDto, JsonError> {
        let obj = match v {
            Value::Object(o) => o,
            _ => return Err(JsonError),
        };
        let mut d = PdfSettingsDto::default();
        for (k, v) in obj.iter() {
            match k.to_ascii_lowercase().as_str() {
                "format" => d.format = get_str(v)?,
                "orientation" => d.orientation = get_str(v)?,
                "margintop" => d.margin_top = get_str(v)?,
                "marginright" => d.margin_right = get_str(v)?,
                "marginbottom" => d.margin_bottom = get_str(v)?,
                "marginleft" => d.margin_left = get_str(v)?,
                "showpagenumbers" => d.show_page_numbers = get_bool(v)?,
                "showdate" => d.show_date = get_bool(v)?,
                "documenttitle" => d.document_title = get_str(v)?,
                "datetimeformat" => d.date_time_format = get_str(v)?,
                "headerfooterlines" => d.header_footer_lines = get_str(v)?,
                "linecolor" => d.line_color = get_str(v)?,
                _ => {}
            }
        }
        Ok(d)
    }

    pub fn validate(&self) -> Vec<DirectiveError<PdfSettingKey>> {
        let mut errors = Vec::new();
        if let Some(f) = &self.format {
            if !enum_try_parse(&PdfPaperFormat::NAMES, f) {
                errors.push(DirectiveError {
                    key: PdfSettingKey::Format,
                    message: format!("'format' must be one of: {}", PdfPaperFormat::NAMES.join(", ")),
                });
            }
        }
        check_one_of(
            &mut errors,
            PdfSettingKey::Orientation,
            "orientation",
            &self.orientation,
            &ORIENTATIONS,
        );
        check_margin(&mut errors, PdfSettingKey::MarginTop, "marginTop", &self.margin_top);
        check_margin(
            &mut errors,
            PdfSettingKey::MarginRight,
            "marginRight",
            &self.margin_right,
        );
        check_margin(
            &mut errors,
            PdfSettingKey::MarginBottom,
            "marginBottom",
            &self.margin_bottom,
        );
        check_margin(&mut errors, PdfSettingKey::MarginLeft, "marginLeft", &self.margin_left);
        check_one_of(
            &mut errors,
            PdfSettingKey::HeaderFooterLines,
            "headerFooterLines",
            &self.header_footer_lines,
            &HEADER_FOOTER_LINE_MODES,
        );
        if let Some(c) = &self.line_color {
            if !hex_color().is_match(c) {
                errors.push(DirectiveError {
                    key: PdfSettingKey::LineColor,
                    message: "'lineColor' must be a hex color, e.g. #b3b3b3".to_string(),
                });
            }
        }
        errors
    }
}

fn check_one_of(
    errors: &mut Vec<DirectiveError<PdfSettingKey>>,
    key: PdfSettingKey,
    name: &str,
    value: &Option<String>,
    allowed: &[&str],
) {
    let Some(v) = value else {
        return;
    };
    if !allowed.iter().any(|a| crate::text::eq_ignore_case(a, v)) {
        errors.push(DirectiveError {
            key,
            message: format!("'{}' must be one of: {}", name, allowed.join(", ")),
        });
    }
}

fn check_margin(
    errors: &mut Vec<DirectiveError<PdfSettingKey>>,
    key: PdfSettingKey,
    name: &str,
    value: &Option<String>,
) {
    let Some(v) = value else {
        return;
    };
    if !css_length().is_match(v) {
        errors.push(DirectiveError {
            key,
            message: format!("'{name}' must be a length with a unit, e.g. 2cm, 0.5in, or 12mm"),
        });
    }
}
