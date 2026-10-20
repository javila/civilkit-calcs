// private static string Expand(string source)
fn Expand(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (has_errors, expanded) = macro_parser.Parse(source);
    assert!(!has_errors, "{expanded}");
    expanded
}
