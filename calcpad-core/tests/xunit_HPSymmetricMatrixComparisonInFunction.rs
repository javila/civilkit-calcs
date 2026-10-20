#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Large/HPSymmetricMatrixComparisonInFunction.cs by tools/transcribe_xunit.py.
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

const RandomMatrixA: &str = "a = random(mfill(symmetric(n); 1))";
const RandomMatrixB: &str = "b = random(mfill(symmetric(n); 1))";
const OrthogonalMatrix: &str = "q = submatrix(qr(random(mfill(matrix(n; n); 1))); 1; n; 1; n)";
const WellConditionedMatrix: &str = "a = q*vec2diag(0.55 + range(1; n; 1)/n)*transp(q)";
const PositiveDefiniteArray: [&str; 4] = [
    "n = 250",
    "a = symmetric(n)",
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
        (&format!("c_hp = a_hp {} b_hp", o)).to_string(),
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
            (&format!("c_hp = {}(hp(a); hp(b))", func)).to_string(),
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
fn HPSymmetricMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('*', "10^-11"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('*', "10^-11"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixScalarForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixScalarModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarOperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarSymmetricMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarSymmetricMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarSymmetricMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('*', "10^-11"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarSymmetricMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarSymmetricMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarSymmetricMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarSymmetricMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixOperatorTestHelper('⦼', "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixTan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("tan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("csc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixTanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Tanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Csch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("coth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAsin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAcos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAtan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAtan2() {
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
        &TestCalc::CompareWithTolerance("f(a; b)", "c_hp", "10^-13"),
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAcsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAsec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAcot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAsinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAcosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAtanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAcsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAsech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAcoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acoth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixLog() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixLn() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ln"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixLog2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log_2"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixExp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("exp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSqrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sqrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCbrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cbrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixRoot() {
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
fn HPSymmetricMatrixRound() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("round"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixFloor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("floor"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCeiling() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ceiling"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixTrunc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("trunc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixMin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("min"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixMax() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("max"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSum() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sum"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSumsq() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sumsq"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSrss() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("srss"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAverage() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("average"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixProduct() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("product"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixMean() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mean"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixTake() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("take"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixLine() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("line"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSpline() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("spline"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSortCols() {
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
fn HPSymmetricMatrixRSortCols() {
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
fn HPSymmetricMatrixSortRows() {
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
fn HPSymmetricMatrixRSortRows() {
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
fn HPSymmetricMatrixOrderCols() {
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
fn HPSymmetricMatrixRevOrderCols() {
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
fn HPSymmetricMatrixOrderRows() {
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
fn HPSymmetricMatrixMcount() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        "x = 1",
        "a = round(random(mfill(symmetric(n); 1000)))",
        "f(a; x) = mcount(a; x)",
        "f(a; x) ≡ f(hp(a); x)",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixHprod() {
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
fn HPSymmetricMatrixFprod() {
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
fn HPSymmetricMatrixKprod() {
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
fn HPSymmetricMatrixMnorm1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mnorm_1"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixMnorm2() {
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
fn HPSymmetricMatrixMnormi() {
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
fn HPSymmetricMatrixCond1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = cond_1(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCond2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = cond_2(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCondE() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = cond_e(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCondI() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = cond_i(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixDet() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = det(a)",
        &TestCalc::CompareWithToleranceDirect("f(a)", "f(hp(a))", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixRank() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&["n = 100", RandomMatrixA, "f(a) = rank(a)", "r = f(a) ≡ f(hp(a))"]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPSymmetricMatrixTrace() {
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
fn HPSymmetricMatrixTransp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("transp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixAdj() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = adj(a)",
        "c_hp = adj(hp(a))",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-8"),
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCofactor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = cofactor(a)",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-8"),
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixEigenvals() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        RandomMatrixA,
        "f(a) = eigenvals(a)",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-10"),
        "count(r; 0; 1)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixEigenvecs() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        RandomMatrixA,
        "f(a) = eigenvecs(a)",
        "r = abs(abs(f(a)) - abs(f(hp(a)))) ≤ 10^-10",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixEigen() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        RandomMatrixA,
        "f(a) = eigen(a)",
        "r = abs(abs(f(a)) - abs(f(hp(a)))) ≤ 10^-10*max(abs(f(a));1)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCholesky() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(PositiveDefiniteTestHelper("cholesky", "10^-10"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixLu() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(PositiveDefiniteTestHelper("lu", "10^-14"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixQr() {
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
fn HPSymmetricMatrixSvd() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        RandomMatrixA,
        "f(a) = svd(a)",
        "c_hp = svd(hp(a))",
        "r = abs(abs(f(a)) - abs(f(hp(a)))) ≤ 10^-8*max(abs(f(a)); 1)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixInverse() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        OrthogonalMatrix,
        WellConditionedMatrix,
        "f(a) = inverse(a)",
        &TestCalc::CompareWithTolerance("f(a)", "f(hp(a))", "10^-8"),
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixLsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixEquationTestHelper("lsolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixClsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixEquationTestHelper("clsolve", "10^-8"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSlsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "Tol = 10^-5",
        "n = 250",
        "a = symmetric_hp(n)",
        "v = transp(hp([50000; 20000; 10000; 5000; 2000; 1000; 500; 200; 100; 50]))",
        "$Repeat{ add(v; a; i; i) @ i = 1 : n}",
        "b = random(fill(vector_hp(n); 1))",
        "f(a; b) = slsolve(a; b)",
        "r = a * f(a; b) - b ≤ 10^-5",
        "count(r; 0; 1)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixMsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixMultiEquationTestHelper("msolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixCmsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixMultiEquationTestHelper("cmsolve", "10^-8"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPSymmetricMatrixSmsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "Tol = 10^-5",
        "n = 250",
        "a = symmetric_hp(n)",
        "v = transp(hp([50000; 20000; 10000; 5000; 2000; 1000; 500; 200; 100; 50]))",
        "$Repeat{ add(v; a; i; i) @ i = 1 : n}",
        "b = random(mfill(matrix_hp(n; 2); 1))",
        "f(a; b) = smsolve(a; b)",
        "r = a * f(a; b) - b ≤ 10^-5",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
