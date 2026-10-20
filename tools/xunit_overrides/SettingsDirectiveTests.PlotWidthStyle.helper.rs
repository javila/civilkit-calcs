// Regex.Match(Render(source), @"width:(\d+)pt"): the first "width:" followed by digits and "pt".
fn PlotWidthStyle(source: &str) -> Option<String> {
    let html = render(source);
    let mut from = 0;
    while let Some(i) = html[from..].find("width:") {
        let rest = &html[from + i + 6..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() && rest[digits.len()..].starts_with("pt") {
            return Some(digits);
        }
        from += i + 6;
    }
    None
}
