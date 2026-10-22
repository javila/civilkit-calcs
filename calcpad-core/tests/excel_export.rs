//! The Excel export (calcpad-rs's own feature): the variables a parse records with
//! `capture_values`, written by `openxml::variables_export::workbook` and read back with the
//! workbook reader that `#read` uses.
use calcpad_core::openxml::{excel_data, variables_export, zip};
use calcpad_core::parsers::expression_parser::ExpressionParser;

const SHEET: &str = "'Excel export check\na = 2\nL = ? {4.5}m\n#UI F = 12.5kN\nM = F*L\nv = [1; 2; 3]*1m\nK = [1; 2|3; 4]\na = a + 1\nb = a*2";

fn book() -> Vec<u8> {
    let mut p = ExpressionParser::new();
    p.capture_values = true;
    p.parse(SHEET, true);
    variables_export::workbook(&p.captured_values).unwrap()
}

fn sheet(book: &[u8], name: &str, end: &str) -> Vec<Vec<String>> {
    excel_data::read(book, name, "A1", end).unwrap().unwrap()
}

fn s(rows: &[&[&str]]) -> Vec<Vec<String>> {
    rows.iter().map(|r| r.iter().map(|c| c.to_string()).collect()).collect()
}

#[test]
fn variables_in_calculation_order_with_values_and_units() {
    let b = book();
    assert_eq!(
        sheet(&b, "Variables", "C9"),
        s(&[
            &["Name", "Value", "Unit"],
            &["a", "3", ""],
            &["L", "4.5", "m"],
            &["F", "12.5", "kN"],
            &["M", "56.25", "kNm"],
            &["v", "vector of 3 (sheet \"v\")", ""],
            &["K", "matrix 2 × 2 (sheet \"K\")", ""],
            &["b", "6", ""],
        ])
    );
}

#[test]
fn vectors_and_matrices_have_their_own_sheets() {
    let b = book();
    assert_eq!(
        sheet(&b, "v", "C4"),
        s(&[
            &["Index", "Value", "Unit"],
            &["1", "1", "m"],
            &["2", "2", "m"],
            &["3", "3", "m"]
        ])
    );
    assert_eq!(
        sheet(&b, "K", "D3"),
        s(&[&["", "1", "2", "Unit"], &["1", "1", "2", ""], &["2", "3", "4", ""]])
    );
}

#[test]
fn inputs_are_the_fields_and_controls() {
    assert_eq!(
        sheet(&book(), "Inputs", "C3"),
        s(&[&["Name", "Value", "Unit"], &["L", "4.5", "m"], &["F", "12.5", "kN"]])
    );
}

#[test]
fn numbers_are_numeric_cells_and_text_is_text() {
    let b = book();
    let parts = zip::read(&b).unwrap();
    let names: Vec<&str> = parts.iter().map(|p| p.name.as_str()).collect();
    let first = parts
        .iter()
        .find(|p| p.name.ends_with("sheet1.xml"))
        .unwrap_or_else(|| panic!("{names:?}"));
    let xml = String::from_utf8(first.data.clone()).unwrap();
    // "56.25" (M) is a number, its unit an inline string.
    assert!(xml.contains("<x:c r=\"B5\" t=\"n\"><x:v>56.25</x:v></x:c>"), "{xml}");
    assert!(
        xml.contains("<x:c r=\"C5\" t=\"inlineStr\"><x:is><x:t>kNm</x:t></x:is></x:c>"),
        "{xml}"
    );
}

#[test]
fn nothing_is_recorded_without_the_option() {
    let mut p = ExpressionParser::new();
    p.parse(SHEET, true);
    assert!(p.captured_values.is_empty());
}

#[test]
fn the_sheets_survive_json_and_name_the_variable_of_a_vector_sheet() {
    let mut p = ExpressionParser::new();
    p.capture_values = true;
    p.parse(SHEET, true);
    let sheets = variables_export::sheets(&p.captured_values);
    let names: Vec<&str> = sheets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Variables", "v", "K", "Inputs"]);
    assert_eq!(sheets[1].variable.as_deref(), Some("v"));
    assert_eq!(sheets[0].variable, None);
    let back = variables_export::sheets_from_json(&variables_export::sheets_to_json(&sheets));
    assert_eq!(back, sheets);
}

fn plots(text: &str) -> Vec<calcpad_core::plotter::capture::CapturedChart> {
    let mut p = ExpressionParser::new();
    p.capture_values = true;
    p.plot_settings.vector_graphics = true;
    p.parse(text, true);
    p.captured_plots.clone()
}

#[test]
fn a_plot_records_its_curves_axes_and_colours() {
    let c = plots("$Plot{x^2 & 2*x @ x = 0 : 2}");
    assert_eq!(c.len(), 1);
    let c = &c[0];
    assert_eq!(c.x_name, "x");
    assert_eq!(c.series.len(), 2);
    assert_eq!(c.series[0].name, "x^2");
    assert_eq!(c.series[1].name, "2*x");
    assert_eq!(c.series[0].color, "FF6347");
    assert_eq!(c.series[1].color, "9ACD32");
    // The adaptive plot starts from 32 points and adds more where the curve bends; it ends at (2, 4).
    assert!(c.series[0].points.len() >= 32);
    let last = *c.series[0].points.last().unwrap();
    assert!((last.0 - 2.0).abs() < 1e-12 && (last.1 - 4.0).abs() < 1e-12);
    // The limits the SVG used.
    // (the step is added 512 times, so the limits carry a little noise)
    assert!(
        c.bounds.0 <= 0.0 && (c.bounds.2 - 2.0).abs() < 1e-9 && (c.bounds.3 - 4.0).abs() < 1e-9,
        "{:?}",
        c.bounds
    );
}

#[test]
fn a_parametric_plot_and_units_are_kept_and_a_map_is_not_a_chart() {
    let c = plots("$Plot{3*cos(t)*1m | 3*sin(t)*1m @ t = 0 : 6.28}");
    assert_eq!(c[0].series[0].name, "3*cos(t)*1m | 3*sin(t)*1m");
    assert_eq!(c[0].x_unit, "m");
    assert_eq!(c[0].y_unit, "m");
    assert_eq!(plots("$Map{x*y @ x = 0 : 1 & y = 0 : 1}").len(), 0);
    // The data survives the JSON hop to the browser's Excel module.
    let json = calcpad_core::plotter::capture::charts_to_json(&c);
    assert_eq!(calcpad_core::plotter::capture::charts_from_json(&json), c);
}
