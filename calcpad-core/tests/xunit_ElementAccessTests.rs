#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/MathParser/ElementAccessTests.cs by tools/transcribe_xunit.py.
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

#[test]
fn VectorElement() {
    let mut calc = TestCalc::new(ms());
    assert_equal(20.0, calc.RunAll(&["v = [10; 20; 30]", "v.2"]));
}
#[test]
fn VectorElementInExpression() {
    let mut calc = TestCalc::new(ms());
    assert_equal(40.0, calc.RunAll(&["v = [10; 20; 30]", "v.1 + v.3"]));
}
#[test]
fn VectorElementWithComputedIndex() {
    let mut calc = TestCalc::new(ms());
    assert_equal(30.0, calc.RunAll(&["v = [10; 20; 30]", "i = 1", "v.(i + 2)"]));
}
#[test]
fn VectorElementAssignment() {
    let mut calc = TestCalc::new(ms());
    assert_equal(99.0, calc.RunAll(&["v = [10; 20; 30]", "v.2 = 99", "v.2"]));
}
#[test]
fn MatrixElement() {
    let mut calc = TestCalc::new(ms());
    assert_equal(4.0, calc.RunAll(&["m = [1; 2 | 3; 4]", "m.(2; 2)"]));
}
#[test]
fn ElementAccessOnUndeclaredName() {
    let mut calc = TestCalc::new(ms());
    assert_equal(20.0, calc.RunAll(&["f(x) = x.2", "f([10; 20; 30])"]));
}
#[test]
fn DotIsNotANameChar() {
    let mut calc = TestCalc::new(ms());
    assert_throws_any(|| {
        calc.RunAll(&["a = 5", "a.b = 2"]);
    });
}
#[test]
fn DecimalLiteralIsUnaffected() {
    let mut calc = TestCalc::new(ms());
    assert_equal(3.5, calc.Run("1.25 + 2.25"));
}
#[test]
fn CustomUnitIsUnaffected() {
    let mut calc = TestCalc::new(ms());
    assert_equal(4096.0, calc.RunAll(&[".bit = 1", ".byte = 8*bit", "2*256*byte/bit"]));
}
#[test]
fn ValidatorRejectsDottedNames_case1() {
    let name = "a1";
    let expected = true;
    assert_equal(expected.clone(), calcpad_core::validator::is_variable(name));
}
#[test]
fn ValidatorRejectsDottedNames_case2() {
    let name = "a_1";
    let expected = true;
    assert_equal(expected.clone(), calcpad_core::validator::is_variable(name));
}
#[test]
fn ValidatorRejectsDottedNames_case3() {
    let name = "a,b";
    let expected = true;
    assert_equal(expected.clone(), calcpad_core::validator::is_variable(name));
}
#[test]
fn ValidatorRejectsDottedNames_case4() {
    let name = "α₁";
    let expected = true;
    assert_equal(expected.clone(), calcpad_core::validator::is_variable(name));
}
#[test]
fn ValidatorRejectsDottedNames_case5() {
    let name = "a.b";
    let expected = false;
    assert_equal(expected.clone(), calcpad_core::validator::is_variable(name));
}
#[test]
fn ValidatorRejectsDottedNames_case6() {
    let name = "v.1";
    let expected = false;
    assert_equal(expected.clone(), calcpad_core::validator::is_variable(name));
}
