#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Matrices/HP/Small/HPMatrixFunctionInFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPMatrixCreation() {
    let mut m = "3";
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = matrix_hp(a; b)", &format!("f({}; {})", m, n)]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPIdentityMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = identity_hp(a)", &format!("f({})", n)]);
    assert_equal_str("[1 0 0|0 1 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiagonalMatrixCreation() {
    let mut n = "3";
    let mut d = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = diagonal_hp(a; b)", &format!("f({}; {})", n, d)]);
    assert_equal_str("[7 0 0|0 7 0|0 0 7]", &calc.ToString());
}
#[test]
fn HPColumnMatrixCreation() {
    let mut m = "3";
    let mut c = "11";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = column_hp(a; b)", &format!("f({}; {})", m, c)]);
    assert_equal_str("[11|11|11]", &calc.ToString());
}
#[test]
fn HPUtriangMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = utriang_hp(a)", &format!("f({})", n)]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPLtriangMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = ltriang_hp(a)", &format!("f({})", n)]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPSymmetricMatrixCreation() {
    let mut n = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = symmetric_hp(a)", &format!("f({})", n)]);
    assert_equal_str("[0 0 0|0 0 0|0 0 0]", &calc.ToString());
}
#[test]
fn HPVec2diagMatrixCreation() {
    let mut v = "hp([11; 12; 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = vec2diag(a)", &format!("f({})", v)]);
    assert_equal_str("[11 0 0|0 12 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPDiag2vecMatrixCreation() {
    let mut m = "hp([11; 12; 17| 2; 4; 4| 1; 2; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = diag2vec(a)", &format!("f({})", m)]);
    assert_equal_str("[11 4 5]", &calc.ToString());
}
#[test]
fn IsHPMatrix() {
    let mut m = "hp([11; 12; 17| 2; 4; 4| 1; 2; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = ishp(a)", &format!("f({})", m)]);
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn HPJoinColsMatrixCreation() {
    let mut c1 = "hp([1; 7; 5; 4; 9])";
    let mut c2 = "hp([2; 4; 5; 19])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = join_cols(a; b)", &format!("f({}; {})", c1, c2)]);
    assert_equal_str("[1 2|7 4|5 5|4 19|9 0]", &calc.ToString());
}
#[test]
fn HPJoinRowsMatrixCreation() {
    let mut r1 = "hp([1; 7; 5; 4; 9])";
    let mut r2 = "hp([2; 4; 5; 19])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = join_rows(a; b)", &format!("f({}; {})", r1, r2)]);
    assert_equal_str("[1 7 5 4 9|2 4 5 19 0]", &calc.ToString());
}
#[test]
fn HPAugmentMatrixCreation() {
    let mut m1 = "hp([1|2])";
    let mut m2 = "hp([2; 7|5; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = augment(a; b)", &format!("f({}; {})", m1, m2)]);
    assert_equal_str("[1 2 7|2 5 9]", &calc.ToString());
}
#[test]
fn HPStackMatrixCreation() {
    let mut m1 = "hp([1; 2| 7; 9])";
    let mut m2 = "hp([4; 4|11; 35])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = stack(a; b)", &format!("f({}; {})", m1, m2)]);
    assert_equal_str("[1 2|7 9|4 4|11 35]", &calc.ToString());
}
#[test]
fn HPResizeMatrix() {
    let mut M = "hp([1; 2; 4; 5| 7; 9; 9; 155])";
    let mut m = "2";
    let mut n = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b; c) = mresize(a; b; c)", &format!("f({}; {}; {})", M, m, n)]);
    assert_equal_str("[1 2|7 9]", &calc.ToString());
}
#[test]
fn HPMatrixFill() {
    let mut M = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut x = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfill(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[1 1 1|1 1 1|1 1 1]", &calc.ToString());
}
#[test]
fn HPMatrixCopy() {
    let mut A = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut B = "hp([2; 2; 7|11; 45; 6|1; 2; 9])";
    let mut i = "3";
    let mut j = "3";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = copy(a; b; c; d)",
        &format!("f({}; {}; {}; {})", A, B, i, j),
    ]);
    assert_equal_str("[2 2 7|11 45 6|1 2 1]", &calc.ToString());
}
#[test]
fn HPMatrixAdd() {
    let mut A = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut B = "hp([2; 2; 7|11; 45; 6|1; 2; 9])";
    let mut i = "1";
    let mut j = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = add(a; b; c; d)",
        &format!("f({}; {}; {}; {})", A, B, i, j),
    ]);
    assert_equal_str("[3 4 10|15 52 13|10 11 18]", &calc.ToString());
}
#[test]
fn HPMatrixNrows() {
    let mut M = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = n_rows(a)", &format!("f({})", M)]);
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn HPMatrixNcols() {
    let mut M = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = n_cols(a)", &format!("f({})", M)]);
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn HPSubmatrixCreation() {
    let mut M = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d; e) = submatrix(a; b; c; d; e)",
        &format!("f({}; 2; 3; 2; 3)", M),
    ]);
    assert_equal_str("[7 7|9 9]", &calc.ToString());
}
#[test]
fn HPMatrixExtractRows() {
    let mut M = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut i = "hp([3; 3; 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = extract_rows(a; b)", &format!("f({}; {})", M, i)]);
    assert_equal_str("[9 9 9|9 9 9|9 9 9]", &calc.ToString());
}
#[test]
fn HPMatrixExtractCols() {
    let mut M = "hp([1; 2; 3|4; 7; 7|9; 9; 9])";
    let mut i = "hp([1; 1; 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = extract_cols(a; b)", &format!("f({}; {})", M, i)]);
    assert_equal_str("[1 1 1|4 4 4|9 9 9]", &calc.ToString());
}
#[test]
fn HPMatrixHprod() {
    let mut A = "hp([1; 3; 5|7; 2; 6|4; 8; 9])";
    let mut B = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = hprod(a; b)", &format!("f({}; {})", A, B)]);
    assert_equal_str("[2 12 30|7 6 54|20 56 27]", &calc.ToString());
}
#[test]
fn HPMatrixFprod() {
    let mut A = "hp([2; 5; 3|6; 7; 8|1; 4; 9])";
    let mut B = "hp([1; 6; 5|7; 3; 2|8; 5; 4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = fprod(a; b)", &format!("f({}; {})", A, B)]);
    assert_equal_str("190", &calc.ToString());
}
#[test]
fn HPMatrixKprod() {
    let mut A = "hp([3; 2|4; 6])";
    let mut B = "hp([2; 5|1; 4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = kprod(a; b)", &format!("f({}; {})", A, B)]);
    assert_equal_str("[6 15 4 10|3 12 2 8|8 20 12 30|4 16 6 24]", &calc.ToString());
}
#[test]
fn HPMatrixMnorm() {
    let mut M = "hp([1; 4; 6|2; 5; 7|8; 3; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = mnorm(a)", &format!("f({})", M)]);
    assert_equal_str("16.2", &calc.ToString());
}
#[test]
fn HPMatrixMnormE() {
    let mut M = "hp([2; 3; 4|5; 1; 7|6; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = mnorm_e(a)", &format!("f({})", M)]);
    assert_equal_str("16.88", &calc.ToString());
}
#[test]
fn HPMatrixMnorm1() {
    let mut M = "hp([2; 3; 9|5; 6; 7|1; 8; 4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = mnorm_1(a)", &format!("f({})", M)]);
    assert_equal_str("20", &calc.ToString());
}
#[test]
fn HPMatrixMnorm2() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = mnorm_2(a)", &format!("f({})", M)]);
    assert_equal_str("16.07", &calc.ToString());
}
#[test]
fn HPMatrixMnormi() {
    let mut M = "hp([4; 6; 5|3; 8; 2|7; 9; 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = mnorm_i(a)", &format!("f({})", M)]);
    assert_equal_str("17", &calc.ToString());
}
#[test]
fn HPMatrixCond() {
    let mut M = "hp([2; 1; 3|4; 7; 6|8; 5; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cond(a)", &format!("f({})", M)]);
    assert_equal_str("29.42", &calc.ToString());
}
#[test]
fn HPMatrixCondE() {
    let mut M = "hp([3; 6; 5|8; 2; 4|9; 1; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cond_e(a)", &format!("f({})", M)]);
    assert_equal_str("10.71", &calc.ToString());
}
#[test]
fn HPMatrixCond1() {
    let mut M = "hp([5; 3; 1|9; 6; 7|2; 8; 4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cond_1(a)", &format!("f({})", M)]);
    assert_equal_str("11.67", &calc.ToString());
}
#[test]
fn HPMatrixCond2() {
    let mut M = "hp([6; 4; 2|3; 9; 8|5; 1; 7])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cond_2(a)", &format!("f({})", M)]);
    assert_equal_str("3.88", &calc.ToString());
}
#[test]
fn HPMatrixCondi() {
    let mut M = "hp([4; 7; 9|5; 1; 6|8; 2; 3])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cond_i(a)", &format!("f({})", M)]);
    assert_equal_str("10.7", &calc.ToString());
}
#[test]
fn HPMatrixDet() {
    let mut M = "hp([2; 3; 4|5; 6; 7|8; 9; 1])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = det(a)", &format!("f({})", M)]);
    assert_equal_str("27", &calc.ToString());
}
#[test]
fn HPMatrixRank() {
    let mut M = "hp([1; 3; 5|7; 2; 6|4; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = rank(a)", &format!("f({})", M)]);
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn HPMatrixTrace() {
    let mut M = "hp([1; 2; 3|4; 5; 6|7; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = trace(a)", &format!("f({})", M)]);
    assert_equal_str("15", &calc.ToString());
}
#[test]
fn HPMatrixTransp() {
    let mut M = "hp([4; 1; 6|7; 2; 9|3; 8; 5])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = transp(a)", &format!("f({})", M)]);
    assert_equal_str("[4 7 3|1 2 8|6 9 5]", &calc.ToString());
}
#[test]
fn HPMatrixAdj() {
    let mut M = "hp([2; 5; 7|3; 8; 1|6; 4; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = adj(a)", &format!("f({})", M)]);
    assert_equal_str("[68 -17 -51|-21 -24 19|-36 22 1]", &calc.ToString());
}
#[test]
fn HPMatrixCofactor() {
    let mut M = "hp([2; 4; 8|1; 7; 3|5; 9; 6])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = cofactor(a)", &format!("f({})", M)]);
    assert_equal_str("[15 9 -26|48 -28 2|-44 2 10]", &calc.ToString());
}
#[test]
fn HPMatrixEigenvalues() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "M = copy([1; 3; 1|0; 2; 5|0; 0; 9]; symmetric_hp(3); 1; 1)",
        "f(a) = eigenvals(a)",
        "f(M)",
    ]);
    assert_equal_str("[-2.27 2.19 12.09]", &calc.ToString());
}
#[test]
fn HPMatrixEigenvectors() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "M = copy([7; 8; 9|0; 5; 6|0; 0; 3]; symmetric_hp(3); 1; 1)",
        "f(a) = eigenvecs(a)",
        "f(M)",
    ]);
    assert_equal_str(
        "[0.658 -0.0844 -0.748|-0.359 0.838 -0.411|0.661 0.539 0.521]",
        &calc.ToString(),
    );
}
#[test]
fn HPMatrixCholeskyDecomp() {
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric_hp(3); 1; 1)",
        "f(a) = cholesky(a)",
        "f(M)",
    ]);
    assert_equal_str("[2 0.5 1|0 2.18 1.15|0 0 1.92]", &calc.ToString());
}
#[test]
fn HPMatrixLUdecomp() {
    let mut M = "hp([6; 2; 1|8; 7; 3|9; 5; 4])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = lu(a)", &format!("f({})", M)]);
    assert_equal_str("[6 2 1|1.33 4.33 1.67|1.5 0.462 1.73]", &calc.ToString());
}
#[test]
fn HPMatrixSVDdecomp() {
    let mut M = "hp([2; 4; 6|1; 5; 3|9; 7; 8])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = svd(a)", &format!("f({})", M)]);
    assert_equal_str("[-0.433 -0.526 0.732 16.38 -0.535 -0.564 -0.629|-0.32 -0.67 -0.67 3.62 0.83 -0.491 -0.266|-0.842 0.525 -0.122 1.92 -0.159 -0.664 0.73]", &calc .ToString ( ));
}
#[test]
fn HPMatrixInverse() {
    let mut M = "hp([5; 3; 2|1; 4; 7|6; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = inverse(a)", &format!("f({})", M)]);
    assert_equal_str("[0.606 0.333 -0.394|-1 -1 1|0.485 0.667 -0.515]", &calc.ToString());
}
#[test]
fn HPMatrixLsolve() {
    let mut v = "hp([5; 10; 7])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = hp([2; 3; 4|3; 5; 6|4; 6; 7])");
    calc.RunAll(&[&format!("f(a; b) = lsolve(a; b)"), &format!("f(M; {})", v)]);
    assert_equal_str("[-11 5 3]", &calc.ToString());
}
#[test]
fn HPMatrixMsolve() {
    let mut v = "hp([5; 7|10; 14|7; 22])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = hp([2; 3; 4|3; 5; 6|4; 6; 7])");
    calc.RunAll(&[&format!("f(a; b) = msolve(a; b)"), &format!("f(M; {})", v)]);
    assert_equal_str("[-11 9|5 7|3 -8]", &calc.ToString());
}
#[test]
fn HPMatrixSymLsolve() {
    let mut v = "hp([5; 10; 7])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = lsolve(a; b)"), &format!("f(M; {})", v)]);
    assert_equal_str("[-11 5 3]", &calc.ToString());
}
#[test]
fn HPMatrixSymClsolve() {
    let mut v = "hp([7; 14; 22])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = clsolve(a; b)"), &format!("f(M; {})", v)]);
    assert_equal_str("[-0.1 0.857 3.27]", &calc.ToString());
}
#[test]
fn HPMatrixSymSlsolve() {
    let mut v = "hp([7; 14; 22])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = slsolve(a; b)"), &format!("f(M; {})", v)]);
    assert_equal_str("[-0.1 0.857 3.27]", &calc.ToString());
}
#[test]
fn HPMatrixSymMsolve() {
    let mut matrix = "hp([5; 7|10; 14|7; 22])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = msolve(a; b)"), &format!("f(M; {})", matrix)]);
    assert_equal_str("[-11 9|5 7|3 -8]", &calc.ToString());
}
#[test]
fn HPMatrixSymCmsolve() {
    let mut matrix = "hp([5; 7|10; 14|7; 22])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = cmsolve(a; b)"), &format!("f(M; {})", matrix)]);
    assert_equal_str("[0.8 -0.1|1.86 0.857|-0.0286 3.27]", &calc.ToString());
}
#[test]
fn HPMatrixSymSmsolve() {
    let mut matrix = "hp([5; 7|10; 14|7; 22])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = smsolve(a; b)"), &format!("f(M; {})", matrix)]);
    assert_equal_str("[0.8 -0.1|1.86 0.857|-0.0286 3.27]", &calc.ToString());
}
#[test]
fn HPMatrixSymVecMsolve() {
    let mut vector = "hp([5; 7; 10])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([2; 3; 4|0; 5; 6|0; 0; 7]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = msolve(a; b)"), &format!("f(M; {})", vector)]);
    assert_equal_str("[4|-1|0]", &calc.ToString());
}
#[test]
fn HPMatrixSymVecCmsolve() {
    let mut vector = "hp([5; 7; 10])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = cmsolve(a; b)"), &format!("f(M; {})", vector)]);
    assert_equal_str("[0.5|0.571|1.21]", &calc.ToString());
}
#[test]
fn HPMatrixSymVecSmsolve() {
    let mut vector = "hp([5| 7| 10])";
    let mut calc = TestCalc::new(ms());
    calc.Run("M = copy([4; 1; 2|0; 5; 3|0; 0; 6]; symmetric_hp(3); 1; 1)");
    calc.RunAll(&[&format!("f(a; b) = smsolve(a; b)"), &format!("f(M; {})", vector)]);
    assert_equal_str("[0.5|0.571|1.21]", &calc.ToString());
}
#[test]
fn HPMatrixSortCols() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut i = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = sort_cols(a; b)", &format!("f({}; {})", M, i)]);
    assert_equal_str("[2 4 6|1 3 9|5 7 3]", &calc.ToString());
}
#[test]
fn HPMatrixRSortCols() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut i = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = rsort_cols(a; b)", &format!("f({}; {})", M, i)]);
    assert_equal_str("[6 4 2|9 3 1|3 7 5]", &calc.ToString());
}
#[test]
fn HPMatrixSortRows() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut j = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = sort_rows(a; b)", &format!("f({}; {})", M, j)]);
    assert_equal_str("[1 3 9|2 4 6|5 7 3]", &calc.ToString());
}
#[test]
fn HPMatrixRSortRows() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut j = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = rsort_rows(a; b)", &format!("f({}; {})", M, j)]);
    assert_equal_str("[5 7 3|2 4 6|1 3 9]", &calc.ToString());
}
#[test]
fn HPMatrixOrderCols() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut i = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = order_cols(a; b)", &format!("f({}; {})", M, i)]);
    assert_equal_str("[1 2 3]", &calc.ToString());
}
#[test]
fn HPMatrixRevOrderCols() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut i = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = revorder_cols(a; b)", &format!("f({}; {})", M, i)]);
    assert_equal_str("[3 2 1]", &calc.ToString());
}
#[test]
fn HPMatrixOrderRows() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut j = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = order_rows(a; b)", &format!("f({}; {})", M, j)]);
    assert_equal_str("[2 1 3]", &calc.ToString());
}
#[test]
fn HPMatrixRevOrderRows() {
    let mut M = "hp([2; 4; 6|1; 3; 9|5; 7; 3])";
    let mut j = "1";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = revorder_rows(a; b)", &format!("f({}; {})", M, j)]);
    assert_equal_str("[3 1 2]", &calc.ToString());
}
#[test]
fn HPMatrixCount() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mcount(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("3", &calc.ToString());
}
#[test]
fn HPMatrixSearch() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b; c; d) = msearch(a; b; c; d)", &format!("f({}; {}; 1; 1)", M, x)]);
    assert_equal_str("[1 1]", &calc.ToString());
}
#[test]
fn HPMatrixFind() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfind(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[1 2 3|1 1 2]", &calc.ToString());
}
#[test]
fn HPMatrixFindEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfind_eq(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[1 2 3|1 1 2]", &calc.ToString());
}
#[test]
fn HPMatrixFindNotEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfind_ne(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[1 1 2 2 3 3|2 3 2 3 1 3]", &calc.ToString());
}
#[test]
fn HPMatrixFindLessThan() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfind_lt(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[2|2]", &calc.ToString());
}
#[test]
fn HPMatrixFindLessOrEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfind_le(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[1 2 2 3|1 1 2 2]", &calc.ToString());
}
#[test]
fn HPMatrixFindGreaterThan() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfind_gt(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[3|3]", &calc.ToString());
}
#[test]
fn HPMatrixFindGreaterOrEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b) = mfind_ge(a; b)", &format!("f({}; {})", M, x)]);
    assert_equal_str("[2 3|3 3]", &calc.ToString());
}
#[test]
fn HPMatrixHlookup() {
    let mut M = "hp([2; 3; 7|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b; c; d) = hlookup(a; b; c; d)", &format!("f({}; {}; 1; 2)", M, x)]);
    assert_equal_str("[7]", &calc.ToString());
}
#[test]
fn HPMatrixHlookupEqual() {
    let mut M = "hp([2; 3; 7|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = hlookup_eq(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[7]", &calc.ToString());
}
#[test]
fn HPMatrixHlookupNotEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = hlookup_ne(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[2 1 7]", &calc.ToString());
}
#[test]
fn HPMatrixHlookupLessThan() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = hlookup_lt(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[2 1 7]", &calc.ToString());
}
#[test]
fn HPMatrixHlookupLessOrEqual() {
    let mut M = "hp([2; 3; 7|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = hlookup_le(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[2 1 7]", &calc.ToString());
}
#[test]
fn HPMatrixHlookupGreaterThan() {
    let mut M = "hp([2; 3; 10|2; 1; 4|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = hlookup_gt(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[4]", &calc.ToString());
}
#[test]
fn HPMatrixHlookupGreaterOrEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = hlookup_ge(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[]", &calc.ToString());
}
#[test]
fn HPMatrixVlookup() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b; c; d) = vlookup(a; b; c; d)", &format!("f({}; {}; 1; 2)", M, x)]);
    assert_equal_str("[3 1]", &calc.ToString());
}
#[test]
fn HPMatrixVlookupEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = vlookup_eq(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[3 1]", &calc.ToString());
}
#[test]
fn HPMatrixVlookupNotEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = vlookup_ne(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[2]", &calc.ToString());
}
#[test]
fn HPMatrixVlookupLessThan() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "15";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = vlookup_lt(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[3 1 2]", &calc.ToString());
}
#[test]
fn HPMatrixVlookupLessOrEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = vlookup_le(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[3 1 2]", &calc.ToString());
}
#[test]
fn HPMatrixVlookupGreaterThan() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = vlookup_gt(a; b; c; d)",
        &format!("f({}; {}; 1; 2)", M, x),
    ]);
    assert_equal_str("[2]", &calc.ToString());
}
#[test]
fn HPMatrixVlookupGreaterOrEqual() {
    let mut M = "hp([2; 3; 4|2; 1; 7|6; 2; 9])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&[
        "f(a; b; c; d) = vlookup_ge(a; b; c; d)",
        &format!("f({}; {}; 1; 3)", M, x),
    ]);
    assert_equal_str("[4 7 9]", &calc.ToString());
}
#[test]
fn HPMatrixMin() {
    let mut M = "hp([1; 3; 5|7; 2; 6|4; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = min(a)", &format!("f({})", M)]);
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn HPMatrixMax() {
    let mut M = "hp([1; 3; 5|7; 2; 6|4; 8; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = max(a)", &format!("f({})", M)]);
    assert_equal_str("9", &calc.ToString());
}
#[test]
fn HPMatrixSum() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sum(a)", &format!("f({})", M)]);
    assert_equal_str("45", &calc.ToString());
}
#[test]
fn HPMatrixSumSq() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = sumsq(a)", &format!("f({})", M)]);
    assert_equal_str("285", &calc.ToString());
}
#[test]
fn HPMatrixSrss() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = srss(a)", &format!("f({})", M)]);
    assert_equal_str("16.88", &calc.ToString());
}
#[test]
fn HPMatrixAverage() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = average(a)", &format!("f({})", M)]);
    assert_equal_str("5", &calc.ToString());
}
#[test]
fn HPMatrixProduct() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = product(a)", &format!("f({})", M)]);
    assert_equal_str("362880", &calc.ToString());
}
#[test]
fn HPMatrixMean() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = mean(a)", &format!("f({})", M)]);
    assert_equal_str("4.15", &calc.ToString());
}
#[test]
fn HPMatrixTake() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b; c) = take(a; b; c)", &format!("f(1; 2; {})", M)]);
    assert_equal_str("5", &calc.ToString());
}
#[test]
fn HPMatrixLine() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b; c) = line(a; b; c)", &format!("f(4; 5; {})", M)]);
    assert_equal_str(" Undefined ", &calc.ToString());
}
#[test]
fn HPMatrixSpline() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a; b; c) = spline(a; b; c)", &format!("f(2; 2; {})", M)]);
    assert_equal_str("2", &calc.ToString());
}
#[test]
fn HPMatrixNot() {
    let mut M = "[0; 5; 7|8; 2; 6|4; 1; 0]";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = not(a)", &format!("f({})", M)]);
    assert_equal_str("[1 0 0|0 0 0|0 0 1]", &calc.ToString());
}
#[test]
fn HPMatrixOr() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = or(a)", &format!("f({})", M)]);
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn HPMatrixAnd() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = and(a)", &format!("f({})", M)]);
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn HPMatrixXor() {
    let mut M = "hp([3; 5; 7|8; 2; 6|4; 1; 9])";
    let mut calc = TestCalc::new(ms());
    calc.RunAll(&["f(a) = xor(a)", &format!("f({})", M)]);
    assert_equal_str("1", &calc.ToString());
}
