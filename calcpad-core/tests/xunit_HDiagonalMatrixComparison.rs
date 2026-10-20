#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Large/HPDiagonalMatrixComparison.cs by tools/transcribe_xunit.py.
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

const RandomMatrixA: &str = "a = random(diagonal(n; 1))";
const RandomMatrixB: &str = "b = random(diagonal(n; 1))";
const WellConditionedMatrix: &str = "a = vec2diag((0.55 + range(0; n - 1; 1))/n)";

#[track_caller]
fn OperatorTestHelper(o: char, tol: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (RandomMatrixB).to_string(),
        (&format!("c = a {} b", o)).to_string(),
        ("a_hp = hp(a)").to_string(),
        ("b_hp = hp(b)").to_string(),
        (&format!("c_hp = a_hp {} b_hp", o)).to_string(),
        (&format!("r = if({} ≡ 0; c_hp ≡ c; abs(c_hp - c) ≤ {})", tol, tol)).to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn FunctionTestHelper(func: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (&format!("c = {}(a)", func)).to_string(),
        (&format!("c_hp = {}(hp(a))", func)).to_string(),
        ("r = c ≡ c_hp").to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn ScalarTestHelper(func: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (&format!("c = {}(a)", func)).to_string(),
        (&format!("c_hp = {}(hp(a))", func)).to_string(),
        (&TestCalc::CompareWithTolerance("c", "c_hp", "10^-14")).to_string(),
    ]
}
#[track_caller]
fn InterpolationTestHelper(func: &str) -> Vec<String> {
    vec![
        ("m = 250").to_string(),
        ("n = 500").to_string(),
        ("i = random(m - 1) + 1").to_string(),
        ("j = random(n - 1) + 1").to_string(),
        (RandomMatrixA).to_string(),
        (&format!("c = {}(i; j; a)", func)).to_string(),
        (&format!("c_hp = {}(i; j; hp(a))", func)).to_string(),
        (&TestCalc::CompareWithTolerance("c", "c_hp", "10^-14")).to_string(),
    ]
}
#[track_caller]
fn PositiveDefiniteTestHelper(func: &str, tol: &str) -> Vec<String> {
    vec![
        ("n = 250").to_string(),
        (WellConditionedMatrix).to_string(),
        (&format!("c = {}(a)", func)).to_string(),
        (&format!("c_hp = {}(hp(a))", func)).to_string(),
        (&TestCalc::CompareWithTolerance("c", "c_hp", tol)).to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn MatrixEquationTestHelper(func: &str, tol: &str) -> Vec<String> {
    vec![
        ("n = 250").to_string(),
        (WellConditionedMatrix).to_string(),
        ("b = random(fill(vector(n); 1))").to_string(),
        (&format!("c = {}(a; b)", func)).to_string(),
        (&format!("c_hp = {}(hp(a); hp(b))", func)).to_string(),
        (&TestCalc::CompareWithTolerance("c", "c_hp", tol)).to_string(),
        ("count(r; 0; 1)").to_string(),
    ]
}
#[track_caller]
fn MatrixMultiEquationTestHelper(func: &str, tol: &str) -> Vec<String> {
    vec![
        ("n = 250").to_string(),
        (WellConditionedMatrix).to_string(),
        ("b = random(mfill(matrix(n; 2); 1))").to_string(),
        (&format!("c = {}(a; b)", func)).to_string(),
        (&format!("c_hp = {}(hp(a); hp(b))", func)).to_string(),
        (&TestCalc::CompareWithTolerance("c", "c_hp", tol)).to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}

#[test]
fn HPDiagonalMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('*', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixTan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("tan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("csc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixTanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Tanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Csch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("coth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAsin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAcos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAtan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAtan2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        RandomMatrixB,
        "c = atan2(a; b)",
        "c_hp = atan2(hp(a); hp(b))",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAcsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAsec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAcot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAsinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAcosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAtanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAcsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAsech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAcoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acoth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixLog() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixLn() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ln"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixLog2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log_2"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixExp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("exp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSqrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sqrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCbrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cbrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixRoot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "nth = 4",
        RandomMatrixA,
        "c = root(a; nth)",
        "c_hp = root(hp(a); nth)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixRound() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("round"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixFloor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("floor"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCeiling() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ceiling"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixTrunc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("trunc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("min"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMax() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("max"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSum() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sum"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSumsq() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sumsq"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSrss() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("srss"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAverage() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("average"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixProduct() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("product"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMean() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mean"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixTake() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("take"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixLine() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("line"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSpline() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("spline"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSortCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "i = 50",
        RandomMatrixA,
        "c = sort_cols(a; i)",
        "c_hp = sort_cols(hp(a); i)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixRSortCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "i = 50",
        RandomMatrixA,
        "c = rsort_cols(a; i)",
        "c_hp = rsort_cols(hp(a); i)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSortRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "j = 50",
        RandomMatrixA,
        "c = sort_rows(a; j)",
        "c_hp = sort_rows(hp(a); j)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixRSortRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "j = 50",
        RandomMatrixA,
        "c = rsort_rows(a; j)",
        "c_hp = rsort_rows(hp(a); j)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixOrderCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "i = 50",
        RandomMatrixA,
        "c = order_cols(a; i)",
        "c_hp = order_cols(hp(a); i)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixRevOrderCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "i = 50",
        RandomMatrixA,
        "c = revorder_cols(a; i)",
        "c_hp = revorder_cols(hp(a); i)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixOrderRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "j = 50",
        RandomMatrixA,
        "c = order_rows(a; j)",
        "c_hp = order_rows(hp(a); j)",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMcount() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "x = 1",
        "a = round(random(diagonal(n; 1000)))",
        "c = mcount(a; x)",
        "c_hp = mcount(hp(a); x)",
        "c ≡ c_hp",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixHprod() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        RandomMatrixB,
        "c = hprod(a; b)",
        "c_hp = hprod(hp(a); hp(b))",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixFprod() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        RandomMatrixB,
        "c = fprod(a; b)",
        "c_hp = fprod(hp(a); hp(b))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixKprod() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 100",
        RandomMatrixA,
        RandomMatrixB,
        "c = kprod(a; b)",
        "c_hp = kprod(hp(a); hp(b))",
        "r = c ≡ c_hp",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMnorm1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mnorm_1"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMnorm2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        RandomMatrixA,
        "c = mnorm_2(a)",
        "c_hp = mnorm_2(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMnormi() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        "c = mnorm_i(a)",
        "c_hp = mnorm_i(hp(a))",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCond1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = cond_1(a)",
        "c_hp = cond_1(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCond2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        WellConditionedMatrix,
        "c = cond_2(a)",
        "c_hp = cond_2(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCondE() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = cond_e(a)",
        "c_hp = cond_e(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCondI() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = cond_i(a)",
        "c_hp = cond_i(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixDet() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        RandomMatrixA,
        "c = det(a)",
        "c_hp = det(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixRank() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 100",
        RandomMatrixA,
        "c = rank(a)",
        "c_hp = rank(hp(a))",
        "r = c ≡ c_hp",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixTrace() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        "c = trace(a)",
        "c_hp = trace(hp(a))",
        "r = abs(c - c_hp) ≤ 10^-12",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPDiagonalMatrixTransp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("transp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixAdj() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = adj(a)",
        "c_hp = adj(hp(a))",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-8"),
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCofactor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = cofactor(a)",
        "c_hp = cofactor(hp(a))",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-8"),
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixEigenvals() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = eigenvals(a)",
        "c_hp = eigenvals(hp(a))",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-12"),
        "count(r; 0; 1)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixEigenvecs() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = eigenvecs(a)",
        "c_hp = eigenvecs(hp(a))",
        "r = abs(abs(c) - abs(c_hp)) ≤ 10^-12",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixEigen() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        WellConditionedMatrix,
        "c = eigen(a)",
        "c_hp = eigen(hp(a))",
        "r = abs(abs(c) - abs(c_hp)) ≤ 10^-12*max(abs(c);1)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCholesky() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(PositiveDefiniteTestHelper("cholesky", "10^-10"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixLu() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(PositiveDefiniteTestHelper("lu", "10^-14"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixQr() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        RandomMatrixA,
        "c = qr(a)",
        "c_hp = qr(hp(a))",
        "r = abs(c - c_hp) ≤ 10^-12",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSvd() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        RandomMatrixA,
        "c = svd(a)",
        "c_hp = svd(hp(a))",
        "r = abs(abs(c) - abs(c_hp)) ≤ 10^-8*max(abs(c); 1)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixInverse() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = inverse(a)",
        "c_hp = inverse(hp(a))",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-8"),
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixLsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixEquationTestHelper("lsolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixClsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixEquationTestHelper("clsolve", "10^-8"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSlsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "Tol = 10^-5",
        "n = 250",
        "a = vec2diag((1000 + range_hp(0; n - 1; 1))/1234)",
        "b = random(fill(vector_hp(n); 1))",
        "c = slsolve(a; b)",
        "r = a * c - b ≤ 10^-5",
        "count(r; 0; 1)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixMsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixMultiEquationTestHelper("msolve", "10^-8"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixCmsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixMultiEquationTestHelper("cmsolve", "10^-8"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPDiagonalMatrixSmsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "Tol = 10^-5",
        "n = 250",
        "a = vec2diag((1000 + range_hp(0; n - 1; 1))/1234)",
        "b = random(mfill(matrix_hp(n; 2); 1))",
        "c = smsolve(a; b)",
        "r = a * c - b ≤ 10^-5",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
