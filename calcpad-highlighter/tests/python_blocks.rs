//! `#python` blocks through the endpoint layer: the editor must treat the block's lines as Python, not Calcpad.
use calcpad_highlighter::api::{self, MemoryFiles};
use serde_json::{json, Value};

fn call(f: impl Fn(&MemoryFiles, &str) -> String, req: Value) -> Value {
    serde_json::from_str(&f(&MemoryFiles::new(), &req.to_string())).unwrap()
}

const SHEET: &str = "a = 5\n#python\nimport numpy as np\nx = np.array([1, 2, 3])   # it's a comment\nprint(\"50% { }\")\nexport(M_max=1.5, V=2)\n#end python\nb = a + M_max + V\n";

#[test]
fn python_lines_are_not_linted_as_calcpad() {
    let v = call(api::lint, json!({ "content": SHEET }));
    let d = v["diagnostics"].as_array().unwrap();
    assert!(
        d.iter().all(|x| !(2..=6).contains(&x["line"].as_i64().unwrap())),
        "{d:?}"
    );
    assert!(d.iter().all(|x| x["severity"] != "error"), "{d:?}");
    assert_eq!(v["errorCount"], 0);
}

#[test]
fn an_exported_name_is_defined_for_the_lines_after_the_block() {
    let v = call(api::lint, json!({ "content": SHEET }));
    let d = v["diagnostics"].as_array().unwrap();
    assert!(
        d.iter()
            .all(|x| !x["message"].as_str().unwrap().contains("Undefined variable")),
        "{d:?}"
    );
    // without the export the name is undefined, as usual
    let w = call(
        api::lint,
        json!({ "content": "#python\nx = 1\n#end python\nb = M_max + 1" }),
    );
    assert!(w["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["message"].as_str().unwrap().contains("M_max")));
}

#[test]
fn only_the_delimiters_are_highlighted() {
    let v = call(api::highlight, json!({ "content": SHEET, "includeText": true }));
    let t = v["tokens"].as_array().unwrap();
    assert!(
        t.iter().all(|x| !(2..=5).contains(&x["line"].as_i64().unwrap())),
        "{t:?}"
    );
    let kw: Vec<&str> = t
        .iter()
        .filter(|x| x["type"] == "Keyword")
        .map(|x| x["text"].as_str().unwrap())
        .collect();
    assert_eq!(kw, ["#python", "#end python"]);
    assert!(t.iter().any(|x| x["text"] == "b"));
}

#[test]
fn an_open_block_is_an_error() {
    let v = call(api::lint, json!({ "content": "a = 1\n#python\nx = 1\n" }));
    let d = v["diagnostics"].as_array().unwrap();
    let e = d.iter().find(|x| x["code"] == "CPD-3801").unwrap();
    assert_eq!(e["line"], 1);
    assert_eq!(v["errorCount"], 1);
}

#[test]
fn prettify_leaves_the_block_alone() {
    let content = "#if 1 > 0\n    a = 1\n#end if\n#python\n  for i in range(3):\n\tprint(i)   \n\n\n#end python\n";
    let v = call(
        |_, r| api::prettify(r),
        json!({ "content": content, "indentUnit": "    " }),
    );
    let out = v["content"].as_str().unwrap();
    assert!(
        out.contains("#python\n  for i in range(3):\n\tprint(i)   \n\n\n#end python\n"),
        "{out:?}"
    );
}

#[test]
fn a_worksheet_without_a_block_is_unchanged_by_all_this() {
    let v = call(api::lint, json!({ "content": "a = 1\nb = a + z" }));
    assert!(v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["message"].as_str().unwrap().contains("'z'")));
}

#[test]
fn the_definitions_name_the_python_export() {
    let v = call(api::definitions, json!({ "content": SHEET }));
    let var = v["variables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["name"] == "M_max")
        .expect("M_max listed");
    assert_eq!(var["expression"], "exported from Python");
}

#[test]
fn hovering_python_text_resolves_nothing() {
    let v = call(
        api::symbol_at_position,
        json!({ "content": SHEET, "line": 3, "column": 0 }),
    );
    assert!(v.is_null());
}
