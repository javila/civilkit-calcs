// named-options
// private static string Render(string source, bool forPrint = false, bool enableUi = false, bool showHiddenOutput = false)
fn Render(source: &str, named: &[(&str, bool)]) -> String {
    let get = |k: &str| named.iter().find(|(n, _)| *n == k).map(|(_, v)| *v).unwrap_or(false);
    let mut parser = new_expression_parser(&[("ForPrint", get("forPrint")), ("EnableUi", get("enableUi"))]);
    parser.math_settings.borrow_mut().show_hidden_output = get("showHiddenOutput");
    parser.parse(source, true);
    parser.html_result
}
