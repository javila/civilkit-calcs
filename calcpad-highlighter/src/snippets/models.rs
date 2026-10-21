//! `Snippets/Models/SnippetItem.cs`.

use super::calcpad_type::CalcpadType;
use serde::Deserialize;

/// A snippet item passed to the frontend for autocomplete/insertion. For functions it also
/// carries signature information for type checking.
#[derive(Clone, Debug, PartialEq)]
pub struct SnippetItem {
    /// Text to insert; `§` is the cursor placeholder.
    pub insert: String,
    /// Linter classification: "Command", "Macro", "Variable", "Function", "Keyword", "Unit",
    /// "Constant", "Operator" or "Data". None or empty makes the snippet UI-only.
    pub keyword_type: Option<String>,
    /// Short label for tooltips, the insert tab and completion details.
    pub description: String,
    /// Long-form Markdown description; falls back to `description` when None.
    pub documentation: Option<String>,
    /// Optional Calcpad usage example.
    pub example: Option<String>,
    /// Description of the return value; falls back to the return type name when None.
    pub return_type_description: Option<String>,
    /// Optional label (defaults to the description when None).
    pub label: Option<String>,
    /// Category path using `/` as separator, e.g. "Functions/Trigonometric".
    pub category: String,
    /// When true the snippet is withheld from the frontend but still feeds the linter sets.
    pub internal_only: bool,
    /// Quick typing shortcut without the `~` prefix.
    pub quick_type: Option<String>,
    /// Parameters with expected types (functions only).
    pub parameters: Option<Vec<SnippetParameter>>,
    /// When true, parameter count validation is skipped entirely.
    pub accepts_any_count: bool,
    /// Return type of the function; None for non-functions.
    pub return_type: Option<CalcpadType>,
    /// Whether the function operates element-wise on vectors and matrices.
    pub is_element_wise: bool,
}

impl SnippetItem {
    /// Minimum number of required parameters; 0 when `accepts_any_count`.
    pub fn min_params(&self) -> i32 {
        if self.accepts_any_count {
            return 0;
        }
        self.parameters.as_ref().map_or(0, |ps| {
            ps.iter().filter(|p| !p.is_optional && !p.is_variadic).count() as i32
        })
    }

    /// Maximum number of parameters; -1 when variadic or `accepts_any_count` (unlimited).
    pub fn max_params(&self) -> i32 {
        let variadic = self
            .parameters
            .as_ref()
            .is_some_and(|ps| ps.iter().any(|p| p.is_variadic));
        if self.accepts_any_count || variadic {
            -1
        } else {
            self.parameters.as_ref().map_or(0, |ps| ps.len() as i32)
        }
    }

    /// Expected parameter type at `index`; for indices beyond the declared parameters the
    /// variadic parameter's type is used, otherwise [`ParameterType::Any`].
    pub fn get_parameter_type(&self, index: usize) -> ParameterType {
        let Some(ps) = self.parameters.as_ref().filter(|ps| !ps.is_empty()) else {
            return ParameterType::Any;
        };
        if let Some(p) = ps.get(index) {
            return p.param_type;
        }
        ps.iter()
            .find(|p| p.is_variadic)
            .map_or(ParameterType::Any, |p| p.param_type)
    }
}

/// A parameter in a function snippet.
#[derive(Clone, Debug, PartialEq)]
pub struct SnippetParameter {
    pub name: String,
    /// Type constraint for validation (`Type` in C#).
    pub param_type: ParameterType,
    pub type_description: Option<String>,
    pub description: Option<String>,
    pub is_optional: bool,
    /// Marks the repeating parameter whose type applies to all remaining arguments.
    pub is_variadic: bool,
}

/// A parameter type constraint for function validation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub enum ParameterType {
    /// Any type allowed.
    #[default]
    Any,
    /// Scalar value only.
    Scalar,
    /// Vector only.
    Vector,
    /// Matrix only.
    Matrix,
    /// Integer value (indices, counters).
    Integer,
    /// Boolean/condition expression.
    Boolean,
    /// Unevaluated expression, for special functions.
    Expression,
    /// Various types accepted; type checking is skipped for this parameter.
    Various,
}

impl ParameterType {
    /// The variant name, equal to C# `Enum.ToString()`.
    pub fn name(self) -> &'static str {
        match self {
            ParameterType::Any => "Any",
            ParameterType::Scalar => "Scalar",
            ParameterType::Vector => "Vector",
            ParameterType::Matrix => "Matrix",
            ParameterType::Integer => "Integer",
            ParameterType::Boolean => "Boolean",
            ParameterType::Expression => "Expression",
            ParameterType::Various => "Various",
        }
    }
}
