//! The Word export for the browser, as its own module: resvg and its font code would roughly
//! double the engine, so the page loads this only when a document is exported. The engine
//! renders the worksheet with `captureOpenXml`; its HTML and equations come here.
use wasm_bindgen::prelude::*;

/// Adds a font file (TTF/OTF) for the text of SVG drawings.
#[wasm_bindgen]
pub fn add_font(data: &[u8]) {
    calcpad_docx::fonts::add_font(data.to_vec());
}

/// The .docx bytes for a rendered page and the JSON array of its equations.
#[wasm_bindgen]
pub fn docx(html: &str, expressions_json: &str) -> Vec<u8> {
    let expressions: Vec<String> = serde_json::from_str(expressions_json).unwrap_or_default();
    calcpad_docx::html_to_docx(html, &expressions, None)
}

/// The .xlsx bytes for a rendered page and the JSON of the variable sheets the engine made
/// (`xlsxSheets`): the page as a "Report" sheet, then those sheets.
#[wasm_bindgen]
pub fn xlsx(html: &str, sheets_json: &str) -> Result<Vec<u8>, JsError> {
    let value: serde_json::Value = serde_json::from_str(sheets_json).map_err(|e| JsError::new(&e.to_string()))?;
    let sheets = calcpad_core::openxml::variables_export::sheets_from_json(&value);
    calcpad_docx::html_to_xlsx(html, &sheets, None).map_err(|e| JsError::new(&e))
}
