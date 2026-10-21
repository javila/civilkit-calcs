/// Type of a value in the linter's type tracker (`Linter/Models/CalcpadType.cs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CalcpadType {
    Unknown = 0,
    Value,
    Vector,
    Matrix,
    CustomUnit,
    Function,
    InlineMacro,
    MultilineMacro,
    Various,
}

impl CalcpadType {
    /// The variant name, equal to C# `Enum.ToString()`.
    pub fn name(self) -> &'static str {
        match self {
            CalcpadType::Unknown => "Unknown",
            CalcpadType::Value => "Value",
            CalcpadType::Vector => "Vector",
            CalcpadType::Matrix => "Matrix",
            CalcpadType::CustomUnit => "CustomUnit",
            CalcpadType::Function => "Function",
            CalcpadType::InlineMacro => "InlineMacro",
            CalcpadType::MultilineMacro => "MultilineMacro",
            CalcpadType::Various => "Various",
        }
    }

    /// Parses a variant name (case-sensitive, as `Enum.Parse` without `ignoreCase`).
    pub fn from_name(name: &str) -> Option<CalcpadType> {
        Some(match name {
            "Unknown" => CalcpadType::Unknown,
            "Value" => CalcpadType::Value,
            "Vector" => CalcpadType::Vector,
            "Matrix" => CalcpadType::Matrix,
            "CustomUnit" => CalcpadType::CustomUnit,
            "Function" => CalcpadType::Function,
            "InlineMacro" => CalcpadType::InlineMacro,
            "MultilineMacro" => CalcpadType::MultilineMacro,
            "Various" => CalcpadType::Various,
            _ => return None,
        })
    }
}
