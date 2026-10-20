// Mirrors the real pipeline: macro expansion, then expression parsing.
fn Render(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (has_errors, unwrapped) = macro_parser.Parse(source);
    assert!(!has_errors, "{unwrapped}");
    let mut parser = new_expression_parser(&[]);
    parser.parse(&unwrapped, true);
    parser.html_result
}
