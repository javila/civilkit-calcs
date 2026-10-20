//! Transcribed from Calcpad.Tests/Matrices/HP/Large/HPLowerTriangularMatrixComparisonInFunction.cs by tools/transcribe_xunit.py.
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

const RandomMatrixA: &str = "a = random(mfill(ltriang(n); 1))";
const RandomMatrixB: &str = "b = random(mfill(ltriang(n); 1))";
const WellConditionedMatrix: &str = "a = diagonal(n; 1) + random(mfill(ltriang(n); 0.1))";
const PositiveDefiniteArray: [&str; 4] = [
    "n = 250",
    "a = ltriang(n)",
    "v = [50000; 20000; 10000; 5000; 2000; 1000; 500; 200; 100; 50]",
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
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (&format!("f(a) = {}(a)", func)).to_string(),
        (&format!("c_hp = {}(hp(a))", func)).to_string(),
        ("r = f(a) ≡ f(hp(a))").to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn ScalarTestHelper(func: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (&format!("f(a) = {}(a)", func)).to_string(),
        (&TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-14")).to_string(),
    ]
}
#[track_caller]
fn InterpolationTestHelper(func: &str) -> Vec<String> {
    vec![
        ("n = 500").to_string(),
        ("i = random(n - 1) + 1").to_string(),
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
        (&format!("c_hp = b {} a_hp", o)).to_string(),
        (&format!(
            "r = if({} ≡ 0; f(b; a_hp) ≡ f(b; a); abs(f(b; a_hp) - f(b; a)) ≤ {})",
            tol, tol
        ))
            .to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}

#[test]
fn HPLowerTriangularMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('*', "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('*', "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixScalarForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixScalarModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarLowerTriangularMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarLowerTriangularMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarLowerTriangularMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('*', "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarLowerTriangularMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarLowerTriangularMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarLowerTriangularMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarLowerTriangularMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixTan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("tan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("csc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixTanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Tanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Csch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("coth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAsin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAcos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAtan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAtan2() {
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
fn HPLowerTriangularMatrixAcsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAsec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAcot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAsinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAcosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAtanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAcsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAsech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAcoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acoth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixLog() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixLn() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ln"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixLog2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log_2"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixExp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("exp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSqrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sqrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCbrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cbrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixRoot() {
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
fn HPLowerTriangularMatrixRound() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("round"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixFloor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("floor"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCeiling() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ceiling"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixTrunc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("trunc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixMin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("min"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixMax() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("max"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSum() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sum"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSumsq() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sumsq"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSrss() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("srss"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAverage() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("average"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixProduct() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("product"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixMean() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mean"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixTake() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("take"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixLine() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("line"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSpline() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("spline"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSortCols() {
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
fn HPLowerTriangularMatrixRSortCols() {
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
fn HPLowerTriangularMatrixSortRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "j = 50",
        RandomMatrixA,
        "f(a; j) = sort_rows(a; j)",
        "r = f(a; j) ≡ f(hp(a); j)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixRSortRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "j = 50",
        RandomMatrixA,
        "f(a; j) = rsort_rows(a; j)",
        "r = f(a; j) ≡ f(hp(a); j)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixOrderCols() {
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
fn HPLowerTriangularMatrixRevOrderCols() {
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
fn HPLowerTriangularMatrixOrderRows() {
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
fn HPLowerTriangularMatrixMcount() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "x = 1",
        "a = round(random(mfill(ltriang(n); 1000)))",
        "f(a; x) = mcount(a; x)",
        "f(a; x) ≡ f(hp(a); x)",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixHprod() {
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
fn HPLowerTriangularMatrixFprod() {
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
fn HPLowerTriangularMatrixKprod() {
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
fn HPLowerTriangularMatrixMnorm1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mnorm_1"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixMnorm2() {
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
fn HPLowerTriangularMatrixMnormi() {
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
fn HPLowerTriangularMatrixCond1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        WellConditionedMatrix,
        "f(a) = cond_1(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCond2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        WellConditionedMatrix,
        "f(a)  = cond_2(a)",
        &TestCalc::CompareWithToleranceDirect("f(a) ", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixCondE() {
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
fn HPLowerTriangularMatrixCondI() {
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
fn HPLowerTriangularMatrixDet() {
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
fn HPLowerTriangularMatrixRank() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 100",
        WellConditionedMatrix,
        "f(a) = rank(a)",
        "r = f(a) ≡ f(hp(a))",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixTrace() {
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
fn HPLowerTriangularMatrixTransp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("transp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixAdj() {
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
fn HPLowerTriangularMatrixCofactor() {
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
fn HPLowerTriangularMatrixLu() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(PositiveDefiniteTestHelper("lu", "10^-14"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixQr() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        RandomMatrixA,
        "f(a) = qr(a)",
        "c_hp = qr(hp(a))",
        "r = abs(f(a) - f(hp(a))) ≤ 10^-12",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixSvd() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        WellConditionedMatrix,
        "f(a) = svd(a)",
        "c_hp = svd(hp(a))",
        "r = abs(abs(f(a)) - abs(f(hp(a)))) ≤ 10^-8*max(abs(f(a)); 1)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixInverse() {
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
fn HPLowerTriangularMatrixLsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixEquationTestHelper("lsolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPLowerTriangularMatrixMsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixMultiEquationTestHelper("msolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
