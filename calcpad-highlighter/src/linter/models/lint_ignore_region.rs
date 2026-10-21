//! LintIgnoreRegion.cs
use crate::text::StrSet;

/// A region of original source lines in which specific diagnostic codes are suppressed.
#[derive(Clone, Debug)]
pub struct LintIgnoreRegion {
    /// First suppressed original source line (0-based, inclusive).
    pub start_line: i32,
    /// Last suppressed original source line (0-based, inclusive); `i32::MAX` suppresses to end of file.
    pub end_line: i32,
    /// CPD codes to suppress (ignore case). An empty set suppresses all codes within the region.
    pub codes: StrSet,
}

impl LintIgnoreRegion {
    pub fn new(start_line: i32, end_line: i32, codes: StrSet) -> Self {
        LintIgnoreRegion {
            start_line,
            end_line,
            codes,
        }
    }
}
