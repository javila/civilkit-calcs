// Regex width:(\d+)pt over the rendered plot; plots are out of scope for phase 1.
fn PlotWidthStyle(source: &str) -> Option<String> {
    let html = render(source);
    let i = html.find("width:")?;
    let rest = &html[i + 6..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() && rest[digits.len()..].starts_with("pt") {
        Some(digits)
    } else {
        None
    }
}
