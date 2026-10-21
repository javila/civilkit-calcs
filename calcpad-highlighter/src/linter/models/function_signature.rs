//! FunctionSignature.cs
use super::CalcpadType;
use crate::snippets::ParameterType;

/// The signature of a built-in function including parameter types and return type.
#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub min_params: i32,
    /// Maximum number of parameters (-1 for unlimited/variadic).
    pub max_params: i32,
    /// Expected types for each parameter position (the last one applies to variadic functions).
    pub parameter_types: Vec<ParameterType>,
    pub return_type: CalcpadType,
    /// Whether the function can operate element-wise on vectors/matrices.
    pub is_element_wise: bool,
    /// When true, parameter count validation is skipped entirely.
    pub accepts_any_count: bool,
    pub description: String,
}

impl Default for FunctionSignature {
    fn default() -> Self {
        FunctionSignature {
            name: String::new(),
            min_params: 0,
            max_params: 0,
            parameter_types: Vec::new(),
            return_type: CalcpadType::Value,
            is_element_wise: false,
            accepts_any_count: false,
            description: String::new(),
        }
    }
}

impl FunctionSignature {
    /// The expected parameter type for a parameter index; the last defined type for variadic functions.
    pub fn get_parameter_type(&self, index: usize) -> ParameterType {
        if self.parameter_types.is_empty() {
            return ParameterType::Any;
        }
        if index < self.parameter_types.len() {
            return self.parameter_types[index];
        }
        self.parameter_types[self.parameter_types.len() - 1]
    }

    /// Whether an actual argument type is compatible with the expected parameter type.
    /// Element-wise functions also accept vectors and matrices where a scalar is expected.
    pub fn is_argument_compatible(&self, expected: ParameterType, actual: CalcpadType) -> bool {
        if self.is_element_wise
            && (actual == CalcpadType::Vector || actual == CalcpadType::Matrix)
            && matches!(
                expected,
                ParameterType::Scalar | ParameterType::Integer | ParameterType::Any
            )
        {
            return true;
        }
        Self::is_type_compatible(expected, actual)
    }

    pub fn is_type_compatible(expected: ParameterType, actual: CalcpadType) -> bool {
        if actual == CalcpadType::Unknown
            || actual == CalcpadType::Various
            || expected == ParameterType::Various
            || expected == ParameterType::Boolean
        {
            return true;
        }
        match expected {
            ParameterType::Any => true,
            ParameterType::Scalar => actual == CalcpadType::Value,
            ParameterType::Vector => actual == CalcpadType::Vector,
            ParameterType::Matrix => actual == CalcpadType::Matrix || actual == CalcpadType::Vector,
            ParameterType::Integer => actual == CalcpadType::Value,
            ParameterType::Expression => true,
            _ => true,
        }
    }

    pub fn get_type_name(t: ParameterType) -> &'static str {
        match t {
            ParameterType::Any => "any",
            ParameterType::Scalar => "scalar",
            ParameterType::Vector => "vector",
            ParameterType::Matrix => "matrix",
            ParameterType::Integer => "integer",
            ParameterType::Boolean => "boolean",
            ParameterType::Expression => "expression",
            ParameterType::Various => "various",
        }
    }
}
