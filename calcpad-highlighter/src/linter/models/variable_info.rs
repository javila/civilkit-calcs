//! VariableInfo.cs
use super::CalcpadType;

/// Holds type information and metadata about a Calcpad variable or definition.
#[derive(Clone, Debug)]
pub struct VariableInfo {
    /// The name of the variable/function/macro.
    pub name: String,
    /// The inferred type of the variable.
    pub type_: CalcpadType,
    /// Line number where defined (0-based).
    pub line_number: i32,
    /// Column where defined (0-based).
    pub column: i32,
    /// For functions/macros: parameter names.
    pub parameters: Vec<String>,
    /// The raw expression on the right side of the assignment.
    pub expression: String,
    /// For functions: the inferred return type based on the body expression.
    pub return_type: CalcpadType,
    /// For custom units: the unit name without the leading dot.
    pub unit_name: String,
    /// Whether this variable/function was defined with #const (readonly).
    pub is_const: bool,
    /// Source of the definition (local file or included file).
    pub source: String,
    /// User-provided description from a metadata comment on the preceding line.
    pub description: Option<String>,
    /// User-provided type hints per parameter.
    pub param_types: Option<Vec<String>>,
    /// User-provided descriptions per parameter.
    pub param_descriptions: Option<Vec<String>>,
}

impl Default for VariableInfo {
    fn default() -> Self {
        VariableInfo {
            name: String::new(),
            type_: CalcpadType::Unknown,
            line_number: 0,
            column: 0,
            parameters: Vec::new(),
            expression: String::new(),
            return_type: CalcpadType::Unknown,
            unit_name: String::new(),
            is_const: false,
            source: "local".to_string(),
            description: None,
            param_types: None,
            param_descriptions: None,
        }
    }
}

impl VariableInfo {
    /// `ParameterCount`
    pub fn parameter_count(&self) -> i32 {
        self.parameters.len() as i32
    }

    /// Whether this variable supports element access via .() syntax.
    pub fn supports_element_access(&self) -> bool {
        matches!(
            self.type_,
            CalcpadType::Vector | CalcpadType::Matrix | CalcpadType::Various | CalcpadType::Unknown
        )
    }

    /// Whether this is a $ suffixed identifier (macro or string variable).
    pub fn is_dollar_suffixed(&self) -> bool {
        self.name.ends_with('$')
    }
}

impl std::fmt::Display for VariableInfo {
    /// `ToString()`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.type_ {
            CalcpadType::Function => write!(f, "{}({})", self.name, self.parameters.join("; ")),
            CalcpadType::InlineMacro | CalcpadType::MultilineMacro => {
                if self.parameters.is_empty() {
                    write!(f, "{}", self.name)
                } else {
                    write!(f, "{}({})", self.name, self.parameters.join("; "))
                }
            }
            CalcpadType::CustomUnit => write!(f, ".{}", self.unit_name),
            _ => write!(f, "{}", self.name),
        }
    }
}
