// private static int Count(string html, string value) => Regex.Count(html, Regex.Escape(value));
fn Count(html: impl AsRef<str>, value: &str) -> f64 {
    let html = html.as_ref();
    html.matches(value).count() as f64
}
