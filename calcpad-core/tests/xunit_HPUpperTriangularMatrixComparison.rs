#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Large/HPUpperTriangularMatrixComparison.cs by tools/transcribe_xunit.py.
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

const RandomMatrixA: &str = "a = random(mfill(utriang(n); 1))";
const RandomMatrixB: &str = "b = random(mfill(utriang(n); 1))";
const WellConditionedMatrix: &str = "a = diagonal(n; 1) + random(mfill(utriang(n); 0.1))";
const PositiveDefiniteArray: [&str; 4] = [
    "n = 250",
    "a = utriang(n)",
    "v = transp([50000; 20000; 10000; 5000; 2000; 1000; 500; 200; 100; 50])",
    "$Repeat{add(v; a; i; i) @ i = 1 : n}",
];

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
        ("m = 250").to_string(),
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
        ("m = 250").to_string(),
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
    PositiveDefiniteArray
        .iter()
        .map(|s| s.to_string())
        .chain(vec![
            (&format!("c = {}(a)", func)).to_string(),
            (&format!("c_hp = {}(hp(a))", func)).to_string(),
            (&TestCalc::CompareWithTolerance("c", "c_hp", tol)).to_string(),
            ("mcount(r; 0)").to_string(),
        ])
        .collect::<Vec<String>>()
}
#[track_caller]
fn MatrixEquationTestHelper(func: &str, tol: &str) -> Vec<String> {
    PositiveDefiniteArray
        .iter()
        .map(|s| s.to_string())
        .chain(vec![
            ("b = random(fill(vector(n); 1))").to_string(),
            (&format!("c = {}(a; b)", func)).to_string(),
            (&format!("c_hp = {}(hp(a); hp(b))", func)).to_string(),
            (&TestCalc::CompareWithTolerance("c", "c_hp", tol)).to_string(),
            ("count(r; 0; 1)").to_string(),
        ])
        .collect::<Vec<String>>()
}
#[track_caller]
fn MatrixMultiEquationTestHelper(func: &str, tol: &str) -> Vec<String> {
    PositiveDefiniteArray
        .iter()
        .map(|s| s.to_string())
        .chain(vec![
            ("b = random(mfill(matrix(n; 2); 1))").to_string(),
            (&format!("c = {}(a; b)", func)).to_string(),
            (&format!("c_hp = {}(hp(a); hp(b))", func)).to_string(),
            (&TestCalc::CompareWithTolerance("c", "c_hp", tol)).to_string(),
            ("mcount(r; 0)").to_string(),
        ])
        .collect::<Vec<String>>()
}

#[test]
fn HPUpperTriangularMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('*', "10^-11"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixTan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("tan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("csc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixTanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Tanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Csch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("coth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAsin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAcos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAtan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAtan2() {
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
fn HPUpperTriangularMatrixAcsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAsec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAcot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAsinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAcosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAtanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAcsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAsech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAcoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acoth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixLog() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixLn() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ln"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixLog2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log_2"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixExp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("exp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSqrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sqrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCbrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cbrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixRoot() {
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
fn HPUpperTriangularMatrixRound() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("round"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixFloor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("floor"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCeiling() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ceiling"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixTrunc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("trunc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixMin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("min"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixMax() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("max"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSum() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sum"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSumsq() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sumsq"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSrss() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("srss"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAverage() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("average"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixProduct() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("product"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixMean() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mean"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixTake() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("take"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixLine() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("line"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSpline() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("spline"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixSortCols() {
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
fn HPUpperTriangularMatrixRSortCols() {
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
fn HPUpperTriangularMatrixSortRows() {
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
fn HPUpperTriangularMatrixRSortRows() {
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
fn HPUpperTriangularMatrixOrderCols() {
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
fn HPUpperTriangularMatrixRevOrderCols() {
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
fn HPUpperTriangularMatrixOrderRows() {
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
fn HPUpperTriangularMatrixMcount() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "x = 1",
        "a = round(random(mfill(utriang(n); 1000)))",
        "c = mcount(a; x)",
        "c_hp = mcount(hp(a); x)",
        "c ≡ c_hp",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixHprod() {
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
fn HPUpperTriangularMatrixFprod() {
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
fn HPUpperTriangularMatrixKprod() {
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
fn HPUpperTriangularMatrixMnorm1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mnorm_1"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixMnorm2() {
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
fn HPUpperTriangularMatrixMnormi() {
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
fn HPUpperTriangularMatrixCond1() {
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
fn HPUpperTriangularMatrixCond2() {
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
fn HPUpperTriangularMatrixCondE() {
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
fn HPUpperTriangularMatrixCondI() {
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
fn HPUpperTriangularMatrixDet() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "c = det(a)",
        "c_hp = det(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixRank() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 100",
        "n = 100",
        WellConditionedMatrix,
        "c = rank(a)",
        "c_hp = rank(hp(a))",
        "r = c ≡ c_hp",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixTrace() {
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
fn HPUpperTriangularMatrixTransp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("transp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixAdj() {
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
fn HPUpperTriangularMatrixCofactor() {
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
fn HPUpperTriangularMatrixLu() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(PositiveDefiniteTestHelper("lu", "10^-14"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixQr() {
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
fn HPUpperTriangularMatrixSvd() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        WellConditionedMatrix,
        "c = svd(a)",
        "c_hp = svd(hp(a))",
        "r = abs(abs(c) - abs(c_hp)) ≤ 10^-8*max(abs(c); 1)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixInverse() {
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
fn HPUpperTriangularMatrixLsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixEquationTestHelper("lsolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixMsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixMultiEquationTestHelper("msolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
