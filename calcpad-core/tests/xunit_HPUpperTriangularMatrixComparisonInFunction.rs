//! Transcribed from Calcpad.Tests/Matrices/HP/Large/HPUpperTriangularMatrixComparisonInFunction.cs by tools/transcribe_xunit.py.
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
const RandomNum: &str = "b = random(50)";

#[track_caller]
fn OperatorTestHelper(o: char, tol: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (RandomMatrixB).to_string(),
        (&format!("f(a; b) = a {} b", o)).to_string(),
        ("a_hp = hp(a)").to_string(),
        ("b_hp = hp(b)").to_string(),
        (&format!(
            "r = if({} ≡ 0; f(a_hp; b_hp) ≡ f(a; b); abs(f(a_hp; b_hp) - f(a; b)) ≤ {})",
            tol, tol
        ))
            .to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn FunctionTestHelper(func: &str) -> Vec<String> {
    vec![
        ("m = 250").to_string(),
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (&format!("f(a) = {}(a)", func)).to_string(),
        ("r = f(a) ≡ f(hp(a))").to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn ScalarTestHelper(func: &str) -> Vec<String> {
    vec![
        ("m = 250").to_string(),
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (&format!("f(a) = {}(a)", func)).to_string(),
        (&TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-14")).to_string(),
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
        (&format!("f(i; j; a) = {}(i; j; a)", func)).to_string(),
        (&TestCalc::CompareWithTolerance("f(i; j; a)", "f(i; j; hp(a))", "10^-14")).to_string(),
    ]
}
#[track_caller]
fn PositiveDefiniteTestHelper(func: &str, tol: &str) -> Vec<String> {
    PositiveDefiniteArray
        .iter()
        .map(|s| s.to_string())
        .chain(vec![
            (&format!("f(a) = {}(a)", func)).to_string(),
            (&TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", tol)).to_string(),
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
            (&format!("f(a; b) = {}(a; b)", func)).to_string(),
            (&TestCalc::CompareWithTolerance("f(a; b)", "f(hp(a); hp(b))", tol)).to_string(),
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
            (&format!("f(a; b) = {}(a; b)", func)).to_string(),
            (&TestCalc::CompareWithTolerance("f(a; b)", "f(hp(a); hp(b))", tol)).to_string(),
            ("mcount(r; 0)").to_string(),
        ])
        .collect::<Vec<String>>()
}
#[track_caller]
fn MatrixScalarOperatorTestHelper(o: char, tol: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (RandomNum).to_string(),
        (&format!("f(a; b) = a {} b", o)).to_string(),
        ("a_hp = hp(a)").to_string(),
        (&format!(
            "r = if({} ≡ 0; f(a_hp; b) ≡ f(a; b); abs(f(a_hp; b) - f(a; b)) ≤ {})",
            tol, tol
        ))
            .to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn ScalarMatrixOperatorTestHelper(o: char, tol: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (RandomNum).to_string(),
        (&format!("f(b; a) = b {} a", o)).to_string(),
        ("a_hp = hp(a)").to_string(),
        (&format!(
            "r = if({} ≡ 0; f(b; a_hp) ≡ f(b; a); abs(f(b; a_hp) - f(b; a)) ≤ {})",
            tol, tol
        ))
            .to_string(),
        ("mcount(r; 0)").to_string(),
    ]
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
fn HPUpperTriangularMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('*', "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixScalarForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixScalarModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarUpperTriangularMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarUpperTriangularMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarUpperTriangularMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('*', "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarUpperTriangularMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarUpperTriangularMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarUpperTriangularMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarUpperTriangularMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('⦼', "0"));
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
        "f(a; b) = atan2(a; b)",
        "r = f(a; b) ≡ f(hp(a); hp(b))",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixMatMul() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        RandomMatrixB,
        "f(a; b) = a * b",
        "c_hp = matmul(hp(a); hp(b))",
        "r = abs(abs(f(a; b)) - abs(c_hp)) ≤ 10^-8*max(abs(f(a; b)); 1)",
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
        "f(a; nth) = root(a; nth)",
        "r = f(a; nth) ≡ f(hp(a); nth)",
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
        "f(a; i) = sort_cols(a; i)",
        "r = f(a; i) ≡ f(hp(a); i)",
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
        "f(a; i) = rsort_cols(a; i)",
        "r = f(a; i) ≡ f(hp(a); i)",
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
        "f(a; j) = sort_rows(a; j)",
        "c_hp = sort_rows(hp(a); j)",
        "r = f(a; j) ≡ f(hp(a); j)",
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
        "f(a; j) = rsort_rows(a; j)",
        "c_hp = rsort_rows(hp(a); j)",
        "r = f(a; j) ≡ f(hp(a); j)",
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
        "f(a; i) = order_cols(a; i)",
        "r = f(a; i) ≡ f(hp(a); i)",
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
        "f(a; i) = revorder_cols(a; i)",
        "r = f(a; i) ≡ f(hp(a); i)",
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
        "f(a; j) = order_rows(a; j)",
        "r = f(a; j) ≡ f(hp(a); j)",
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
        "f(a; x) = mcount(a; x)",
        "f(a; x) ≡ f(hp(a); x)",
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
        "f(a; b) = hprod(a; b)",
        "r = f(a; b) ≡ f(hp(a); hp(b))",
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
        "f(a; b) = fprod(a; b)",
        &TestCalc::CompareWithToleranceDirect("f(a; b)", "f(hp(a); hp(b))", "10^-12"),
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
        "f(a; b) = kprod(a; b)",
        "r = f(a; b) ≡ f(hp(a); hp(b))",
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
        "f(a) = mnorm_2(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixMnormi() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        "f(a) = mnorm_i(a)",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCond1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "f(a) = cond_1(a)",
        "c_hp = cond_1(hp(a))",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCond2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        WellConditionedMatrix,
        "f(a) = cond_2(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCondE() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "f(a) = cond_e(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixCondI() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "f(a) = cond_i(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixDet() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "f(a) = det(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
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
        "f(a) = rank(a)",
        "r = f(a) ≡ f(hp(a))",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPUpperTriangularMatrixTrace() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomMatrixA,
        "f(a) = trace(a)",
        "r = abs(f(a) - f(hp(a))) ≤ 10^-12",
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
        "f(a) = adj(a)",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-8"),
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
        "f(a) = cofactor(a)",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-8"),
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
        "f(a) = qr(a)",
        "r = abs(f(a) - f(hp(a))) ≤ 10^-12",
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
        "f(a) = svd(a)",
        "r = abs(abs(f(a)) - abs(f(hp(a)))) ≤ 10^-8*max(abs(f(a)); 1)",
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
        "f(a) = inverse(a)",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-8"),
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
