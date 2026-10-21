//! WebAssembly bindings of calcpad-highlighter. Each endpoint of Calcpad.Server that fronts
//! Calcpad.Highlighter is a function from the request JSON to the response JSON; where the
//! server would answer an HTTP error the answer is `{"status": n, "error": "..."}`.
use calcpad_highlighter::api::{self, MemoryFiles};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static FILES: RefCell<MemoryFiles> = RefCell::new(MemoryFiles::new());
}

/// The highlighter version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Makes a text file visible to `#include`.
#[wasm_bindgen]
pub fn set_file(path: &str, content: &str) {
    FILES.with(|f| f.borrow_mut().set(path, content));
}

#[wasm_bindgen]
pub fn clear_files() {
    FILES.with(|f| f.borrow_mut().clear());
}

/// `POST /api/calcpad/highlight`
#[wasm_bindgen]
pub fn highlight(request_json: &str) -> String {
    FILES.with(|f| api::highlight(&f.borrow(), request_json))
}

/// `POST /api/calcpad/lint`
#[wasm_bindgen]
pub fn lint(request_json: &str) -> String {
    FILES.with(|f| api::lint(&f.borrow(), request_json))
}

/// `POST /api/calcpad/definitions`
#[wasm_bindgen]
pub fn definitions(request_json: &str) -> String {
    FILES.with(|f| api::definitions(&f.borrow(), request_json))
}

/// `POST /api/calcpad/symbol-at-position`
#[wasm_bindgen]
pub fn symbol_at_position(request_json: &str) -> String {
    FILES.with(|f| api::symbol_at_position(&f.borrow(), request_json))
}

/// `POST /api/calcpad/prettify`
#[wasm_bindgen]
pub fn prettify(request_json: &str) -> String {
    api::prettify(request_json)
}

/// `GET /api/calcpad/snippets[?category=...]`; an empty category means all.
#[wasm_bindgen]
pub fn snippets(category: &str) -> String {
    calcpad_highlighter::snippets::snippets_json(if category.trim().is_empty() {
        None
    } else {
        Some(category)
    })
}

/// The tokens of an unwrapped code listing (see `calcpad_highlighter::api::tokenize_listing`).
/// The engine calls this through `globalThis.calcpadTokenizeListing`.
#[wasm_bindgen]
pub fn tokenize_listing(text: &str) -> String {
    api::tokenize_listing(text)
}
