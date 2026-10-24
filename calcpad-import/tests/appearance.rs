//! How text and pictures look: styles read from the file, a picture beside text, quiet plain pictures.
//! Every file is invented here.

use calcpad_import::{import_xmcd, ImportResult};

const TINY_PNG: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

fn sheet(styles: &str, regions: &str) -> String {
    format!(
        r##"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30"><settings><presentation><textRendering><textStyles>{styles}</textStyles></textRendering></presentation></settings><regions>{regions}</regions><binaryContent><item item-id="1">{TINY_PNG}</item></binaryContent></worksheet>"##
    )
}

fn text_region(id: u32, left: f64, top: f64, h: f64, base: f64, paras: &str) -> String {
    format!(
        r#"<region region-id="{id}" left="{left}" top="{top}" width="200" height="{h}" align-y="{base}"><text>{paras}</text></region>"#
    )
}

fn png_region(id: u32, left: f64, top: f64, w: f64, h: f64, base: f64) -> String {
    format!(
        r#"<region region-id="{id}" left="{left}" top="{top}" width="{w}" height="{h}" align-y="{base}"><png item-idref="1"/></region>"#
    )
}

fn styles() -> String {
    r##"<textStyle name="Normal"><blockAttr text-align="left"/><inlineAttr font-family="Times New Roman" font-size="12" font-weight="normal" font-style="normal" underline="false" color="inherit"/></textStyle>
<textStyle name="Heading 1"><blockAttr text-align="left"/><inlineAttr font-family="Arial" font-size="14" font-weight="bold" font-style="normal" underline="false" color="#800000"/></textStyle>"##
        .to_string()
}

fn lines(r: &ImportResult) -> Vec<String> {
    r.calcpad.lines().map(str::to_string).collect()
}

#[test]
fn a_picture_beside_two_lines_of_text_is_one_row_with_the_text_stacked() {
    // A picture 18..64 high; one text starts above its baseline, the next sits lower, both to its right.
    let xml = sheet(
        &styles(),
        &(png_region(1, 6.0, 18.0, 50.0, 46.0, 18.0)
            + &text_region(2, 78.0, 6.75, 14.0, 18.0, r#"<p style="Normal">Upper line</p>"#)
            + &text_region(3, 78.0, 27.75, 36.0, 42.0, r#"<p style="Normal">Lower line</p>"#)
            + &text_region(4, 6.0, 120.0, 14.0, 132.0, r#"<p style="Normal">Below</p>"#)),
    );
    let res = import_xmcd(&xml);
    let l = lines(&res);
    let at = |needle: &str| {
        l.iter()
            .position(|x| x.contains(needle))
            .unwrap_or_else(|| panic!("{needle}: {l:?}"))
    };
    let side = at("imported-side");
    let (img, up, low, below) = (at("<img"), at("Upper line"), at("Lower line"), at("Below"));
    // The picture, then both texts in order, inside the one block, and the later text outside it.
    assert!(side < img && img < up && up < low && low < below, "{l:?}");
    assert_eq!(l.iter().filter(|x| x.contains("imported-side")).count(), 1);
    let close = l
        .iter()
        .enumerate()
        .filter(|(i, x)| *i > low && x.as_str() == "'</div>")
        .map(|(i, _)| i)
        .nth(1);
    assert!(close.is_some_and(|c| c < below), "{l:?}");
}

#[test]
fn text_below_a_picture_is_not_pulled_beside_it() {
    let xml = sheet(
        &styles(),
        &(png_region(1, 6.0, 18.0, 50.0, 46.0, 18.0)
            + &text_region(2, 6.0, 80.0, 14.0, 92.0, r#"<p style="Normal">Under it</p>"#)),
    );
    assert!(!import_xmcd(&xml).calcpad.contains("imported-side"));
}

#[test]
fn a_plain_picture_region_is_quiet_and_keeps_its_size() {
    let xml = sheet(&styles(), &png_region(7, 6.0, 18.0, 120.0, 30.0, 18.0));
    let res = import_xmcd(&xml);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("width:120.0pt"), "{}", res.calcpad);
    assert!(!res.calcpad.contains("static picture"), "{}", res.calcpad);
    assert!(!res.calcpad.contains("other-region"), "{}", res.calcpad);
    assert!(res.report.issues.is_empty(), "{:?}", res.report.issues);
}

#[test]
fn a_style_gives_font_size_and_colour_and_a_heading_stays_a_heading() {
    let xml = sheet(
        &styles(),
        &text_region(
            1,
            6.0,
            10.0,
            14.0,
            20.0,
            r#"<p style="Heading 1">Title</p><p style="Normal">Body text</p>"#,
        ),
    );
    let res = import_xmcd(&xml);
    let l = lines(&res);
    let h = l.iter().find(|x| x.contains("Title")).expect("heading");
    assert!(h.starts_with("'<h1 "), "{h}");
    assert!(h.contains("class=\"imp-c-800000\""), "{h}");
    assert!(
        h.contains("font-family:Arial,Helvetica") && h.contains("font-size:14pt"),
        "{h}"
    );
    let b = l.iter().find(|x| x.contains("Body text")).expect("body");
    assert!(
        b.contains("font-family:Times New Roman,Times") && b.contains("font-size:12pt"),
        "{b}"
    );
    assert!(!b.contains("class="), "inherit is no colour: {b}");
    // No quote character that the engine would read as a switch to maths.
    assert!(!h.contains('\'') || h.starts_with('\''), "{h}");
    assert_eq!(h.matches('\'').count(), 1, "{h}");
    assert_eq!(b.matches('\'').count(), 1, "{b}");
}

#[test]
fn a_run_overrides_the_style_and_colours_work_on_a_light_and_a_dark_page() {
    let xml = sheet(
        &styles(),
        &text_region(
            1,
            6.0,
            10.0,
            14.0,
            20.0,
            r##"<p style="Normal">plain <f family="Courier New" size="9"><c val="#008080"><b>teal mono</b></c></f> end</p>"##,
        ),
    );
    let res = import_xmcd(&xml);
    let l = lines(&res);
    let t = l.iter().find(|x| x.contains("teal mono")).expect("line");
    assert!(t.contains("font-family:Courier New,Courier"), "{t}");
    assert!(t.contains("font-size:9pt"), "{t}");
    assert!(t.contains("<b>teal mono</b>"), "{t}");
    assert!(t.contains("imp-c-008080"), "{t}");
    // One rule for the light page and one that lightens the colour on the dark page.
    let style = l.iter().find(|x| x.starts_with("'<style>")).expect("style block");
    assert!(style.contains(".imp-c-008080{color:#008080}"), "{style}");
    let dark = style
        .split(".dark-theme .imp-c-008080{color:")
        .nth(1)
        .expect("dark rule");
    let hex = &dark[..7];
    let lum = |i: usize| f64::from(u8::from_str_radix(&hex[i..i + 2], 16).unwrap());
    assert!(
        lum(1).max(lum(3)).max(lum(5)) > 150.0,
        "not light enough on a dark page: {hex}"
    );
}

#[test]
fn black_white_and_greys_are_left_to_the_apps_text_colour() {
    let xml = sheet(
        &styles(),
        &text_region(
            1,
            6.0,
            10.0,
            14.0,
            20.0,
            r##"<p style="Normal"><c val="#000000">black</c> <c val="#ffffff">white</c></p>"##,
        ),
    );
    let res = import_xmcd(&xml);
    assert!(!res.calcpad.contains("imp-c-"), "{}", res.calcpad);
    assert!(!res.calcpad.contains("<style>"), "{}", res.calcpad);
}

#[test]
fn alignment_from_the_paragraph_is_kept() {
    let xml = sheet(
        &styles(),
        &text_region(
            1,
            6.0,
            10.0,
            14.0,
            20.0,
            r#"<p style="Normal" text-align="justify">Justified</p><p style="Normal" text-align="left">Left</p>"#,
        ),
    );
    let l = lines(&import_xmcd(&xml));
    let j = l.iter().find(|x| x.contains("Justified")).expect("line");
    assert!(j.contains("text-align:justify"), "{j}");
    let t = l.iter().find(|x| x.contains("Left")).expect("line");
    assert!(!t.contains("text-align"), "{t}");
}

#[test]
fn the_text_lines_run_through_the_engine_without_error() {
    let xml = sheet(
        &styles(),
        &(png_region(1, 6.0, 18.0, 50.0, 46.0, 18.0)
            + &text_region(
                2,
                78.0,
                6.75,
                14.0,
                18.0,
                r##"<p style="Heading 1"><c val="#008080">Chapter</c></p>"##,
            )
            + &text_region(3, 78.0, 27.75, 36.0, 42.0, r#"<p style="Normal">Lower line</p>"#)),
    );
    let res = import_xmcd(&xml);
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&res.calcpad, true);
    let html = parser.html_result.clone();
    assert!(!html.contains("class=\"err\""), "{html}");
    assert!(html.contains("<style>"), "{html}");
    assert!(html.contains("Lower line") && html.contains("Chapter"), "{html}");
    assert!(html.contains("<img"), "{html}");
}
