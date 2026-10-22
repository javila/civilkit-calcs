//! The Excel report export (`html_to_xlsx`): the Report sheet read back from the package.
use calcpad_core::openxml::variables_export::Sheet as DataSheet;
use calcpad_core::openxml::xml::{self, Element};
use calcpad_core::openxml::zip;
use calcpad_docx::html_to_xlsx;

fn part(book: &[u8], name: &str) -> Option<Vec<u8>> {
    zip::read(book)
        .unwrap()
        .into_iter()
        .find(|e| e.name == name)
        .map(|e| e.data)
}

fn xml_part(book: &[u8], name: &str) -> Element {
    xml::parse(&String::from_utf8(part(book, name).unwrap_or_else(|| panic!("no part {name}"))).unwrap()).unwrap()
}

/// (row, col) to the cell's text or number text, for sheet N.
fn cells(book: &[u8], sheet: usize) -> Vec<(u32, u32, String)> {
    let root = xml_part(book, &format!("xl/worksheets/sheet{sheet}.xml"));
    let mut out = Vec::new();
    let mut rows = Vec::new();
    root.descendants("row", &mut rows);
    for row in rows {
        let r: u32 = row.attr("r").unwrap().parse().unwrap();
        for c in row.elements().filter(|e| xml::local(&e.name) == "c") {
            let reference = c.attr("r").unwrap();
            let letters: String = reference.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
            let col = letters.bytes().fold(0u32, |a, b| a * 26 + (b - b'A' + 1) as u32);
            let text = c.inner_text();
            out.push((r, col, text));
        }
    }
    out
}

fn at(cells: &[(u32, u32, String)], row: u32, col: u32) -> Option<&str> {
    cells
        .iter()
        .find(|(r, c, _)| *r == row && *c == col)
        .map(|(_, _, t)| t.as_str())
}

fn sheet_names(book: &[u8]) -> Vec<String> {
    let wb = xml_part(book, "xl/workbook.xml");
    let mut sheets = Vec::new();
    wb.descendants("sheet", &mut sheets);
    sheets.iter().map(|s| s.attr("name").unwrap().to_string()).collect()
}

const PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, 0x49, 0x48, 0x44, 0x52, 0, 0, 0, 2, 0, 0, 0, 2, 8,
    6, 0, 0, 0, 0x72, 0xB6, 0x0D, 0x24, 0, 0, 0, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0, 0, 0, 2, 0, 1,
    0xE5, 0x27, 0xDE, 0xFC, 0, 0, 0, 0, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn b64(data: &[u8]) -> String {
    calcpad_core::parsers::expression_parser::base64_encode(data)
}

fn variable_sheets() -> Vec<DataSheet> {
    vec![
        DataSheet {
            name: "Variables".into(),
            rows: vec![
                vec!["Name".into(), "Value".into(), "Unit".into()],
                vec!["h".into(), "550".into(), "".into()],
                vec!["V".into(), "vector of 2 (sheet \"V\")".into(), "".into()],
            ],
            variable: None,
        },
        DataSheet {
            name: "V".into(),
            rows: vec![
                vec!["Index".into(), "Value".into(), "Unit".into()],
                vec!["1".into(), "1".into(), "".into()],
            ],
            variable: Some("V".into()),
        },
        DataSheet {
            name: "Inputs".into(),
            rows: vec![vec!["Name".into(), "Value".into(), "Unit".into()]],
            variable: None,
        },
    ]
}

fn page() -> String {
    format!(
        r#"<html><body>
<h3>Beam <hr/><small>According to the code</small></h3>
<p>Span - <span class="eq"><var>h</var> = 550</span> mm, and <span class="eq"><var>A</var> = 12</span> mm<sup>2</sup></p>
<p>Width with <span class="eq"><var>w</var> = 5</span> with diameter</p>
<p><span class="eq"><var>M</var> = <var>q</var> &middot; <var>L</var> = 10 &middot; 5 = 50 <i>kN</i><i>&middot;</i><i>m</i></span></p>
<p><span class="eq"><var>V</var> = <b class="b0">[</b><span class="matrix"><span class="tr"><span class="td">1</span><span class="td">2</span></span></span><b class="b0">]</b></span></p>
<p><span class="eq"><var>V</var> = <span class="b0">[</span>1; 2<span class="b0">]</span> = <b class="b0">[</b>1 <i>m</i> 2 <i>m</i><b class="b0">]</b></span></p>
<p><span class="eq"><var>x</var> = <span class="dvc">a<span class="dvl"></span>b</span> = 1.5&middot;10<sup>-3</sup></span></p>
<table><tr><th>Bar</th><th>Area</th></tr><tr><td>1</td><td>2.5</td></tr><tr><td>2</td><td>4</td></tr></table>
<table><tr><td><h4>Layout</h4><p>text in a layout table</p></td></tr></table>
<p style="display:none">hidden text</p>
<svg width="40" height="20" viewBox="0 0 40 20" xmlns="http://www.w3.org/2000/svg"><rect width="40" height="20" fill="red"/></svg>
<img alt="dot" width="10" src="data:image/png;base64,{}"/>
<p>last</p>
</body></html>"#,
        b64(PNG)
    )
}

#[test]
fn the_report_sheet_comes_first_then_the_variables() {
    let book = html_to_xlsx(&page(), &variable_sheets(), &[], None).unwrap();
    assert_eq!(sheet_names(&book), ["Report", "Variables", "V", "Inputs"]);
}

#[test]
fn headings_text_and_equations_are_cells_in_document_order() {
    let book = html_to_xlsx(&page(), &variable_sheets(), &[], None).unwrap();
    let c = cells(&book, 1);
    assert_eq!(at(&c, 1, 1), Some("Description"));
    assert_eq!(at(&c, 2, 1), Some("Beam"));
    assert_eq!(at(&c, 3, 1), Some("According to the code"));
    // A label before an equation is its description; the unit written after the value is its unit.
    assert_eq!(
        (at(&c, 4, 1), at(&c, 4, 2), at(&c, 4, 5), at(&c, 4, 6)),
        (Some("Span"), Some("h"), Some("550"), Some("mm"))
    );
    // The exponent of that unit follows too.
    assert_eq!(
        (at(&c, 5, 2), at(&c, 5, 5), at(&c, 5, 6)),
        (Some("A"), Some("12"), Some("mm\u{00B2}"))
    );
    // A word that is not a unit is text.
    assert_eq!((at(&c, 6, 2), at(&c, 6, 5), at(&c, 6, 6)), (Some("w"), Some("5"), None));
    assert!(c.iter().any(|(_, _, t)| t.contains("with diameter")));
    // name, expression, expression with values, value and its own unit
    assert_eq!(
        (at(&c, 8, 2), at(&c, 8, 3), at(&c, 8, 4), at(&c, 8, 5), at(&c, 8, 6)),
        (
            Some("M"),
            Some("q \u{00B7} L"),
            Some("10 \u{00B7} 5"),
            Some("50"),
            Some("kN\u{00B7}m")
        )
    );
    // a value written as m times a power of ten is a number
    let row = c.iter().find(|(_, col, t)| *col == 2 && t == "x").unwrap().0;
    assert_eq!(at(&c, row, 3), Some("(a)/(b)"));
    assert_eq!(at(&c, row, 5).and_then(|t| t.parse::<f64>().ok()), Some(0.0015));
    assert!(!c.iter().any(|(_, _, t)| t.contains("hidden")));
}

#[test]
fn a_matrix_value_links_to_its_sheet() {
    let book = html_to_xlsx(&page(), &variable_sheets(), &[], None).unwrap();
    let c = cells(&book, 1);
    let row = c.iter().find(|(_, col, t)| *col == 2 && t == "V").unwrap().0;
    assert_eq!(at(&c, row, 3), Some("[1; 2]"));
    assert_eq!(at(&c, row, 5), Some("see sheet \"V\""));
    let sheet = String::from_utf8(part(&book, "xl/worksheets/sheet1.xml").unwrap()).unwrap();
    assert!(sheet.contains("location=\"'V'!A1\""), "{sheet}");
}

#[test]
fn a_vector_written_inline_links_to_its_sheet_too() {
    let book = html_to_xlsx(&page(), &variable_sheets(), &[], None).unwrap();
    let c = cells(&book, 1);
    let rows: Vec<u32> = c
        .iter()
        .filter(|(_, col, t)| *col == 2 && t == "V")
        .map(|(r, _, _)| *r)
        .collect();
    assert_eq!(rows.len(), 2);
    let row = rows[1];
    assert_eq!(
        (at(&c, row, 3), at(&c, row, 4), at(&c, row, 5)),
        (Some("[1; 2]"), Some("[1 m 2 m]"), Some("see sheet \"V\""))
    );
}

#[test]
fn a_plain_table_is_a_cell_range_and_a_layout_table_is_read_in_order() {
    let book = html_to_xlsx(&page(), &variable_sheets(), &[], None).unwrap();
    let c = cells(&book, 1);
    let header = c.iter().find(|(_, col, t)| *col == 1 && t == "Bar").unwrap().0;
    assert_eq!(
        (at(&c, header, 2), at(&c, header + 1, 1), at(&c, header + 1, 2)),
        (Some("Area"), Some("1"), Some("2.5"))
    );
    assert_eq!(at(&c, header + 2, 2), Some("4"));
    let layout = c.iter().find(|(_, _, t)| t == "Layout").unwrap().0;
    assert_eq!(at(&c, layout + 1, 1), Some("text in a layout table"));
}

#[test]
fn drawings_and_images_are_pictures_in_the_package() {
    let book = html_to_xlsx(&page(), &variable_sheets(), &[], None).unwrap();
    let drawing = xml_part(&book, "xl/drawings/drawing1.xml");
    let mut anchors = Vec::new();
    drawing.descendants("oneCellAnchor", &mut anchors);
    assert_eq!(anchors.len(), 2);
    let names: Vec<String> = zip::read(&book)
        .unwrap()
        .into_iter()
        .map(|e| e.name)
        .filter(|n| n.starts_with("xl/media/"))
        .collect();
    assert_eq!(names.len(), 2);
    // The SVG was drawn to a PNG.
    let svg_png = part(&book, "xl/media/image1.png").unwrap();
    assert_eq!(&svg_png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    // The text after the pictures still follows them, below the space they take.
    let c = cells(&book, 1);
    let last = c.iter().find(|(_, _, t)| t == "last").unwrap().0;
    let from_rows: Vec<u32> = {
        let mut rows = Vec::new();
        drawing.descendants("row", &mut rows);
        rows.iter().map(|r| r.inner_text().parse().unwrap()).collect()
    };
    assert!(from_rows.iter().all(|r| *r < last));
}

#[test]
fn the_variables_sheet_takes_the_units_the_text_gave() {
    let book = html_to_xlsx(&page(), &variable_sheets(), &[], None).unwrap();
    let c = cells(&book, 2);
    assert_eq!(
        (at(&c, 2, 1), at(&c, 2, 2), at(&c, 2, 3), at(&c, 2, 4)),
        (
            Some("h"),
            Some("550"),
            Some("mm"),
            Some("unit read from the text after the value")
        )
    );
    assert_eq!(at(&c, 1, 4), Some("Note"));
}

fn sample_plots() -> Vec<calcpad_core::plotter::capture::CapturedChart> {
    use calcpad_core::plotter::capture::{CapturedChart, CapturedSeries};
    let series = |name: &str, color: &str, k: f64| CapturedSeries {
        name: name.into(),
        color: color.into(),
        points: (0..=4).map(|i| (i as f64, k * i as f64)).collect(),
        dot: false,
        fill: false,
    };
    vec![CapturedChart {
        x_name: "x".into(),
        x_unit: "m".into(),
        y_unit: "kN".into(),
        series: vec![series("2x", "FF6347", 2.0), series("3x", "9ACD32", 3.0)],
        bounds: (-1.2e-17, 0.0, 4.0, 11.999999999999998),
    }]
}

const PLOT_PAGE: &str = "<html><body><p>before</p><svg class=\"plot\" data-plot=\"svg\" width=\"500px\" height=\"350px\" style=\"width:370pt; height:259pt;\" viewbox=\" 0 0 500 350\"><rect width=\"500\" height=\"350\"/></svg><svg width=\"100px\" height=\"80px\"><rect width=\"100\" height=\"80\"/></svg><p>after</p></body></html>";

#[test]
fn a_plot_is_a_native_chart_with_its_data_on_its_own_sheet() {
    let book = html_to_xlsx(PLOT_PAGE, &variable_sheets(), &sample_plots(), None).unwrap();
    // One chart for the plot, one picture for the drawing that is not a plot.
    let names = sheet_names(&book);
    assert_eq!(names.last().map(String::as_str), Some("Plot data"));
    assert!(part(&book, "xl/charts/chart1.xml").is_some());
    assert!(part(&book, "xl/charts/chart2.xml").is_none());
    assert!(part(&book, "xl/media/image1.png").is_some());
    assert!(part(&book, "xl/media/image2.png").is_none());
    let drawing = xml_part(&book, "xl/drawings/drawing1.xml");
    let mut frames = Vec::new();
    drawing.descendants("graphicFrame", &mut frames);
    assert_eq!(frames.len(), 1);
    let rels = String::from_utf8(part(&book, "xl/drawings/_rels/drawing1.xml.rels").unwrap()).unwrap();
    assert!(rels.contains("relationships/chart\" Target=\"../charts/chart1.xml\""));
    let types = String::from_utf8(part(&book, "[Content_Types].xml").unwrap()).unwrap();
    assert!(types.contains("/xl/charts/chart1.xml"));
    // The data: two columns per curve, headers in rows 1 and 2, values from row 3.
    let data = cells(&book, names.len());
    assert_eq!(at(&data, 2, 1), Some("x [m]"));
    assert_eq!(at(&data, 2, 2), Some("2x [kN]"));
    assert_eq!(at(&data, 7, 2), Some("8"));
    assert_eq!(at(&data, 7, 4), Some("12"));
}

#[test]
fn the_chart_xml_is_in_schema_order_and_points_at_the_data() {
    let book = html_to_xlsx(PLOT_PAGE, &variable_sheets(), &sample_plots(), None).unwrap();
    let chart = xml_part(&book, "xl/charts/chart1.xml");
    let order = |el: &Element| -> Vec<String> { el.elements().map(|e| xml::local(&e.name).to_string()).collect() };
    assert_eq!(order(&chart), ["roundedCorners", "chart", "spPr"]);
    let c = chart.elements().find(|e| xml::local(&e.name) == "chart").unwrap();
    assert_eq!(
        order(c),
        ["autoTitleDeleted", "plotArea", "legend", "plotVisOnly", "dispBlanksAs"]
    );
    let mut area = Vec::new();
    chart.descendants("plotArea", &mut area);
    assert_eq!(order(area[0]), ["layout", "scatterChart", "valAx", "valAx", "spPr"]);
    let mut sc = Vec::new();
    chart.descendants("scatterChart", &mut sc);
    assert_eq!(
        order(sc[0]),
        ["scatterStyle", "varyColors", "ser", "ser", "axId", "axId"]
    );
    let mut ser = Vec::new();
    chart.descendants("ser", &mut ser);
    assert_eq!(
        order(ser[0]),
        ["idx", "order", "tx", "spPr", "marker", "xVal", "yVal", "smooth"]
    );
    let mut ax = Vec::new();
    chart.descendants("valAx", &mut ax);
    assert_eq!(
        order(ax[0]),
        [
            "axId",
            "scaling",
            "delete",
            "axPos",
            "majorGridlines",
            "title",
            "numFmt",
            "majorTickMark",
            "minorTickMark",
            "tickLblPos",
            "spPr",
            "crossAx",
            "crosses",
            "crossBetween",
            "majorUnit"
        ]
    );
    let mut f = Vec::new();
    chart.descendants("f", &mut f);
    let refs: Vec<String> = f.iter().map(|e| e.inner_text()).collect();
    assert_eq!(
        refs,
        [
            "'Plot data'!$A$3:$A$7",
            "'Plot data'!$B$3:$B$7",
            "'Plot data'!$C$3:$C$7",
            "'Plot data'!$D$3:$D$7"
        ]
    );
    // The axes carry the plotted limits and titles with units.
    let mut max = Vec::new();
    ax[1].descendants("max", &mut max);
    assert_eq!(max[0].attr("val"), Some("12"));
    // Clean limits and a round major unit, whatever noise the plotter's arithmetic left.
    let mut unit = Vec::new();
    ax[1].descendants("majorUnit", &mut unit);
    assert_eq!(unit[0].attr("val"), Some("2"));
    assert!(chart.inner_text().contains("x [m]"));
}

#[test]
fn a_plot_without_captured_data_stays_a_picture() {
    let book = html_to_xlsx(PLOT_PAGE, &variable_sheets(), &[], None).unwrap();
    assert!(part(&book, "xl/charts/chart1.xml").is_none());
    assert!(part(&book, "xl/media/image2.png").is_some());
    assert!(!sheet_names(&book).contains(&"Plot data".to_string()));
}
