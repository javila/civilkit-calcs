#![cfg(feature = "xunit_vm")]
//! Transcribed from Calcpad.Tests/Vectors/HP/HPVectorFunctionTests.cs by tools/transcribe_xunit.py.
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
fn HPVectorCreation() {
    let mut n = 3.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("vector_hp({})", n));
    assert_equal_str("[0 0 0]", &calc.ToString());
}
#[test]
fn HPVectorFill() {
    let mut vector = "hp([0; 0; 0; 1; 2; 3])";
    let mut n = 4.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("fill({};{})", vector, n));
    assert_equal_str("[4 4 4 4 4 4]", &calc.ToString());
}
#[test]
fn HPVectorRange() {
    let mut x = "1";
    let mut y = 4.0;
    let mut s = 1.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("range({}; {}; {})", x, y, s));
    assert_equal_str("[1 2 3 4]", &calc.ToString());
}
#[test]
fn HPVectorJoin() {
    let mut vector1 = "hp([4; 8; 5; 2; 5; 1])";
    let mut vector2 = "hp([-3.5; 5; 4.25; 0])";
    let mut n = 3.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("join({};{};{})", vector1, vector2, n));
    assert_equal_str("[4 8 5 2 5 1 -3.5 5 4.25 0 3]", &calc.ToString());
}
#[test]
fn HPVectorResize() {
    let mut vector = "hp([0.5; 0.866; 1; -0.5; -0.866; -1; 0])";
    let mut n = 3.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("resize({}; {})", vector, n));
    assert_equal_str("[0.5 0.866 1]", &calc.ToString());
}
#[test]
fn HPVectorLen() {
    let mut vector = "hp([1; 4; 4; 5; 2; 12; 54])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("len({})", vector));
    assert_equal_str("7", &calc.ToString());
}
#[test]
fn HPVectorSize() {
    let mut vector = "hp([1; 4; 4; 5; 2; 12; 54])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("size({})", vector));
    assert_equal_str("7", &calc.ToString());
}
#[test]
fn HPVectorSlice() {
    let mut vector = "hp([-3; 0; 4.5; -7; 2.25])";
    let mut x = "2";
    let mut y = 4.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("slice({};{};{})", vector, x, y));
    assert_equal_str("[0 4.5 -7]", &calc.ToString());
}
#[test]
fn HPVectorFirst() {
    let mut vector = "hp([-0.5; 3; 0; -7.8; 5.6; 4.2; -6; 1.8; -3.1])";
    let mut n = 4.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("first({};{})", vector, n));
    assert_equal_str("[-0.5 3 0 -7.8]", &calc.ToString());
}
#[test]
fn HPVectorLast() {
    let mut vector = "hp([-0.5; 3; 0; -7.8; 5.6; 4.2; -6; 1.8; -3.1])";
    let mut n = 5.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("last({};{})", vector, n));
    assert_equal_str("[5.6 4.2 -6 1.8 -3.1]", &calc.ToString());
}
#[test]
fn HPVectorExtract() {
    let mut vector = "hp([8.2; 0; 6.9; -2.8; 3; 8.2; -7; 4.6])";
    let mut i = "hp([1; 3; 2; 4; 4; 5])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("extract({};{})", vector, i));
    assert_equal_str("[8.2 6.9 0 -2.8 -2.8 3]", &calc.ToString());
}
#[test]
fn HPVectorNorm1() {
    let mut vector = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("norm_1({})", vector));
    assert_equal_str("19.5", &calc.ToString());
}
#[test]
fn HPVectorNorm2() {
    let mut vector = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("norm_2({})", vector));
    assert_equal_str("10.56", &calc.ToString());
}
#[test]
fn HPVectorNormP() {
    let mut vector = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut p = 4.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("norm_p({}; {})", vector, p));
    assert_equal_str("8.32", &calc.ToString());
}
#[test]
fn HPVectorNormInfinity() {
    let mut vector = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("norm_i({})", vector));
    assert_equal_str("7.8", &calc.ToString());
}
#[test]
fn HPVectorUnit() {
    let mut vector = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("unit({})", vector));
    assert_equal_str("[0.474 -0.199 0 0.739 -0.436]", &calc.ToString());
}
#[test]
fn HPVectorDot() {
    let mut vector1 = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut vector2 = "hp([-3.5; 5; 4.25; 0; 10])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("dot({}; {})", vector1, vector2));
    assert_equal_str("-74", &calc.ToString());
}
#[test]
fn HPVectorCross() {
    let mut vector1 = "hp([5; -2.1; 0])";
    let mut vector2 = "hp([4.25; 0; 10])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("cross({}; {})", vector1, vector2));
    assert_equal_str("[-21 -50 8.93]", &calc.ToString());
}
#[test]
fn HPVectorSort() {
    let mut vector = "hp([2.9; -8; 0; 12; -6.5])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sort({})", vector));
    assert_equal_str("[-8 -6.5 0 2.9 12]", &calc.ToString());
}
#[test]
fn HPVectorRsort() {
    let mut vector = "hp([2.9; -8; 0; 12; -6.5])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("rsort({})", vector));
    assert_equal_str("[12 2.9 0 -6.5 -8]", &calc.ToString());
}
#[test]
fn HPVectorOrder() {
    let mut vector = "hp([2.9; -8; 0; 12; -6.5])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("order({})", vector));
    assert_equal_str("[2 5 3 1 4]", &calc.ToString());
}
#[test]
fn HPVectorRevorder() {
    let mut vector = "hp([2.9; -8; 0; 12; -6.5])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("revorder({})", vector));
    assert_equal_str("[4 1 3 5 2]", &calc.ToString());
}
#[test]
fn HPVectorReverse() {
    let mut vector = "hp([2.9; -8; 0; 12; -6.5])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("reverse({})", vector));
    assert_equal_str("[-6.5 12 0 -8 2.9]", &calc.ToString());
}
#[test]
fn HPVectorCount() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut x = "5.25";
    let mut i = 2.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("count({}; {}; {})", vector, x, i));
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn HPVectorSearch() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut x = "2";
    let mut i = 3.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("search({}; {}; {})", vector, x, i));
    assert_equal_str("4", &calc.ToString());
}
#[test]
fn HPVectorFind() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 4; 4; 4])";
    let mut x = "4";
    let mut i = 1.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("find({}; {}; {})", vector, x, i));
    assert_equal_str("[5 6 7]", &calc.ToString());
}
#[test]
fn HPVectorFindNe() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut x = "2";
    let mut i = 3.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("find_ne({}; {}; {})", vector, x, i));
    assert_equal_str("[3 5 6 7]", &calc.ToString());
}
#[test]
fn HPVectorFindLt() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut x = "8";
    let mut i = 1.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("find_lt({}; {}; {})", vector, x, i));
    assert_equal_str("[2 3 4 6 7]", &calc.ToString());
}
#[test]
fn HPVectorFindLe() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut x = "2";
    let mut i = 1.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("find_le({}; {}; {})", vector, x, i));
    assert_equal_str("[2 4]", &calc.ToString());
}
#[test]
fn HPVectorFindGt() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut x = "8";
    let mut i = 1.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("find_gt({}; {}; {})", vector, x, i));
    assert_equal_str("[1]", &calc.ToString());
}
#[test]
fn HPVectorFindGe() {
    let mut vector = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut x = "5.25";
    let mut i = 1.0;
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("find_ge({}; {}; {})", vector, x, i));
    assert_equal_str("[1 3 5]", &calc.ToString());
}
#[test]
fn HPVectorLookup() {
    let mut vector1 = "hp([6.2; 6.2; 0; 6.2; 7.4])";
    let mut vector2 = "hp([5; -2.1; 0; 7.8; 11])";
    let mut x = "6.2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lookup({};{}; {})", vector1, vector2, x));
    assert_equal_str("[5 -2.1 7.8]", &calc.ToString());
}
#[test]
fn HPVectorLookupNe() {
    let mut vector1 = "hp([12; -1.2; 5.25; 2; 8])";
    let mut vector2 = "hp([5; -2.1; 0; 7.8; 11])";
    let mut x = "5.25";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lookup_ne({}; {}; {})", vector1, vector2, x));
    assert_equal_str("[5 -2.1 7.8 11]", &calc.ToString());
}
#[test]
fn HPVectorLookupLt() {
    let mut vector1 = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut vector2 = "hp([1.5; 2.1; 3.2; 4.4; 5.5; 6.9; 7.111])";
    let mut x = "5.25";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lookup_lt({}; {}; {})", vector1, vector2, x));
    assert_equal_str("[2.1 4.4 6.9 7.11]", &calc.ToString());
}
#[test]
fn HPVectorLookupLe() {
    let mut vector1 = "hp([12; -1.2; 5.25; 2; 8])";
    let mut vector2 = "hp([1; 6; 78; -24; 5.3])";
    let mut x = "7";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lookup_le({}; {}; {})", vector1, vector2, x));
    assert_equal_str("[6 78 -24]", &calc.ToString());
}
#[test]
fn HPVectorLookupGt() {
    let mut vector1 = "hp([12; -1.2; 5.25; 2; 8; 3; 4])";
    let mut vector2 = "hp([0; -6.7; 3.8; -2; 9.5; 7.2; 0; -1.3; 5])";
    let mut x = "4";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lookup_gt({}; {}; {})", vector1, vector2, x));
    assert_equal_str("[0 3.8 9.5]", &calc.ToString());
}
#[test]
fn HPVectorLookupGe() {
    let mut vector1 = "hp([12; -1.2; 5.25; 2; 8])";
    let mut vector2 = "hp([5; -2.1; 0; 7.8; -4.6])";
    let mut x = "2";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("lookup_ge({};{};{})", vector1, vector2, x));
    assert_equal_str("[5 0 7.8 -4.6]", &calc.ToString());
}
#[test]
fn HPVectorMin() {
    let mut vector = "hp([77; -24; 1.1; 2.3; -5.6])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("min({})", vector));
    assert_equal_str("-24", &calc.ToString());
}
#[test]
fn HPVectorMax() {
    let mut vector = "hp([12; 44; -27; 56; 199.9; -2.4])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("max({})", vector));
    assert_equal_str("199.9", &calc.ToString());
}
#[test]
fn HPVectorSum() {
    let mut vector = "hp([10.5; 23.2; -5.1; 8.4; 15.3])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sum({})", vector));
    assert_equal_str("52.3", &calc.ToString());
}
#[test]
fn HPVectorSumsq() {
    let mut vector = "hp([1.5; 2.3; -4.4; 6.2; -7.1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("sumsq({})", vector));
    assert_equal_str("115.75", &calc.ToString());
}
#[test]
fn HPVectorSrss() {
    let mut vector = "hp([3; 4; 12])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("srss({})", vector));
    assert_equal_str("13", &calc.ToString());
}
#[test]
fn HPVectorAverage() {
    let mut vector = "hp([5.5; 7.3; -2.8; 4.0; 10.1])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("average({})", vector));
    assert_equal_str("4.82", &calc.ToString());
}
#[test]
fn HPVectorProduct() {
    let mut vector = "hp([1.2; 3.0; -2.0; 4.5; 5.0])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("product({})", vector));
    assert_equal_str("-162", &calc.ToString());
}
#[test]
fn HPVectorMean() {
    let mut vector = "hp([9.1; 15.4; 7.3; 12.0; -3.5])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("Mean({})", vector));
    assert_equal_str(" Undefined ", &calc.ToString());
}
#[test]
fn HPVectorTake() {
    let mut vector = "hp([0; 3.29; 2; -1.5; 7])";
    let mut x = "3.8";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("take({}; {})", x, vector));
    assert_equal_str("-1.5", &calc.ToString());
}
#[test]
fn HPVectorLine() {
    let mut vector = "hp([0; 3.29; 2; -1.5; 7])";
    let mut x = "4.5";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("line({}; {})", x, vector));
    assert_equal_str("2.75", &calc.ToString());
}
#[test]
fn HPVectorSpline() {
    let mut vector = "hp([0; 3.29; 2; -1.5; 7])";
    let mut x = "4.5";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("spline({}; {} )", x, vector));
    assert_equal_str("1.39", &calc.ToString());
}
#[test]
fn HPVectorNot() {
    let mut vector = "hp([1; 0; 22; 3.7; 55; -6.3; 0])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("not({})", vector));
    assert_equal_str("[0 1 0 0 0 0 1]", &calc.ToString());
}
#[test]
fn HPVectorOr() {
    let mut vector = "hp([1; 0; 22; 3.7; 55; -6.3; 0])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("or({})", vector));
    assert_equal_str("1", &calc.ToString());
}
#[test]
fn HPVectorAnd() {
    let mut vector = "hp([1; 0; 22; 3.7; 55; -6.3; 0])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("and({})", vector));
    assert_equal_str("0", &calc.ToString());
}
#[test]
fn HPVectorXor() {
    let mut vector = "hp([1; 0; 22; 3.7; 55; -6.3; 0])";
    let mut calc = TestCalc::new(ms());
    calc.Run(&format!("xor({})", vector));
    assert_equal_str("1", &calc.ToString());
}
