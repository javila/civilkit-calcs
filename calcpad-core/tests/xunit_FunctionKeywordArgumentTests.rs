//! Transcribed from Calcpad.Tests/MathParser/FunctionKeywordArgumentTests.cs by tools/transcribe_xunit.py.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::identity_op,
    clippy::needless_borrow,
    clippy::redundant_clone,
    clippy::bool_assert_comparison,
    clippy::useless_format,
    clippy::clone_on_copy,
    unused_imports,
    dead_code,
    suspicious_double_ref_op,
    noop_method_call,
    clippy::needless_borrows_for_generic_args
)]
mod common;
use common::*;

// private static TestCalc NewCalc() => new(new());
fn NewCalc() -> TestCalc {
    TestCalc::new(ms())
}

#[test]
fn OmittedOptionalArguments_UseDefaults() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2; z = 3) = x + y + z");
    assert_equal(6.0, calc.Run("f(1)"));
}
#[test]
fn PositionalArgumentsOverrideDefaults() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2; z = 3) = x + y + z");
    assert_equal(111.0, calc.Run("f(1; 10; 100)"));
}
#[test]
fn KeywordArguments_CanBeGivenInAnyOrder() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2; z = 3) = x + y + z");
    assert_equal(111.0, calc.Run("f(z = 100; x = 1; y = 10)"));
}
#[test]
fn PositionalFollowedByKeyword_IsAllowed() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2; z = 3) = x + y + z");
    assert_equal(103.0, calc.Run("f(1; z = 100)"));
}
#[test]
fn VectorDefault_WithSemicolons() {
    let mut calc = NewCalc();
    calc.Run("f(x; v = [1; 2; 3]) = x + sum(v)");
    assert_equal(7.0, calc.Run("f(1)"));
    assert_equal(31.0, calc.Run("f(1; v = [10; 20])"));
}
#[test]
fn KeywordArgumentsWithoutSpaces_AreRecognized() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_equal(11.0, calc.Run("f(x=1;y=10)"));
}
#[test]
fn DefaultsAreEvaluatedAtCallTime() {
    let mut calc = NewCalc();
    calc.Run("a = 5");
    calc.Run("f(x; y = a) = x + y");
    assert_equal(6.0, calc.Run("f(1)"));
    calc.Run("a = 50");
    assert_equal(51.0, calc.Run("f(1)"));
}
#[test]
fn DefaultExpression_CanContainFunctionCalls() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = sqr(16)) = x + y");
    assert_equal(5.0, calc.Run("f(1)"));
}
#[test]
fn DefaultCanBeAVector() {
    let mut calc = NewCalc();
    calc.Run("f(x; v = [1; 2; 3]) = x*len(v)");
    assert_equal(6.0, calc.Run("f(2)"));
}
#[test]
fn FunctionWithoutDefaults_IsUnaffected() {
    let mut calc = NewCalc();
    calc.Run("f(x; y) = x + y");
    assert_equal(3.0, calc.Run("f(1; 2)"));
}
#[test]
fn NestedCalls_ResolveDefaults() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_equal(7.0, calc.Run("f(f(1); 4)"));
}
#[test]
fn NestedCallInsideAResolvedArgumentList_IsAlsoResolved() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2; z = 3) = x + y + z");
    assert_equal(13.0, calc.Run("f(f(1); 4)"));
    assert_equal(6.0, calc.Run("f(x = f(1; z = 0); y = 0)"));
}
#[test]
fn DefaultCallingAnotherFunctionWithDefaults() {
    let mut calc = NewCalc();
    calc.Run("g(a; b = 4) = a + b");
    calc.Run("f(x; y = g(1)) = x + y");
    assert_equal(6.0, calc.Run("f(1)"));
}
#[test]
fn MissingRequiredArgument_Throws() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_throws(|| {
        calc.Run("f()");
    });
}
#[test]
fn DuplicateArgument_Throws() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_throws(|| {
        calc.Run("f(1; x = 3)");
    });
}
#[test]
fn RequiredParameterAfterOptional_Throws() {
    let mut calc = NewCalc();
    assert_throws(|| {
        calc.Run("f(x = 1; y) = x + y");
    });
}
#[test]
fn PositionalAfterKeyword_Throws() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_throws(|| {
        calc.Run("f(x = 1; 2)");
    });
}
#[test]
fn IndentedDefinition_IsNotTreatedAsACall() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    calc.Run("   f(x; y = 20) = x + y");
    assert_equal(21.0, calc.Run("f(1)"));
}
#[test]
fn KeywordArgument_WithUnits() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2kg) = x + y");
    assert_equal(3.0, calc.Run("f(1kg)"));
    assert_equal(6.0, calc.Run("f(1kg; y = 5kg)"));
}
#[test]
fn Redefinition_KeepsWorking() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_equal(3.0, calc.Run("f(1)"));
    calc.Run("f(x; y = 20) = x + y");
    assert_equal(21.0, calc.Run("f(1)"));
}
#[test]
fn Default_IsNotCapturedByCallerParameter() {
    let mut calc = NewCalc();
    calc.Run("a = 1");
    calc.Run("f(x; y = a) = x + y");
    calc.Run("g(a) = f(0)");
    assert_equal(1.0, calc.Run("g(50)"));
}
#[test]
fn Default_CanReferenceEarlierParameter() {
    let mut calc = NewCalc();
    calc.Run("f(b; h = b) = b*h");
    assert_equal(9.0, calc.Run("f(3)"));
    assert_equal(6.0, calc.Run("f(3; h = 2)"));
}
#[test]
fn Default_CanReferenceVariableDefinedLater() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = b) = x + y");
    calc.Run("b = 7");
    assert_equal(8.0, calc.Run("f(1)"));
}
#[test]
fn GlobalChangeInDefault_ClearsCallerCache() {
    let mut calc = NewCalc();
    calc.Run("a = 1");
    calc.Run("f(x; y = a) = x + y");
    calc.Run("g(x) = f(x)");
    assert_equal(2.0, calc.Run("g(1)"));
    calc.Run("a = 100");
    assert_equal(101.0, calc.Run("g(1)"));
}
#[test]
fn KeywordArguments_WorkWithoutDefaults() {
    let mut calc = NewCalc();
    calc.Run("f(x; y) = x - y");
    assert_equal(4.0, calc.Run("f(y = 1; x = 5)"));
}
#[test]
fn UnknownKeywordArgument_Throws() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_throws(|| {
        calc.Run("f(1; z = 3)");
    });
}
#[test]
fn RecursiveDefault_Throws() {
    let mut calc = NewCalc();
    calc.Run("g(x) = x");
    calc.Run("f(x; y = g(x)) = x + y");
    assert_throws(|| {
        calc.Run("g(x) = f(x)");
    });
}
#[test]
fn DefaultsInsideSolver() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x - y");
    assert_equal_precision(2.0, calc.Run("$Root{f(x) = 0 @ x = 0 : 10}"), 6);
}
#[test]
fn EmptyTrailingArgument_Throws() {
    let mut calc = NewCalc();
    calc.Run("f(x; y = 2) = x + y");
    assert_throws(|| {
        calc.Run("f(1;)");
    });
}
