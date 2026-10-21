//! UnwrappedListingHighlightTests.cs: the unwrapped listing is highlighted by the real
//! tokenizer, emitting the canonical token names as span classes so the listing, the editor and
//! the exported HTML share one palette.
use calcpad_wasm::{convert, set_listing_tokenizer};
use serde_json::Value;

const DOCUMENT: &str = "#def half$(x$)\nx$/2\n#end def\n'A plain comment\n'<b>markup</b>\n'<!-- a note -->\n#settings {\"decimals\": 4}\n#UI {\"type\": \"entry\"} d = 10m\nL = half$(4)*1m\n$Plot{L @ d = 0 : 1}\n";

fn unwrapped() -> String {
    set_listing_tokenizer(Some(calcpad_highlighter::api::tokenize_listing));
    let req = serde_json::json!({ "content": DOCUMENT, "forceUnwrappedCode": true });
    let v: Value = serde_json::from_str(&convert(&req.to_string(), false)).unwrap();
    v["html"].as_str().unwrap().to_string()
}

#[test]
fn canonical_token_classes_are_emitted() {
    let html = unwrapped();
    for class in [
        "comment",
        "htmlComment",
        "tag",
        "htmlContent",
        "keyword",
        "command",
        "variable",
        "localVariable",
        "units",
        "const",
        "operator",
        "bracket",
        "settingsJson",
    ] {
        assert!(html.contains(&format!("class=\"{class}\"")), "{class}");
    }
}

#[test]
fn legacy_token_classes_are_gone() {
    let html = unwrapped();
    for class in ["number", "htmltag"] {
        assert!(!html.contains(&format!("class=\"{class}\"")), "{class}");
    }
}

#[test]
fn ui_json_block_is_one_span_and_the_assignment_follows() {
    let html = unwrapped();
    assert!(html.contains("<span class=\"settingsJson\">{&quot;type&quot;: &quot;entry&quot;}</span>"));
    assert!(html.contains("<span class=\"variable\">d</span>"));
}

#[test]
fn macro_calls_are_already_expanded() {
    let html = unwrapped();
    assert!(!html.contains("half$"));
    assert!(html.contains("<span class=\"const\">4</span><span class=\"operator\">/</span>"));
}

#[test]
fn without_a_tokenizer_the_listing_is_plain_and_says_so() {
    set_listing_tokenizer(None);
    let req = serde_json::json!({ "content": "a = 1", "forceUnwrappedCode": true });
    let v: Value = serde_json::from_str(&convert(&req.to_string(), false)).unwrap();
    assert_eq!(v["needsHighlighter"], true);
    assert!(!v["html"].as_str().unwrap().contains("class=\"variable\""));
}
