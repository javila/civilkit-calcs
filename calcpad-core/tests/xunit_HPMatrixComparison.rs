//! Transcribed from Calcpad.Tests/Matrices/HP/Large/HPMatrixComparison.cs by tools/transcribe_xunit.py.
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

const RandomMatrixA: &str = "a = random(mfill(matrix(m; n); 1))";
const RandomMatrixB: &str = "b = random(mfill(matrix(m; n); 1))";
const RandomSquareMatrix: &str = "a = random(mfill(matrix(n; n); 1))";
const WellConditionedMatrix: &str = "a = submatrix(qr(random(mfill(matrix(n; n); 1))); 1; n; 1; n)";
const PositiveDefiniteArray: [&str; 5] = [
    "n = 250",
    "s = symmetric(n)",
    "v = transp([50000; 20000; 10000; 5000; 2000; 1000; 500; 200; 100; 50])",
    "$Repeat{add(v; s; i; i) @ i = 1 : n}",
    "a = copy(s; matrix(n; n); 1; 1)",
];
const RandomNum: &str = "b = random(50)";

#[track_caller]
fn OperatorTestHelper(o: char, tol: &str) -> Vec<String> {
    vec![
        ("m = 500").to_string(),
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
#[track_caller]
fn MatrixScalarTestHelper(o: char) -> Vec<String> {
    vec![
        ("m = 500").to_string(),
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (RandomNum).to_string(),
        (&format!("c = a {} b", o)).to_string(),
        (&format!("c_hp = hp(a) {} b", o)).to_string(),
        ("r = c ≡ c_hp").to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}
#[track_caller]
fn ScalarMatrixTestHelper(o: char, tol: &str) -> Vec<String> {
    vec![
        ("m = 500").to_string(),
        ("n = 500").to_string(),
        (RandomMatrixA).to_string(),
        (RandomNum).to_string(),
        (&format!("c = b {} a", o)).to_string(),
        (&format!("c_hp = b {} hp(a)", o)).to_string(),
        (&format!("r = if({} ≡ 0; c_hp ≡ c; abs(c_hp - c) ≤ {})", tol, tol)).to_string(),
        ("mcount(r; 0)").to_string(),
    ]
}

#[test]
fn HPMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('*', "10^-11"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(OperatorTestHelper('⦼', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixScalarAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarTestHelper('+'));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixScalarSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarTestHelper('-'));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixScalarMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarTestHelper('*'));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixScalarDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarTestHelper('/'));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixScalarForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarTestHelper('÷'));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixScalarIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarTestHelper('\\'));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixScalarModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixScalarTestHelper('⦼'));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarMatrixAddition() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixTestHelper('+', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarMatrixSubtraction() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixTestHelper('-', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarMatrixMultiplication() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixTestHelper('*', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarMatrixDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixTestHelper('/', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarMatrixForceDivisionBar() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixTestHelper('÷', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarMatrixIntegerDivision() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixTestHelper('\\', "0"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPScalarMatrixModulo() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarMatrixTestHelper('⦼', "10^-14"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixSin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixTan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("tan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("csc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixSec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixSinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixTanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Tanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Csch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixSech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("Sech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("coth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAsin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asin"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAcos() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acos"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAtan() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atan"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAtan2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixAcsc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAsec() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asec"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAcot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acot"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAsinh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asinh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAcosh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acosh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAtanh() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("atanh"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAcsch() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acsch"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAsech() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("asech"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAcoth() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("acoth"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixLog() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixLn() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ln"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixLog2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("log_2"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixExp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("exp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixSqrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("sqrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCbrt() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("cbrt"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixRoot() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixRound() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("round"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixFloor() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("floor"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixCeiling() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("ceiling"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixTrunc() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("trunc"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixMin() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("min"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixMax() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("max"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixSum() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sum"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixSumsq() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("sumsq"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixSrss() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("srss"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixAverage() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("average"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixProduct() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("product"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixMean() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mean"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixTake() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("take"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixLine() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("line"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixSpline() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(InterpolationTestHelper("spline"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixSortCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixRSortCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixSortRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixRSortRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixOrderCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixRevOrderCols() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixOrderRows() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixMcount() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
        "n = 500",
        "x = 1",
        "a = round(random(mfill(matrix(m; n); 1000)))",
        "c = mcount(a; x)",
        "c_hp = mcount(hp(a); x)",
        "c ≡ c_hp",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixHprod() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixFprod() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
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
fn HPMatrixKprod() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 75",
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
fn HPMatrixMnorm1() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(ScalarTestHelper("mnorm_1"));
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixMnorm2() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 200",
        "n = 200",
        RandomMatrixA,
        "c = mnorm_2(a)",
        "c_hp = mnorm_2(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixMnormi() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "m = 250",
        "n = 500",
        RandomMatrixA,
        "c = mnorm_i(a)",
        "c_hp = mnorm_i(hp(a))",
        &TestCalc::CompareWithTolerance("c", "c_hp", "10^-12"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixCond1() {
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
fn HPMatrixCond2() {
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
fn HPMatrixCondE() {
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
fn HPMatrixCondI() {
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
fn HPMatrixDet() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        RandomSquareMatrix,
        "c = det(a)",
        "c_hp = det(hp(a))",
        &TestCalc::CompareWithToleranceDirect("c", "c_hp", "10^-8"),
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixRank() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 100",
        RandomSquareMatrix,
        "c = rank(a)",
        "c_hp = rank(hp(a))",
        "r = c ≡ c_hp",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixTrace() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 500",
        RandomSquareMatrix,
        "c = trace(a)",
        "c_hp = trace(hp(a))",
        "r = abs(c - c_hp) ≤ 10^-12",
    ]);
    assert_equal(1.0, result.clone());
}
#[test]
fn HPMatrixTransp() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(FunctionTestHelper("transp"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixAdj() {
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
fn HPMatrixCofactor() {
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
fn HPMatrixLu() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(PositiveDefiniteTestHelper("lu", "10^-14"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixQr() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 250",
        RandomSquareMatrix,
        "c = qr(a)",
        "c_hp = qr(hp(a))",
        "r = abs(c - c_hp) ≤ 10^-12",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixSvd() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.RunAll(&[
        "n = 200",
        RandomSquareMatrix,
        "c = svd(a)",
        "c_hp = svd(hp(a))",
        "r = abs(abs(c) - abs(c_hp)) ≤ 10^-8*max(abs(c); 1)",
        "mcount(r; 0)",
    ]);
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixInverse() {
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
fn HPMatrixLsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixEquationTestHelper("lsolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
#[test]
fn HPMatrixMsolve() {
    let mut calc = TestCalc::new(ms());
    let mut result = calc.Run(MatrixMultiEquationTestHelper("msolve", "10^-12"));
    assert_equal(0.0, result.clone());
}
