// private static int CountOccurrences(string text, string value)
fn CountOccurrences(text: impl AsRef<str>, value: &str) -> f64 {
    text.as_ref().matches(value).count() as f64
}
