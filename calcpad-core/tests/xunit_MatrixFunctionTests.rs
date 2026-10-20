#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/MatrixFunctionTests.cs by tools/transcribe_xunit.py.
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
fn MatrixCreation() {
    let mut m = "3";
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("matrix({};{})", m, n));
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn IdentityMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("identity({})", n));
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn DiagonalMatrixCreation() {
    let mut n = "3";
    let mut d = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("diagonal({}; {})", n, d));
    assert_equal_str("[7 0 0|0 7 0|0 0 7]", &calc.ToString());
}
#[test]
fn ColumnMatrixCreation() {
    let mut m = "3";
    let mut c = "11";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("column({}; {})", m, c));
    assert_equal_str("[11|11|11]", &calc.ToString());
}
#[test]
fn UtriangMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("utriang({})", n));
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn LtriangMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("ltriang({})", n));
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn SymmetricMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("symmetric({})", n));
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn Vec2diagMatrixCreation() {
    let mut v = "[11; 12; 1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vec2diag({})", v));
    assert_equal_str("[11 0 0|0 12 0|0 0 1]", &calc.ToString());
}
#[test]
fn Diag2vecMatrixCreation() {
    let mut m = "[11; 12; 17| 2; 4; 4| 1; 2; 5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("diag2vec({})", m));
    assert_equal_str("[11 4 5]", &calc.ToString());
}
#[test]
fn Vec2colMatrixCreation() {
    let mut v = "[1; 7; 5; 4; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vec2col({})", v));
    assert_equal_str("[1|7|5|4|9]", &calc.ToString());
}
#[test]
fn JoinColsMatrixCreation() {
    let mut c1 = "[1; 7; 5; 4; 9]";
    let mut c2 = "[2; 4; 5; 19]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("join_cols({};{})", c1, c2));
    assert_equal_str("[1 2|7 4|5 5|4 19|9 0]", &calc.ToString());
}
#[test]
fn JoinRowsMatrixCreation() {
    let mut r1 = "[1; 7; 5; 4; 9]";
    let mut r2 = "[2; 4; 5; 19]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("join_rows({};{})", r1, r2));
    assert_equal_str("[1 7 5 4 9|2 4 5 19 0]", &calc.ToString());
}
#[test]
fn AugmentMatrixCreation() {
    let mut m1 = "[1|2]";
    let mut m2 = "[2; 7|5; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("augment({};{})", m1, m2));
    assert_equal_str("[1 2 7|2 5 9]", &calc.ToString());
}
#[test]
fn StackMatrixCreation() {
    let mut m1 = "[1; 2| 7; 9]";
    let mut m2 = "[4; 4|11; 35]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("stack({};{})", m1, m2));
    assert_equal_str("[1 2|7 9|4 4|11 35]", &calc.ToString());
}
#[test]
fn ResizeMatrix() {
    let mut M = "[1; 2; 4; 5| 7; 9; 9; 155]";
    let mut m = "2";
    let mut n = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mresize({};{};{})", M, m, n));
    assert_equal_str("[1 2|7 9]", &calc.ToString());
}
#[test]
fn MatrixFill() {
    let mut M = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfill({};{})", M, x));
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn MatrixCopy() {
    let mut A = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut B = "[2; 2; 7|11; 45; 6|1; 2; 9]";
    let mut i = "3";
    let mut j = "3";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("copy({};{};{};{})", A, B, i, j));
    assert_equal_str("[2 2 7|11 45 6|1 2 1]", &calc.ToString());
}
#[test]
fn MatrixAdd() {
    let mut A = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut B = "[2; 2; 7|11; 45; 6|1; 2; 9]";
    let mut i = "1";
    let mut j = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("add({};{};{};{})", A, B, i, j));
    assert_equal_str("[3 4 10|15 52 13|10 11 18]", &calc.ToString());
}
#[test]
fn MatrixNrows() {
    let mut M = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("n_rows({})", M));
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn MatrixNcols() {
    let mut M = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("n_cols({})", M));
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn SubmatrixCreation() {
    let mut M = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("submatrix({}; 2; 3; 2; 3)", M));
    assert_equal_str("[7 7|9 9]", &calc.ToString());
}
#[test]
fn MatrixExtractRows() {
    let mut M = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut i = "[3; 3; 3]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("extract_rows({}; {})", M, i));
    assert_equal_str("[9 9 9|9 9 9|9 9 9]", &calc.ToString());
}
#[test]
fn MatrixExtractCols() {
    let mut M = "[1; 2; 3|4; 7; 7|9; 9; 9]";
    let mut i = "[1; 1; 1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("extract_cols({}; {})", M, i));
    assert_equal_str("[1 1 1|4 4 4|9 9 9]", &calc.ToString());
}
#[test]
fn MatrixHprod() {
    let mut A = "[1; 3; 5|7; 2; 6|4; 8; 9]";
    let mut B = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hprod({}; {})", A, B));
    assert_equal_str("[2 12 30|7 6 54|20 56 27]", &calc.ToString());
}
#[test]
fn MatrixFprod() {
    let mut A = "[2; 5; 3|6; 7; 8|1; 4; 9]";
    let mut B = "[1; 6; 5|7; 3; 2|8; 5; 4]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("fprod({}; {})", A, B));
    assert_equal_str("190", &calc.ToString());
}
#[test]
fn MatrixKprod() {
    let mut A = "[3; 2|4; 6]";
    let mut B = "[2; 5|1; 4]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("kprod({}; {})", A, B));
    assert_equal_str("[6 15 4 10|3 12 2 8|8 20 12 30|4 16 6 24]", &calc.ToString());
}
#[test]
fn MatrixMnorm() {
    let mut M = "[1; 4; 6|2; 5; 7|8; 3; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mnorm({})", M));
    assert_equal_str("16.2", &calc.ToString());
}
#[test]
fn MatrixMnormE() {
    let mut M = "[2; 3; 4|5; 1; 7|6; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mnorm_e({})", M));
    assert_equal_str("16.88", &calc.ToString());
}
#[test]
fn MatrixMnorm1() {
    let mut M = "[2; 3; 9|5; 6; 7|1; 8; 4]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mnorm_1({})", M));
    assert_equal_str("20", &calc.ToString());
}
#[test]
fn MatrixMnorm2() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mnorm_2({})", M));
    assert_equal_str("16.07", &calc.ToString());
}
#[test]
fn MatrixMnormi() {
    let mut M = "[4; 6; 5|3; 8; 2|7; 9; 1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mnorm_i({})", M));
    assert_equal_str("17", &calc.ToString());
}
#[test]
fn MatrixCond() {
    let mut M = "[2; 1; 3|4; 7; 6|8; 5; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cond({})", M));
    assert_equal_str("29.42", &calc.ToString());
}
#[test]
fn MatrixCondE() {
    let mut M = "[3; 6; 5|8; 2; 4|9; 1; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cond_e({})", M));
    assert_equal_str("10.71", &calc.ToString());
}
#[test]
fn MatrixCond1() {
    let mut M = "[5; 3; 1|9; 6; 7|2; 8; 4]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cond_1({})", M));
    assert_equal_str("11.67", &calc.ToString());
}
#[test]
fn MatrixCond2() {
    let mut M = "[6; 4; 2|3; 9; 8|5; 1; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cond_2({})", M));
    assert_equal_str("3.88", &calc.ToString());
}
#[test]
fn MatrixCondi() {
    let mut M = "[4; 7; 9|5; 1; 6|8; 2; 3]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cond_i({})", M));
    assert_equal_str("10.7", &calc.ToString());
}
#[test]
fn MatrixDet() {
    let mut M = "[2; 3; 4|5; 6; 7|8; 9; 1]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("det({})", M));
    assert_equal_str("27", &calc.ToString());
}
#[test]
fn MatrixRank() {
    let mut M = "[1; 3; 5|7; 2; 6|4; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("rank({})", M));
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn MatrixTrace() {
    let mut M = "[1; 2; 3|4; 5; 6|7; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("trace({})", M));
    assert_equal_str("15", &calc.ToString());
}
#[test]
fn MatrixTransp() {
    let mut M = "[4; 1; 6|7; 2; 9|3; 8; 5]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("transp({})", M));
    assert_equal_str("[4 7 3|1 2 8|6 9 5]", &calc.ToString());
}
#[test]
fn MatrixAdj() {
    let mut M = "[2; 5; 7|3; 8; 1|6; 4; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("adj({})", M));
    assert_equal_str("[68 -17 -51|-21 -24 19|-36 22 1]", &calc.ToString());
}
#[test]
fn MatrixCofactor() {
    let mut M = "[2; 4; 8|1; 7; 3|5; 9; 6]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cofactor({})", M));
    assert_equal_str("[15 9 -26|48 -28 2|-44 2 10]", &calc.ToString());
}
#[test]
fn MatrixEigenvalues() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "M = copy([1; 3; 1|0; 2; 5|0; 0; 9]; symmetric(3); 1; 1)",
        "eigenvals(M)",
    ]);
    assert_equal_str("[-2.27 2.19 12.09]", &calc.ToString());
}
#[test]
fn MatrixEigenvectors() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "M = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric(3); 1; 1)",
        "eigenvecs(M)",
    ]);
    assert_equal_str(
        "[0.658 -0.0844 -0.748|-0.359 0.838 -0.411|0.661 0.539 0.521]",
        &calc.ToString(),
    );
}
#[test]
fn MatrixCholeskyDecomp() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric(3); 1; 1)", "cholesky(M)"]);
    assert_equal_str("[2 0.5 1|0 2.18 1.15|0 0 1.92]", &calc.ToString());
}
#[test]
fn MatrixLUdecomp() {
    let mut M = "[6; 2; 1|8; 7; 3|9; 5; 4]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lu({})", M));
    assert_equal_str("[6 2 1|1.33 4.33 1.67|1.5 0.462 1.73]", &calc.ToString());
}
#[test]
fn MatrixSVDdecomp() {
    let mut M = "[2; 4; 6|1; 5; 3|9; 7; 8]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("svd({})", M));
    assert_equal_str("[-0.433 -0.526 0.732 16.38 -0.535 -0.564 -0.629|-0.32 -0.67 -0.67 3.62 0.83 -0.491 -0.266|-0.842 0.525 -0.122 1.92 -0.159 -0.664 0.73]", &calc .ToString ( ));
}
#[test]
fn MatrixInverse() {
    let mut M = "[5; 3; 2|1; 4; 7|6; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("inverse({})", M));
    assert_equal_str("[0.606 0.333 -0.394|-1 -1 1|0.485 0.667 -0.515]", &calc.ToString());
}
#[test]
fn MatrixLsolve() {
    let mut v = "[5; 10; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = [2; 3; 4|3; 5; 6|4; 6; 7]");
    calc.Run(&format!("lsolve(M; {})", v));
    assert_equal_str("[-11 5 3]", &calc.ToString());
}
#[test]
fn MatrixMsolve() {
    let mut v = "[5; 7|10; 14|7; 22]";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = [2; 3; 4|3; 5; 6|4; 6; 7]");
    calc.Run(&format!("msolve(M; {})", v));
    assert_equal_str("[-11 9|5 7|3 -8]", &calc.ToString());
}
#[test]
fn MatrixSymLsolve() {
    let mut v = "[5; 10; 7]";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric(3); 1; 1)");
    calc.Run(&format!("lsolve(M; {})", v));
    assert_equal_str("[-11 5 3]", &calc.ToString());
}
#[test]
fn MatrixSymClsolve() {
    let mut v = "[7; 14; 22]";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric(3); 1; 1)");
    calc.Run(&format!("clsolve(M; {})", v));
    assert_equal_str("[-0.1 0.857 3.27]", &calc.ToString());
}
#[test]
fn MatrixSymMsolve() {
    let mut v = "[5; 7|10; 14|7; 22]";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric(3); 1; 1)");
    calc.Run(&format!("msolve(M; {})", v));
    assert_equal_str("[-11 9|5 7|3 -8]", &calc.ToString());
}
#[test]
fn MatrixSymCmsolve() {
    let mut v = "[5; 7|10; 14|7; 22]";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric(3); 1; 1)");
    calc.Run(&format!("cmsolve(M; {})", v));
    assert_equal_str("[0.8 -0.1|1.86 0.857|-0.0286 3.27]", &calc.ToString());
}
#[test]
fn MatrixSortCols() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut i = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sort_cols({};{})", M, i));
    assert_equal_str("[2 4 6|1 3 9|5 7 3]", &calc.ToString());
}
#[test]
fn MatrixRSortCols() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut i = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("rsort_cols({};{})", M, i));
    assert_equal_str("[6 4 2|9 3 1|3 7 5]", &calc.ToString());
}
#[test]
fn MatrixSortRows() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut j = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sort_rows({};{})", M, j));
    assert_equal_str("[1 3 9|2 4 6|5 7 3]", &calc.ToString());
}
#[test]
fn MatrixRSortRows() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut j = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("rsort_rows({};{})", M, j));
    assert_equal_str("[5 7 3|2 4 6|1 3 9]", &calc.ToString());
}
#[test]
fn MatrixOrderCols() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut i = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("order_cols({};{})", M, i));
    assert_equal_str("[1 2 3]", &calc.ToString());
}
#[test]
fn MatrixRevOrderCols() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut i = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("revorder_cols({};{})", M, i));
    assert_equal_str("[3 2 1]", &calc.ToString());
}
#[test]
fn MatrixOrderRows() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut j = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("order_rows({};{})", M, j));
    assert_equal_str("[2 1 3]", &calc.ToString());
}
#[test]
fn MatrixRevOrderRows() {
    let mut M = "[2; 4; 6|1; 3; 9|5; 7; 3]";
    let mut j = "1";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("revorder_rows({};{})", M, j));
    assert_equal_str("[3 1 2]", &calc.ToString());
}
#[test]
fn MatrixCount() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mcount({};{})", M, x));
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn MatrixSearch() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("msearch({};{}; 1; 1)", M, x));
    assert_equal_str("[1 1]", &calc.ToString());
}
#[test]
fn MatrixFind() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfind({};{})", M, x));
    assert_equal_str("[1 2 3|1 1 2]", &calc.ToString());
}
#[test]
fn MatrixFindEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfind_eq({};{})", M, x));
    assert_equal_str("[1 2 3|1 1 2]", &calc.ToString());
}
#[test]
fn MatrixFindNotEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfind_ne({};{})", M, x));
    assert_equal_str("[1 1 2 2 3 3|2 3 2 3 1 3]", &calc.ToString());
}
#[test]
fn MatrixFindLessThan() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfind_lt({};{})", M, x));
    assert_equal_str("[2|2]", &calc.ToString());
}
#[test]
fn MatrixFindLessOrEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfind_le({};{})", M, x));
    assert_equal_str("[1 2 2 3|1 1 2 2]", &calc.ToString());
}
#[test]
fn MatrixFindGreaterThan() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfind_gt({};{})", M, x));
    assert_equal_str("[3|3]", &calc.ToString());
}
#[test]
fn MatrixFindGreaterOrEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mfind_ge({};{})", M, x));
    assert_equal_str("[2 3|3 3]", &calc.ToString());
}
#[test]
fn MatrixHlookup() {
    let mut M = "[2; 3; 7|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hlookup({};{}; 1; 2)", M, x));
    assert_equal_str("[7]", &calc.ToString());
}
#[test]
fn MatrixHlookupEqual() {
    let mut M = "[2; 3; 7|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hlookup_eq({};{}; 1; 2)", M, x));
    assert_equal_str("[7]", &calc.ToString());
}
#[test]
fn MatrixHlookupNotEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hlookup_ne({};{}; 1; 2)", M, x));
    assert_equal_str("[2 1 7]", &calc.ToString());
}
#[test]
fn MatrixHlookupLessThan() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hlookup_lt({};{}; 1; 2)", M, x));
    assert_equal_str("[2 1 7]", &calc.ToString());
}
#[test]
fn MatrixHlookupLessOrEqual() {
    let mut M = "[2; 3; 7|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hlookup_le({};{}; 1; 2)", M, x));
    assert_equal_str("[2 1 7]", &calc.ToString());
}
#[test]
fn MatrixHlookupGreaterThan() {
    let mut M = "[2; 3; 10|2; 1; 4|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hlookup_gt({};{}; 1; 2)", M, x));
    assert_equal_str("[4]", &calc.ToString());
}
#[test]
fn MatrixHlookupGreaterOrEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("hlookup_ge({};{}; 1; 2)", M, x));
    assert_equal_str("[]", &calc.ToString());
}
#[test]
fn MatrixVlookup() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vlookup({};{}; 1; 2)", M, x));
    assert_equal_str("[3 1]", &calc.ToString());
}
#[test]
fn MatrixVlookupEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vlookup_eq({};{}; 1; 2)", M, x));
    assert_equal_str("[3 1]", &calc.ToString());
}
#[test]
fn MatrixVlookupNotEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vlookup_ne({};{}; 1; 2)", M, x));
    assert_equal_str("[2]", &calc.ToString());
}
#[test]
fn MatrixVlookupLessThan() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "15";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vlookup_lt({};{}; 1; 2)", M, x));
    assert_equal_str("[3 1 2]", &calc.ToString());
}
#[test]
fn MatrixVlookupLessOrEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vlookup_le({};{}; 1; 2)", M, x));
    assert_equal_str("[3 1 2]", &calc.ToString());
}
#[test]
fn MatrixVlookupGreaterThan() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vlookup_gt({};{}; 1; 2)", M, x));
    assert_equal_str("[2]", &calc.ToString());
}
#[test]
fn MatrixVlookupGreaterOrEqual() {
    let mut M = "[2; 3; 4|2; 1; 7|6; 2; 9]";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vlookup_ge({};{}; 1; 3)", M, x));
    assert_equal_str("[4 7 9]", &calc.ToString());
}
#[test]
fn MatrixMin() {
    let mut M = "[1; 3; 5|7; 2; 6|4; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("min({})", M));
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn MatrixMax() {
    let mut M = "[1; 3; 5|7; 2; 6|4; 8; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("max({})", M));
    assert_equal_str("9", &calc.ToString());
}
#[test]
fn MatrixSum() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sum({})", M));
    assert_equal_str("45", &calc.ToString());
}
#[test]
fn MatrixSumSq() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sumsq({})", M));
    assert_equal_str("285", &calc.ToString());
}
#[test]
fn MatrixSrss() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("srss({})", M));
    assert_equal_str("16.88", &calc.ToString());
}
#[test]
fn MatrixAverage() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("average({})", M));
    assert_equal_str("5", &calc.ToString());
}
#[test]
fn MatrixProduct() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("product({})", M));
    assert_equal_str("362880", &calc.ToString());
}
#[test]
fn MatrixMean() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("mean({})", M));
    assert_equal_str("4.15", &calc.ToString());
}
#[test]
fn MatrixTake() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("take(1; 2; {})", M));
    assert_equal_str("5", &calc.ToString());
}
#[test]
fn MatrixLine() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("line(4; 5; {})", M));
    assert_equal_str(" Undefined ", &calc.ToString());
}
#[test]
fn MatrixSpline() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("spline(2; 2; {})", M));
    assert_equal_str("2", &calc.ToString());
}
#[test]
fn MatrixNot() {
    let mut M = "[0; 5; 7|8; 2; 6|4; 1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("not({})", M));
    assert_equal_str("[1 0 0|0 0 0|0 0 1]", &calc.ToString());
}
#[test]
fn MatrixOr() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("or({})", M));
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn MatrixAnd() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("and({})", M));
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn MatrixXor() {
    let mut M = "[3; 5; 7|8; 2; 6|4; 1; 9]";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("xor({})", M));
    assert_equal_str("1", &calc.ToString());
}
