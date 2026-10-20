// named-options
// private static string Render(string source, bool debug = false)
fn Render(source: &str, named: &[(&str, bool)]) -> String {
    let debug = named.iter().any(|(n, v)| *n == "debug" && *v);
    let mut parser = new_expression_parser(&[("Debug", debug)]);
    parser.parse(source, true);
    parser.html_result.replace("\r\n", "\n")
}
