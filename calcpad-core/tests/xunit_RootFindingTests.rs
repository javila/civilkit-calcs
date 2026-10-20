//! Transcribed from Calcpad.Tests/Scalars/RootFindingTests.cs by tools/transcribe_xunit.py.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::identity_op
)]
mod common;
use common::*;

const Tol: f64 = 1e-14;
const P: &str = "p(x) = x + 1.11111";
const _functions: [&str; 62] = [
    "f_01(x) = x^3 - 1",
    "f_02(x) = x^2*(x^2/3 + sqrt(2)*sin(x)) - sqrt(3)/18",
    "f_03(x) = 11*x^11 - 1",
    "f_04(x) = x^3 + 1",
    "f_05(x) = x^3 - 2*x - 5",
    "f_06(x) = 2*x* e^(-5) + 1 - 2*e^(-5*x)",
    "f_07(x) = 2*x* e^(-10) + 1 - 2*e^(-10*x)",
    "f_08(x) = 2*x* e^(-20) + 1 - 2*e^(-20*x)",
    "f_09(x) = (1 + (1 - 5)^2)*x^2 - (1 - 5*x)^2",
    "f_10(x) = (1 + (1 - 10)^2)*x^2 - (1 - 10*x)^2",
    "f_11(x) = (1 + (1 - 20)^2)*x^2 - (1 - 20*x)^2",
    "f_12(x) = x^2 - (1 - x)^5",
    "f_13(x) = x^2 - (1 - x)^10",
    "f_14(x) = x^2 - (1 - x)^20",
    "f_15(x) = (1 + (1 - 5)^4)*x - (1 - 5*x)^4",
    "f_16(x) = (1 + (1 - 10)^4)*x - (1 - 10*x)^4",
    "f_17(x) = (1 + (1 - 20)^4)*x - (1 - 20*x)^4",
    "f_18(x) = e^(-5*x)*(x - 1) + x^5",
    "f_19(x) = e^(-10*x)*(x - 1) + x^10",
    "f_20(x) = e^(-20*x)*(x - 1) + x^20",
    "f_21(x) = x^2 + sin(x/5) - 1/4",
    "f_22(x) = x^2 + sin(x/10) - 1/4",
    "f_23(x) = x^2 + sin(x/20) - 1/4",
    "f_24(x) = (x + 2)*(x + 1)*(x - 3)^3",
    "f_25(x) = (x - 4)^5*ln(x)",
    "f_26(x) = (sin(x) - x/4)^3",
    "f_27(x) = (81 - p(x)*(108 - p(x)*(54 - p(x)*(12 - p(x)))))*sign(p(x) - 3)",
    "f_28(x) = sin((x - 7.143)^3)",
    "f_29(x) = e^((x - 3)^5) - 1",
    "f_30(x) = e^((x - 3)^5) - e^(x - 1)",
    "f_31(x) = π - 1/x",
    "f_32(x) = 4 - tan(x)",
    "f_33(x) = cos(x) - x^3",
    "f_34(x) = cos(x) - x",
    "f_35(x) = sqr(abs(x - 2/3))*if(x ≤ 2/3; 1; -1) - 0.1",
    "f_36(x) = abs(x - 2/3)^0.2*if(x ≤ 2/3; 1; -1)",
    "f_37(x) = (x - 7/9)^3 + (x - 7/9)*10^-3",
    "f_38(x) = if(x ≤ 1/3; -0.5; 0.5)",
    "f_39(x) = if(x ≤ 1/3; -10^-3; 1 - 10^-3)",
    "f_40(x) = if(x ≡ 0; 0; 1/(x - 2/3))",
    "f_41(x) = 2*x* e^-5 - 2*e^(-5*x) + 1",
    "f_42(x) = (x^2 - x - 6)*(x^2 - 3*x + 2)",
    "f_43(x) = x^3",
    "f_44(x) = x^5",
    "f_45(x) = x^7",
    "f_46(x) = (e^(-5*x) - x - 0.5)/x^5",
    "f_47(x) = 1/sqrt(x) - 2*ln(5*10^3*sqrt(x)) + 0.8",
    "f_48(x) = 1/sqrt(x) - 2*ln(5*10^7*sqrt(x)) + 0.8",
    "f_49(x) = if(x ≤ 0; -x^3 - x - 1; x^(1/3) - x - 1)",
    "f_50(x) = x^3 - 2*x - x + 3",
    "f_51(x) = ln(x)",
    "f_52(x) = (10 - x)*e^(-10*x) - x^10 + 1",
    "f_53(x) = e^sin(x) - x - 1",
    "f_54(x) = 2*sin(x) - 1",
    "f_55(x) = (x - 1)*e^-x",
    "f_56(x) = (x - 1)^3 - 1",
    "f_57(x) = e^(x^2 + 7*x - 30) - 1",
    "f_58(x) = atan(x) - 1",
    "f_59(x) = e^x - 2*x - 1",
    "f_60(x) = e^-x - x - sin(x)",
    "f_61(x) = x^2 - sin(x)^2 - 1",
    "f_62(x) = sin(x) - x/2",
];
const _commands: [&str; 62] = [
    "x_01 = $Root{f_01(x) @ x = 0.5 : 1.5}",
    "x_02 = $Root{f_02(x) @ x = 0.1 : 1}",
    "x_03 = $Root{f_03(x) @ x = 0.1 : 1}",
    "x_04 = $Root{f_04(x) @ x = -1.8 : 0}",
    "x_05 = $Root{f_05(x) @ x = 2 : 3}",
    "x_06 = $Root{f_06(x) @ x = 0 : 1}",
    "x_07 = $Root{f_07(x) @ x = 0 : 1}",
    "x_08 = $Root{f_08(x) @ x = 0 : 1}",
    "x_09 = $Root{f_09(x) @ x = 0 : 1}",
    "x_10 = $Root{f_10(x) @ x = 0 : 1}",
    "x_11 = $Root{f_11(x) @ x = 0 : 1}",
    "x_12 = $Root{f_12(x) @ x = 0 : 1}",
    "x_13 = $Root{f_13(x) @ x = 0 : 1}",
    "x_14 = $Root{f_14(x) @ x = 0 : 1}",
    "x_15 = $Root{f_15(x) @ x = 0 : 1}",
    "x_16 = $Root{f_16(x) @ x = 0 : 1}",
    "x_17 = $Root{f_17(x) @ x = 0 : 1}",
    "x_18 = $Root{f_18(x) @ x = 0 : 1}",
    "x_19 = $Root{f_19(x) @ x = 0 : 1}",
    "x_20 = $Root{f_20(x) @ x = 0 : 1}",
    "x_21 = $Root{f_21(x) @ x = 0 : 1}",
    "x_22 = $Root{f_22(x) @ x = 0 : 1}",
    "x_23 = $Root{f_23(x) @ x = 0 : 1}",
    "x_24 = $Root{f_24(x) @ x = 2.6 : 4.6}",
    "x_25 = $Root{f_25(x) @ x = 3.6 : 5.6}",
    "x_26 = $Root{f_26(x) @ x = 2 : 4}",
    "x_27 = $Root{f_27(x) @ x = 1 : 3}",
    "x_28 = $Root{f_28(x) @ x = 7 : 8}",
    "x_29 = $Root{f_29(x) @ x = 2.6 : 3.6}",
    "x_30 = $Root{f_30(x) @ x = 4 : 5}",
    "x_31 = $Root{f_31(x) @ x = 0.05 : 5}",
    "x_32 = $Root{f_32(x) @ x = 0 : 1.5}",
    "x_33 = $Root{f_33(x) @ x = 0 : 4}",
    "x_34 = $Root{f_34(x) @ x = -11 : 9}",
    "x_35 = $Root{f_35(x) @ x = -11 : 9}",
    "x_36 = $Find{f_36(x) @ x = -11 : 9}",
    "x_37 = $Root{f_37(x) @ x = -11 : 9}",
    "x_38 = $Find{f_38(x) @ x = -11 : 9}",
    "x_39 = $Find{f_39(x) @ x = -11 : 9}",
    "x_40 = $Find{f_40(x) @ x = -11 : 9}",
    "x_41 = $Root{f_41(x) @ x = 0 : 10}",
    "x_42 = $Root{f_42(x) @ x = 0 : 4}",
    "x_43 = $Root{f_43(x) @ x = -1 : 1.5}",
    "x_44 = $Root{f_44(x) @ x = -1 : 1.5}",
    "x_45 = $Root{f_45(x) @ x = -1 : 1.5}",
    "x_46 = $Root{f_46(x) @ x = 0.09 : 0.7}",
    "x_47 = $Root{f_47(x) @ x = 0.0005 : 0.5}",
    "x_48 = $Root{f_48(x) @ x = 0.0005 : 0.5}",
    "x_49 = $Root{f_49(x) @ x = -1 : 1}",
    "x_50 = $Root{f_50(x) @ x = -3 : 2}",
    "x_51 = $Root{f_51(x) @ x = 0.5 : 5}",
    "x_52 = $Root{f_52(x) @ x = 0.5 : 8}",
    "x_53 = $Root{f_53(x) @ x = 1.0 : 4}",
    "x_54 = $Root{f_54(x) @ x = 0.1 : π / 3}",
    "x_55 = $Root{f_55(x) @ x = 0.0 : 1.5}",
    "x_56 = $Root{f_56(x) @ x = 1.5 : 3}",
    "x_57 = $Root{f_57(x) @ x = 2.6 : 3.5}",
    "x_58 = $Root{f_58(x) @ x = 1.0 : 8}",
    "x_59 = $Root{f_59(x) @ x = 0.2 : 3}",
    "x_60 = $Root{f_60(x) @ x = 0.0 : 2}",
    "x_61 = $Root{f_61(x) @ x = -1 : 2}",
    "x_62 = $Root{f_62(x) @ x = π / 2 : π}",
];
const _roots: [f64; 62] = [
    1.0000000000000000e+000,
    3.9942229171096816e-001,
    8.0413309750366435e-001,
    -1.0000000000000000e+000,
    2.0945514815423265e+000,
    1.3825715505682409e-001,
    6.9314088687023462e-002,
    3.4657359020853859e-002,
    1.0961179679779243e-001,
    5.2478603436810184e-002,
    2.5623747619988209e-002,
    3.4595481584824200e-001,
    2.4512233375330719e-001,
    1.6492095727644099e-001,
    3.6171081789040638e-003,
    1.5147133478389139e-004,
    7.6685951221853389e-006,
    5.1615351875793358e-001,
    5.3952222690841578e-001,
    5.5270466667848783e-001,
    4.0999201798913715e-001,
    4.5250914557764121e-001,
    4.7562684859606241e-001,
    3.0000000000014557e+000,
    4.0000000953674322e+000,
    2.4745767873682780e+000,
    1.8891601562500000e+000,
    7.1429999999963911e+000,
    3.0003906250000001e+000,
    4.2671683045421238e+000,
    3.1830988618379069e-001,
    1.3258176636680326e+000,
    8.6547403310161442e-001,
    7.3908513321516067e-001,
    6.5666666666666662e-001,
    6.6666666666661423e-001,
    7.7777777777777779e-001,
    3.3333333333332860e-001,
    3.3333333333332860e-001,
    6.6666666666664298e-001,
    1.3825715505682407e-001,
    2.0000000000000000e+000,
    3.6379788070917130e-012,
    -5.9604644775390625e-008,
    3.8146972656250000e-006,
    1.0162439229354953e-001,
    7.7325232006127979e-003,
    1.2763049457355641e-003,
    -6.8232780382801927e-001,
    -2.1038034027355366e+000,
    1.0000000000000000e+000,
    1.0000408355647270e+000,
    1.6968123868097513e+000,
    5.2359877559829882e-001,
    1.0000000000000000e+000,
    2.0000000000000000e+000,
    3.0000000000000000e+000,
    1.5574077246549021e+000,
    1.2564312086261697e+000,
    3.5446310437502532e-001,
    1.4044916482153411e+000,
    1.8954942670339812e+000,
];

#[test]
fn Root_F_01() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[0]);
    let mut result = calc.Run(_commands[0]);
    assert_equal_tol(_roots[0], result, Tol);
    result = calc.Run("f_01(x_01)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_02() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[1]);
    let mut result = calc.Run(_commands[1]);
    assert_equal_tol(_roots[1], result, Tol);
    result = calc.Run("f_02(x_02)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_03() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[2]);
    let mut result = calc.Run(_commands[2]);
    assert_equal_tol(_roots[2], result, Tol);
    result = calc.Run("f_03(x_03)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_04() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[3]);
    let mut result = calc.Run(_commands[3]);
    assert_equal_tol(_roots[3], result, Tol);
    result = calc.Run("f_04(x_04)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_05() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[4]);
    let mut result = calc.Run(_commands[4]);
    assert_equal_tol(_roots[4], result, Tol);
    result = calc.Run("f_05(x_05)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_06() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[5]);
    let mut result = calc.Run(_commands[5]);
    assert_equal_tol(_roots[5], result, Tol);
    result = calc.Run("f_06(x_06)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_07() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[6]);
    let mut result = calc.Run(_commands[6]);
    assert_equal_tol(_roots[6], result, Tol);
    result = calc.Run("f_07(x_07)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_08() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[7]);
    let mut result = calc.Run(_commands[7]);
    assert_equal_tol(_roots[7], result, Tol);
    result = calc.Run("f_08(x_08)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_09() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[8]);
    let mut result = calc.Run(_commands[8]);
    assert_equal_tol(_roots[8], result, Tol);
    result = calc.Run("f_09(x_09)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_10() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[9]);
    let mut result = calc.Run(_commands[9]);
    assert_equal_tol(_roots[9], result, Tol);
    result = calc.Run("f_10(x_10)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_11() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[10]);
    let mut result = calc.Run(_commands[10]);
    assert_equal_tol(_roots[10], result, Tol);
    result = calc.Run("f_11(x_11)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_12() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[11]);
    let mut result = calc.Run(_commands[11]);
    assert_equal_tol(_roots[11], result, Tol);
    result = calc.Run("f_12(x_12)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_13() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[12]);
    let mut result = calc.Run(_commands[12]);
    assert_equal_tol(_roots[12], result, Tol);
    result = calc.Run("f_13(x_13)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_14() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[13]);
    let mut result = calc.Run(_commands[13]);
    assert_equal_tol(_roots[13], result, Tol);
    result = calc.Run("f_14(x_14)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_15() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[14]);
    let mut result = calc.Run(_commands[14]);
    assert_equal_tol(_roots[14], result, Tol);
    result = calc.Run("f_15(x_15)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_16() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[15]);
    let mut result = calc.Run(_commands[15]);
    assert_equal_tol(_roots[15], result, Tol);
    result = calc.Run("f_16(x_16)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_17() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[16]);
    let mut result = calc.Run(_commands[16]);
    assert_equal_tol(_roots[16], result, Tol);
    result = calc.Run("f_17(x_17)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_18() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[17]);
    let mut result = calc.Run(_commands[17]);
    assert_equal_tol(_roots[17], result, Tol);
    result = calc.Run("f_18(x_18)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_19() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[18]);
    let mut result = calc.Run(_commands[18]);
    assert_equal_tol(_roots[18], result, Tol);
    result = calc.Run("f_19(x_19)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_20() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[19]);
    let mut result = calc.Run(_commands[19]);
    assert_equal_tol(_roots[19], result, Tol);
    result = calc.Run("f_20(x_20)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_21() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[20]);
    let mut result = calc.Run(_commands[20]);
    assert_equal_tol(_roots[20], result, Tol);
    result = calc.Run("f_21(x_21)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_22() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[21]);
    let mut result = calc.Run(_commands[21]);
    assert_equal_tol(_roots[21], result, Tol);
    result = calc.Run("f_22(x_22)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_23() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[22]);
    let mut result = calc.Run(_commands[22]);
    assert_equal_tol(_roots[22], result, Tol);
    result = calc.Run("f_23(x_23)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_24() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[23]);
    let mut result = calc.Run(_commands[23]);
    assert_equal_tol(_roots[23], result, Tol);
    result = calc.Run("f_24(x_24)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_25() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[24]);
    let mut result = calc.Run(_commands[24]);
    assert_equal_tol(_roots[24], result, Tol);
    result = calc.Run("f_25(x_25)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_26() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[25]);
    let mut result = calc.Run(_commands[25]);
    assert_equal_tol(_roots[25], result, Tol);
    result = calc.Run("f_26(x_26)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_27() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(P);
    calc.Run(_functions[26]);
    let mut result = calc.Run(_commands[26]);
    assert_equal_tol(_roots[26], result, Tol);
    result = calc.Run("f_27(x_27)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_28() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[27]);
    let mut result = calc.Run(_commands[27]);
    assert_equal_tol(_roots[27], result, Tol);
    result = calc.Run("f_28(x_28)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_29() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[28]);
    let mut result = calc.Run(_commands[28]);
    assert_equal_tol(_roots[28], result, Tol);
    result = calc.Run("f_29(x_29)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_30() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[29]);
    let mut result = calc.Run(_commands[29]);
    assert_equal_tol(_roots[29], result, Tol);
    result = calc.Run("f_30(x_30)");
    assert_equal_tol(0.0, result, Tol * 100.0);
}
#[test]
fn Root_F_31() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[30]);
    let mut result = calc.Run(_commands[30]);
    assert_equal_tol(_roots[30], result, Tol);
    result = calc.Run("f_31(x_31)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_32() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[31]);
    let mut result = calc.Run(_commands[31]);
    assert_equal_tol(_roots[31], result, Tol);
    result = calc.Run("f_32(x_32)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_33() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[32]);
    let mut result = calc.Run(_commands[32]);
    assert_equal_tol(_roots[32], result, Tol);
    result = calc.Run("f_33(x_33)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_34() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[33]);
    let mut result = calc.Run(_commands[33]);
    assert_equal_tol(_roots[33], result, Tol);
    result = calc.Run("f_34(x_34)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_35() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[34]);
    let mut result = calc.Run(_commands[34]);
    assert_equal_tol(_roots[34], result, Tol);
    result = calc.Run("f_35(x_35)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_36() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[35]);
    let mut result = calc.Run(_commands[35]);
    assert_equal_tol(_roots[35], result, Tol);
}
#[test]
fn Root_F_37() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[36]);
    let mut result = calc.Run(_commands[36]);
    assert_equal_tol(_roots[36], result, Tol);
    result = calc.Run("f_37(x_37)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_38() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[37]);
    let mut result = calc.Run(_commands[37]);
    assert_equal_tol(_roots[37], result, Tol);
}
#[test]
fn Root_F_39() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[38]);
    let mut result = calc.Run(_commands[38]);
    assert_equal_tol(_roots[38], result, Tol);
}
#[test]
fn Root_F_40() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[39]);
    let mut result = calc.Run(_commands[39]);
    assert_equal_tol(_roots[39], result, Tol);
}
#[test]
fn Root_F_41() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[40]);
    let mut result = calc.Run(_commands[40]);
    assert_equal_tol(_roots[40], result, Tol);
    result = calc.Run("f_41(x_41)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_42() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[41]);
    let mut result = calc.Run(_commands[41]);
    assert_equal_tol(_roots[41], result, Tol);
    result = calc.Run("f_42(x_42)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_43() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[42]);
    let mut result = calc.Run(_commands[42]);
    assert_equal_tol(_roots[42], result, Tol);
    result = calc.Run("f_43(x_43)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_44() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[43]);
    let mut result = calc.Run(_commands[43]);
    assert_equal_tol(_roots[43], result, Tol);
    result = calc.Run("f_44(x_44)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_45() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[44]);
    let mut result = calc.Run(_commands[44]);
    assert_equal_tol(_roots[44], result, Tol);
    result = calc.Run("f_45(x_45)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_46() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[45]);
    let mut result = calc.Run(_commands[45]);
    assert_equal_tol(_roots[45], result, Tol);
    result = calc.Run("f_46(x_46)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_47() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[46]);
    let mut result = calc.Run(_commands[46]);
    assert_equal_tol(_roots[46], result, Tol);
    result = calc.Run("f_47(x_47)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_48() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[47]);
    let mut result = calc.Run(_commands[47]);
    assert_equal_tol(_roots[47], result, Tol);
    result = calc.Run("f_48(x_48)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_49() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[48]);
    let mut result = calc.Run(_commands[48]);
    assert_equal_tol(_roots[48], result, Tol);
    result = calc.Run("f_49(x_49)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_50() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[49]);
    let mut result = calc.Run(_commands[49]);
    assert_equal_tol(_roots[49], result, Tol);
    result = calc.Run("f_50(x_50)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_51() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[50]);
    let mut result = calc.Run(_commands[50]);
    assert_equal_tol(_roots[50], result, Tol);
    result = calc.Run("f_51(x_51)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_52() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[51]);
    let mut result = calc.Run(_commands[51]);
    assert_equal_tol(_roots[51], result, Tol);
    result = calc.Run("f_52(x_52)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_53() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[52]);
    let mut result = calc.Run(_commands[52]);
    assert_equal_tol(_roots[52], result, Tol);
    result = calc.Run("f_53(x_53)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_54() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[53]);
    let mut result = calc.Run(_commands[53]);
    assert_equal_tol(_roots[53], result, Tol);
    result = calc.Run("f_54(x_54)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_55() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[54]);
    let mut result = calc.Run(_commands[54]);
    assert_equal_tol(_roots[54], result, Tol);
    result = calc.Run("f_55(x_55)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_56() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[55]);
    let mut result = calc.Run(_commands[55]);
    assert_equal_tol(_roots[55], result, Tol);
    result = calc.Run("f_56(x_56)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_57() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[56]);
    let mut result = calc.Run(_commands[56]);
    assert_equal_tol(_roots[56], result, Tol);
    result = calc.Run("f_57(x_57)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_58() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[57]);
    let mut result = calc.Run(_commands[57]);
    assert_equal_tol(_roots[57], result, Tol);
    result = calc.Run("f_58(x_58)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_59() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[58]);
    let mut result = calc.Run(_commands[58]);
    assert_equal_tol(_roots[58], result, Tol);
    result = calc.Run("f_59(x_59)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_60() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[59]);
    let mut result = calc.Run(_commands[59]);
    assert_equal_tol(_roots[59], result, Tol);
    result = calc.Run("f_60(x_60)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_61() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[60]);
    let mut result = calc.Run(_commands[60]);
    assert_equal_tol(_roots[60], result, Tol);
    result = calc.Run("f_61(x_61)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Root_F_62() {
    let mut calc = TestCalc::new(ms().Degrees(1.0));
    calc.Run(_functions[61]);
    let mut result = calc.Run(_commands[61]);
    assert_equal_tol(_roots[61], result, Tol);
    result = calc.Run("f_62(x_62)");
    assert_equal_tol(0.0, result, Tol);
}
