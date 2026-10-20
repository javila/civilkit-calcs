//! Error constructors, generated from Calcpad.Core/Exceptions.cs by tools/gen_exceptions.py.
//! Function names keep the C# names so call sites read the same as upstream.
#![allow(non_snake_case, dead_code)]

use crate::math_parser_exception::MathParserException;
use crate::messages;
use crate::netfmt::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Items {
    Index,
    Count,
    NumRows,
    NumCols,
    Argument,
    Result,
    Variable,
    IndexTarget,
    Limit,
    Value,
}

fn item_to_string(item: Items) -> &'static str {
    match item {
        Items::Index => messages::Index,
        Items::Count => messages::Count,
        Items::NumRows => messages::Number_of_rows,
        Items::NumCols => messages::Number_of_columns,
        Items::Argument => messages::Argument,
        Items::Result => messages::Result,
        Items::Variable => messages::Variable,
        Items::IndexTarget => messages::Index_target,
        Items::Limit => messages::Limit,
        Items::Value => messages::Value,
    }
}

pub fn IncompleteExpression() -> MathParserException {
    MathParserException::new(messages::Incomplete_expression.to_string())
}

pub fn MissingLeftBracket() -> MathParserException {
    MathParserException::new(messages::Missing_left_bracket.to_string())
}

pub fn MissingRightBracket() -> MathParserException {
    MathParserException::new(messages::Missing_right_bracket.to_string())
}

pub fn InvalidSymbol(c: char) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_symbol_0, &[&c.to_string()]))
}

pub fn InvalidUnits(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_units_0, &[s]))
}

pub fn InvalidLiteral(s: &str, literal: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Cannot_evaluate_0_as_1, &[s, literal]))
}

pub fn InvalidOperator(c: char) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_operator_0, &[&c.to_string()]))
}

pub fn PowerNotUnitless() -> MathParserException {
    MathParserException::new(messages::Power_must_be_unitless.to_string())
}

pub fn ResultIsNotUnits() -> MathParserException {
    MathParserException::new(
        messages::The_expression_on_the_right_does_not_evaluate_to_units.to_string(),
    )
}

pub fn CannotEvaluateFunction(s: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Cannot_evaluate_the_function_f_for_v_equals_0,
        &[s],
    ))
}

pub fn FunctionNotDefined(s: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::The_function_f_is_not_defined_for_v_equals_0,
        &[s],
    ))
}

pub fn InconsistentUnits(u1: &str, u2: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Inconsistent_units_0_and_1, &[u1, u2]))
}

pub fn IterationLimits(l1: &str, l2: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Limits_out_of_range_0_1, &[l1, l2]))
}

pub fn InvalidUnitsFunction(function: &str, unit: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Invalid_units_for_function_0_1,
        &[function, unit],
    ))
}

pub fn RootUnitless() -> MathParserException {
    MathParserException::new(messages::Root_index_must_be_unitless.to_string())
}

pub fn RootComplex() -> MathParserException {
    MathParserException::new(messages::Root_index_cannot_be_a_complex_number.to_string())
}

pub fn RootInteger() -> MathParserException {
    MathParserException::new(messages::Root_index_must_be_integer_more_than_1.to_string())
}

pub fn FactorialArgumentOutOfRange() -> MathParserException {
    MathParserException::new(messages::Argument_out_of_range_for_n_factorial.to_string())
}

pub fn FactorialArgumentUnitless() -> MathParserException {
    MathParserException::new(messages::The_argument_of_n_factorial_must_be_unitless.to_string())
}

pub fn FactorialArgumentPositiveInteger() -> MathParserException {
    MathParserException::new(
        messages::The_argument_of_n_factorial_must_be_a_positive_integer.to_string(),
    )
}

pub fn FactorialArgumentComplex() -> MathParserException {
    MathParserException::new(messages::The_argument_of_n_factorial_cannot_be_complex.to_string())
}

pub fn RemainderUnits(u1: &str, u2: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Cannot_evaluate_remainder_0_1_Denominator_must_be_unitless,
        &[u1, u2],
    ))
}

pub fn BothValuesInteger() -> MathParserException {
    MathParserException::new(messages::Both_values_must_be_integers.to_string())
}

pub fn VariableNotExist(name: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Variable_0_does_not_exist, &[name]))
}

pub fn UnitNotExist(name: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Unit_0_does_not_exist, &[name]))
}

pub fn InteruptedByUser() -> MathParserException {
    MathParserException::new(messages::Interrupted_by_user.to_string())
}

pub fn CalculationsNotActive() -> MathParserException {
    MathParserException::new(messages::Calculations_not_active.to_string())
}

pub fn ExpressionEmpty() -> MathParserException {
    MathParserException::new(messages::Expression_is_empty.to_string())
}

pub fn MissingOperand() -> MathParserException {
    MathParserException::new(messages::Missing_operand.to_string())
}

pub fn InvalidFunction(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_function_0, &[s]))
}

pub fn InvalidVariableName(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_variable_name_0, &[s]))
}

pub fn CannotEvaluateAsType(literal: &str, type_: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Cannot_evaluate_0_as_1, &[literal, type_]))
}

pub fn StackLeak() -> MathParserException {
    MathParserException::new(messages::Stack_memory_leak_Invalid_expression.to_string())
}

pub fn StackEmpty() -> MathParserException {
    MathParserException::new(messages::Stack_empty_Invalid_expression.to_string())
}

pub fn UndefinedInputField() -> MathParserException {
    MathParserException::new(messages::Undefined_input_field.to_string())
}

pub fn UndefinedVariableOrUnits(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Undefined_variable_or_units_0, &[s]))
}

pub fn ErrorEvaluatingAsFunction(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Error_evaluating_0_as_function, &[s]))
}

pub fn ErrorEvaluatingAsFunctionOrOperator(s: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Error_evaluating_0_as_function_or_operator,
        &[s],
    ))
}

pub fn CannotRewriteUnits(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Cannot_rewrite_existing_units_0, &[s]))
}

pub fn InconsistentTargetUnits(sourceUnits: &str, targetUnits: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::The_calculated_units_0_are_inconsistent_with_the_target_units_1,
        &[sourceUnits, targetUnits],
    ))
}

pub fn InvalidCharacter(c: char) -> MathParserException {
    MathParserException::new(fmt(
        messages::Invalid_character_exception,
        &[&c.to_string()],
    ))
}

pub fn ImproperAssignment() -> MathParserException {
    MathParserException::new(messages::Improper_use_of_the_assignment_operator_equals.to_string())
}

pub fn MissingLeftSolverBracket() -> MathParserException {
    MathParserException::new(messages::Missing_left_bracket_in_solver_command.to_string())
}

pub fn MissingRightSolverBracket() -> MathParserException {
    MathParserException::new(messages::Missing_right_bracket_in_solver_command.to_string())
}

pub fn InvalidMacro(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_macro_identifier_0, &[s]))
}

pub fn InvalidSolver(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_solver_command_definition_0, &[s]))
}

pub fn ErrorParsingUnits(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Error_parsing_0_as_units, &[s]))
}

pub fn ErrorParsingNumber(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Error_parsing_0_as_number, &[s]))
}

pub fn MissingDelimiter(delimiter: char, script: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Missing_delimiter_0_in_solver_command_1,
        &[&delimiter.to_string(), script],
    ))
}

pub fn MultipleAssignments(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::More_than_one_operator_equals_in_0, &[s]))
}

pub fn NotConstantExpression(s: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::The_expression_on_the_right_side_must_be_constant_0,
        &[s],
    ))
}

pub fn InconsistentUnits1(variable: &str, units: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Inconsistent_units_for_0_equals_1,
        &[variable, units],
    ))
}

pub fn InconsistentUnits2(variable: &str, u1: &str, u2: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Inconsistent_units_for_0_equals_1_2,
        &[variable, u1, u2],
    ))
}

pub fn NoSolution(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::No_solution_for_0, &[s]))
}

pub fn RecursionNotAllowed(s: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Recursion_is_not_allowed_in_function_definition_0,
        &[s],
    ))
}

pub fn AssignmentPreceded() -> MathParserException {
    MathParserException::new(
        messages::The_assignment_equals_must_be_preceded_by_custom_function_or_variable.to_string(),
    )
}

pub fn AssignmentNotFirst() -> MathParserException {
    MathParserException::new(
        messages::Assignment_equals_must_be_the_first_operator_in_the_expression.to_string(),
    )
}

pub fn InvalidSyntax(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_syntax_0, &[s]))
}

pub fn InvalidSyntax2(s1: &str, s2: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_syntax_0_1, &[s1, s2]))
}

pub fn UnexpectedDelimiter() -> MathParserException {
    MathParserException::new(messages::Unexpected_delimiter.to_string())
}

pub fn InvalidNumberOfArguments() -> MathParserException {
    MathParserException::new(messages::Invalid_number_of_arguments.to_string())
}

pub fn ResultNotReal(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::The_result_is_not_a_real_number_0, &[s]))
}

pub fn MissingFunctionParameter() -> MathParserException {
    MathParserException::new(messages::Missing_parameter_in_function_definition.to_string())
}

pub fn MissingFunctionDelimiter() -> MathParserException {
    MathParserException::new(messages::Missing_delimiter_in_function_definition.to_string())
}

pub fn InvalidFunctionToken(name: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Invalid_token_in_function_definition_0,
        &[name],
    ))
}

pub fn CircularReference(name: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Circular_reference_detected_for_function_0,
        &[name],
    ))
}

pub fn CircularMacroReference(name: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Circular_reference_detected_for_macro_0,
        &[name],
    ))
}

pub fn InvalidFunctionDefinition() -> MathParserException {
    MathParserException::new(messages::Invalid_function_definition_exception.to_string())
}

pub fn ArgumentOutOfRange(function: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Argument_out_of_range_for_0_x, &[function]))
}

pub fn ConditionEmpty() -> MathParserException {
    MathParserException::new(messages::Condition_cannot_be_empty.to_string())
}

pub fn ConditionNotInitialized() -> MathParserException {
    MathParserException::new(messages::Condition_block_not_initialized_with_if.to_string())
}

pub fn DuplicateElse() -> MathParserException {
    MathParserException::new(messages::Duplicate_else_in_condition_block.to_string())
}

pub fn ElseIfAfterElse() -> MathParserException {
    MathParserException::new(
        messages::else_if_is_not_allowed_after_else_in_condition_block.to_string(),
    )
}

pub fn ConditionComplex() -> MathParserException {
    MathParserException::new(messages::Condition_cannot_evaluate_to_a_complex_number.to_string())
}

pub fn ConditionResultInvalid(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Condition_result_is_invalid_0, &[s]))
}

pub fn DuplicateMacroParameters(s: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Duplicate_macro_parameter_names_0_and_1,
        &[s, s],
    ))
}

pub fn UndefinedMacro(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Macro_not_defined_0, &[s]))
}

pub fn RequiredParameterAfterOptional(name: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Required_parameter_after_optional_0, &[name]))
}

pub fn UnknownKeywordArgument(name: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Unknown_keyword_argument_0, &[name]))
}

pub fn DuplicateArgument(name: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Duplicate_argument_0, &[name]))
}

pub fn MissingMapItem(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Missing_0_in_surface_map_command, &[s]))
}

pub fn PlotLimitsIdentical() -> MathParserException {
    MathParserException::new(messages::The_limits_of_plot_area_are_identical.to_string())
}

pub fn ErrorWritingPngFile(path: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Error_writing_a_png_file_to_0, &[path]))
}

pub fn ErrorWritingSvgFile(path: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Error_writing_a_svg_file_to_0, &[path]))
}

pub fn ErrorConvertingPngToBase64() -> MathParserException {
    MathParserException::new(messages::Error_converting_png_to_Base64.to_string())
}

pub fn InconsistentUnitsOperation(ua: &str, op: char, ub: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Inconsistent_units_0_1_2,
        &[ua, &op.to_string(), ub],
    ))
}

pub fn UnitsToComplexPower() -> MathParserException {
    MathParserException::new(messages::Units_cannon_be_raised_to_complex_power.to_string())
}

pub fn CannotEvaluateRemainder(ua: &str, ub: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Cannot_evaluate_remainder_0_1_The_denominator_must_be_unitless,
        &[ua, ub],
    ))
}

pub fn MissingVectorOpeningBracket() -> MathParserException {
    MathParserException::new(messages::Missing_vector_opening_bracket.to_string())
}

pub fn BracketMismatch() -> MathParserException {
    MathParserException::new(messages::Bracket_mismatch.to_string())
}

pub fn IndexOutOfRange(index: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Index_out_of_range_0, &[index]))
}

pub fn CannotAssignVectorToScalar() -> MathParserException {
    MathParserException::new(messages::Cannot_assign_vector_to_scalar.to_string())
}

pub fn MissingVectorClosingBracket() -> MathParserException {
    MathParserException::new(messages::Missing_vector_closing_bracket.to_string())
}

pub fn MustBeScalar(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::_must_be_scalar
    ))
}

pub fn MustBeReal(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::_must_be_real
    ))
}

pub fn MustBePositiveInteger(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::_must_be_positive_integer
    ))
}

pub fn MustBeMatrix(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::_must_be_matrix
    ))
}

pub fn MustBeVector(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::_must_be_vector
    ))
}

pub fn StepCannotBeZero() -> MathParserException {
    MathParserException::new(messages::Step_cannot_be_zero.to_string())
}

pub fn VectorSizeLimit() -> MathParserException {
    MathParserException::new(fmt(messages::Vector_size_cannot_exceed_0, &["10000000"]))
}

pub fn CrossProductVectorDimensions() -> MathParserException {
    MathParserException::new(
        messages::Cross_product_is_definedonly_for_vectors_with_2_and_3_elements.to_string(),
    )
}

pub fn FunctionOnlyRealMode(name: &str) -> MathParserException {
    MathParserException::new(fmt(
        messages::Function_0_is_not_defined_in_complex_mode,
        &[name],
    ))
}

pub fn MatrixDimensions() -> MathParserException {
    MathParserException::new(messages::Matrix_dimensions_do_not_match.to_string())
}

pub fn MatrixNotSquare() -> MathParserException {
    MathParserException::new(messages::Matrix_must_be_square.to_string())
}

pub fn InvalidLpNormArgument() -> MathParserException {
    MathParserException::new(messages::The_Lp_norm_argument_must_be_p_ge_1.to_string())
}

pub fn MatrixMustBeSymmetric() -> MathParserException {
    MathParserException::new(messages::Matrix_must_be_symmetric.to_string())
}

pub fn MatrixNotPositiveDefinite() -> MathParserException {
    MathParserException::new(messages::Matrix_is_not_positive_definite.to_string())
}

pub fn MatrixSingular() -> MathParserException {
    MathParserException::new(messages::Matrix_is_singular.to_string())
}

pub fn MatrixCloseToSingular() -> MathParserException {
    MathParserException::new(messages::Matrix_is_singular_or_close_to_singular.to_string())
}

pub fn ComplexVectorsAndMatricesNotSupported() -> MathParserException {
    MathParserException::new(
        messages::Vectors_and_matrices_are_not_supported_in_complex_mode.to_string(),
    )
}

pub fn CannotInterpolateWithNonScalarValue() -> MathParserException {
    MathParserException::new(messages::Cannot_interpolate_with_non_scalar_value.to_string())
}

pub fn JacobiFailed() -> MathParserException {
    MathParserException::new(messages::Jacobi_iteration_failed.to_string())
}

pub fn InvalidOperand(value: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_operand_0, &[value]))
}

pub fn InvalidArgument(value: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_argument_0, &[value]))
}

pub fn ArgumentNotEigenvalue(value: &str) -> MathParserException {
    MathParserException::new(fmt(messages::The_argument_is_not_an_eigenvalue_0, &[value]))
}

pub fn QLAlgorithmFailed() -> MathParserException {
    MathParserException::new(messages::The_QL_algorithm_failed_to_converge.to_string())
}

pub fn MatirixSizeLimit() -> MathParserException {
    MathParserException::new(fmt(messages::Matrix_size_cannot_exceed_0, &["1000000"]))
}

pub fn EmbeddedDataSizeLimit() -> MathParserException {
    MathParserException::new(fmt(messages::Embedded_data_cannot_exceed_0_MB, &["10"]))
}

pub fn DatagridSizeLimit() -> MathParserException {
    MathParserException::new(fmt(
        messages::Datagrid_size_cannot_exceed_0_cells,
        &["100000"],
    ))
}

pub fn MatrixNotHigh() -> MathParserException {
    MathParserException::new(
        messages::The_number_of_matrix_rows_must_be_greater_than_or_equal_to_the_number_of_columns
            .to_string(),
    )
}

pub fn FunctionDefinitionInSolver() -> MathParserException {
    MathParserException::new(messages::Function_definition_not_allowed_in_solver_block.to_string())
}

pub fn InvalidFormatString(s: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_format_string_0, &[s]))
}

pub fn VariableNotMatrix(name: &str) -> MathParserException {
    MathParserException::new(fmt(messages::The_variable_is_not_a_matrix_0, &[name]))
}

pub fn FileNotFound(path: &str) -> MathParserException {
    MathParserException::new(fmt(messages::File_not_found_0, &[path]))
}

pub fn MissingFileName() -> MathParserException {
    MathParserException::new(messages::Missing_file_name.to_string())
}

pub fn InvalidType(type_: char) -> MathParserException {
    MathParserException::new(fmt(messages::Invalid_type_0, &[&type_.to_string()]))
}

pub fn PathNotFound(path: &str) -> MathParserException {
    MathParserException::new(fmt(messages::Path_not_found_0, &[path]))
}

pub fn FileFormatNotSupported(ext: &str) -> MathParserException {
    MathParserException::new(fmt(messages::ThisFileFormatIsNotSupported0, &[ext]))
}

pub fn MustBeHpMatrix(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::MustBeHpMatrix
    ))
}

pub fn MustBeHpVector(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::MustBeHpVector
    ))
}

pub fn MatrixOneOrTwoRows() -> MathParserException {
    MathParserException::new(
        messages::TheInputMatrixForFFTMustHaveOneRowForRealAndTwoRowsForComplexData.to_string(),
    )
}

pub fn MissingMatrixIndex(matrix: &str) -> MathParserException {
    MathParserException::new(fmt(messages::MissingMatrixIndexIn0, &[matrix]))
}

pub fn CounterMustBeASingleVariableName() -> MathParserException {
    MathParserException::new(messages::CounterMustBeASingleVariableName.to_string())
}

pub fn InvalidNumberOfIndexes() -> MathParserException {
    MathParserException::new(messages::InvalidNumberOfIndexes.to_string())
}

pub fn InfiniteLoop(maxCount: i32) -> MathParserException {
    MathParserException::new(fmt(
        messages::PossiblyInfiniteLoopDetected,
        &[&maxCount.to_string()],
    ))
}

pub fn CannotModifyConstant(name: &str) -> MathParserException {
    MathParserException::new(fmt(messages::CannotModifyConstant, &[name]))
}

pub fn InvalidNumber(s: &str) -> MathParserException {
    InvalidLiteral(s, "number")
}
