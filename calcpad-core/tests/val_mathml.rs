//! The opt-in `valMathml` setting: `#val` writes a result as MathML, the unit once after the brackets.
//! Off (the default) `#val` is unchanged.
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn render(source: &str) -> String {
    let mut p = ExpressionParser::new();
    p.parse(source, true);
    p.html_result.clone()
}

#[test]
fn val_is_unchanged_unless_the_setting_is_on() {
    let h = render("x = [1.5; 2.5]*kN\n#val\nx\n#end val\n");
    assert!(h.contains("[1.5, 2.5]") && !h.contains("<mtable>"), "{h}");
}

#[test]
fn a_vector_is_a_bracketed_table_with_the_unit_once() {
    let h = render("#settings {\"valMathml\": true}\n#round 2\nx = [1.5; 2.5; -3]*kN\n#val\ntransp(x)\n#end val\n");
    assert!(
        h.contains("<mo>[</mo><mtable><mtr><mtd><mn>1.5</mn></mtd><mtd><mn>2.5</mn></mtd><mtd><mo>\u{2212}</mo><mn>3</mn></mtd></mtr></mtable><mo>]</mo>"),
        "{h}"
    );
    assert_eq!(h.matches("<mi mathvariant=\"normal\">kN</mi>").count(), 1, "{h}");
}

#[test]
fn a_scalar_has_its_number_and_a_compound_unit() {
    let h = render("#settings {\"valMathml\": true}\n#round 3\ng = 1234.5678*kN*m/s^2\n#val\ng\n#end val\n");
    assert!(h.contains("<mn>1234.568</mn>"), "{h}");
    assert!(
        h.contains("<mo>\u{22C5}</mo>") && h.contains("<msup><mi mathvariant=\"normal\">s</mi><mn>2</mn></msup>"),
        "{h}"
    );
}
