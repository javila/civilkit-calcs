//! Translator features checked against the engine: each case builds a small invented worksheet,
//! converts it, runs the produced Calcpad text through the engine and compares the printed
//! numbers with values worked out by hand.

use calcpad_import_xmcd::{import_xmcd, Code, ImportResult};

// ---- builders (the same shapes as the plain-XML worksheet) ---------------------------------------

fn sheet(regions: &str) -> String {
    format!(
        r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10"><settings><calculation><builtInVariables array-origin="1"/></calculation></settings><regions>{regions}</regions></worksheet>"#
    )
}

fn sheet_of(inners: &[String]) -> String {
    let regions: String = inners
        .iter()
        .enumerate()
        .map(|(i, inner)| {
            let top = i as f64 * 20.0;
            format!(
                r#"<region region-id="{}" left="0" top="{top}" width="10" height="10" align-y="{}"><math>{inner}</math></region>"#,
                i + 1,
                top + 10.0
            )
        })
        .collect();
    sheet(&regions)
}

pub fn n(name: &str) -> String {
    format!("<ml:id>{name}</ml:id>")
}
pub fn r(v: &str) -> String {
    format!("<ml:real>{v}</ml:real>")
}
pub fn app(op: &str, args: &[String]) -> String {
    format!("<ml:apply><ml:{op}/>{}</ml:apply>", args.join(""))
}
pub fn call(name: &str, args: &[String]) -> String {
    format!(
        "<ml:apply>{}<ml:sequence>{}</ml:sequence></ml:apply>",
        n(name),
        args.join("")
    )
}
pub fn define(name: &str, value: &str) -> String {
    format!("<ml:define>{}{}</ml:define>", n(name), value)
}
pub fn eval(expr: &str) -> String {
    format!("<ml:eval>{expr}</ml:eval>")
}
pub fn eval_in(expr: &str, unit: &str) -> String {
    format!(
        "<ml:eval>{expr}<ml:unitOverride>{}</ml:unitOverride></ml:eval>",
        n(unit)
    )
}

pub fn convert(inners: &[String]) -> ImportResult {
    import_xmcd(&sheet_of(inners))
}

/// The engine's printed results, one string per line that printed something (tags removed).
pub fn engine_lines(code: &str) -> Vec<String> {
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&format!("#round 10\n{code}"), true);
    let html = parser.html_result.clone();
    assert!(!html.contains("class=\"err\""), "engine error in:\n{code}\n---\n{html}");
    html.lines()
        .map(|l| {
            let mut out = String::new();
            let mut in_tag = false;
            for c in l.chars() {
                match c {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    c if !in_tag => out.push(c),
                    _ => {}
                }
            }
            out.replace("&#8722;", "-").trim().to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

/// The last number printed on a line, after the last `=`.
pub fn last_number(line: &str) -> f64 {
    let tail = line.rsplit('=').next().unwrap();
    let digits: String = tail
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'E' | '+'))
        .collect();
    digits.parse().unwrap_or_else(|_| panic!("no number in {line:?}"))
}

pub fn assert_close(got: f64, want: f64) {
    assert!(
        (got - want).abs() <= 1e-9 * want.abs().max(1.0),
        "got {got}, want {want}"
    );
}

// ---- 1. units written with a middle dot -----------------------------------------------------------

#[test]
fn middle_dot_units_are_units() {
    // M := 5 kN.m ; M shown in N.m ; M / (2 kN.m)
    let res = convert(&[
        define("M", &app("mult", &[r("5"), n("kN\u{00B7}m")])),
        eval_in(&n("M"), "N\u{00B7}m"),
        eval(&app("div", &[n("M"), app("mult", &[r("2"), n("kN\u{00B7}m")])])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("M = 5*(kN*m)"), "{}", res.calcpad);
    assert!(res.calcpad.contains("M|(N*m)"), "{}", res.calcpad);
    let out = engine_lines(&res.calcpad);
    let nums: Vec<f64> = out.iter().filter(|l| l.contains('=')).map(|l| last_number(l)).collect();
    // Defined as 5 kN*m (shown as given); in N*m that is 5000; the ratio is 2.5.
    assert_close(nums[0], 5.0);
    assert_close(nums[1], 5000.0);
    assert_close(nums[2], 2.5);
}

#[test]
fn a_dotted_name_with_a_non_unit_part_stays_undefined() {
    let res = convert(&[eval(&n("kN\u{00B7}zz"))]);
    assert!(res.report.gaps().any(|i| i.code == Code::UndefinedVariable));
}

// ---- 2. integrals and derivatives -------------------------------------------------------------------

fn lambda(var: &str, body: &str) -> String {
    format!("<ml:lambda><ml:boundVars>{}</ml:boundVars>{body}</ml:lambda>", n(var))
}
fn integral(var: &str, body: &str, lo: &str, hi: &str) -> String {
    format!(
        "<ml:apply><ml:integral/>{}<ml:lowerBound>{lo}</ml:lowerBound><ml:upperBound>{hi}</ml:upperBound></ml:apply>",
        lambda(var, body)
    )
}
fn derivative(var: &str, body: &str) -> String {
    format!(
        "<ml:apply><ml:derivative/>{}<ml:degree><ml:placeholder/></ml:degree></ml:apply>",
        lambda(var, body)
    )
}
fn func(name: &str, params: &[&str], body: &str) -> String {
    let p: String = params.iter().map(|x| n(x)).collect();
    format!(
        "<ml:define><ml:function>{}<ml:boundVars>{p}</ml:boundVars></ml:function>{body}</ml:define>",
        n(name)
    )
}

#[test]
fn integral_of_a_constant_load_keeps_its_units() {
    // w = 3 kN/m over 0..6 m is 18 kN.
    let res = convert(&[
        define("w", &app("div", &[app("mult", &[r("3"), n("kN")]), n("m")])),
        define("L", &app("mult", &[r("6"), n("m")])),
        define("F", &integral("x", &n("w"), &r("0"), &n("L"))),
        eval_in(&n("F"), "kN"),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("$Integral{w @ x = 0*(L) : L}"), "{}", res.calcpad);
    let nums: Vec<f64> = engine_lines(&res.calcpad)
        .iter()
        .filter(|l| l.contains('='))
        .map(|l| last_number(l))
        .collect();
    assert_close(*nums.last().unwrap(), 18.0);
}

#[test]
fn integral_of_a_function_of_the_bound_variable() {
    // q(x) = 2 x, from 1 to 4: 16 - 1 = 15.
    let res = convert(&[
        func("q", &["t"], &app("mult", &[r("2"), n("t")])),
        eval(&integral("x", &call("q", &[n("x")]), &r("1"), &r("4"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums: Vec<f64> = engine_lines(&res.calcpad)
        .iter()
        .filter(|l| l.contains('='))
        .map(|l| last_number(l))
        .collect();
    assert_close(*nums.last().unwrap(), 15.0);
}

#[test]
fn derivative_is_taken_at_the_current_value() {
    // d/dx x^3 at x = 3 is 27.
    let res = convert(&[
        define("x", &r("3")),
        eval(&derivative("x", &app("pow", &[n("x"), r("3")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums: Vec<f64> = engine_lines(&res.calcpad)
        .iter()
        .filter(|l| l.contains('='))
        .map(|l| last_number(l))
        .collect();
    assert_close(*nums.last().unwrap(), 27.0);
}

#[test]
fn calculus_that_cannot_be_read_faithfully_is_a_precise_gap() {
    // No value for x: the source could not calculate it either.
    let res = convert(&[eval(&derivative("x", &app("pow", &[n("x"), r("3")])))]);
    let g = res
        .report
        .gaps()
        .find(|i| i.code == Code::CalculusOperator)
        .expect("gap");
    assert!(g.detail.as_deref().unwrap_or("").contains("no value"), "{:?}", g);
    // A bound that uses the integration variable's name is ambiguous.
    let res = convert(&[define("x", &r("2")), eval(&integral("x", &n("x"), &r("0"), &n("x")))]);
    let g = res
        .report
        .gaps()
        .find(|i| i.code == Code::CalculusOperator)
        .expect("gap");
    assert!(g.detail.as_deref().unwrap_or("").contains("bound"), "{:?}", g);
    // A second derivative has a degree other than 1: not read.
    let second = "<ml:apply><ml:derivative/><ml:lambda><ml:boundVars><ml:id>x</ml:id></ml:boundVars><ml:id>x</ml:id></ml:lambda><ml:degree><ml:real>2</ml:real></ml:degree></ml:apply>";
    let res = convert(&[define("x", &r("2")), eval(second)]);
    assert!(res.report.gaps().any(|i| i.code == Code::CalculusOperator));
}
