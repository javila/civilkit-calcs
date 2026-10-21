//! The endpoint layer (src/api.rs): wire shapes, UTF-16 columns, status answers and includes
//! from the in-memory file system.
use calcpad_highlighter::api::{self, MemoryFiles};
use serde_json::{json, Value};

fn call(f: impl Fn(&MemoryFiles, &str) -> String, files: &MemoryFiles, req: Value) -> Value {
    serde_json::from_str(&f(files, &req.to_string())).unwrap()
}

#[test]
fn highlight_columns_are_utf16_units() {
    let content = "'caf\u{e9} \u{1F600} text' + x";
    let v = call(
        api::highlight,
        &MemoryFiles::new(),
        json!({ "content": content, "includeText": true }),
    );
    let tokens = v["tokens"].as_array().unwrap();
    let comment = tokens.iter().find(|t| t["type"] == "Comment").unwrap();
    assert_eq!(
        comment["length"].as_i64().unwrap() as usize,
        content.encode_utf16().count() - 4
    );
    let x = tokens.iter().find(|t| t["text"] == "x").unwrap();
    assert_eq!(
        x["column"].as_i64().unwrap() as usize,
        content.encode_utf16().count() - 1
    );
    assert_eq!(x["typeId"], 5);
}

#[test]
fn highlight_omits_text_unless_asked() {
    let v = call(api::highlight, &MemoryFiles::new(), json!({ "content": "a = 1" }));
    assert!(v["tokens"][0]["text"].is_null());
}

#[test]
fn empty_content_is_a_400() {
    let v = call(api::lint, &MemoryFiles::new(), json!({ "content": "  " }));
    assert_eq!(v["status"], 400);
    assert_eq!(v["error"], "Content is required");
}

#[test]
fn lint_reports_diagnostics_in_the_server_shape() {
    let v = call(api::lint, &MemoryFiles::new(), json!({ "content": "r = zz + 1" }));
    assert_eq!(v["errorCount"], 1);
    let d = &v["diagnostics"][0];
    assert_eq!(d["code"], "CPD-3301");
    assert_eq!(d["severity"], "error");
    assert_eq!(d["severityId"], 0);
    assert_eq!(d["source"], "Calcpad Linter");
    assert_eq!(
        (d["line"].as_i64(), d["column"].as_i64(), d["endColumn"].as_i64()),
        (Some(0), Some(4), Some(6))
    );
}

#[test]
fn includes_resolve_from_the_in_memory_files() {
    let mut files = MemoryFiles::new();
    files.set("/lib/k.cpd", "k = 5\n");
    let req = json!({ "content": "#include k.cpd\nr = k*2", "sourceFilePath": "/lib/main.cpd" });
    let v = call(api::lint, &files, req.clone());
    assert_eq!(v["errorCount"], 0, "{v}");
    let without = call(api::lint, &MemoryFiles::new(), req);
    assert!(without["errorCount"].as_i64().unwrap() > 0);
}

#[test]
fn definitions_lists_every_kind() {
    let v = call(
        api::definitions,
        &MemoryFiles::new(),
        json!({ "content": "a = [1; 2]\nf(x; y = 1) = x*y\n#def m$ = 5" }),
    );
    assert_eq!(v["variables"][0]["name"], "a");
    assert_eq!(v["variables"][0]["type"], "Vector");
    assert_eq!(v["functions"][0]["name"], "f");
    assert_eq!(v["functions"][0]["parameters"], json!(["x", "y"]));
    assert_eq!(v["macros"][0]["name"], "m$");
    assert_eq!(v["macros"][0]["isMultiline"], false);
}

#[test]
fn symbol_at_position_and_null() {
    let files = MemoryFiles::new();
    let v = call(
        api::symbol_at_position,
        &files,
        json!({ "content": "a = 2\nb = a + a", "line": 1, "column": 4 }),
    );
    assert_eq!(v["symbolName"], "a");
    assert_eq!(v["kind"], "variable");
    assert_eq!(v["locations"].as_array().unwrap().len(), 3);
    let none = api::symbol_at_position(
        &files,
        &json!({ "content": "a = 2", "line": 0, "column": 2 }).to_string(),
    );
    assert_eq!(none, "null");
}

#[test]
fn prettify_indents_blocks() {
    let v: Value = serde_json::from_str(&api::prettify(
        &json!({ "content": "#if 1 > 0\nx = 1\n#end if" }).to_string(),
    ))
    .unwrap();
    assert_eq!(v["content"], "#if 1 > 0\n\tx = 1\n#end if");
}
