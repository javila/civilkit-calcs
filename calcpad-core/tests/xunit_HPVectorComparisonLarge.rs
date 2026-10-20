//! Transcribed from Calcpad.Tests/Vectors/HP/HPVectorComparisonLarge.cs by tools/transcribe_xunit.py.
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

#[track_caller]
fn OperatorTestHelper(o: char, tol: &str) -> Vec<String> {
    vec![
        ("n = 1000").to_string(),
        ("a = random(fill(vector(n); 1))").to_string(),
        ("b = random(fill(vector(n); 1))").to_string(),
        (&format!("c = a {} b", o)).to_string(),
        ("a_hp = hp(a)").to_string(),
        ("b_hp = hp(b)").to_string(),
        (&format!("c_hp = a_hp {} b_hp", o)).to_string(),
        (&format!("r = if({} ≡ 0; c_hp ≡ c; abs(c_hp - c) ≤ {})", tol, tol)).to_string(),
        ("check = count(r; 0; 1) ≡ 0").to_string(),
    ]
}
#[track_caller]
fn FunctionTestHelper(f: &str) -> Vec<String> {
    vec![
        ("n = 1000").to_string(),
        ("a = random(fill(vector(n); 1))").to_string(),
        (&format!("c = {}(a)", f)).to_string(),
        (&format!("c_hp = {}(hp(a))", f)).to_string(),
        ("r = c ≡ c_hp").to_string(),
        ("check = count(r; 0; 1) ≡ 0").to_string(),
    ]
}
#[track_caller]
fn ScalarTestHelper(f: &str) -> Vec<String> {
    vec![
        ("n = 1000").to_string(),
        ("a = random(fill(vector(n); 1))").to_string(),
        (&format!("c = {}(a)", f)).to_string(),
        (&format!("c_hp = {}(hp(a))", f)).to_string(),
        (&TestCalc::CompareWithTolerance("c", "c_hp", "10^-14")).to_string(),
    ]
}

#[test]
fn HPVectorAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('+', "0"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('-', "0"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('*', "0"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('/', "0"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('÷', "0"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('\\', "0"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('⦼', "0"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sin"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cos"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorTan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("tan"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("csc"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sec"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cot"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sinh"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cosh"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorTanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("tanh"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("csch"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sech"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("coth"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAsin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asin"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAcos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acos"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAtan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atan"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAcsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsc"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAsec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asec"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAcot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acot"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAsinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asinh"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAcosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acosh"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAtanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atanh"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAcsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsch"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAsech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asech"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAcoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acoth"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorLog() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorLn() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ln"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorLog2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log_2"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorExp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("exp"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSqrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sqrt"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCbrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cbrt"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorRoot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 1000",
        "nth = 100",
        "a = random(fill(vector(n); 1))",
        "c = root(a; nth)",
        "c_hp = root(hp(a); nth)",
        "r = c ≡ c_hp",
        "check = count(r; 0; 1) ≡ 0",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorRound() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("round"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorFloor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("floor"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorCeiling() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ceiling"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorTrunc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("trunc"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorMin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("min"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorMax() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("max"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSum() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sum"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSumsq() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sumsq"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSrss() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("srss"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorAverage() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("average"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorProduct() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("product"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorMean() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mean"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorTake() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("take"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorLine() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("Line"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSpline() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("spline"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorSort() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sort"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorRsort() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("rsort"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorOrder() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("order"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorRevorder() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("revorder"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorReverse() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("reverse"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorNorm1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("norm_1"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorNorm2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("norm_2"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorNormP() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 1000",
        "a = random(fill(vector(n); 1))",
        "c = norm_p(a; 5)",
        "c_hp = norm_p(hp(a); 5)",
        "c ≡ c_hp",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorNormI() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("norm_i"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorUnit() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 1000",
        "a = random(fill(vector(n); 1))",
        "c = unit(a)",
        "c_hp = unit(hp(a))",
        "r = abs(c - c_hp) < 10^-12*abs(c)",
        "check = count(r; 0; 1) ≡ 0",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPVectorDot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 1000",
        "a = random(fill(vector(n); 1))",
        "b = random(fill(vector(n); 1))",
        "c = dot(a; b)",
        "c_hp = dot(hp(a); hp(b))",
        "abs(c - c_hp) < 10^-12*abs(c)",
    ]);
    assert_equal(1.0, result.clone());
}
