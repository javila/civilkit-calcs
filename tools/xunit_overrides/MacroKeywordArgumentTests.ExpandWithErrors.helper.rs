// private static string ExpandWithErrors(string source)
fn ExpandWithErrors(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (_, expanded) = macro_parser.Parse(source);
    expanded
}
