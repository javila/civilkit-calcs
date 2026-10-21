//! Office Math (OMML) from the XmlWriter port (Output/XmlWriter.cs), captured with
//! `ExpressionParser::capture_open_xml` as Calcpad.Server's /docx does. Upstream has no tests
//! for it; each expectation below follows XmlWriter.cs.
use calcpad_core::openxml::xml;
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn capture(source: &str) -> (Vec<String>, String) {
    let mut p = ExpressionParser::new();
    p.capture_open_xml = true;
    p.parse(source, true);
    (p.open_xml_expressions.clone(), p.html_result.clone())
}

fn one(source: &str) -> String {
    let (x, _) = capture(source);
    assert_eq!(x.len(), 1, "{source}: {x:?}");
    // Each equation is well-formed XML inside an oMath with the two namespaces.
    let doc = format!(
        "<m:oMath xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">{}</m:oMath>",
        x[0]
    );
    xml::parse(&doc).unwrap_or_else(|e| panic!("{source}: {e}\n{}", x[0]));
    x[0].clone()
}

const VAR: &str = "<w:rPr><w:color w:val=\"0066DD\"/></w:rPr>";

#[test]
fn numbers_variables_and_assignment_are_runs() {
    assert_eq!(
        one("a = 2"),
        format!("<m:r>{VAR}<m:t>a</m:t></m:r><m:r><m:t>=</m:t></m:r><m:r><m:t>2</m:t></m:r>")
    );
}

#[test]
fn every_equation_is_captured_and_numbered_in_the_html() {
    let (x, html) = capture("a = 2\n'text\nb = a + 1\nc = b*a");
    assert_eq!(x.len(), 3);
    for i in 0..3 {
        assert!(html.contains(&format!("<span class=\"eq\" id=\"eq-{i}\">")), "{html}");
    }
    // Without the capture the spans carry no ids (the HTML of the preview is unchanged).
    let mut p = ExpressionParser::new();
    p.parse("a = 2", true);
    assert!(p.open_xml_expressions.is_empty());
    assert!(!p.html_result.contains("id=\"eq-"));
}

#[test]
fn division_power_and_root() {
    let s = one("x = 2^3/4 + sqr(9)");
    assert!(s.contains("<m:f><m:num><m:sSup><m:e><m:r><m:t>2</m:t></m:r></m:e><m:sup><m:r><m:t>3</m:t></m:r></m:sup></m:sSup></m:num><m:den><m:r><m:t>4</m:t></m:r></m:den></m:f>"), "{s}");
    assert!(
        s.contains(
            "<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg/><m:e><m:r><m:t>9</m:t></m:r></m:e></m:rad>"
        ),
        "{s}"
    );
    let c = one("y = root(8; 3)");
    assert!(c.contains("<m:rad><m:deg><m:r><m:t>3</m:t></m:r></m:deg>"), "{c}");
}

#[test]
fn subscripts_and_units() {
    let s = one("f_ck = 25MPa");
    assert!(
        s.starts_with(&format!(
            "<m:sSub><m:e><m:r>{VAR}<m:t>f</m:t></m:r></m:e><m:sub><m:r><m:t>ck</m:t></m:r></m:sub></m:sSub>"
        )),
        "{s}"
    );
    // Value, thin space, units in Cambria Math as normal text.
    assert!(s.ends_with("<m:r><m:t>25</m:t></m:r><m:r><m:t>\u{2009}</m:t></m:r><m:r><m:rPr><m:nor/></m:rPr><w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\" /><w:sz w:val=\"22\" /></w:rPr><m:t>MPa</m:t></m:r>"), "{s}");
    // In an expression units are written as typed (here a fraction); a result's units divide
    // with the division slash (XmlWriter.UnitDivision).
    let (x, _) = capture("d = 6m\nt = 2s\nv = d/t");
    assert!(x[2].contains("<m:t>\u{2215}</m:t>"), "{}", x[2]);
}

#[test]
fn large_numbers_are_powers_of_ten() {
    let s = one("y = 1.5*10^20");
    assert!(s.contains("<m:t>\u{00D7}</m:t></m:r><m:sSup><m:e><m:r><m:t>10</m:t></m:r></m:e><m:sup><m:r><m:t>20</m:t></m:r></m:sup></m:sSup>"), "{s}");
}

#[test]
fn conditions_vectors_and_matrices() {
    let s = one("b = if(2 > 1; 1; 0)");
    assert!(
        s.contains("<m:d><m:dPr><m:begChr m:val=\"{\"/><m:endChr m:val=\" \"/></m:dPr><m:e><m:m>"),
        "{s}"
    );
    assert!(
        s.contains("<m:t> if </m:t>") && s.contains("<m:t> else:</m:t>") && s.contains("<m:t>&gt;</m:t>"),
        "{s}"
    );
    let v = one("v = [1; 2; 3]");
    assert!(
        v.starts_with("<m:acc><m:accPr><m:chr m:val=\"\u{20D7}\"/></m:accPr>"),
        "a vector name has an arrow accent: {v}"
    );
    assert!(v.contains("<m:begChr m:val=\"[\"/><m:endChr m:val=\"]\"/>"), "{v}");
    let m = one("M = [1; 2|3; 4]");
    assert!(
        m.contains("<m:count m:val=\"2\"/><m:mcJc m:val=\"center\"/>") && m.matches("<m:mr>").count() == 2,
        "{m}"
    );
}

#[test]
fn sums_and_solver_blocks() {
    let s = one("s = $Sum{k^2 @ k = 1 : 3}");
    assert!(
        s.contains("<m:nary><m:naryPr><m:chr m:val=\"\u{2211}\"/><m:limLoc m:val=\"undOvr\"/></m:naryPr><m:sub>"),
        "{s}"
    );
    let r = one("x = $Root{x^2 - 2 @ x = 0 : 2}");
    assert!(
        r.contains("<m:t>\u{2208}</m:t>") && r.contains("<m:begChr m:val=\"{\"/><m:endChr m:val=\"}\"/>"),
        "{r}"
    );
}

#[test]
fn a_formatted_unit_power_is_not_doubled() {
    // FixPowerFormatting: upstream kept the number's first character in front of the
    // reformatted number, so a unit power "2" became "22" (DEVIATIONS D6).
    let s = one("#format F2\nA = 3m^2");
    assert!(!s.contains("<m:t>22</m:t>"), "{s}");
}
