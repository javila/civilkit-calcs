//! Port of Output/TextWriter.cs.
use super::*;

pub struct TextWriter {
    base: WriterBase,
}

impl TextWriter {
    pub const UNIT_DIVISION: char = '\u{2215}';
    pub const UNIT_PRODUCT: char = '\u{00B7}';

    pub fn new(settings: Option<&MathSettings>, phasor: bool) -> TextWriter {
        TextWriter {
            base: WriterBase::new(settings, phasor),
        }
    }

    pub fn with_decimals(decimals: i32) -> TextWriter {
        let mut base = WriterBase::new(None, false);
        base.decimals = decimals;
        TextWriter { base }
    }
}

impl OutputWriter for TextWriter {
    fn base(&self) -> &WriterBase {
        &self.base
    }

    fn format_type(&self) -> OutputFormat {
        OutputFormat::Text
    }

    fn angle_units(&self) -> [&'static str; 3] {
        ["\u{00B0}", "", " grad"]
    }

    fn unit_string(&self, units: &UnitRef) -> String {
        units.text().replace('\u{2031}', "\u{2031} ")
    }

    fn format_input(
        &self,
        s: &str,
        units: &Option<UnitRef>,
        _line: i32,
        _is_calculated: bool,
    ) -> String {
        match units {
            None => s.to_string(),
            Some(u) => format!("{} {}", s, u.text()),
        }
    }

    fn format_subscript(&self, sa: &str, sb: &str) -> String {
        format!("{}_{}", sa, sb)
    }

    fn append_subscript(&self, sa: &str, sb: &str) -> String {
        format!("{}.{}", sa, sb)
    }

    fn format_variable(&self, name: &str, _value: &str, _is_bold: bool) -> String {
        if name.starts_with('\u{20D7}') {
            name['\u{20D7}'.len_utf8()..].to_string()
        } else {
            name.to_string()
        }
    }

    fn format_units(&self, s: &str) -> String {
        s.to_string()
    }

    fn format_function(&self, s: &str) -> String {
        s.to_string()
    }

    fn format_switch(&self, sa: &[String], _level: i32) -> String {
        format!("switch({})", sa.join("; "))
    }

    fn format_if(&self, sc: &str, sa: &str, sb: &str, _level: i32) -> String {
        format!("if({}; {}; {})", sc, sa, sb)
    }

    fn format_root(&self, s: &str, _level: i32, n: &str) -> String {
        match n {
            "2" => format!("\u{221A}({})", s),
            "3" => format!("\u{00B3}\u{221A}({})", s),
            "4" => format!("\u{2074}\u{221A}({})", s),
            _ => format!("root({}; {})", s, n),
        }
    }

    fn format_operator(&self, c: char) -> String {
        format_operator_helper(c)
    }

    fn format_power(&self, sa: &str, sb: &str, _level: i32, _order: i32) -> String {
        format!("{}^{}", sa, sb)
    }

    fn format_division(&self, sa: &str, sb: &str, _level: i32) -> String {
        format!("{}/{}", sa, sb)
    }

    fn format_nary(&self, symbol: &str, sub: &str, sup: &str, expr: &str) -> String {
        format!("{}{{{}; {}...{}}}", symbol, expr, sub, sup)
    }

    fn format_value(&self, value: &Value) -> String {
        let u = value.units();
        let fs = u.as_ref().and_then(|x| x.format_string());
        let mut s = self.format_complex(value.re(), value.im(), fs.as_deref());
        let u_text = u.as_ref().map(|x| x.text()).unwrap_or_default();
        if u_text.is_empty() {
            return s;
        }
        if !(value.is_real() || self.base.phasor && s.contains('\u{2220}')) {
            s = self.add_brackets(&s, 0, '(', ')');
        }
        if u_text == "\u{00B0}" {
            return format!("{}{}", s, u_text);
        }
        format!("{} {}", s, u_text)
    }

    fn add_brackets(&self, s: &str, _level: i32, left: char, right: char) -> String {
        format!("{}{}{}", left, s, right)
    }

    fn format_abs(&self, s: &str, _level: i32) -> String {
        format!("|{}|", s)
    }

    fn format_real(&self, d: f64, format: Option<&str>, zero_small: bool) -> String {
        let s = self.format_number_helper(d, format);
        if d.is_nan() || d.is_infinite() {
            return s;
        }
        let i = match s.rfind('E') {
            Some(i) if i > 0 => i,
            _ => return s,
        };
        if zero_small && s[i + 1..].starts_with('-') {
            return "0".to_string();
        }
        s
    }

    fn format_complex(&self, re: f64, im: f64, format: Option<&str>) -> String {
        if re.is_nan() && im.is_nan() {
            return " Undefined ".to_string();
        }
        if re.is_infinite() && im.is_infinite() {
            return "\u{221E}".to_string();
        }
        self.format_complex_helper(re, im, format)
    }

    fn format_block(&self, sa: &[String]) -> String {
        format!(" \u{2502} {}", sa.join(" \u{2502} "))
    }

    fn close_curly_brackets(&self, sa: &str, _level: i32) -> String {
        sa.to_string()
    }
}
