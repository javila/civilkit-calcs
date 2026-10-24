//! Translator features checked against the engine: each case builds a small invented worksheet,
//! converts it, runs the produced Calcpad text through the engine and compares the printed
//! numbers with values worked out by hand.

use calcpad_import::{import_xmcd, Code, ImportResult};

// ---- builders (the same shapes as the plain-XML worksheet) ---------------------------------------

fn sheet(regions: &str, origin: i64) -> String {
    format!(
        r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10"><settings><calculation><builtInVariables array-origin="{origin}"/></calculation></settings><regions>{regions}</regions></worksheet>"#
    )
}

fn sheet_of(inners: &[String], origin: i64) -> String {
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
    sheet(&regions, origin)
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
    import_xmcd(&sheet_of(inners, 1))
}

pub fn convert_origin(inners: &[String], origin: i64) -> ImportResult {
    import_xmcd(&sheet_of(inners, origin))
}

/// The engine's printed results, one string per line that printed something (tags removed).
pub fn engine_lines(code: &str) -> Vec<String> {
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&format!("#round 10\n{code}"), true);
    let html = parser.html_result.clone();
    assert!(!html.contains("class=\"err\""), "engine error in:\n{code}\n---\n{html}");
    html.replace("<p>", "\n<p>")
        .lines()
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
pub fn try_last_number(line: &str) -> Option<f64> {
    let tail = line.rsplit('=').next().unwrap();
    let digits: String = tail
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'E' | '+'))
        .collect();
    digits.parse().ok()
}

pub fn last_number(line: &str) -> f64 {
    try_last_number(line).unwrap_or_else(|| panic!("no number in {line:?}"))
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

// ---- 3. range variables and element assignment ---------------------------------------------------------

fn range(first: &str, last: &str) -> String {
    format!("<ml:range>{}{}</ml:range>", r(first), r(last))
}
fn index1(base: &str, idx: &str) -> String {
    format!("<ml:apply><ml:indexer/>{}{}</ml:apply>", n(base), idx)
}
fn set_element(base: &str, idx: &str, value: &str) -> String {
    format!("<ml:define>{}{}</ml:define>", index1(base, idx), value)
}
fn numbers(code: &str) -> Vec<f64> {
    engine_lines(code)
        .iter()
        .filter(|l| l.contains('=') && !l.contains('['))
        .filter_map(|l| try_last_number(l))
        .collect()
}

#[test]
fn range_with_element_assignment_fills_a_vector_origin_zero() {
    // i = 0..4, x[i] = i^2 + 1, so x[3] = 10 and x[0] = 1.
    let res = convert_origin(
        &[
            define("i", &range("0", "4")),
            set_element("x", &n("i"), &app("plus", &[app("pow", &[n("i"), r("2")]), r("1")])),
            eval(&index1("x", &r("3"))),
            eval(&index1("x", &r("0"))),
        ],
        0,
    );
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.report.notes().any(|i| i.code == Code::RangeLoop));
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 10.0);
    assert_close(nums[nums.len() - 1], 1.0);
}

#[test]
fn range_with_element_assignment_origin_one_and_units() {
    // i = 1..4, y[i] = i * L with L = 2 m: y[3] = 6 m.
    let res = convert(&[
        define("L", &app("mult", &[r("2"), n("m")])),
        define("i", &range("1", "4")),
        set_element("y", &n("i"), &app("mult", &[n("i"), n("L")])),
        eval(&index1("y", &r("3"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums = numbers(&res.calcpad);
    assert_close(*nums.last().unwrap(), 6.0);
}

#[test]
fn a_range_variable_used_anywhere_else_is_reported() {
    let res = convert_origin(
        &[define("i", &range("0", "4")), eval(&app("mult", &[n("i"), r("2")]))],
        0,
    );
    let g = res.report.gaps().find(|i| i.code == Code::RangeVariable).expect("gap");
    assert!(
        g.detail.as_deref().unwrap_or("").contains("only read as the loop"),
        "{g:?}"
    );
    // The range definition itself converted (with a note).
    assert!(res.report.notes().any(|i| i.code == Code::RangeLoop));
}

#[test]
fn element_assignments_that_cannot_be_a_fill_loop_say_why() {
    // The array already has a value.
    let res = convert_origin(
        &[
            define("i", &range("0", "2")),
            define("x", &app("mult", &[r("2"), r("3")])),
            set_element("x", &n("i"), &n("i")),
        ],
        0,
    );
    let g = res
        .report
        .gaps()
        .find(|i| i.code == Code::ElementAssignment)
        .expect("gap");
    assert!(
        g.detail.as_deref().unwrap_or("").contains("already has a value"),
        "{g:?}"
    );
    // The value reads the array it fills.
    let res = convert_origin(
        &[
            define("i", &range("1", "3")),
            set_element(
                "x",
                &n("i"),
                &app("plus", &[index1("x", &app("minus", &[n("i"), r("1")])), r("1")]),
            ),
        ],
        0,
    );
    let g = res
        .report
        .gaps()
        .find(|i| i.code == Code::ElementAssignment)
        .expect("gap");
    assert!(g.detail.as_deref().unwrap_or("").contains("reads the array"), "{g:?}");
    // A literal index is not a loop.
    let res = convert(&[set_element("x", &r("2"), &r("5"))]);
    let g = res
        .report
        .gaps()
        .find(|i| i.code == Code::ElementAssignment)
        .expect("gap");
    assert!(
        g.detail
            .as_deref()
            .unwrap_or("")
            .contains("not made of range variables"),
        "{g:?}"
    );
    // A range with a step, or with a bound that is not a literal, stays a range-variable gap.
    let stepped =
        "<ml:range><ml:sequence><ml:real>0</ml:real><ml:real>2</ml:real></ml:sequence><ml:real>10</ml:real></ml:range>";
    let res = convert(&[define("j", stepped)]);
    assert!(res.report.gaps().any(|i| i.code == Code::RangeVariable));
    // A bound that does not convert keeps the range a gap.
    let res = convert(&[define(
        "k",
        &format!("<ml:range>{}{}</ml:range>", r("1"), n("undefined_bound")),
    )]);
    assert!(res.report.gaps().any(|i| i.code == Code::UndefinedVariable));
}

// ---- 4. programs -------------------------------------------------------------------------------------------

fn program(stmts: &[String]) -> String {
    format!("<ml:program>{}</ml:program>", stmts.join(""))
}
fn local(name: &str, value: &str) -> String {
    format!("<ml:localDefine>{}{}</ml:localDefine>", n(name), value)
}
fn local_el(base: &str, idx: &str, value: &str) -> String {
    format!("<ml:localDefine>{}{}</ml:localDefine>", index1(base, idx), value)
}
fn for_in(var: &str, lo: &str, hi: &str, body: &[String]) -> String {
    format!(
        "<ml:for>{}<ml:range>{lo}{hi}</ml:range>{}</ml:for>",
        n(var),
        program(body)
    )
}
fn if_then(test: &str, then: &[String], otherwise: Option<&[String]>) -> String {
    let e = otherwise
        .map(|o| format!("<ml:else>{}</ml:else>", program(o)))
        .unwrap_or_default();
    format!(
        "<ml:if><ml:test>{test}</ml:test><ml:then>{}</ml:then>{e}</ml:if>",
        program(then)
    )
}
fn while_do(cond: &str, body: &[String]) -> String {
    format!("<ml:while>{cond}{}</ml:while>", program(body))
}
fn gt(a: &str, b: &str) -> String {
    app("greaterThan", &[a.to_string(), b.to_string()])
}
fn lt(a: &str, b: &str) -> String {
    app("lessThan", &[a.to_string(), b.to_string()])
}
fn plus(a: &str, b: &str) -> String {
    app("plus", &[a.to_string(), b.to_string()])
}
fn times(a: &str, b: &str) -> String {
    app("mult", &[a.to_string(), b.to_string()])
}
fn sq(a: &str) -> String {
    app("pow", &[a.to_string(), r("2")])
}
fn last(code: &str) -> f64 {
    *numbers(code).last().expect("a printed result")
}

#[test]
fn straight_line_program_is_the_expression_with_its_locals_substituted() {
    // f(x) = (a = 2x; b = a + 1; b * a): f(3) = 7 * 6 = 42.
    let res = convert(&[
        format!(
            "<ml:define><ml:function>{}<ml:boundVars>{}</ml:boundVars></ml:function>{}</ml:define>",
            n("f"),
            n("x"),
            program(&[
                local("a", &times(&r("2"), &n("x"))),
                local("b", &plus(&n("a"), &r("1"))),
                times(&n("b"), &n("a")),
            ])
        ),
        eval(&call("f", &[r("3")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(!res.calcpad.contains("#"), "no statements are needed: {}", res.calcpad);
    assert_close(last(&res.calcpad), 42.0);
}

#[test]
fn loop_with_an_accumulator_and_literal_bounds() {
    // s = 0; for i in 1..4: s = s + i^2; s  -> 1 + 4 + 9 + 16 = 30
    let res = convert(&[define(
        "S",
        &program(&[
            local("s", &r("0")),
            for_in("i", &r("1"), &r("4"), &[local("s", &plus(&n("s"), &sq(&n("i"))))]),
            n("s"),
        ]),
    )]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("#for i_p1 = 1 : 4"), "{}", res.calcpad);
    assert_close(last(&res.calcpad), 30.0);
}

#[test]
fn loop_bounds_that_are_names_count_up_or_down_like_the_source() {
    // Ascending 1..5 and descending 5..1 both visit five values: sum 15, count 5.
    for (lo, hi) in [(1.0, 5.0), (5.0, 1.0)] {
        let res = convert(&[
            define("lo", &r(&lo.to_string())),
            define("hi", &r(&hi.to_string())),
            define(
                "T",
                &program(&[
                    local("t", &r("0")),
                    local("c", &r("0")),
                    for_in(
                        "k",
                        &n("lo"),
                        &n("hi"),
                        &[local("t", &plus(&n("t"), &n("k"))), local("c", &plus(&n("c"), &r("1")))],
                    ),
                    plus(&times(&n("t"), &r("100")), &n("c")),
                ]),
            ),
        ]);
        assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
        assert_close(last(&res.calcpad), 1505.0);
    }
}

#[test]
fn if_break_and_while() {
    // First i with i^2 > 10 is 4.
    let res = convert(&[define(
        "F",
        &program(&[
            local("r", &r("0")),
            for_in(
                "i",
                &r("1"),
                &r("10"),
                &[if_then(
                    &gt(&sq(&n("i")), &r("10")),
                    &[local("r", &n("i")), "<ml:break/>".to_string()],
                    None,
                )],
            ),
            n("r"),
        ]),
    )]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 4.0);
    // Doubling until at least 100 starting from 1 gives 128; an if/else picks the sign.
    let res = convert(&[
        define(
            "W",
            &program(&[
                local("w", &r("1")),
                while_do(&lt(&n("w"), &r("100")), &[local("w", &times(&n("w"), &r("2")))]),
                n("w"),
            ]),
        ),
        define(
            "Q",
            &program(&[
                local("q", &r("0")),
                if_then(
                    &gt(&n("W"), &r("100")),
                    &[local("q", &r("1"))],
                    Some(&[local("q", &r("-1"))]),
                ),
                n("q"),
            ]),
        ),
        eval(&plus(&n("W"), &n("Q"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 129.0);
}

#[test]
fn locals_do_not_leak_into_the_sheet() {
    // x = 10 outside; the program's own x is separate: P = 5 + 1 + 2 = 8, and x is still 10.
    let res = convert(&[
        define("x", &r("10")),
        define(
            "P",
            &program(&[
                local("x", &r("5")),
                for_in("i", &r("1"), &r("2"), &[local("x", &plus(&n("x"), &n("i")))]),
                n("x"),
            ]),
        ),
        eval(&plus(&times(&n("x"), &r("100")), &n("P"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 1008.0);
}

#[test]
fn a_local_array_created_whole_can_be_filled_element_by_element() {
    let zeros =
        "<ml:matrix rows=\"3\" cols=\"1\"><ml:real>0</ml:real><ml:real>0</ml:real><ml:real>0</ml:real></ml:matrix>";
    let res = convert(&[
        define(
            "V",
            &program(&[
                local("v", zeros),
                for_in("i", &r("1"), &r("3"), &[local_el("v", &n("i"), &sq(&n("i")))]),
                n("v"),
            ]),
        ),
        eval(&index1("V", &r("3"))),
        eval(&index1("V", &r("2"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 9.0);
    assert_close(nums[nums.len() - 1], 4.0);
}

#[test]
fn units_survive_in_a_program() {
    // s = 0 m; for i in 1..3: s = s + i * 2 m  -> 12 m
    let res = convert(&[define(
        "D",
        &program(&[
            local("s", &times(&r("0"), &n("m"))),
            for_in(
                "i",
                &r("1"),
                &r("3"),
                &[local("s", &plus(&n("s"), &times(&n("i"), &times(&r("2"), &n("m")))))],
            ),
            n("s"),
        ]),
    )]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 12.0);
}

#[test]
fn programs_that_cannot_be_read_faithfully_say_why() {
    let reason = |res: &ImportResult| -> String {
        res.report
            .gaps()
            .find(|i| i.code == Code::ProgramBlock)
            .and_then(|i| i.detail.clone())
            .unwrap_or_default()
    };
    // try/catch.
    let res = convert(&[define(
        "C",
        &program(&[
            format!(
                "<ml:tryCatch>{}{}</ml:tryCatch>",
                program(&[local("a", &r("1"))]),
                program(&[r("0")])
            ),
            r("1"),
        ]),
    )]);
    assert!(reason(&res).contains("tryCatch"), "{}", reason(&res));
    // A function whose body loops.
    let res = convert(&[format!(
        "<ml:define><ml:function>{}<ml:boundVars>{}</ml:boundVars></ml:function>{}</ml:define>",
        n("h"),
        n("x"),
        program(&[
            local("s", &r("0")),
            for_in("i", &r("1"), &n("x"), &[local("s", &n("i"))]),
            n("s")
        ])
    )]);
    assert!(reason(&res).contains("usable inside a block"), "{}", reason(&res));
    // A gap leaves the name blocked for later uses.
    let res = convert(&[
        define(
            "G",
            &program(&[
                format!(
                    "<ml:tryCatch>{}{}</ml:tryCatch>",
                    program(&[local("g", &r("1"))]),
                    program(&[r("0")])
                ),
                n("g"),
            ]),
        ),
        eval(&n("G")),
    ]);
    assert!(res.report.gaps().any(|i| i.code == Code::DependsOnGap));
}

// ---- 5. matrix functions the engine has ---------------------------------------------------------------------

fn index2(base: &str, i: &str, j: &str) -> String {
    format!(
        "<ml:apply><ml:indexer/>{}<ml:sequence>{i}{j}</ml:sequence></ml:apply>",
        n(base)
    )
}
fn mat(rows: usize, cols: usize, items: &[&str]) -> String {
    let body: String = items.iter().map(|v| r(v)).collect();
    format!(r#"<ml:matrix rows="{rows}" cols="{cols}">{body}</ml:matrix>"#)
}

#[test]
fn lsolve_and_augment_with_more_than_two_arguments() {
    // [2 0; 0 4] x = [2; 8] gives x = [1; 2].
    let res = convert(&[
        define("A", &mat(2, 2, &["2", "0", "0", "4"])),
        define("b", &mat(2, 1, &["2", "8"])),
        define("X", &call("lsolve", &[n("A"), n("b")])),
        eval(&index1("X", &r("1"))),
        eval(&index1("X", &r("2"))),
        define("M", &call("augment", &[n("b"), n("b"), n("b")])),
        eval(&index2("M", &r("2"), &r("3"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 3], 1.0);
    assert_close(nums[nums.len() - 2], 2.0);
    assert_close(nums[nums.len() - 1], 8.0);
}

#[test]
fn submatrix_indexes_follow_the_array_origin() {
    // A = [1 2 3; 4 5 6; 7 8 10]; submatrix(A, 1, 2, 0, 1) at origin 0 is rows 1..2, columns 0..1
    // = [4 5; 7 8], so S[0, 1] = 5 and S[1, 1] = 8.
    let res = convert_origin(
        &[
            define("A", &mat(3, 3, &["1", "4", "7", "2", "5", "8", "3", "6", "10"])),
            define("S", &call("submatrix", &[n("A"), r("1"), r("2"), r("0"), r("1")])),
            eval(&index2("S", &r("0"), &r("1"))),
            eval(&index2("S", &r("1"), &r("1"))),
        ],
        0,
    );
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 5.0);
    assert_close(nums[nums.len() - 1], 8.0);
}

// ---- gap lines show the original maths ----------------------------------------------------------------

fn gap_line<'a>(res: &'a ImportResult, id: &str) -> &'a str {
    res.calcpad
        .lines()
        .find(|l| l.contains(&format!("R{id} ")))
        .unwrap_or_else(|| panic!("no gap line for R{id} in\n{}", res.calcpad))
}

#[test]
fn an_unconverted_region_is_shown_with_its_real_names() {
    let res = convert(&[
        define("y", &call("vendor_fn", &[n("a"), r("2")])),
        define("z", &app("plus", &[n("y"), r("1")])),
        define("s", "<ml:str>abc</ml:str>"),
        eval(&app("mult", &[call("rows", &[n("Input_N")]), n("kN")])),
    ]);
    // Maths goes in a #noc block (typeset, never evaluated), one block per region.
    assert!(
        res.calcpad.contains("#noc\ny = vendor_fn(a; 2)\n#end noc"),
        "{}",
        res.calcpad
    );
    assert!(res.calcpad.contains("#noc\nz = y + 1\n#end noc"), "{}", res.calcpad);
    assert!(res.calcpad.contains("n_rows(Input_N)*kN\n#end noc"), "{}", res.calcpad);
    // A string is not Calcpad maths: it is code text, not an equation.
    assert!(res.calcpad.contains("<code>s = \"\"abc\"\"</code>"), "{}", res.calcpad);
    // The marker and the codes are in an HTML comment, with a small muted note.
    assert!(
        gap_line(&res, "1").starts_with("'<!-- [import-gap R1 "),
        "{}",
        gap_line(&res, "1")
    );
    assert!(
        gap_line(&res, "1").contains("<small>not calculated:"),
        "{}",
        gap_line(&res, "1")
    );
    // No stand-in inside any equation block.
    let mut in_noc = false;
    for l in res.calcpad.lines() {
        match l {
            "#noc" => in_noc = true,
            "#end noc" => in_noc = false,
            _ if in_noc => assert!(!l.contains('?'), "stand-in in {l}"),
            _ => {}
        }
    }
}

/// The page's HTML for some Calcpad text, to look at what is rendered, not only the text.
fn html_of(code: &str) -> String {
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(code, true);
    parser.html_result.clone()
}

#[test]
fn gap_regions_render_as_equations_without_errors() {
    let res = convert(&[
        define(
            "N",
            &app(
                "mult",
                &[
                    call("submatrix", &[n("Input_N"), r("1"), n("n_n"), r("3"), r("3")]),
                    n("cm"),
                ],
            ),
        ),
        define("y", &call("vendor_fn", &[n("a"), r("2")])),
        define("k", &app("plus", &[n("c"), n("d")])),
        app("equal", &[n("fB"), plus(&times(&n("c"), &n("L")), &n("d"))]),
        define("s", "<ml:str>abc</ml:str>"),
        define("a", &r("2")),
        eval(&app("plus", &[n("a"), r("1")])),
    ]);
    let html = html_of(&res.calcpad);
    assert!(
        !html.contains("class=\"err\""),
        "engine error in\n{}\n---\n{html}",
        res.calcpad
    );
    // The gap equations are typeset (class eq, names in <var>), not prose.
    assert!(html.contains("<b>submatrix</b>"), "{html}");
    assert!(html.contains("<var>vendor</var><sub>fn</sub>"), "{html}");
    assert!(
        html.contains("<var>f</var><sub>B</sub>") || html.contains("<var>fB</var>"),
        "{html}"
    );
    assert!(
        !html.contains("?name?") && !html.contains("?gap?") && !html.contains("?call?"),
        "{html}"
    );
    // They are not evaluated: nothing is defined by them, the later regions still calculate.
    assert!(html.contains("<var>a</var> + 1 = 2 + 1 = 3"), "{html}");
}

#[test]
fn equality_is_written_the_way_the_engine_reads_it() {
    // 2 = 2 is true (1), 2 = 3 is false (0).
    let res = convert(&[
        eval(&app("equal", &[r("2"), r("2")])),
        eval(&app("equal", &[r("2"), r("3")])),
    ]);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 1.0);
    assert_close(nums[nums.len() - 1], 0.0);
}

#[test]
fn ranges_with_run_time_bounds_and_two_dimensions_preallocate() {
    // n = 4, i = 1..n, x[i] = i^2: x[3] = 9.
    let res = convert(&[
        define("n", &r("4")),
        define("i", &format!("<ml:range>{}{}</ml:range>", r("1"), n("n"))),
        set_element("x", &n("i"), &sq(&n("i"))),
        eval(&index1("x", &r("3"))),
        eval(&index1("x", &r("4"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("x = vector(max("), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 9.0);
    assert_close(nums[nums.len() - 1], 16.0);
    // A descending range 3..1: x[2] = 20.
    let res = convert(&[
        define("i", &format!("<ml:range>{}{}</ml:range>", r("3"), r("1"))),
        set_element("x", &n("i"), &times(&n("i"), &r("10"))),
        eval(&index1("x", &r("2"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 20.0);
    // Origin 0 with a run-time end: y[3] = 6.
    let res = convert_origin(
        &[
            define("n", &r("3")),
            define("i", &format!("<ml:range>{}{}</ml:range>", r("0"), n("n"))),
            set_element("y", &n("i"), &times(&r("2"), &n("i"))),
            eval(&index1("y", &r("3"))),
        ],
        0,
    );
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 6.0);
    // Two ranges fill a matrix: M[2,3] = 23 and M[1,2] = 12.
    let res = convert(&[
        define("i", &format!("<ml:range>{}{}</ml:range>", r("1"), r("2"))),
        define("j", &format!("<ml:range>{}{}</ml:range>", r("1"), r("3"))),
        format!(
            "<ml:define>{}{}</ml:define>",
            index2("M", &n("i"), &n("j")),
            plus(&times(&n("i"), &r("10")), &n("j"))
        ),
        eval(&index2("M", &r("2"), &r("3"))),
        eval(&index2("M", &r("1"), &r("2"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("M = matrix("), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 23.0);
    assert_close(nums[nums.len() - 1], 12.0);
}

// ---- arrays built element by element, early returns -------------------------------------------------------

fn index_n(base: &str, idx: &[String]) -> String {
    format!(
        "<ml:apply><ml:indexer/>{}<ml:sequence>{}</ml:sequence></ml:apply>",
        n(base),
        idx.join("")
    )
}
fn local_el2(base: &str, i: &str, j: &str, value: &str) -> String {
    format!(
        "<ml:localDefine>{}{}</ml:localDefine>",
        index_n(base, &[i.to_string(), j.to_string()]),
        value
    )
}
fn ret(value: &str) -> String {
    format!("<ml:return>{value}</ml:return>")
}

#[test]
fn a_program_that_grows_an_array_in_loops_preallocates_it() {
    // K[i, j] = 10 i + j over i = 1..3, j = 1..2: K[3,2] = 32, K[2,1] = 21, and K has 3 rows.
    let res = convert(&[
        define(
            "K",
            &program(&[
                for_in(
                    "i",
                    &r("1"),
                    &r("3"),
                    &[for_in(
                        "j",
                        &r("1"),
                        &r("2"),
                        &[local_el2(
                            "k",
                            &n("i"),
                            &n("j"),
                            &plus(&times(&n("i"), &r("10")), &n("j")),
                        )],
                    )],
                ),
                n("k"),
            ]),
        ),
        eval(&index2("K", &r("3"), &r("2"))),
        eval(&index2("K", &r("2"), &r("1"))),
        eval(&call("rows", &[n("K")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("k_p1 = matrix(3; 2)"), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 3], 32.0);
    assert_close(nums[nums.len() - 2], 21.0);
    assert_close(nums[nums.len() - 1], 3.0);
    // A run-time bound is an expression the engine evaluates: n = 4, q[i] = i^2, 4 rows, q[3] = 9.
    let res = convert(&[
        define("n", &r("4")),
        define(
            "Q",
            &program(&[
                for_in("i", &r("1"), &n("n"), &[local_el("q", &n("i"), &sq(&n("i")))]),
                n("q"),
            ]),
        ),
        eval(&index1("Q", &r("3"))),
        eval(&call("rows", &[n("Q")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 9.0);
    assert_close(nums[nums.len() - 1], 4.0);
}

#[test]
fn growth_that_cannot_be_sized_ahead_is_a_gap_with_the_reason() {
    let reason = |res: &ImportResult| -> String {
        res.report
            .gaps()
            .find(|i| i.code == Code::ProgramBlock)
            .and_then(|i| i.detail.clone())
            .unwrap_or_default()
    };
    // Written under a condition: the array may end up smaller than the loop.
    let res = convert(&[define(
        "G",
        &program(&[
            for_in(
                "i",
                &r("1"),
                &r("3"),
                &[if_then(&gt(&n("i"), &r("1")), &[local_el("g", &n("i"), &n("i"))], None)],
            ),
            n("g"),
        ]),
    )]);
    assert!(reason(&res).contains("cannot be known ahead"), "{}", reason(&res));
    // Indexed by a counter, not by the loop variable.
    let res = convert(&[define(
        "H",
        &program(&[
            local("c", &r("0")),
            for_in(
                "i",
                &r("1"),
                &r("3"),
                &[local("c", &plus(&n("c"), &r("1"))), local_el("h", &n("c"), &n("i"))],
            ),
            n("h"),
        ]),
    )]);
    assert!(reason(&res).contains("cannot be known ahead"), "{}", reason(&res));
    // A bound that is itself a local of the program cannot be written ahead of the loop.
    let res = convert(&[define(
        "J",
        &program(&[
            local("m", &r("3")),
            for_in("i", &r("1"), &n("m"), &[local_el("j", &n("i"), &n("i"))]),
            n("j"),
        ]),
    )]);
    assert!(reason(&res).contains("cannot be known ahead"), "{}", reason(&res));
}

#[test]
fn early_return_from_a_loop_uses_a_result_and_a_done_flag() {
    // The first i with i^2 > 20 is 5; with a threshold nothing reaches the program returns 0.
    for (limit, want) in [(20.0, 5.0), (200.0, 0.0)] {
        let res = convert(&[define(
            "R",
            &program(&[
                for_in(
                    "i",
                    &r("1"),
                    &r("10"),
                    &[if_then(
                        &gt(&sq(&n("i")), &r(&limit.to_string())),
                        &[ret(&n("i"))],
                        None,
                    )],
                ),
                r("0"),
            ]),
        )]);
        assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
        assert!(res.calcpad.contains("done_p1"), "{}", res.calcpad);
        assert_close(last(&res.calcpad), want);
    }
    // Nested loops: the first (i, j) with i*j > 6 is (2, 4), returned as 204; the outer loop stops too.
    let res = convert(&[define(
        "R",
        &program(&[
            local("count", &r("0")),
            for_in(
                "i",
                &r("1"),
                &r("5"),
                &[for_in(
                    "j",
                    &r("1"),
                    &r("5"),
                    &[
                        local("count", &plus(&n("count"), &r("1"))),
                        if_then(
                            &gt(&times(&n("i"), &n("j")), &r("6")),
                            &[ret(&plus(&times(&n("i"), &r("100")), &n("j")))],
                            None,
                        ),
                    ],
                )],
            ),
            r("-1"),
        ]),
    )]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 204.0);
}

// ---- function bodies that are not a single expression --------------------------------------------------------

fn function(name: &str, params: &[&str], body: &str) -> String {
    let p: String = params.iter().map(|x| n(x)).collect();
    format!(
        "<ml:define><ml:function>{}<ml:boundVars>{p}</ml:boundVars></ml:function>{body}</ml:define>",
        n(name)
    )
}

#[test]
fn function_bodies_with_conditions_and_returns_become_one_expression() {
    // f(x): y = x; if x > 10 then y = 10; y * 2. f(3) = 6, f(30) = 20.
    let res = convert(&[
        function(
            "f",
            &["x"],
            &program(&[
                local("y", &n("x")),
                if_then(&gt(&n("x"), &r("10")), &[local("y", &r("10"))], None),
                times(&n("y"), &r("2")),
            ]),
        ),
        eval(&call("f", &[r("3")])),
        eval(&call("f", &[r("30")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(!res.calcpad.contains('#'), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 6.0);
    assert_close(nums[nums.len() - 1], 20.0);
    // g(x): early returns. g(-5) = -1, g(0) = 0, g(7) = 1.
    let res = convert(&[
        function(
            "g",
            &["x"],
            &program(&[
                if_then(&lt(&n("x"), &r("0")), &[ret(&r("-1"))], None),
                if_then(&app("equal", &[n("x"), r("0")]), &[ret(&r("0"))], None),
                r("1"),
            ]),
        ),
        eval(&call("g", &[r("-5")])),
        eval(&call("g", &[r("0")])),
        eval(&call("g", &[r("7")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 3], -1.0);
    assert_close(nums[nums.len() - 2], 0.0);
    assert_close(nums[nums.len() - 1], 1.0);
    // A short literal loop is unrolled: t = x*1 + x*2 + x*3, so s(2) = 12.
    let res = convert(&[
        function(
            "s",
            &["x"],
            &program(&[
                local("t", &r("0")),
                for_in(
                    "k",
                    &r("1"),
                    &r("3"),
                    &[local("t", &plus(&n("t"), &times(&n("x"), &n("k"))))],
                ),
                n("t"),
            ]),
        ),
        eval(&call("s", &[r("2")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 12.0);
    // Units pass through a condition: k(3 m) = 3 m, k(1 m) = 2 m.
    let m2 = times(&r("2"), &n("m"));
    let res = convert(&[
        function(
            "k",
            &["x"],
            &program(&[if_then(&gt(&n("x"), &m2), &[ret(&n("x"))], None), m2.clone()]),
        ),
        eval(&call("k", &[times(&r("3"), &n("m"))])),
        eval(&call("k", &[times(&r("1"), &n("m"))])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 3.0);
    assert_close(nums[nums.len() - 1], 2.0);
}

#[test]
fn a_function_with_a_run_time_loop_names_the_missing_engine_feature() {
    let res = convert(&[function(
        "h",
        &["x"],
        &program(&[
            local("t", &r("0")),
            for_in("k", &r("1"), &n("x"), &[local("t", &plus(&n("t"), &n("k")))]),
            n("t"),
        ]),
    )]);
    let g = res.report.gaps().find(|i| i.code == Code::ProgramBlock).expect("gap");
    let d = g.detail.as_deref().unwrap_or("");
    assert!(
        d.contains("$While") && d.contains("loop statement usable inside a block"),
        "{d}"
    );
    // And the readable line still shows the function.
    assert!(res.calcpad.contains("<code>h(x) = program:</code>"), "{}", res.calcpad);
    assert!(res.calcpad.contains("#noc\nt = 0\n#end noc"), "{}", res.calcpad);
    assert!(
        res.calcpad.contains("'&emsp;<code>for k = 1 to x:</code>"),
        "{}",
        res.calcpad
    );
}

// ---- constraint equations ---------------------------------------------------------------------------------------

#[test]
fn an_equation_on_its_own_is_a_constraint_not_undefined_names() {
    // f_B = c*L + d with c and d unknown (a solve block would find them).
    let eq = app("equal", &[n("fB"), plus(&times(&n("c"), &n("L")), &n("d"))]);
    let res = convert(&[
        define("L", &times(&r("6"), &n("m"))),
        eq,
        eval(&app("mult", &[r("2"), r("3")])),
    ]);
    let codes: Vec<Code> = res.report.gaps().map(|i| i.code).collect();
    assert_eq!(codes, vec![Code::ConstraintEquation], "{:?}", res.report.issues);
    let l = gap_line(&res, "2");
    assert!(l.contains("constraint-equation"), "{l}");
    assert!(res.calcpad.contains("#noc\nfB = c*L + d\n#end noc"), "{}", res.calcpad);
    // The constraint does not stop the sheet: the later region converted.
    let nums = numbers(&res.calcpad);
    assert_close(*nums.last().unwrap(), 6.0);
    // Any other bare expression is still a bare expression.
    let res = convert(&[eval(&n("zz"))]);
    assert!(!res.report.gaps().any(|i| i.code == Code::ConstraintEquation));
}

#[test]
fn nth_root_with_an_empty_degree_is_the_square_root() {
    // sqrt(16 + 9) = 5.
    let res = convert(&[eval(
        "<ml:apply><ml:nthRoot/><ml:placeholder/><ml:apply><ml:plus/><ml:real>16</ml:real><ml:real>9</ml:real></ml:apply></ml:apply>",
    )]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_close(last(&res.calcpad), 5.0);
    // A degree that is given is not read.
    let res = convert(&[eval(
        "<ml:apply><ml:nthRoot/><ml:real>3</ml:real><ml:real>8</ml:real></ml:apply>",
    )]);
    assert!(res.report.gaps().any(|i| i.code == Code::UnknownElement));
}

#[test]
fn a_transposed_row_is_a_vector_the_engine_can_count() {
    // v := (4 7 9)^T ; len(v) = 3 ; max(v) = 9 ; w := (v^T)^T keeps working as a vector.
    let tr = |x: &str| format!("<ml:apply><ml:transpose/>{x}</ml:apply>");
    let res = convert(&[
        define("v", &tr(&mat(1, 3, &["4", "7", "9"]))),
        define("w", &tr(&tr(&n("v")))),
        eval(&call("length", &[n("w")])),
        eval(&call("max", &[n("v")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(!res.calcpad.contains("transp("), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 3.0);
    assert_close(nums[nums.len() - 1], 9.0);
}
