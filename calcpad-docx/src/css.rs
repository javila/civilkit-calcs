//! Ports of CssParser.cs and CssColors.cs: CSS sizes and colours as Word wants them.

pub const ERR_FG: &str = "DC143C";
pub const OK_FG: &str = "008000";
pub const ERR_BG: &str = "FFEEEE";
pub const OK_BG: &str = "F0FFF0";

/// `CssColor.NameToHex`: the CSS named colours (143).
const NAMED: [(&str, &str); 143] = [
    ("aliceblue", "F0F8FF"),
    ("antiquewhite", "FAEBD7"),
    ("aqua", "00FFFF"),
    ("aquamarine", "7FFFD4"),
    ("azure", "F0FFFF"),
    ("beige", "F5F5DC"),
    ("bisque", "FFE4C4"),
    ("black", "000000"),
    ("blanchedalmond", "FFEBCD"),
    ("blue", "0000FF"),
    ("blueviolet", "8A2BE2"),
    ("brown", "A52A2A"),
    ("burlywood", "DEB887"),
    ("cadetblue", "5F9EA0"),
    ("chartreuse", "7FFF00"),
    ("chocolate", "D2691E"),
    ("coral", "FF7F50"),
    ("cornflowerblue", "6495ED"),
    ("cornsilk", "FFF8DC"),
    ("crimson", "DC143C"),
    ("cyan", "00FFFF"),
    ("darkblue", "00008B"),
    ("darkcyan", "008B8B"),
    ("darkgoldenrod", "B8860B"),
    ("darkgray", "A9A9A9"),
    ("darkgreen", "006400"),
    ("darkkhaki", "BDB76B"),
    ("darkmagenta", "8B008B"),
    ("darkolivegreen", "556B2F"),
    ("darkorange", "FF8C00"),
    ("darkorchid", "9932CC"),
    ("darkred", "8B0000"),
    ("darksalmon", "E9967A"),
    ("darkseagreen", "8FBC8F"),
    ("darkslateblue", "483D8B"),
    ("darkslategray", "2F4F4F"),
    ("darkturquoise", "00CED1"),
    ("darkviolet", "9400D3"),
    ("deeppink", "FF1493"),
    ("deepskyblue", "00BFFF"),
    ("dimgray", "696969"),
    ("dimgrey", "696969"),
    ("dodgerblue", "1E90FF"),
    ("firebrick", "B22222"),
    ("floralwhite", "FFFAF0"),
    ("forestgreen", "228B22"),
    ("fuchsia", "FF00FF"),
    ("gainsboro", "DCDCDC"),
    ("ghostwhite", "F8F8FF"),
    ("gold", "FFD700"),
    ("goldenrod", "DAA520"),
    ("gray", "808080"),
    ("grey", "808080"),
    ("green", "008000"),
    ("greenyellow", "ADFF2F"),
    ("honeydew", "F0FFF0"),
    ("hotpink", "FF69B4"),
    ("indianred", "CD5C5C"),
    ("indigo", "4B0082"),
    ("ivory", "FFFFF0"),
    ("khaki", "F0E68C"),
    ("lavender", "E6E6FA"),
    ("lavenderblush", "FFF0F5"),
    ("lawngreen", "7CFC00"),
    ("lemonchiffon", "FFFACD"),
    ("lightblue", "ADD8E6"),
    ("lightcoral", "F08080"),
    ("lightcyan", "E0FFFF"),
    ("lightgoldenrodyellow", "FAFAD2"),
    ("lightgray", "D3D3D3"),
    ("lightgreen", "90EE90"),
    ("lightpink", "FFB6C1"),
    ("lightsalmon", "FFA07A"),
    ("lightseagreen", "20B2AA"),
    ("lightskyblue", "87CEFA"),
    ("lightslategray", "778899"),
    ("lightsteelblue", "B0C4DE"),
    ("lightyellow", "FFFFE0"),
    ("lime", "00FF00"),
    ("limegreen", "32CD32"),
    ("linen", "FAF0E6"),
    ("magenta", "FF00FF"),
    ("maroon", "800000"),
    ("mediumaquamarine", "66CDAA"),
    ("mediumblue", "0000CD"),
    ("mediumorchid", "BA55D3"),
    ("mediumpurple", "9370DB"),
    ("mediumseagreen", "3CB371"),
    ("mediumslateblue", "7B68EE"),
    ("mediumspringgreen", "00FA9A"),
    ("mediumturquoise", "48D1CC"),
    ("mediumvioletred", "C71585"),
    ("midnightblue", "191970"),
    ("mintcream", "F5FFFA"),
    ("mistyrose", "FFE4E1"),
    ("moccasin", "FFE4B5"),
    ("navajowhite", "FFDEAD"),
    ("navy", "000080"),
    ("oldlace", "FDF5E6"),
    ("olive", "808000"),
    ("olivedrab", "6B8E23"),
    ("orange", "FFA500"),
    ("orangered", "FF4500"),
    ("orchid", "DA70D6"),
    ("palegoldenrod", "EEE8AA"),
    ("palegreen", "98FB98"),
    ("paleturquoise", "AFEEEE"),
    ("palevioletred", "DB7093"),
    ("papayawhip", "FFEFD5"),
    ("peachpuff", "FFDAB9"),
    ("peru", "CD853F"),
    ("pink", "FFC0CB"),
    ("plum", "DDA0DD"),
    ("powderblue", "B0E0E6"),
    ("purple", "800080"),
    ("rebeccapurple", "663399"),
    ("red", "FF0000"),
    ("rosybrown", "BC8F8F"),
    ("royalblue", "4169E1"),
    ("saddlebrown", "8B4513"),
    ("salmon", "FA8072"),
    ("sandybrown", "F4A460"),
    ("seagreen", "2E8B57"),
    ("seashell", "FFF5EE"),
    ("sienna", "A0522D"),
    ("silver", "C0C0C0"),
    ("skyblue", "87CEEB"),
    ("slateblue", "6A5ACD"),
    ("slategray", "708090"),
    ("snow", "FFFAFA"),
    ("springgreen", "00FF7F"),
    ("steelblue", "4682B4"),
    ("tan", "D2B48C"),
    ("teal", "008080"),
    ("thistle", "D8BFD8"),
    ("tomato", "FF6347"),
    ("turquoise", "40E0D0"),
    ("violet", "EE82EE"),
    ("wheat", "F5DEB3"),
    ("white", "FFFFFF"),
    ("whitesmoke", "F5F5F5"),
    ("yellow", "FFFF00"),
    ("yellowgreen", "9ACD32"),
];

pub fn name_to_hex(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    NAMED.iter().find(|(k, _)| *k == n).map(|(_, v)| *v)
}

/// `CssColor.RgbToHex`: "rgb(r, g, b)" as "RRGGBB".
pub fn rgb_to_hex(s: &str) -> Option<String> {
    let inner = s.get(4..s.len().checked_sub(1)?)?;
    let parts: Vec<u8> = inner
        .split(',')
        .map(|p| p.trim().parse::<u8>())
        .collect::<Result<_, _>>()
        .ok()?;
    if parts.len() != 3 {
        return None;
    }
    Some(format!("{:02X}{:02X}{:02X}", parts[0], parts[1], parts[2]))
}

/// `CssToOpenXmlColor`
pub fn css_to_openxml_color(color: &str) -> Option<String> {
    if let Some(h) = color.strip_prefix('#') {
        return Some(h.to_string());
    }
    if color.starts_with("rgb") {
        return rgb_to_hex(color);
    }
    name_to_hex(color).map(|s| s.to_string())
}

/// `CssParser.GetCSSParameter`: the value of `parameter` in a style attribute.
pub fn css_parameter(css: &str, parameter: &str) -> Option<String> {
    let i = css.to_ascii_lowercase().find(&parameter.to_ascii_lowercase())?;
    let colon = css[i..].find(':')? + i + 1;
    let end = css[colon..].find(';').map(|e| e + colon).unwrap_or(css.len());
    Some(css[colon..end].trim().to_string())
}

#[derive(Clone, Copy)]
enum CssUnit {
    Px = 0,
    Pt = 1,
    Pc = 2,
    Mm = 3,
    Cm = 4,
    In = 5,
}

const SCALE: [[f64; 6]; 6] = [
    [1.0, 72.0 / 96.0, 1.0 / 16.0, 127.0 / 480.0, 127.0 / 4800.0, 1.0 / 96.0],
    [96.0 / 72.0, 1.0, 1.0 / 12.0, 127.0 / 360.0, 127.0 / 3600.0, 1.0 / 72.0],
    [16.0, 12.0, 1.0, 127.0 / 30.0, 127.0 / 300.0, 1.0 / 6.0],
    [480.0 / 127.0, 360.0 / 127.0, 30.0 / 127.0, 1.0, 1.0 / 10.0, 5.0 / 127.0],
    [4800.0 / 127.0, 3600.0 / 127.0, 300.0 / 127.0, 10.0, 1.0, 50.0 / 127.0],
    [96.0, 72.0, 6.0, 25.4, 2.54, 1.0],
];

fn css_unit(u: &str) -> Option<CssUnit> {
    Some(match u.to_ascii_lowercase().as_str() {
        "" | "px" => CssUnit::Px,
        "pt" => CssUnit::Pt,
        // EM and REM are read as picas, as upstream does.
        "pc" | "em" | "rem" => CssUnit::Pc,
        "mm" => CssUnit::Mm,
        "cm" => CssUnit::Cm,
        "in" => CssUnit::In,
        _ => return None,
    })
}

/// `CssParser.ParseSize(s)` in pixels; 0 for a size it cannot read.
pub fn parse_size_px(s: &str) -> f64 {
    let s = s.trim();
    let chars: Vec<char> = s.chars().collect();
    let mut n = chars.len().saturating_sub(1);
    let mut i = n;
    while i > 0 {
        let c = chars[i];
        if c.is_ascii_digit() || c == '.' {
            n = i + 1;
            break;
        }
        i -= 1;
    }
    let value: String = chars[..n.min(chars.len())].iter().collect();
    let units: String = chars[n.min(chars.len())..].iter().collect();
    let (Some(u), Ok(d)) = (css_unit(&units), value.trim().parse::<f64>()) else {
        return 0.0;
    };
    d * SCALE[u as usize][CssUnit::Px as usize]
}

/// `CssParser.GetSizeCSSParameter(css, parameter)` in pixels.
pub fn css_size_px(css: &str, parameter: &str) -> f64 {
    match css_parameter(css, parameter) {
        Some(s) if !s.is_empty() => parse_size_px(&s),
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_colours() {
        assert_eq!(parse_size_px("190pt"), 190.0 * (96.0 / 72.0));
        assert_eq!(parse_size_px("120"), 120.0);
        assert_eq!(parse_size_px("12in"), 1152.0);
        // As upstream: a single digit before the unit is not found ("2in" reads as 0).
        assert_eq!(parse_size_px("2in"), 0.0);
        assert_eq!(
            css_size_px("width:190pt; height: 30cm", "height"),
            30.0 * (4800.0 / 127.0)
        );
        assert_eq!(css_parameter("float: right;", "float").as_deref(), Some("right"));
        assert_eq!(css_to_openxml_color("rgb(255, 0, 16)").as_deref(), Some("FF0010"));
        assert_eq!(css_to_openxml_color("SteelBlue").as_deref(), Some("4682B4"));
        assert_eq!(css_to_openxml_color("#abc").as_deref(), Some("abc"));
    }
}
