//! Port of CalcpadError.cs.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalcpadErrorSource {
    Macro,
    Expression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalcpadError {
    pub source_line: i32,
    pub output_line: i32,
    pub message: String,
    pub source: CalcpadErrorSource,
}
