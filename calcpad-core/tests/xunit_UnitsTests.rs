//! Transcribed from Calcpad.Tests/Scalars/UnitsTests.cs by tools/transcribe_xunit.py.
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

#[track_caller]
fn Test_f64(expression: &str, expected: f64) {
    let mut result = TestCalc::new(ms()).Run(expression);
    assert!(Math::Abs(expected - result) < Tol * Math::Abs(expected));
}
#[track_caller]
fn Test_str(expression: &str, expected: &str) {
    let mut calc = TestCalc::new(ms());
    calc.Run(expression);
    let mut result = calc.ToString();
    assert_equal(expected, result);
}

#[test]
fn Test_Percent() {
    Test_f64("1|%", 100.0);
}
#[test]
fn Test_Permile() {
    Test_f64("1|‰", 1000.0);
}
#[test]
fn Test_dag() {
    Test_f64("dag|g", 10.0);
}
#[test]
fn Test_hg() {
    Test_f64("hg|g", 100.0);
}
#[test]
fn Test_kg() {
    Test_f64("kg|g", 1000.0);
}
#[test]
fn Test_t() {
    Test_f64("t|g", 1000000.0);
}
#[test]
fn Test_kt() {
    Test_f64("kt|g", 1000000000.0);
}
#[test]
fn Test_Mt() {
    Test_f64("Mt|g", 1000000000000.0);
}
#[test]
fn Test_Gt() {
    Test_f64("Gt|g", 1e+15);
}
#[test]
fn Test_dg() {
    Test_f64("dg|g", 1e-1);
}
#[test]
fn Test_cg() {
    Test_f64("cg|g", 1e-2);
}
#[test]
fn Test_mg() {
    Test_f64("mg|g", 1e-3);
}
#[test]
fn Test_μg() {
    Test_f64("μg|g", 1e-6);
}
#[test]
fn Test_ng() {
    Test_f64("ng|g", 1e-9);
}
#[test]
fn Test_pg() {
    Test_f64("pg|g", 1e-12);
}
#[test]
fn Test_Da() {
    Test_f64("Da|kg", 1.6605390666050505e-27);
}
#[test]
fn Test_u() {
    Test_f64("u/kg", 1.6605390666050505e-27);
}
#[test]
fn Test_gr() {
    Test_f64("gr|g", 0.06479891);
}
#[test]
fn Test_dr() {
    Test_f64("dr|g", 1.7718451953125);
}
#[test]
fn Test_oz() {
    Test_f64("oz|g", 28.349523125);
}
#[test]
fn Test_lb() {
    Test_f64("lb|g", 453.59237);
}
#[test]
fn Test_lbm() {
    Test_f64("lbm/g", 453.59237);
}
#[test]
fn Test_lb_m() {
    Test_f64("lb_m/g", 453.59237);
}
#[test]
fn Test_kipm() {
    Test_f64("kipm|kg", 453.59237);
}
#[test]
fn Test_kip_m() {
    Test_f64("kip_m|kg", 453.59237);
}
#[test]
fn Test_klbk() {
    Test_f64("klb|kg", 453.59237);
}
#[test]
fn Test_st() {
    Test_f64("st|g", 6350.29318);
}
#[test]
fn Test_qr() {
    Test_f64("qr|g", 12700.58636);
}
#[test]
fn Test_cwt_US() {
    Test_f64("cwt_US|g", 45359.237);
}
#[test]
fn Test_cwt_UK() {
    Test_f64("cwt_UK|g", 50802.34544);
}
#[test]
fn Test_ton_US() {
    Test_f64("ton_US|g", 907184.74);
}
#[test]
fn Test_ton_UK() {
    Test_f64("ton_UK|g", 1016046.9088);
}
#[test]
fn Test_slug() {
    Test_f64("slug|g", 14593.90294);
}
#[test]
fn Test_km() {
    Test_f64("km|m", 1e+3);
}
#[test]
fn Test_dm() {
    Test_f64("dm|m", 1e-1);
}
#[test]
fn Test_cm() {
    Test_f64("cm|m", 1e-2);
}
#[test]
fn Test_mm() {
    Test_f64("mm|m", 1e-3);
}
#[test]
fn Test_μm() {
    Test_f64("μm|m", 1e-6);
}
#[test]
fn Test_nm() {
    Test_f64("nm|m", 1e-9);
}
#[test]
fn Test_pm() {
    Test_f64("pm|m", 1e-12);
}
#[test]
fn Test_AU() {
    Test_f64("AU|m", 149597870700.0);
}
#[test]
fn Test_ly() {
    Test_f64("ly|m", 9460730472580800.0);
}
#[test]
fn Test_th() {
    Test_f64("th|m", 2.54E-05);
}
#[test]
fn Test_in() {
    Test_f64("in|m", 0.0254);
}
#[test]
fn Test_ft() {
    Test_f64("ft|m", 0.3048);
}
#[test]
fn Test_yd() {
    Test_f64("yd|m", 0.9144);
}
#[test]
fn Test_ch() {
    Test_f64("ch|m", 20.1168);
}
#[test]
fn Test_fur() {
    Test_f64("fur|m", 201.168);
}
#[test]
fn Test_mi() {
    Test_f64("mi|m", 1609.344);
}
#[test]
fn Test_ftm() {
    Test_f64("ftm|m", 1.8288);
}
#[test]
fn Test_ftm_US() {
    Test_f64("ftm_US|m", 1.8288);
}
#[test]
fn Test_ftm_UK() {
    Test_f64("ftm_UK|m", 1.852);
}
#[test]
fn Test_cable() {
    Test_f64("cable|m", 182.88);
}
#[test]
fn Test_cable_UK() {
    Test_f64("cable_UK|m", 185.2);
}
#[test]
fn Test_cable_US() {
    Test_f64("cable_US|m", 219.456);
}
#[test]
fn Test_nmi() {
    Test_f64("nmi|m", 1852.0);
}
#[test]
fn Test_li() {
    Test_f64("li|m", 0.201168);
}
#[test]
fn Test_rod() {
    Test_f64("rod|m", 5.0292);
}
#[test]
fn Test_pole() {
    Test_f64("pole|m", 5.0292);
}
#[test]
fn Test_perch() {
    Test_f64("perch|m", 5.0292);
}
#[test]
fn Test_lea() {
    Test_f64("lea|m", 4828.032);
}
#[test]
fn Test_a() {
    Test_f64("a|m^2", 100.0);
}
#[test]
fn Test_daa() {
    Test_f64("daa|m^2", 1000.0);
}
#[test]
fn Test_ha() {
    Test_f64("ha|m^2", 10000.0);
}
#[test]
fn Test_rood() {
    Test_f64("rood|m^2", 1011.7141056);
}
#[test]
fn Test_ac() {
    Test_f64("ac|m^2", 4046.8564224);
}
#[test]
fn Test_L() {
    Test_f64("L|dm^3", 1.0);
}
#[test]
fn Test_dL() {
    Test_f64("dL|L", 0.1);
}
#[test]
fn Test_cL() {
    Test_f64("cL|L", 0.01);
}
#[test]
fn Test_mL() {
    Test_f64("mL|cm^3", 1.0);
}
#[test]
fn Test_μL() {
    Test_f64("μL|mm^3", 1.0);
}
#[test]
fn Test_nL() {
    Test_f64("nL|L", 1e-9);
}
#[test]
fn Test_pL() {
    Test_f64("pL|L", 1e-12);
}
#[test]
fn Test_hL() {
    Test_f64("hL|L", 100.0);
}
#[test]
fn Test_fl_dr_UK() {
    Test_f64("fl_dr_UK|L", 3.5516328125e-3);
}
#[test]
fn Test_fl_oz_UK() {
    Test_f64("fl_oz_UK|L", 0.0284130625);
}
#[test]
fn Test_gi_UK() {
    Test_f64("gi_UK|L", 0.1420653125);
}
#[test]
fn Test_pt_UK() {
    Test_f64("pt_UK|L", 0.56826125);
}
#[test]
fn Test_qt_UK() {
    Test_f64("qt_UK|L", 1.1365225);
}
#[test]
fn Test_gal_UK() {
    Test_f64("gal_UK|L", 4.54609);
}
#[test]
fn Test_bbl_UK() {
    Test_f64("bbl_UK|L", 163.65924);
}
#[test]
fn Test_pk_UK() {
    Test_f64("pk_UK|L", 9.09218);
}
#[test]
fn Test_bu_UK() {
    Test_f64("bu_UK|L", 36.36872);
}
#[test]
fn Test_fl_dr_US() {
    Test_f64("fl_dr_US|L", 3.6966911953125e-3);
}
#[test]
fn Test_fl_oz_US() {
    Test_f64("fl_oz_US|L", 0.0295735295625);
}
#[test]
fn Test_gi_US() {
    Test_f64("gi_US|L", 0.11829411825);
}
#[test]
fn Test_pt_US() {
    Test_f64("pt_US|L", 0.473176473);
}
#[test]
fn Test_qt_US() {
    Test_f64("qt_US|L", 0.946352946);
}
#[test]
fn Test_gal_US() {
    Test_f64("gal_US|L", 3.785411784);
}
#[test]
fn Test_bbl_US() {
    Test_f64("bbl_US|L", 119.240471196);
}
#[test]
fn Test_pt_dry() {
    Test_f64("pt_dry|L", 0.5506104713575);
}
#[test]
fn Test_qt_dry() {
    Test_f64("qt_dry|L", 1.101220942715);
}
#[test]
fn Test_gal_dry() {
    Test_f64("gal_dry|L", 4.40488377086);
}
#[test]
fn Test_bbl_dry() {
    Test_f64("bbl_dry|L", 115.628198985075);
}
#[test]
fn Test_pk_US() {
    Test_f64("pk_US|L", 8.80976754172);
}
#[test]
fn Test_bu_US() {
    Test_f64("bu_US|L", 35.23907016688);
}
#[test]
fn Test_ms() {
    Test_f64("ms|s", 1e-3);
}
#[test]
fn Test_μs() {
    Test_f64("μs|s", 1e-6);
}
#[test]
fn Test_ns() {
    Test_f64("ns|s", 1e-9);
}
#[test]
fn Test_ps() {
    Test_f64("ps|s", 1e-12);
}
#[test]
fn Test_min() {
    Test_f64("min|s", 60.0);
}
#[test]
fn Test_h() {
    Test_f64("h|min", 60.0);
}
#[test]
fn Test_d() {
    Test_f64("d|h", 24.0);
}
#[test]
fn Test_w() {
    Test_f64("w|h", 7.0 * 24.0);
}
#[test]
fn Test_y() {
    Test_f64("y|h", 365.0 * 24.0);
}
#[test]
fn Test_kmh() {
    Test_f64("kmh|km/h", 1.0);
}
#[test]
fn Test_mph() {
    Test_f64("mph|mi/h", 1.0);
}
#[test]
fn Test_knot() {
    Test_f64("knot|nmi/h", 1.0);
}
#[test]
fn Test_Hz() {
    Test_f64("Hz|s^-1", 1.0);
}
#[test]
fn Test_kHz() {
    Test_f64("kHz|Hz", 1e+3);
}
#[test]
fn Test_MHz() {
    Test_f64("MHz|Hz", 1e+6);
}
#[test]
fn Test_GHz() {
    Test_f64("GHz|Hz", 1e+9);
}
#[test]
fn Test_THz() {
    Test_f64("THz|Hz", 1e+12);
}
#[test]
fn Test_mHz() {
    Test_f64("mHz|Hz", 1e-3);
}
#[test]
fn Test_μHz() {
    Test_f64("μHz|Hz", 1e-6);
}
#[test]
fn Test_nHz() {
    Test_f64("nHz|Hz", 1e-9);
}
#[test]
fn Test_pHz() {
    Test_f64("pHz|Hz", 1e-12);
}
#[test]
fn Test_rpm() {
    Test_f64("rpm|Hz", 1.0 / 60.0);
}
#[test]
fn Test_kA() {
    Test_f64("kA|A", 1e+3);
}
#[test]
fn Test_MA() {
    Test_f64("MA|A", 1e+6);
}
#[test]
fn Test_GA() {
    Test_f64("GA|A", 1e+9);
}
#[test]
fn Test_TA() {
    Test_f64("TA|A", 1e+12);
}
#[test]
fn Test_mA() {
    Test_f64("mA|A", 1e-3);
}
#[test]
fn Test_μA() {
    Test_f64("μA|A", 1e-6);
}
#[test]
fn Test_nA() {
    Test_f64("nA|A", 1e-9);
}
#[test]
fn Test_pA() {
    Test_f64("pA|A", 1e-12);
}
#[test]
fn Test_Ah() {
    Test_f64("Ah|A*h", 1.0);
}
#[test]
fn Test_mAh() {
    Test_f64("mAh|Ah", 1e-3);
}
#[test]
fn Test_K_C() {
    Test_f64("K|°C", 1.0 - 273.15);
}
#[test]
fn Test_F_C() {
    Test_f64("°F|°C", (1.0 - 32.0) * 5.0 / 9.0);
}
#[test]
fn Test_R_C() {
    Test_f64("°R|°C", 5.0 / 9.0 - 273.15);
}
#[test]
fn Test_ΔC_C() {
    Test_f64("Δ°C|°C", 1.0);
}
#[test]
fn Test_ΔF_C() {
    Test_f64("Δ°F|°C", 5.0 / 9.0);
}
#[test]
fn Test_C_K() {
    Test_f64("°C|K", 1.0 + 273.15);
}
#[test]
fn Test_F_K() {
    Test_f64("°F|K", (1.0 + 459.67) * 5.0 / 9.0);
}
#[test]
fn Test_R_K() {
    Test_f64("°R|K", 5.0 / 9.0);
}
#[test]
fn Test_ΔC_K() {
    Test_f64("Δ°C|K", 1.0);
}
#[test]
fn Test_ΔF_K() {
    Test_f64("Δ°F|K", 5.0 / 9.0);
}
#[test]
fn Test_C_F() {
    Test_f64("°C|°F", 9.0 / 5.0 + 32.0);
}
#[test]
fn Test_K_F() {
    Test_f64("K|°F", 9.0 / 5.0 - 459.67);
}
#[test]
fn Test_R_F() {
    Test_f64("°R|°F", 1.0 - 459.67);
}
#[test]
fn Test_ΔC_F() {
    Test_f64("Δ°C|°F", 9.0 / 5.0);
}
#[test]
fn Test_ΔF_F() {
    Test_f64("Δ°F|°F", 1.0);
}
#[test]
fn Test_C_R() {
    Test_f64("°C|°R", 9.0 / 5.0 + 491.67);
}
#[test]
fn Test_K_R() {
    Test_f64("K|°R", 9.0 / 5.0);
}
#[test]
fn Test_F_R() {
    Test_f64("°F|°R", 1.0 + 459.67);
}
#[test]
fn Test_ΔC_R() {
    Test_f64("Δ°C|°R", 9.0 / 5.0);
}
#[test]
fn Test_ΔF_R() {
    Test_f64("Δ°F|°R", 1.0);
}
#[test]
fn Test_C_ΔC() {
    Test_f64("°C|Δ°C", 1.0);
}
#[test]
fn Test_K_ΔC() {
    Test_f64("K|Δ°C", 1.0);
}
#[test]
fn Test_F_ΔC() {
    Test_f64("°F|Δ°C", 5.0 / 9.0);
}
#[test]
fn Test_R_ΔC() {
    Test_f64("°R|Δ°C", 5.0 / 9.0);
}
#[test]
fn Test_ΔF_ΔC() {
    Test_f64("Δ°F|Δ°C", 5.0 / 9.0);
}
#[test]
fn Test_C_ΔF() {
    Test_f64("°C|Δ°F", 9.0 / 5.0);
}
#[test]
fn Test_K_ΔF() {
    Test_f64("K|Δ°F", 9.0 / 5.0);
}
#[test]
fn Test_F_ΔF() {
    Test_f64("°F|Δ°F", 1.0);
}
#[test]
fn Test_R_ΔF() {
    Test_f64("°R|Δ°F", 1.0);
}
#[test]
fn Test_ΔC_ΔF() {
    Test_f64("Δ°C|Δ°F", 9.0 / 5.0);
}
#[test]
fn Test_N() {
    Test_f64("N|kg*m/s^2", 1.0);
}
#[test]
fn Test_daN() {
    Test_f64("daN|N", 1e+1);
}
#[test]
fn Test_hN() {
    Test_f64("hN|N", 1e+2);
}
#[test]
fn Test_kN() {
    Test_f64("kN|N", 1e+3);
}
#[test]
fn Test_MN() {
    Test_f64("MN|N", 1e+6);
}
#[test]
fn Test_GN() {
    Test_f64("GN|N", 1e+9);
}
#[test]
fn Test_TN() {
    Test_f64("TN|N", 1e+12);
}
#[test]
fn Test_Nm() {
    Test_f64("Nm|N*m", 1.0);
}
#[test]
fn Test_kNm() {
    Test_f64("kNm|kN*m", 1.0);
}
#[test]
fn Test_gf() {
    Test_f64("gf|N", 0.00980665);
}
#[test]
fn Test_kgf() {
    Test_f64("kgf|N", 9.80665);
}
#[test]
fn Test_tf() {
    Test_f64("tf|N", 9806.65);
}
#[test]
fn Test_dyn() {
    Test_f64("dyn|N", 1e-5);
}
#[test]
fn Test_ozf() {
    Test_f64("ozf|N", 0.278013851);
}
#[test]
fn Test_oz_f() {
    Test_f64("oz_f|N", 0.278013851);
}
#[test]
fn Test_lbf() {
    Test_f64("lbf|N", 4.4482216153);
}
#[test]
fn Test_lb_f() {
    Test_f64("lb_f|N", 4.4482216153);
}
#[test]
fn Test_kip() {
    Test_f64("kip|N", 4448.2216153);
}
#[test]
fn Test_kipf() {
    Test_f64("kipf|N", 4448.2216153);
}
#[test]
fn Test_kip_f() {
    Test_f64("kip_f|N", 4448.2216153);
}
#[test]
fn Test_tonf_US() {
    Test_f64("tonf_US|N", 8896.443230521);
}
#[test]
fn Test_tonf_UK() {
    Test_f64("tonf_UK|N", 9964.01641818352);
}
#[test]
fn Test_pdl() {
    Test_f64("pdl|N", 0.138254954376);
}
#[test]
fn Test_Pa() {
    Test_f64("Pa|N/m^2", 1.0);
}
#[test]
fn Test_daPa() {
    Test_f64("daPa|Pa", 1e+1);
}
#[test]
fn Test_hPa() {
    Test_f64("hPa|Pa", 1e+2);
}
#[test]
fn Test_kPa() {
    Test_f64("kPa|Pa", 1e+3);
}
#[test]
fn Test_MPa() {
    Test_f64("MPa|Pa", 1e+6);
}
#[test]
fn Test_GPa() {
    Test_f64("GPa|Pa", 1e+9);
}
#[test]
fn Test_TPa() {
    Test_f64("TPa|Pa", 1e+12);
}
#[test]
fn Test_dPa() {
    Test_f64("dPa|Pa", 1e-1);
}
#[test]
fn Test_cPa() {
    Test_f64("cPa|Pa", 1e-2);
}
#[test]
fn Test_mPa() {
    Test_f64("mPa|Pa", 1e-3);
}
#[test]
fn Test_μPa() {
    Test_f64("μPa|Pa", 1e-6);
}
#[test]
fn Test_nPa() {
    Test_f64("nPa|Pa", 1e-9);
}
#[test]
fn Test_pPa() {
    Test_f64("pPa|Pa", 1e-12);
}
#[test]
fn Test_bar() {
    Test_f64("bar|Pa", 100000.0);
}
#[test]
fn Test_mbar() {
    Test_f64("mbar|Pa", 100.0);
}
#[test]
fn Test_μbar() {
    Test_f64("μbar|Pa", 0.1);
}
#[test]
fn Test_atm() {
    Test_f64("atm|Pa", 101325.0);
}
#[test]
fn Test_mmHg() {
    Test_f64("mmHg|Pa", 133.322387415);
}
#[test]
fn Test_at() {
    Test_f64("at|Pa", 98066.5);
}
#[test]
fn Test_Torr() {
    Test_f64("Torr|Pa", 133.32236842);
}
#[test]
fn Test_osi() {
    Test_f64("osi|Pa", 430.922330894662);
}
#[test]
fn Test_osf() {
    Test_f64("osf|Pa", 2.99251618676848);
}
#[test]
fn Test_psi() {
    Test_f64("psi|Pa", 6894.75729322959);
}
#[test]
fn Test_ksi() {
    Test_f64("ksi|Pa", 6894757.29322959);
}
#[test]
fn Test_tsi() {
    Test_f64("tsi|Pa", 15444256.3366971);
}
#[test]
fn Test_psf() {
    Test_f64("psf|Pa", 47.880258980761);
}
#[test]
fn Test_ksf() {
    Test_f64("ksf|Pa", 47880.258980761);
}
#[test]
fn Test_tsf() {
    Test_f64("tsf|Pa", 107251.780115952);
}
#[test]
fn Test_inHg() {
    Test_f64("inHg|Pa", 3386.389);
}
#[test]
fn Test_P() {
    Test_f64("P|Pa*s", 0.1);
}
#[test]
fn Test_cP() {
    Test_f64("cP|Pa*s", 0.001);
}
#[test]
fn Test_St() {
    Test_f64("St|m^2/s", 0.0001);
}
#[test]
fn Test_cSt() {
    Test_f64("cSt|m^2/s", 0.000001);
}
#[test]
fn Test_J() {
    Test_f64("J|kg*m^2*s^-2", 1.0);
}
#[test]
fn Test_kJ() {
    Test_f64("kJ|J", 1e+3);
}
#[test]
fn Test_MJ() {
    Test_f64("MJ|J", 1e+6);
}
#[test]
fn Test_GJ() {
    Test_f64("GJ|J", 1e+9);
}
#[test]
fn Test_TJ() {
    Test_f64("TJ|J", 1e+12);
}
#[test]
fn Test_mJ() {
    Test_f64("mJ|J", 1e-3);
}
#[test]
fn Test_μJ() {
    Test_f64("μJ|J", 1e-6);
}
#[test]
fn Test_nJ() {
    Test_f64("nJ|J", 1e-9);
}
#[test]
fn Test_pJ() {
    Test_f64("pJ|J", 1e-12);
}
#[test]
fn Test_Wh() {
    Test_f64("Wh|J", 3600.0);
}
#[test]
fn Test_kWh() {
    Test_f64("kWh|J", 3600000.0);
}
#[test]
fn Test_MWh() {
    Test_f64("MWh|J", 3600000000.0);
}
#[test]
fn Test_GWh() {
    Test_f64("GWh|J", 3600000000000.0);
}
#[test]
fn Test_TWh() {
    Test_f64("TWh|J", 3.6e+15);
}
#[test]
fn Test_VAh() {
    Test_f64("VAh|J", 3600.0);
}
#[test]
fn Test_kVAh() {
    Test_f64("kVAh|J", 3600000.0);
}
#[test]
fn Test_MVAh() {
    Test_f64("MVAh|J", 3600000000.0);
}
#[test]
fn Test_GVAh() {
    Test_f64("GVAh|J", 3600000000000.0);
}
#[test]
fn Test_VARh() {
    Test_f64("VARh|J", 3600.0);
}
#[test]
fn Test_kVARh() {
    Test_f64("kVARh|J", 3600000.0);
}
#[test]
fn Test_MVARh() {
    Test_f64("MVARh|J", 3600000000.0);
}
#[test]
fn Test_GVARh() {
    Test_f64("GVARh|J", 3600000000000.0);
}
#[test]
fn Test_erg() {
    Test_f64("erg|J", 1e-7);
}
#[test]
fn Test_eV() {
    Test_f64("eV|J", 1.6021773300241367e-19);
}
#[test]
fn Test_keV() {
    Test_f64("keV|J", 1.6021773300241367e-16);
}
#[test]
fn Test_MeV() {
    Test_f64("MeV|J", 1.6021773300241367e-13);
}
#[test]
fn Test_GeV() {
    Test_f64("GeV|J", 1.6021773300241367e-10);
}
#[test]
fn Test_TeV() {
    Test_f64("TeV|J", 1.6021773300241367e-7);
}
#[test]
fn Test_PeV() {
    Test_f64("PeV|J", 1.6021773300241367e-4);
}
#[test]
fn Test_EeV() {
    Test_f64("EeV|J", 1.6021773300241367e-1);
}
#[test]
fn Test_BTU() {
    Test_f64("BTU|J", 1055.05585262);
}
#[test]
fn Test_therm_US() {
    Test_f64("therm_US|J", 1054.804e+5);
}
#[test]
fn Test_therm_UK() {
    Test_f64("therm_UK|J", 1055.05585262e+5);
}
#[test]
fn Test_quad() {
    Test_f64("quad|J", 1055.05585262e+15);
}
#[test]
fn Test_cal() {
    Test_f64("cal|J", 4.1868);
}
#[test]
fn Test_kcal() {
    Test_f64("kcal|J", 4186.8);
}
#[test]
fn Test_W() {
    Test_f64("W|kg*m^2*s^-3", 1.0);
}
#[test]
fn Test_kW() {
    Test_f64("kW|W", 1e+3);
}
#[test]
fn Test_MW() {
    Test_f64("MW|W", 1e+6);
}
#[test]
fn Test_GW() {
    Test_f64("GW|W", 1e+9);
}
#[test]
fn Test_TW() {
    Test_f64("TW|W", 1e+12);
}
#[test]
fn Test_mW() {
    Test_f64("mW|W", 1e-3);
}
#[test]
fn Test_μW() {
    Test_f64("μW|W", 1e-6);
}
#[test]
fn Test_nW() {
    Test_f64("nW|W", 1e-9);
}
#[test]
fn Test_pW() {
    Test_f64("pW|W", 1e-12);
}
#[test]
fn Test_hp() {
    Test_f64("hp|W", 745.69987158227022);
}
#[test]
fn Test_hp_M() {
    Test_f64("hp_M|W", 735.49875);
}
#[test]
fn Test_ks() {
    Test_f64("ks|W", 735.49875);
}
#[test]
fn Test_hp_E() {
    Test_f64("hp_E|W", 746.0);
}
#[test]
fn Test_hp_S() {
    Test_f64("hp_S|W", 9812.5);
}
#[test]
fn Test_VA() {
    Test_f64("VA|W", 1.0);
}
#[test]
fn Test_kVA() {
    Test_f64("kVA|W", 1e+3);
}
#[test]
fn Test_MVA() {
    Test_f64("MVA|W", 1e+6);
}
#[test]
fn Test_GVA() {
    Test_f64("GVA|W", 1e+9);
}
#[test]
fn Test_TVA() {
    Test_f64("TVA|W", 1e+12);
}
#[test]
fn Test_mVA() {
    Test_f64("mVA|W", 1e-3);
}
#[test]
fn Test_μVA() {
    Test_f64("μVA|W", 1e-6);
}
#[test]
fn Test_nVA() {
    Test_f64("nVA|W", 1e-9);
}
#[test]
fn Test_pVA() {
    Test_f64("pVA|W", 1e-12);
}
#[test]
fn Test_VAR() {
    Test_f64("VAR|W", 1.0);
}
#[test]
fn Test_kVAR() {
    Test_f64("kVAR|W", 1e+3);
}
#[test]
fn Test_MVAR() {
    Test_f64("MVAR|W", 1e+6);
}
#[test]
fn Test_GVAR() {
    Test_f64("GVAR|W", 1e+9);
}
#[test]
fn Test_TVAR() {
    Test_f64("TVAR|W", 1e+12);
}
#[test]
fn Test_mVAR() {
    Test_f64("mVAR|W", 1e-3);
}
#[test]
fn Test_μVAR() {
    Test_f64("μVAR|W", 1e-6);
}
#[test]
fn Test_nVAR() {
    Test_f64("nVAR|W", 1e-9);
}
#[test]
fn Test_pVAR() {
    Test_f64("pVAR|W", 1e-12);
}
#[test]
fn Test_C() {
    Test_f64("C|s*A", 1.0);
}
#[test]
fn Test_kC() {
    Test_f64("kC|C", 1e+3);
}
#[test]
fn Test_MC() {
    Test_f64("MC|C", 1e+6);
}
#[test]
fn Test_GC() {
    Test_f64("GC|C", 1e+9);
}
#[test]
fn Test_TC() {
    Test_f64("TC|C", 1e+12);
}
#[test]
fn Test_mC() {
    Test_f64("mC|C", 1e-3);
}
#[test]
fn Test_μC() {
    Test_f64("μC|C", 1e-6);
}
#[test]
fn Test_nC() {
    Test_f64("nC|C", 1e-9);
}
#[test]
fn Test_pC() {
    Test_f64("pC|C", 1e-12);
}
#[test]
fn Test_V() {
    Test_f64("V|kg*m^2*s^-3*A^-1", 1.0);
}
#[test]
fn Test_kV() {
    Test_f64("kV|V", 1e+3);
}
#[test]
fn Test_MV() {
    Test_f64("MV|V", 1e+6);
}
#[test]
fn Test_GV() {
    Test_f64("GV|V", 1e+9);
}
#[test]
fn Test_TV() {
    Test_f64("TV|V", 1e+12);
}
#[test]
fn Test_mV() {
    Test_f64("mV|V", 1e-3);
}
#[test]
fn Test_μV() {
    Test_f64("μV|V", 1e-6);
}
#[test]
fn Test_nV() {
    Test_f64("nV|V", 1e-9);
}
#[test]
fn Test_pV() {
    Test_f64("pV|V", 1e-12);
}
#[test]
fn Test_F() {
    Test_f64("F|kg^-1*m^-2*s^4*A^2", 1.0);
}
#[test]
fn Test_kF() {
    Test_f64("kF|F", 1e+3);
}
#[test]
fn Test_MF() {
    Test_f64("MF|F", 1e+6);
}
#[test]
fn Test_GF() {
    Test_f64("GF|F", 1e+9);
}
#[test]
fn Test_TF() {
    Test_f64("TF|F", 1e+12);
}
#[test]
fn Test_mF() {
    Test_f64("mF|F", 1e-3);
}
#[test]
fn Test_μF() {
    Test_f64("μF|F", 1e-6);
}
#[test]
fn Test_nF() {
    Test_f64("nF|F", 1e-9);
}
#[test]
fn Test_pF() {
    Test_f64("pF|F", 1e-12);
}
#[test]
fn Test_Ω() {
    Test_f64("Ω|kg*m^2*s^-3*A^-2", 1.0);
}
#[test]
fn Test_kΩ() {
    Test_f64("kΩ|Ω", 1e+3);
}
#[test]
fn Test_MΩ() {
    Test_f64("MΩ|Ω", 1e+6);
}
#[test]
fn Test_GΩ() {
    Test_f64("GΩ|Ω", 1e+9);
}
#[test]
fn Test_TΩ() {
    Test_f64("TΩ|Ω", 1e+12);
}
#[test]
fn Test_mΩ() {
    Test_f64("mΩ|Ω", 1e-3);
}
#[test]
fn Test_μΩ() {
    Test_f64("μΩ|Ω", 1e-6);
}
#[test]
fn Test_nΩ() {
    Test_f64("nΩ|Ω", 1e-9);
}
#[test]
fn Test_pΩ() {
    Test_f64("pΩ|Ω", 1e-12);
}
#[test]
fn Test_S() {
    Test_f64("S|kg^-1*m^-2*s^3*A^2", 1.0);
}
#[test]
fn Test_kS() {
    Test_f64("kS|S", 1e+3);
}
#[test]
fn Test_MS() {
    Test_f64("MS|S", 1e+6);
}
#[test]
fn Test_GS() {
    Test_f64("GS|S", 1e+9);
}
#[test]
fn Test_TS() {
    Test_f64("TS|S", 1e+12);
}
#[test]
fn Test_mS() {
    Test_f64("mS|S", 1e-3);
}
#[test]
fn Test_μS() {
    Test_f64("μS|S", 1e-6);
}
#[test]
fn Test_nS() {
    Test_f64("nS|S", 1e-9);
}
#[test]
fn Test_pS() {
    Test_f64("pS|S", 1e-12);
}
#[test]
fn Test_Mho() {
    Test_f64("℧|S", 1.0);
}
#[test]
fn Test_kMho() {
    Test_f64("k℧|S", 1e+3);
}
#[test]
fn Test_MMho() {
    Test_f64("M℧|S", 1e+6);
}
#[test]
fn Test_GMho() {
    Test_f64("G℧|S", 1e+9);
}
#[test]
fn Test_TMho() {
    Test_f64("T℧|S", 1e+12);
}
#[test]
fn Test_mMho() {
    Test_f64("m℧|S", 1e-3);
}
#[test]
fn Test_μMho() {
    Test_f64("μ℧|S", 1e-6);
}
#[test]
fn Test_nMho() {
    Test_f64("n℧|S", 1e-9);
}
#[test]
fn Test_pMho() {
    Test_f64("p℧|S", 1e-12);
}
#[test]
fn Test_Wb() {
    Test_f64("Wb|kg*m^2*s^-2*A^-1", 1.0);
}
#[test]
fn Test_kWb() {
    Test_f64("kWb|Wb", 1e+3);
}
#[test]
fn Test_MWb() {
    Test_f64("MWb|Wb", 1e+6);
}
#[test]
fn Test_GWb() {
    Test_f64("GWb|Wb", 1e+9);
}
#[test]
fn Test_TWb() {
    Test_f64("TWb|Wb", 1e+12);
}
#[test]
fn Test_mWb() {
    Test_f64("mWb|Wb", 1e-3);
}
#[test]
fn Test_μWb() {
    Test_f64("μWb|Wb", 1e-6);
}
#[test]
fn Test_nWb() {
    Test_f64("nWb|Wb", 1e-9);
}
#[test]
fn Test_pWb() {
    Test_f64("pWb|Wb", 1e-12);
}
#[test]
fn Test_T() {
    Test_f64("T|kg*s^-2*A^-1", 1.0);
}
#[test]
fn Test_kT() {
    Test_f64("kT|T", 1e+3);
}
#[test]
fn Test_MT() {
    Test_f64("MT|T", 1e+6);
}
#[test]
fn Test_GT() {
    Test_f64("GT|T", 1e+9);
}
#[test]
fn Test_TT() {
    Test_f64("TT|T", 1e+12);
}
#[test]
fn Test_mT() {
    Test_f64("mT|T", 1e-3);
}
#[test]
fn Test_μT() {
    Test_f64("μT|T", 1e-6);
}
#[test]
fn Test_nT() {
    Test_f64("nT|T", 1e-9);
}
#[test]
fn Test_pT() {
    Test_f64("pT|T", 1e-12);
}
#[test]
fn Test_H() {
    Test_f64("H|kg*m^2*s^-2*A^-2", 1.0);
}
#[test]
fn Test_kH() {
    Test_f64("kH|H", 1e+3);
}
#[test]
fn Test_MH() {
    Test_f64("MH|H", 1e+6);
}
#[test]
fn Test_GH() {
    Test_f64("GH|H", 1e+9);
}
#[test]
fn Test_TH() {
    Test_f64("TH|H", 1e+12);
}
#[test]
fn Test_mH() {
    Test_f64("mH|H", 1e-3);
}
#[test]
fn Test_μH() {
    Test_f64("μH|H", 1e-6);
}
#[test]
fn Test_nH() {
    Test_f64("nH|H", 1e-9);
}
#[test]
fn Test_pH() {
    Test_f64("pH|H", 1e-12);
}
#[test]
fn Test_Bq() {
    Test_f64("Bq|s^-1", 1.0);
}
#[test]
fn Test_kBq() {
    Test_f64("kBq|Bq", 1e+3);
}
#[test]
fn Test_MBq() {
    Test_f64("MBq|Bq", 1e+6);
}
#[test]
fn Test_GBq() {
    Test_f64("GBq|Bq", 1e+9);
}
#[test]
fn Test_TBq() {
    Test_f64("TBq|Bq", 1e+12);
}
#[test]
fn Test_mBq() {
    Test_f64("mBq|Bq", 1e-3);
}
#[test]
fn Test_μBq() {
    Test_f64("μBq|Bq", 1e-6);
}
#[test]
fn Test_nBq() {
    Test_f64("nBq|Bq", 1e-9);
}
#[test]
fn Test_pBq() {
    Test_f64("pBq|Bq", 1e-12);
}
#[test]
fn Test_Ci() {
    Test_f64("Ci|Bq", 3.7e+10);
}
#[test]
fn Test_Rd() {
    Test_f64("Rd|Bq", 1e+6);
}
#[test]
fn Test_Gy() {
    Test_f64("Gy|m^2*s^-2", 1.0);
}
#[test]
fn Test_kGy() {
    Test_f64("kGy|Gy", 1e+3);
}
#[test]
fn Test_MGy() {
    Test_f64("MGy|Gy", 1e+6);
}
#[test]
fn Test_GGy() {
    Test_f64("GGy|Gy", 1e+9);
}
#[test]
fn Test_TGy() {
    Test_f64("TGy|Gy", 1e+12);
}
#[test]
fn Test_mGy() {
    Test_f64("mGy|Gy", 1e-3);
}
#[test]
fn Test_μGy() {
    Test_f64("μGy|Gy", 1e-6);
}
#[test]
fn Test_nGy() {
    Test_f64("nGy|Gy", 1e-9);
}
#[test]
fn Test_pGy() {
    Test_f64("pGy|Gy", 1e-12);
}
#[test]
fn Test_Sv() {
    Test_f64("Sv|m^2*s^-2", 1.0);
}
#[test]
fn Test_kSv() {
    Test_f64("kSv|Sv", 1e+3);
}
#[test]
fn Test_MSv() {
    Test_f64("MSv|Sv", 1e+6);
}
#[test]
fn Test_GSv() {
    Test_f64("GSv|Sv", 1e+9);
}
#[test]
fn Test_TSv() {
    Test_f64("TSv|Sv", 1e+12);
}
#[test]
fn Test_mSv() {
    Test_f64("mSv|Sv", 1e-3);
}
#[test]
fn Test_μSv() {
    Test_f64("μSv|Sv", 1e-6);
}
#[test]
fn Test_nSv() {
    Test_f64("nSv|Sv", 1e-9);
}
#[test]
fn Test_pSv() {
    Test_f64("pSv|Sv", 1e-12);
}
#[test]
fn Test_lx() {
    Test_f64("lx|lm/m^2", 1.0);
}
#[test]
fn Test_kat() {
    Test_f64("kat|mol/s", 1.0);
}
#[test]
fn Test_deg() {
    Test_f64("deg|°", 1.0);
}
#[test]
fn Test_ang_min() {
    Test_f64("′|°", 1.0 / 60.0);
}
#[test]
fn Test_ang_sec() {
    Test_f64("″|°", 1.0 / 3600.0);
}
#[test]
fn Test_rad() {
    Test_f64("rad|°", 180.0 / Math::PI);
}
#[test]
fn Test_grad() {
    Test_f64("grad|°", 0.9);
}
#[test]
fn Test_rev() {
    Test_f64("rev|°", 360.0);
}
#[test]
fn Test_bbl_UK_to_gal_UK() {
    Test_f64("bbl_UK|gal_UK", 36.0);
}
#[test]
fn Test_bbl_UK_to_qt_UK() {
    Test_f64("bbl_UK|qt_UK", 144.0);
}
#[test]
fn Test_bbl_UK_to_pt_UK() {
    Test_f64("bbl_UK|pt_UK", 288.0);
}
#[test]
fn Test_bbl_UK_to_gi_UK() {
    Test_f64("bbl_UK|gi_UK", 1152.0);
}
#[test]
fn Test_bbl_UK_to_fl_oz_UK() {
    Test_f64("bbl_UK|fl_oz_UK", 5760.0);
}
#[test]
fn Test_bbl_UK_to_fl_dr_UK() {
    Test_f64("bbl_UK|fl_dr_UK", 46080.0);
}
#[test]
fn Test_fl_oz_UK_to_fl_dr_UK() {
    Test_f64("fl_oz_UK|fl_dr_UK", 8.0);
}
#[test]
fn Test_gi_UK_to_fl_oz_UK() {
    Test_f64("gi_UK|fl_oz_UK", 5.0);
}
#[test]
fn Test_pt_UK_to_gi_UK() {
    Test_f64("pt_UK|gi_UK", 4.0);
}
#[test]
fn Test_qt_UK_to_pt_UK() {
    Test_f64("qt_UK|pt_UK", 2.0);
}
#[test]
fn Test_gal_UK_to_qt_UK() {
    Test_f64("gal_UK|qt_UK", 4.0);
}
#[test]
fn Test_bbl_US_to_gal_US() {
    Test_f64("bbl_US|gal_US", 31.5);
}
#[test]
fn Test_bbl_US_to_qt_US() {
    Test_f64("bbl_US|qt_US", 126.0);
}
#[test]
fn Test_bbl_US_to_pt_US() {
    Test_f64("bbl_US|pt_US", 252.0);
}
#[test]
fn Test_bbl_US_to_gi_US() {
    Test_f64("bbl_US|gi_US", 1008.0);
}
#[test]
fn Test_bbl_US_to_fl_oz_US() {
    Test_f64("bbl_US|fl_oz_US", 4032.0);
}
#[test]
fn Test_bbl_US_to_fl_dr_US() {
    Test_f64("bbl_US|fl_dr_US", 32256.0);
}
#[test]
fn Test_fl_oz_US_to_fl_dr_US() {
    Test_f64("fl_oz_US|fl_dr_US", 8.0);
}
#[test]
fn Test_gi_US_to_fl_oz_US() {
    Test_f64("gi_US|fl_oz_US", 4.0);
}
#[test]
fn Test_pt_US_to_gi_US() {
    Test_f64("pt_US|gi_US", 4.0);
}
#[test]
fn Test_qt_US_to_pt_US() {
    Test_f64("qt_US|pt_US", 2.0);
}
#[test]
fn Test_gal_US_to_qt_US() {
    Test_f64("gal_US|qt_US", 4.0);
}
#[test]
fn Test_bu_US_to_pk_US() {
    Test_f64("bu_US|pk_US", 4.0);
}
#[test]
fn Test_bu_US_to_gal_dry() {
    Test_f64("bu_US|gal_dry", 8.0);
}
#[test]
fn Test_bu_US_to_qt_dry() {
    Test_f64("bu_US|qt_dry", 32.0);
}
#[test]
fn Test_bu_US_to_pt_dry() {
    Test_f64("bu_US|pt_dry", 64.0);
}
#[test]
fn Test_gal_dry_to_bbl_dry() {
    Test_f64("26.25gal_dry|bbl_dry", 1.0);
}
#[test]
fn Test_bu_UK_to_pk_UK() {
    Test_f64("bu_UK|pk_UK", 4.0);
}
#[test]
fn Test_kg_m_s2() {
    Test_str("1000kg*1m/s^2", "1kN");
}
#[test]
fn Test_kg_m_s2_tf() {
    Test_str("1000kg*9.80665m/s^2|tf", "1tf");
}
#[test]
fn Test_lb_in_s2() {
    Test_str("1000lb*386.0885826771654in/s^2", "1kip");
}
#[test]
fn Test_ton_ft_s2() {
    // Settings.IsUs defaults to true (US); only ExpressionParser.Parse reads it.
    let html = render("r = 100ton*32.17404855643045ft/s^2");
    assert_contains("200", &html);
}
#[test]
fn Test_ton_ft_s2_UK() {
    let html = render("#settings {\"isUs\": false}\nr = 100ton*32.17404855643045ft/s^2");
    assert_contains("224", &html);
}
#[test]
fn ConcurrentParsers_DoNotShareIsUsSetting() {
    // C#: 32 Task.Run parsers with alternating isUs settings, awaited together.
    let handles: Vec<_> = (0..32)
        .map(|i| {
            std::thread::spawn(move || {
                let is_us = i % 2 == 0;
                let html = render(&format!(
                    "#settings {{\"isUs\": {}}}\nr = 100ton*32.17404855643045ft/s^2",
                    if is_us { "true" } else { "false" }
                ));
                assert_contains(if is_us { "200" } else { "224" }, &html);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}
#[test]
fn Test_MPa_cm2() {
    Test_str("10MPa*1cm^2", "1kN");
}
#[test]
fn Test_ksi_in2() {
    Test_str("10ksi*1in^2", "10kip");
}
#[test]
fn Test_ksf_tf() {
    Test_str("10ksf*1ft^2", "10kip");
}
#[test]
fn Test_kN_cm() {
    Test_str("10kN*50cm", "5kNm");
}
#[test]
fn Test_lbf_in() {
    Test_str("1000lb_f*12in", "1kip·ft");
}
#[test]
fn Test_kN_m2() {
    Test_str("200kN/2m^2", "100kPa");
}
#[test]
fn Test_kip_in2() {
    Test_str("200kip/2in^2", "100ksi");
}
#[test]
fn Test_Nm_cm3() {
    Test_str("2Nm/1000cm^3", "2kPa");
}
#[test]
fn Test_kip_ft_in3() {
    Test_str("2kip*ft/12in^3", "2ksi");
}
#[test]
fn Test_kPa_cm() {
    Test_str("10kPa*200cm", "20kN∕m");
}
#[test]
fn Test_ksf_in() {
    Test_str("10ksf*24in", "20kip∕ft");
}
#[test]
fn Test_kpa_m() {
    Test_str("300kPa/3m", "100kN∕m^3");
}
#[test]
fn Test_ksi_in() {
    Test_str("300ksi/3in", "100kip∕in^3");
}
#[test]
fn Test_A_Ω() {
    Test_str("4A*2Ω", "8V");
}
#[test]
fn Test_W_Div_A() {
    Test_str("4W/2A", "2V");
}
#[test]
fn Test_sqrt_Ω_Div_W() {
    Test_str("sqrt(2Ω*2W)", "2V");
}
#[test]
fn Test_V_Div_Ω() {
    Test_str("4V/2Ω", "2A");
}
#[test]
fn Test_W_Div_V() {
    Test_str("4W/2V", "2A");
}
#[test]
fn Test_sqrt_W_Div_Ω() {
    Test_str("sqrt(8W/2Ω)", "2A");
}
#[test]
fn Test_V_Div_A() {
    Test_str("4V/2A", "2Ω");
}
#[test]
fn Test_V2_Div_W() {
    Test_str("4V^2/2W", "2Ω");
}
#[test]
fn Test_W_Div_A2() {
    Test_str("4W/2A^2", "2Ω");
}
#[test]
fn Test_A_V() {
    Test_str("4A*2V", "8W");
}
#[test]
fn Test_V2_Div_Ω() {
    Test_str("4V^2/2Ω", "2W");
}
#[test]
fn Test_A2_Ω() {
    Test_str("4A^2*2Ω", "8W");
}
#[test]
fn Test_4_Div_2Ω() {
    Test_str("4/2Ω", "2S");
}
#[test]
fn Test_4_Div_2Ω_Mho() {
    Test_str("4/2Ω|℧", "2℧");
}
#[test]
fn Test_V_A() {
    Test_str("4V*2A", "8W");
}
#[test]
fn Test_V_A_VAR() {
    Test_str("4V*2A|VAR", "8VAR");
}
#[test]
fn Test_V_Ω_VA() {
    Test_str("4V*2A|VA", "8VA");
}
#[test]
fn Test_cm2_m() {
    Test_str("10cm^2/m", "0.1cm");
}
#[test]
fn Test_J_kJ() {
    Test_str("100J/kJ", "0.1");
}
#[test]
fn Test_kNm_m() {
    Test_str("5kNm/m", "5kNm∕m");
}
#[test]
fn Test_cm_m() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms()).Run("1|cm/m");
    });
    assert_contains("does not evaluate to units", &e.message);
}
#[test]
fn Test_kN_kPa() {
    Test_str("10kN/5kPa", "2m^2");
}
#[test]
fn Test_1_min() {
    Test_str("1/min", "1min^(-1)");
}
#[test]
fn Test_Hz_min() {
    Test_str("1Hz*1min", "60");
}
#[test]
fn Test_s_min() {
    Test_str("1/s*60s/min|min^-1", "60min^(-1)");
}
#[test]
fn Test_ft_in() {
    Test_str("12ft^-1*in", "1");
}
#[test]
fn Test_sqrt_min_h() {
    Test_str("1h + sqrt(1min*60h)", "2h");
}
#[test]
fn Test_Sum_Length_Metric() {
    Test_str("1m + 5dm + 30cm + 200mm", "2m");
}
#[test]
fn Test_Sum_Mass() {
    Test_str("1t + 1000kg", "2t");
}
#[test]
fn Test_Sum_Length_US1() {
    Test_str("1yd + 2ft + 11in + 1000th", "2yd");
}
#[test]
fn Test_Sum_Length_US2() {
    Test_str("fur + 5*(ch + 3rod + 25li)", "2fur");
}
#[test]
fn Test_Sum_Mass_UK() {
    Test_str("ton_UK + 10*(cwt_UK + 2*(1qr + 1st + 14lb))", "2ton_UK");
}
#[test]
fn Test_Sum_Mass_US() {
    Test_str("1ton_US + 10*(1cwt_US + 50*(1lb + 12oz + 32dr + 875gr))", "2ton_US");
}
#[test]
fn Test_kg_m() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms()).Run("1kg + 1m");
    });
    assert_contains("Inconsistent units", &e.message);
}
#[test]
fn Test_1_m() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms()).Run("1 + m");
    });
    assert_contains("Inconsistent units", &e.message);
}
#[test]
fn Test_Sum_Dimensionless() {
    Test_f64("2 + 30% +45‰", 2.345);
}
#[test]
fn Test_Sum_Angles() {
    Test_str("45° + π/8*rad + 25grad", "90°");
}
#[test]
fn Test_kCal() {
    Test_str("(2kcal/cm^2)/(1000kcal/kg)", "0.02t∕m^2");
}
#[test]
fn Test_kN_kg() {
    Test_str("(2kN/cm^2)/(1000kN/kg)", "0.02t∕m^2");
}
#[test]
fn Test_Subt_F() {
    Test_str("180°F - 18°F", "162°F");
}
#[test]
fn Test_Subt_F_C1() {
    Test_str("212°F - 10°C|°C", "90°C");
}
#[test]
fn Test_Subt_F_C2() {
    Test_str("180°F - 10°C|Δ°C", "90Δ°C");
}
#[test]
fn Test_Subt_C_F() {
    Test_str("30°C - 18°F", "20°C");
}
#[test]
fn Test_Sum_C_F() {
    Test_str("30°C + 18°F", "40°C");
}
#[test]
fn Themodynamics() {
    let mut calc = TestCalc::new(ms());
    calc.Run("σ = 4 * 10 ^ -8 * W * m ^ -2 * K ^ -4");
    calc.Run("I = 625W/m^2");
    calc.Run("T = (I/(4*σ))^0.25");
    let mut result = calc.ToString();
    assert_equal_str("250K", &result);
}
#[test]
fn Test_tan_rad() {
    Test_f64("tan(π/4*rad)", 1.0);
}
#[test]
fn Test_atan_m() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms()).Run("atan(1m)");
    });
    assert_contains("Invalid units for function", &e.message);
}
#[test]
fn Atan_deg() {
    let mut calc = TestCalc::new(ms().Degrees(0.0));
    calc.Run("ReturnAngleUnits = 1");
    calc.Run("atan(1)");
    let mut result = calc.ToString();
    assert_equal_str("45°", &result);
}
#[test]
fn Complex_Assignment() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    calc.Run("z = (2 + 3i)*Ω");
    calc.Run("z");
    let mut result = calc.ToString();
    assert_equal_str("(2 + 3i)Ω", &result);
}
#[test]
fn Test_Sum_Percent() {
    Test_str("1 + 20% + 30‰", "1.23");
}
#[test]
fn Ktf() {
    let mut calc = TestCalc::new(ms());
    calc.Run(".ktf = 1000tf");
    calc.Run("10ktf/100m^2");
    let mut result = calc.ToString();
    assert_equal_str("980.66kPa", &result);
}
#[test]
fn Euro_Pound() {
    let mut calc = TestCalc::new(ms());
    calc.Run(".€ = 1");
    calc.Run(".£ = 2€");
    calc.Run("2000€/t|£/kg");
    let mut result = calc.ToString();
    assert_equal_str("1£/kg", &result);
}
#[test]
fn Euro_Cent() {
    let mut calc = TestCalc::new(ms());
    calc.Run(".€ = 1");
    calc.Run(".¢ = 0.01€");
    calc.Run("10€ + 20¢");
    let mut result = calc.ToString();
    assert_equal_str("10.2€", &result);
}
#[test]
fn Test_Custom_m() {
    let mut e = assert_throws(|| {
        TestCalc::new(ms()).Run(".m = 1");
    });
    assert_contains("Cannot rewrite existing units: m.", &e.message);
}
