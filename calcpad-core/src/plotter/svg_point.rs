//! Port of Plotter/SvgPoint.cs.
use crate::netfmt;

#[derive(Clone, Copy, Debug, Default)]
pub struct SvgPoint {
    pub x: f64,
    pub y: f64,
}

impl SvgPoint {
    /// `new SvgPoint(x, y)`: both coordinates are rounded to 2 decimals.
    pub fn new(x: f64, y: f64) -> SvgPoint {
        SvgPoint {
            x: netfmt::round_even_digits(x, 2),
            y: netfmt::round_even_digits(y, 2),
        }
    }
}

impl std::fmt::Display for SvgPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{}",
            netfmt::double_to_string(self.x),
            netfmt::double_to_string(self.y)
        )
    }
}
