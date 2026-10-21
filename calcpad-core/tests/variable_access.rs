//! `ExpressionParser::variable`: a variable's value after a parse, for callers that read results.
use calcpad_core::base_types::value::Value;
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn real(p: &ExpressionParser, name: &str) -> f64 {
    match p.variable(name) {
        Some(Value::Real(r)) => r.d,
        other => panic!("{name}: {other:?}"),
    }
}

#[test]
fn reads_values_after_a_parse() {
    let mut p = ExpressionParser::new();
    p.parse("a = 3\nb = a^2 + 1\nc = 2kN*3m/(kN*m)", true);
    assert_eq!(real(&p, "a"), 3.0);
    assert_eq!(real(&p, "b"), 10.0);
    assert_eq!(real(&p, "c"), 6.0);
    assert!(p.variable("undefined_name").is_none());
}

#[test]
fn reads_the_value_an_input_field_takes() {
    let mut p = ExpressionParser::new();
    p.parse("L = ? {4.5}m\nA = L*2m", true);
    match p.variable("A") {
        Some(Value::Real(r)) => {
            assert_eq!(r.d, 9.0);
            assert!(r.units.is_some());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_new_parse_replaces_the_values() {
    let mut p = ExpressionParser::new();
    p.parse("a = 1", true);
    p.parse("b = 2", true);
    assert!(p.variable("a").is_none());
    assert_eq!(real(&p, "b"), 2.0);
}
