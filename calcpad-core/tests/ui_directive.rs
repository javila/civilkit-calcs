//! Transcribed by hand from CalcpadCE Calcpad.Tests/UI/UiDirectiveTests.cs (0cf438f). Each
//! [Theory] runs its [InlineData] cases in order; the assertions are the C# ones.
use std::collections::HashMap;

use calcpad_core::parsers::expression_parser::ExpressionParser;
use calcpad_core::ui_dto::UiDto;

fn render_with(source: &str, enable_ui: bool, overrides: Option<&[(&str, &str)]>) -> String {
    let mut parser = ExpressionParser::new();
    parser.enable_ui = enable_ui;
    parser.ui_overrides = overrides.map(|o| {
        o.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>()
    });
    // Mirrors CalcpadService's default (hideErrorLines ?? enableUi).
    parser.show_error_lines = !enable_ui;
    parser.parse(source, true);
    parser.html_result.clone()
}

fn render(source: &str, enable_ui: bool) -> String {
    render_with(source, enable_ui, None)
}

fn render_o(source: &str, enable_ui: bool, overrides: &[(&str, &str)]) -> String {
    render_with(source, enable_ui, Some(overrides))
}

#[track_caller]
fn contains(expected: &str, html: &str) {
    assert!(html.contains(expected), "expected {:?} in\n{}", expected, html);
}

#[track_caller]
fn not_contains(unexpected: &str, html: &str) {
    assert!(
        !html.contains(unexpected),
        "did not expect {:?} in\n{}",
        unexpected,
        html
    );
}

/// `Regex.Matches(html, pattern).Count` for the literal patterns these tests use.
fn count(html: &str, literal: &str) -> usize {
    html.matches(literal).count()
}

/// `UiPrefix().Replace(source, "$1")` with `(?m)^(\s*)#UI\s*(\{[^}]*\})?\s*`.
fn strip_ui_prefix(source: &str) -> String {
    source
        .split('\n')
        .map(|line| {
            let indent_len = line.len() - line.trim_start().len();
            let (indent, rest) = line.split_at(indent_len);
            let Some(mut rest) = rest.strip_prefix("#UI") else {
                return line.to_string();
            };
            rest = rest.trim_start();
            if rest.starts_with('{') {
                if let Some(end) = rest.find('}') {
                    rest = &rest[end + 1..];
                }
            }
            format!("{}{}", indent, rest.trim_start())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[track_caller]
fn assert_report_matches_plain_source(source: &str) {
    let plain = strip_ui_prefix(source);
    assert_eq!(render(&plain, false), render(source, false));
}

#[test]
fn report_mode_is_identical_to_source_without_the_keyword() {
    for source in [
        "#UI L = 10m\nA = L * 2m",
        "#UI {\"type\": \"checkbox\"} flag = 1\nflag * 2",
        "#UI {\"type\": \"dropdown\", \"keys\": [\"Low\", \"High\"], \"values\": [\"1\", \"3\"]} grade = 1",
        "#UI {\"type\": \"radio\", \"keys\": [\"Steel\", \"Concrete\"], \"values\": [\"200GPa\", \"25GPa\"]} E = 200GPa",
        "#UI v = [1; 2; 3]",
        "#UI M = [1; 2; 3 | 4; 5; 6]",
        "#UI Z = vector(5)",
        "#UI G = matrix(3; 4)",
        "#UI {\"type\": \"datagrid\", \"rows\": 3, \"columns\": 4} M = [1; 2; 3 | 4; 5; 6]",
    ] {
        assert_report_matches_plain_source(source);
    }
}

#[test]
fn report_mode_emits_no_ui_markup() {
    let html = render("#UI {\"type\": \"checkbox\"} flag = 1\n#UI v = [1; 2; 3]", false);
    not_contains("calcpad-ui", &html);
    not_contains("data-ui-", &html);
    not_contains("<input", &html);
}

#[test]
fn report_style_adds_class_and_is_the_only_difference() {
    let styled = render("#UI {\"reportStyle\": \"boxed\"} P = 25kN", false);
    let plain = render("P = 25kN", false);
    contains("class=\"boxed\"", &styled);
    assert_eq!(plain, styled.replace(" class=\"boxed\"", ""));
}

#[test]
fn report_style_is_not_emitted_in_ui_mode() {
    not_contains("boxed", &render("#UI {\"reportStyle\": \"boxed\"} P = 25kN", true));
}

#[test]
fn entry_is_auto_detected_and_keeps_the_unit_outside_the_input() {
    let html = render("#UI L = 10m", true);
    contains("class=\"calcpad-ui-input\"", &html);
    contains("data-ui-var=\"L:1\"", &html);
    contains("value=\"10\"", &html);
    not_contains("value=\"10 m\"", &html);
}

#[test]
fn entry_holds_the_value_without_any_of_the_unit() {
    for (source, value) in [
        ("#UI q = 3kN/m", "3"),
        ("#UI a = 9.81m/s^2", "9.81"),
        ("#UI T = 20\u{00B0}C", "20"),
        ("#UI t = 12\u{00B0}", "12"),
        ("#UI r = 50%", "50"),
        ("#UI z = -3.5", "-3.5"),
        ("#UI n = 4", "4"),
    ] {
        let html = render(source, true);
        contains(&format!("value=\"{}\"", value), &html);
        not_contains("&lt;", &html);
    }
}

#[test]
fn assigned_expression_is_an_error() {
    for source in [
        "#UI {\"type\": \"entry\"} k = sin(2)",
        "#UI k = 1 + 2",
        "#UI k = 2*L\nL = 1m",
    ] {
        let html = render(source, true);
        contains("does not support expressions", &html);
        not_contains("calcpad-ui", &html);
    }
}

#[test]
fn assigned_expression_is_allowed_where_the_whole_value_is_replaced() {
    for source in [
        "#UI v = [1; sqrt(4)]",
        "#UI G = matrix(2; 2) + 1",
        "#UI {\"allowExpression\": true} k = 1 + 2",
    ] {
        let html = render(source, true);
        not_contains("does not support expressions", &html);
        contains("calcpad-ui", &html);
    }
}

#[test]
fn inline_comment_is_not_mistaken_for_the_assignment() {
    let html = render("#UI '2&middot;<i>r</i> ='d = 1", true);
    contains("data-ui-var=\"d:1\"", &html);
    not_contains("data-ui-var=\"'2", &html);
}

#[test]
fn inline_comment_is_stripped_from_the_variable_name() {
    for (source, key) in [
        ("#UI 'label ='x = 1", "x:1"),
        ("#UI \"label =\"y = 2", "y:1"),
        ("#UI 'a ='  z  = 3", "z:1"),
    ] {
        contains(&format!("data-ui-var=\"{}\"", key), &render(source, true));
    }
}

#[test]
fn inline_comment_leaves_the_report_unchanged() {
    assert_report_matches_plain_source("#UI '2&middot;<i>r</i> ='d = 1\nA = d*2");
}

#[test]
fn inline_comment_does_not_break_overrides() {
    let html = render_o("#UI '2&middot;<i>r</i> ='d = 1\nA = d*2", false, &[("d:1", "4")]);
    contains("8", &html);
}

#[test]
fn multiple_assignments_each_get_their_own_control() {
    let html = render("#UI '2&middot;<i>r</i> ='d = 1','x = 2'", true);
    contains("data-ui-var=\"d:1\"", &html);
    contains("data-ui-var=\"x:1\"", &html);
    assert_eq!(2, count(&html, "calcpad-ui-input"));
}

#[test]
fn multiple_assignments_share_the_json_properties() {
    let html = render("#UI {\"type\": \"checkbox\", \"style\": \"sw\"} a = 1', 'b = 0", true);
    assert_eq!(2, count(&html, "class=\"calcpad-ui-checkbox sw\""));
}

#[test]
fn multiple_assignments_are_overridden_separately() {
    let html = render_o("#UI d = 1', 'x = 2\nS = d + x", false, &[("x:1", "9")]);
    contains("10", &html);
}

#[test]
fn same_name_twice_on_one_line_gets_two_keys() {
    let html = render("#UI d = 1', 'd = 2", true);
    contains("data-ui-var=\"d:1\"", &html);
    contains("data-ui-var=\"d:2\"", &html);
}

#[test]
fn multiple_assignments_leave_the_report_unchanged() {
    assert_report_matches_plain_source("#UI '2&middot;<i>r</i> ='d = 1','x = 2'\nA = d*x");
}

#[test]
fn multiple_assignments_skip_segments_that_assign_nothing() {
    let html = render("#UI d = 1', 'd", true);
    assert_eq!(1, count(&html, "calcpad-ui-input"));
}

#[test]
fn multiple_datagrids_each_emit_their_own_container() {
    let html = render("#UI v = [1; 2]', 'w = [3; 4; 5]", true);
    contains("data-ui-values=\"1;2\"", &html);
    contains("data-ui-values=\"3;4;5\"", &html);
    assert_eq!(2, count(&html, "class=\"calcpad-ui-datagrid\""));
}

#[test]
fn control_binding_reports_the_source_line() {
    let html = render("'first\n'second\n#UI L = 10m", true);
    contains("data-ui-var=\"L:1\" data-ui-line=\"3\"", &html);
}

#[test]
fn style_is_appended_to_the_control_class() {
    contains(
        "class=\"calcpad-ui-input highlight\"",
        &render("#UI {\"style\": \"highlight\"} d = 2m", true),
    );
}

#[test]
fn entry_keeps_the_variable_name_in_ui_mode() {
    contains(" = ", &render("#UI L = 10m", true));
}

#[test]
fn control_drops_the_name_and_unit_in_ui_mode_but_keeps_them_in_the_report() {
    for (source, name, unit) in [
        (
            "#UI {\"type\": \"radio\", \"keys\": [\"Steel\"], \"values\": [\"200GPa\"]} E = 200GPa",
            "E",
            "GPa",
        ),
        ("#UI {\"type\": \"checkbox\"} flag = 1m", "flag", "m"),
        ("#UI {\"type\": \"datagrid\"} v = [1m; 2m; 3m]", "v", "m"),
    ] {
        let ui_html = render(source, true);
        not_contains("<var>", &ui_html);
        not_contains(&format!("<i>{}</i>", unit), &ui_html);
        let report_html = render(source, false);
        contains(&format!(">{}</var>", name), &report_html);
        contains(&format!("<i>{}</i>", unit), &report_html);
    }
}

#[test]
fn datagrid_leaves_no_equation_behind_in_ui_mode() {
    not_contains("<span class=\"eq\">", &render("#UI v = [1; 2; 3]", true));
}

#[test]
fn checkbox_is_checked_when_the_value_is_one() {
    contains(
        "type=\"checkbox\" class=\"calcpad-ui-checkbox\" data-ui-var=\"flag:1\" data-ui-line=\"1\" checked",
        &render("#UI {\"type\": \"checkbox\"} flag = 1", true),
    );
    not_contains("checked", &render("#UI {\"type\": \"checkbox\"} flag = 0", true));
}

#[test]
fn dropdown_marks_the_matching_option_selected() {
    let html = render(
        "#UI {\"type\": \"dropdown\", \"keys\": [\"Low\", \"Med\"], \"values\": [\"1\", \"2\"]} grade = 2",
        true,
    );
    contains("<option value=\"1\">Low</option>", &html);
    contains("<option value=\"2\" selected>Med</option>", &html);
}

#[test]
fn radio_groups_by_control_key() {
    let html = render(
        "#UI {\"type\": \"radio\", \"keys\": [\"A\", \"B\"], \"values\": [\"1\", \"2\"]} r = 1",
        true,
    );
    contains("name=\"ui-radio-r:1\" value=\"1\" checked", &html);
    contains("name=\"ui-radio-r:1\" value=\"2\"", &html);
}

#[test]
fn radio_checks_the_option_matching_a_unit_value() {
    let html = render(
        "#UI {\"type\": \"radio\", \"keys\": [\"Steel\", \"Concrete\"], \"values\": [\"200GPa\", \"25GPa\"]} E = 200GPa",
        true,
    );
    not_contains("Error", &html);
    contains("value=\"200GPa\" checked", &html);
    contains("value=\"25GPa\">", &html);
}

#[test]
fn radio_override_replaces_the_whole_value_with_its_unit() {
    let html = render_o(
        "#UI {\"type\": \"radio\", \"keys\": [\"Steel\", \"Concrete\"], \"values\": [\"200GPa\", \"25GPa\"]} E = 200GPa\nE",
        true,
        &[("E:1", "25GPa")],
    );
    not_contains("Error", &html);
    contains("value=\"25GPa\" checked", &html);
}

#[test]
fn dropdown_override_replaces_the_whole_value_with_its_unit() {
    let html = render_o(
        "#UI {\"type\": \"dropdown\", \"keys\": [\"Steel\", \"Concrete\"], \"values\": [\"200GPa\", \"25GPa\"]} E = 200GPa\nE",
        true,
        &[("E:1", "25GPa")],
    );
    not_contains("Error", &html);
    contains("<option value=\"25GPa\" selected>", &html);
}

#[test]
fn datagrid_auto_detects_shape_and_values() {
    for (source, rows, columns, values) in [
        ("#UI v = [1; 2; 3]", 1, 3, "1;2;3"),
        ("#UI M = [1; 2; 3 | 4; 5; 6]", 2, 3, "1;2;3|4;5;6"),
        ("#UI Z = vector(5)", 1, 5, "0;0;0;0;0"),
        ("#UI G = matrix(3; 4)", 3, 4, "0;0;0;0|0;0;0;0|0;0;0;0"),
    ] {
        let html = render(source, true);
        contains(
            &format!("data-ui-rows=\"{}\" data-ui-columns=\"{}\"", rows, columns),
            &html,
        );
        contains(&format!("data-ui-values=\"{}\"", values), &html);
    }
}

#[test]
fn datagrid_computed_size_takes_its_shape_from_the_value() {
    for (source, rows, columns) in [
        (
            "#UI r = 3\n#UI c = 4\n#UI {\"type\": \"datagrid\"} G = matrix(r; c)",
            3,
            4,
        ),
        ("x = [1; 2; 3]\ny = [1; 2]\n#UI G = matrix(len(x); len(y))", 3, 2),
        ("n = 5\n#UI Z = vector(n)", 1, 5),
    ] {
        let html = render(source, true);
        not_contains("Error", &html);
        contains(
            &format!("data-ui-rows=\"{}\" data-ui-columns=\"{}\"", rows, columns),
            &html,
        );
    }
}

#[test]
fn datagrid_computed_size_keeps_the_entered_values() {
    let html = render_o(
        "#UI r = 2\n#UI {\"type\": \"datagrid\"} G = matrix(r; r)\nG",
        true,
        &[("G:1", "[1; 2 | 3; 4]")],
    );
    not_contains("Error", &html);
    contains("data-ui-values=\"1;2|3;4\"", &html);
}

#[test]
fn datagrid_computed_size_fits_the_entered_values_to_the_source_shape() {
    for (rows, columns, saved, values) in [
        (2, 2, "[1; 2; 3; 4 | 5; 6; 7; 8 | 9; 10; 11; 12]", "1;2|5;6"),
        (1, 2, "[1; 2; 3; 4 | 5; 6; 7; 8 | 9; 10; 11; 12]", "1;2"),
        (3, 3, "[1; 2 | 3; 4]", "1;2;0|3;4;0|0;0;0"),
    ] {
        let r = rows.to_string();
        let c = columns.to_string();
        let html = render_o(
            "#UI x = 3\n#UI y = 4\n#UI M = matrix(x; y)",
            true,
            &[("x:1", &r), ("y:1", &c), ("M:1", saved)],
        );
        not_contains("Error", &html);
        contains(
            &format!("data-ui-rows=\"{}\" data-ui-columns=\"{}\"", rows, columns),
            &html,
        );
        contains(&format!("data-ui-values=\"{}\"", values), &html);
    }
}

#[test]
fn datagrid_literal_size_fits_the_entered_values_to_the_source_shape() {
    let html = render_o(
        "#UI {\"type\": \"datagrid\"} M = matrix(1; 2)",
        true,
        &[("M:1", "[1; 2; 3 | 4; 5; 6]")],
    );
    contains("data-ui-rows=\"1\" data-ui-columns=\"2\"", &html);
    contains("data-ui-values=\"1;2\"", &html);
}

#[test]
fn datagrid_computed_size_fits_the_entered_values_of_a_vector() {
    let html = render_o(
        "#UI n = 3\n#UI Z = vector(n)",
        true,
        &[("n:1", "2"), ("Z:1", "[7; 8; 9]")],
    );
    not_contains("Error", &html);
    contains("data-ui-rows=\"1\" data-ui-columns=\"2\"", &html);
    contains("data-ui-values=\"7;8\"", &html);
}

#[test]
fn datagrid_fitted_override_drives_the_results() {
    for enable_ui in [true, false] {
        let html = render_o(
            "#UI x = 3\n#UI y = 4\n#UI M = matrix(x; y)\ns = sum(M)",
            enable_ui,
            &[
                ("x:1", "2"),
                ("y:1", "2"),
                ("M:1", "[1; 2; 3; 4 | 5; 6; 7; 8 | 9; 10; 11; 12]"),
            ],
        );
        not_contains("Error", &html);
        contains("14", &html);
    }
}

#[test]
fn datagrid_emits_the_container_outside_the_paragraph() {
    // Assert.Matches(@"</p>\s*<div class=""calcpad-ui-datagrid""", html)
    let html = render("#UI v = [1; 2; 3]", true);
    let found = html.match_indices("</p>").any(|(i, _)| {
        html[i + 4..]
            .trim_start()
            .starts_with("<div class=\"calcpad-ui-datagrid\"")
    });
    assert!(found, "{}", html);
}

#[test]
fn datagrid_declared_shape_pads_and_truncates_the_literal() {
    for (rows, columns, literal, values) in [
        (2, 3, "[1; 2; 3 | 4; 5; 6]", "1;2;3|4;5;6"),
        (3, 4, "[1; 2; 3 | 4; 5; 6]", "1;2;3;0|4;5;6;0|0;0;0;0"),
        (2, 2, "[1; 2; 3]", "1;2|0;0"),
        (1, 2, "[1; 2; 3 | 4; 5; 6]", "1;2"),
        (2, 2, "[1; 2; 3 | 4; 5; 6]", "1;2|4;5"),
    ] {
        let html = render(
            &format!(
                "#UI {{\"type\": \"datagrid\", \"rows\": {}, \"columns\": {}}} T = {}",
                rows, columns, literal
            ),
            true,
        );
        contains(
            &format!("data-ui-rows=\"{}\" data-ui-columns=\"{}\"", rows, columns),
            &html,
        );
        contains(&format!("data-ui-values=\"{}\"", values), &html);
    }
}

#[test]
fn datagrid_declared_shape_does_not_reshape_the_report() {
    assert_report_matches_plain_source(
        "#UI {\"type\": \"datagrid\", \"rows\": 3, \"columns\": 4} M = [1; 2; 3 | 4; 5; 6]\nM",
    );
}

#[test]
fn datagrid_emits_headers() {
    let html = render(
        "#UI {\"type\": \"datagrid\", \"columnHeaders\": [\"a\", \"b\"], \"rowHeaders\": [\"r1\"]} T = [1; 2]",
        true,
    );
    contains("data-ui-col-headers=\"[&quot;a&quot;,&quot;b&quot;]\"", &html);
    contains("data-ui-row-headers=\"[&quot;r1&quot;]\"", &html);
}

#[test]
fn datagrid_seeds_its_cells_from_the_computed_value() {
    for (source, values) in [
        ("#UI {\"type\": \"datagrid\"} G = matrix(2; 2) + 5", "5;5|5;5"),
        ("#UI {\"type\": \"datagrid\"} Z = vector(3) + 2", "2;2;2"),
    ] {
        let html = render(source, true);
        not_contains("Error", &html);
        contains(&format!("data-ui-values=\"{}\"", values), &html);
    }
}

#[test]
fn datagrid_keeps_the_unit_aside_when_the_cells_are_numbers_only() {
    let html = render("#UI {\"type\": \"datagrid\"} U = matrix(1; 2) + 5m", true);
    not_contains("Error", &html);
    contains("data-ui-values=\"5;5\"", &html);
    contains("data-ui-cell-units=\"[[&quot;m&quot;,&quot;m&quot;]]\"", &html);
}

#[test]
fn datagrid_keeps_the_unit_aside_when_the_literal_carries_it() {
    for (source, values, units) in [
        (
            "#UI {\"type\": \"datagrid\"} M = [1m; 2m]",
            "1;2",
            "[[&quot;m&quot;,&quot;m&quot;]]",
        ),
        (
            "#UI {\"type\": \"datagrid\"} M = [1m; 2]",
            "1;2",
            "[[&quot;m&quot;,&quot;&quot;]]",
        ),
        (
            "#UI {\"type\": \"datagrid\"} M = [1kN/m; 2kN/m | 3kN/m; 4kN/m]",
            "1;2|3;4",
            "[[&quot;kN/m&quot;,&quot;kN/m&quot;],[&quot;kN/m&quot;,&quot;kN/m&quot;]]",
        ),
    ] {
        let html = render(source, true);
        not_contains("Error", &html);
        contains(&format!("data-ui-values=\"{}\"", values), &html);
        contains(&format!("data-ui-cell-units=\"{}\"", units), &html);
    }
}

#[test]
fn datagrid_entered_values_keep_the_unit_aside() {
    let html = render_o(
        "#UI {\"type\": \"datagrid\"} M = matrix(1; 2) + 1m",
        true,
        &[("M:1", "[5m; 9m]")],
    );
    not_contains("Error", &html);
    contains("data-ui-values=\"5;9\"", &html);
    contains("data-ui-cell-units=\"[[&quot;m&quot;,&quot;m&quot;]]\"", &html);
}

#[test]
fn datagrid_computed_value_keeps_the_unit_out_of_the_cells() {
    for (source, values, units) in [
        (
            "#UI {\"type\": \"datagrid\"} M = matrix(2; 2)*5m",
            "0;0|0;0",
            "[[&quot;m&quot;,&quot;m&quot;],[&quot;m&quot;,&quot;m&quot;]]",
        ),
        (
            "#UI {\"type\": \"datagrid\", \"forceUnits\": true} M = matrix(1; 2) + 5kN/m",
            "5;5",
            "[[&quot;kN/m&quot;,&quot;kN/m&quot;]]",
        ),
        (
            "#UI {\"type\": \"datagrid\"} M = vector(3)*5m",
            "0;0;0",
            "[[&quot;m&quot;,&quot;m&quot;,&quot;m&quot;]]",
        ),
    ] {
        let html = render(source, true);
        not_contains("Error", &html);
        contains(&format!("data-ui-values=\"{}\"", values), &html);
        contains(&format!("data-ui-cell-units=\"{}\"", units), &html);
    }
}

#[test]
fn datagrid_computed_value_survives_an_edited_value() {
    let html = render_o(
        "#UI {\"type\": \"datagrid\"} M = matrix(2; 2)*5m",
        true,
        &[("M:1", "[7m; 5m | 5m; 5m]")],
    );
    not_contains("Error", &html);
    contains("data-ui-values=\"7;5|5;5\"", &html);
    contains(
        "data-ui-cell-units=\"[[&quot;m&quot;,&quot;m&quot;],[&quot;m&quot;,&quot;m&quot;]]\"",
        &html,
    );
}

#[test]
fn datagrid_pads_the_units_over_the_whole_grid() {
    for (source, units) in [
        (
            "#UI {\"type\": \"datagrid\", \"rows\": 3, \"columns\": 3} M = matrix(2; 2)*5m",
            "[[&quot;m&quot;,&quot;m&quot;,&quot;m&quot;],[&quot;m&quot;,&quot;m&quot;,&quot;m&quot;],[&quot;m&quot;,&quot;m&quot;,&quot;m&quot;]]",
        ),
        (
            "#UI {\"type\": \"datagrid\", \"rows\": 2, \"columns\": 3} M = [1m; 2m]",
            "[[&quot;m&quot;,&quot;m&quot;,&quot;m&quot;],[&quot;m&quot;,&quot;m&quot;,&quot;m&quot;]]",
        ),
    ] {
        contains(&format!("data-ui-cell-units=\"{}\"", units), &render(source, true));
    }
}

#[test]
fn datagrid_unitless_literal_has_no_cell_units() {
    let html = render("#UI {\"type\": \"datagrid\"} M = [1; 2]", true);
    contains("data-ui-values=\"1;2\"", &html);
    not_contains("data-ui-cell-units", &html);
}

#[test]
fn datagrid_force_units_false_leaves_the_units_in_the_literal_cells() {
    let html = render("#UI {\"type\": \"datagrid\", \"forceUnits\": false} M = [1m; 2m]", true);
    contains("data-ui-values=\"1m;2m\"", &html);
    not_contains("data-ui-cell-units", &html);
}

#[test]
fn datagrid_writes_the_unit_into_the_cells_when_the_user_types_it() {
    let html = render(
        "#UI {\"type\": \"datagrid\", \"forceUnits\": false} U = matrix(1; 2) + 5m",
        true,
    );
    contains("data-ui-values=\"5m;5m\"", &html);
    not_contains("data-ui-cell-units", &html);
}

#[test]
fn datagrid_emits_the_declared_widths() {
    let html = render(
        "#UI {\"type\": \"datagrid\", \"width\": \"100%\", \"rowHeaderWidth\": 140, \"columnWidths\": [2, 1]} T = [1; 2]",
        true,
    );
    contains("data-ui-width=\"100%\"", &html);
    contains("data-ui-row-header-width=\"140\"", &html);
    contains("data-ui-column-widths=\"[2,1]\"", &html);
}

#[test]
fn datagrid_width_takes_pixels_or_any_percentage() {
    for (declared, emitted) in [
        ("\"75%\"", "75%"),
        ("\"33.5 %\"", "33.5%"),
        ("\"100%\"", "100%"),
        ("\"full\"", "100%"),
        ("320", "320"),
        ("319.6", "320"),
    ] {
        let html = render(
            &format!("#UI {{\"type\": \"datagrid\", \"width\": {}}} T = [1; 2]", declared),
            true,
        );
        not_contains("Error", &html);
        contains(&format!("data-ui-width=\"{}\"", emitted), &html);
    }
}

#[test]
fn datagrid_width_is_reported_when_it_is_neither_pixels_nor_a_percentage() {
    for declared in ["\"wide\"", "\"0%\"", "\"%\"", "-10"] {
        let html = render(
            &format!("#UI {{\"type\": \"datagrid\", \"width\": {}}} T = [1; 2]", declared),
            true,
        );
        contains("'width' must", &html);
    }
}

#[test]
fn entry_force_units_false_puts_the_unit_in_the_box() {
    let html = render("#UI {\"forceUnits\": false} q = 3kN/m", true);
    contains("data-ui-force-units=\"0\"", &html);
    contains("value=\"3kN/m\"", &html);
    not_contains("<i>kN", &html);
}

#[test]
fn entry_keeps_the_variable_name_in_every_mode() {
    for source in [
        "#UI q = 3kN/m",
        "#UI {\"forceUnits\": false} q = 3kN/m",
        "#UI {\"allowExpression\": true} q = 3kN/m",
    ] {
        contains("<var>q</var> = <input", &render(source, true));
    }
}

#[test]
fn entry_allow_expression_drops_the_substituted_equation() {
    let html = render("#UI {\"allowExpression\": true} A = 2*3", true);
    contains("<var>A</var> = <input", &html);
    not_contains("2 \u{00b7} 3", &html);
}

#[test]
fn entry_force_units_false_override_replaces_the_whole_value() {
    let html = render_o("#UI {\"forceUnits\": false} q = 3kN/m\nq", true, &[("q", "4MN/m")]);
    not_contains("Error", &html);
    contains("value=\"4MN/m\"", &html);
}

#[test]
fn entry_allow_expression_shows_the_source_not_the_result() {
    let html = render("#UI {\"allowExpression\": true} k = 1 + 2", true);
    contains("data-ui-allow-expression=\"1\"", &html);
    contains("value=\"1 + 2\"", &html);
}

#[test]
fn entry_allow_expression_keeps_the_control_when_the_line_errors() {
    let html = render_o("#UI {\"allowExpression\": true} k = 1 + 2", true, &[("k", "1 +")]);
    contains("class=\"calcpad-ui-input\"", &html);
    contains("value=\"1 +\"", &html);
    contains("<p class=\"err\"", &html);
}

#[test]
fn declared_type_is_reported_when_the_value_contradicts_it() {
    for (source, expected) in [
        (
            "#UI {\"type\": \"datagrid\"} x = 5",
            "'datagrid' requires a vector or matrix",
        ),
        (
            "#UI {\"type\": \"datagrid\"} x = 5m",
            "'datagrid' requires a vector or matrix",
        ),
        (
            "#UI {\"type\": \"datagrid\", \"rows\": 2, \"columns\": 2} x = -2.5kN/m",
            "'datagrid' requires a vector or matrix",
        ),
        (
            "#UI {\"type\": \"entry\"} v = [1; 2]",
            "'entry' cannot hold a vector or matrix",
        ),
        (
            "#UI {\"type\": \"entry\"} M = matrix(2; 2)",
            "'entry' cannot hold a vector or matrix",
        ),
        (
            "#UI {\"type\": \"checkbox\"} f = [1]",
            "'checkbox' cannot hold a vector or matrix",
        ),
        (
            "#UI {\"type\": \"dropdown\", \"keys\": [\"A\"], \"values\": [\"1\"]} v = vector(3)",
            "'dropdown' cannot hold a vector or matrix",
        ),
    ] {
        let html = render(source, true);
        contains(expected, &html);
        not_contains("calcpad-ui-", &html);
    }
}

#[test]
fn declared_type_is_accepted_when_the_value_can_be_it() {
    for (source, control) in [
        ("#UI {\"type\": \"datagrid\"} M = 2*vector(3)", "calcpad-ui-datagrid"),
        (
            "#UI {\"type\": \"datagrid\", \"rows\": 2, \"columns\": 2} M = A\nA = [1; 2 | 3; 4]",
            "calcpad-ui-datagrid",
        ),
        ("#UI {\"type\": \"datagrid\"} M = [1; 2]", "calcpad-ui-datagrid"),
        ("#UI {\"type\": \"entry\"} x = 5m", "calcpad-ui-input"),
        (
            "#UI {\"type\": \"entry\", \"allowExpression\": true} v = [1; 2]",
            "calcpad-ui-input",
        ),
    ] {
        let html = render(source, true);
        not_contains("vector or matrix value", &html);
        contains(control, &html);
    }
}

#[test]
fn grid_properties_are_reported_on_controls_that_are_not_grids() {
    for (source, name) in [
        ("#UI {\"type\": \"entry\", \"width\": 300} x = 1", "width"),
        (
            "#UI {\"type\": \"checkbox\", \"rowHeaderWidth\": 40} x = 1",
            "rowHeaderWidth",
        ),
        ("#UI {\"width\": \"100%\"} x = 1", "width"),
        ("#UI {\"columnHeaders\": [\"a\"]} x = 1", "columnHeaders"),
        ("#UI {\"rows\": 2, \"columns\": 2} x = 1", "rows"),
    ] {
        let html = render(source, true);
        contains(&format!("'{}' only applies to a datagrid", name), &html);
        not_contains("calcpad-ui-", &html);
    }
}

#[test]
fn grid_properties_are_accepted_on_a_grid() {
    for source in [
        "#UI {\"type\": \"datagrid\", \"width\": 300, \"rowHeaderWidth\": 40} M = matrix(2; 2)",
        "#UI {\"width\": 300, \"columnWidths\": [2, 1]} T = [1; 2]",
        "#UI {\"rowHeaders\": [\"r1\"]} Z = vector(3)",
    ] {
        let html = render(source, true);
        not_contains("only applies to a datagrid", &html);
        contains("calcpad-ui-datagrid", &html);
    }
}

#[test]
fn mode_properties_are_reported_on_types_that_already_replace_the_value() {
    for (source, name) in [
        (
            "#UI {\"type\": \"dropdown\", \"keys\": [\"A\"], \"values\": [\"1\"], \"forceUnits\": false} x = 1",
            "forceUnits",
        ),
        (
            "#UI {\"type\": \"checkbox\", \"allowExpression\": true} x = 1",
            "allowExpression",
        ),
    ] {
        contains(&format!("'{}' does not apply", name), &render(source, true));
    }
}

#[test]
fn datagrid_headers_keep_their_commas() {
    let html = render(
        "#UI {\"type\": \"datagrid\", \"columnHeaders\": [\"Force, kN\", \"b\"]} T = [1; 2]",
        true,
    );
    contains("data-ui-col-headers=\"[&quot;Force, kN&quot;,&quot;b&quot;]\"", &html);
}

#[test]
fn override_replaces_the_value_and_propagates_to_dependents() {
    let html = render_o("#UI L = 10m\nA = L * 2m", true, &[("L", "12")]);
    contains("value=\"12\"", &html);
    contains("24", &html);
}

#[test]
fn override_preserves_the_unit() {
    not_contains("10", &render_o("#UI L = 10m\nL", true, &[("L", "12")]));
}

#[test]
fn override_applies_in_report_mode_too() {
    contains("24", &render_o("#UI L = 10m\nA = L * 2m", false, &[("L", "12")]));
}

#[test]
fn override_replaces_the_whole_datagrid_literal() {
    contains(
        "data-ui-values=\"7;8;9\"",
        &render_o("#UI v = [1; 2; 3]", true, &[("v", "[7; 8; 9]")]),
    );
}

#[test]
fn invalid_directive_reports_an_error() {
    for source in [
        "#UI {\"type\": \"dropdown\", \"keys\": [\"A\"]} x = 1",
        "#UI {\"type\": \"radio\", \"values\": [\"1\"]} x = 1",
        "#UI {\"type\": \"dropdown\", \"keys\": [\"A\", \"B\"], \"values\": [\"1\"]} x = 1",
        "#UI {\"type\": \"entry\" x = 1",
        "#UI {not json} x = 1",
        "#UI {\"mode\": \"string\"} x = 1",
        "#UI name$ = 1",
        "#UI x",
        "#UI {\"type\": \"slider\"} x = 1",
        "#UI {\"rows\": -1} x = 1",
        "#UI {\"type\": \"datagrid\", \"columns\": 2, \"columnHeaders\": [\"a\", \"b\", \"c\"]} T = [1; 2]",
        "#UI {\"rows\": \"two\"} x = 1",
        "#UI {\"style\": 5} x = 1",
    ] {
        contains("err", &render(source, true));
    }
}

#[test]
fn invalid_directive_renders_no_control() {
    not_contains("calcpad-ui-", &render("#UI {\"type\": \"slider\"} x = 1", true));
}

#[test]
fn parser_and_linter_reject_the_same_properties() {
    let properties = UiDto {
        ty: Some("slider".into()),
        rows: Some(-1),
        mode: Some("string".into()),
        ..Default::default()
    };
    let messages = properties.validate();
    assert_eq!(3, messages.len());
    let html = render(
        "#UI {\"type\": \"slider\", \"rows\": -1, \"mode\": \"string\"} x = 1",
        true,
    );
    for message in messages {
        contains(&message, &html);
    }
}

#[test]
fn inactive_conditional_branch_renders_no_control() {
    not_contains("calcpad-ui-input", &render("#if 0\n#UI L = 10m\n#end if", true));
}

#[test]
fn active_conditional_branch_renders_the_control() {
    contains("calcpad-ui-input", &render("#if 1\n#UI L = 10m\n#end if", true));
}

#[test]
fn skipped_ui_line_does_not_attach_its_control_to_the_next_line() {
    let html = render("#if 0\n#UI L = 10m\n#end if\nW = 5m", true);
    not_contains("data-ui-", &html);
    not_contains("<input", &html);
}

#[test]
fn redefined_variable_gets_a_distinct_key_per_occurrence() {
    let html = render("#UI x = 1\nx = x + 1\n#UI x = 5", true);
    contains("data-ui-var=\"x:1\"", &html);
    contains("data-ui-var=\"x:2\"", &html);
}

#[test]
fn keyed_override_applies_to_that_occurrence_only() {
    let html = render_o("#UI x = 1\n#UI x = 5", true, &[("x:2", "50")]);
    contains("value=\"1\"", &html);
    contains("value=\"50\"", &html);
    not_contains("value=\"5\"", &html);
}

#[test]
fn bare_override_applies_to_every_occurrence_of_that_name() {
    let html = render_o("#UI x = 1\n#UI x = 5", true, &[("x", "7")]);
    assert_eq!(2, count(&html, "value=\"7\""));
}

#[test]
fn looped_control_gets_one_key_per_pass() {
    let html = render("#repeat 3\n#UI x = 1\n#loop", true);
    contains("data-ui-var=\"x:1:1\"", &html);
    contains("data-ui-var=\"x:1:2\"", &html);
    contains("data-ui-var=\"x:1:3\"", &html);
}

#[test]
fn nested_loops_join_the_pass_numbers() {
    let html = render("#repeat 2\n#repeat 2\n#UI x = 1\n#loop\n#loop", true);
    contains("data-ui-var=\"x:1:1.1\"", &html);
    contains("data-ui-var=\"x:1:1.2\"", &html);
    contains("data-ui-var=\"x:1:2.1\"", &html);
    contains("data-ui-var=\"x:1:2.2\"", &html);
}

#[test]
fn control_numbering_is_unaffected_by_loop_iteration_count() {
    contains(
        "data-ui-var=\"b:1\"",
        &render("#repeat 2\n#UI a = 1\n#loop\n#UI b = 2", true),
    );
    contains(
        "data-ui-var=\"b:1\"",
        &render("#repeat 9\n#UI a = 1\n#loop\n#UI b = 2", true),
    );
}

#[test]
fn declaration_override_covers_every_pass_of_a_loop() {
    let html = render_o("#repeat 3\n#UI x = 1\n#loop", true, &[("x:1", "4")]);
    assert_eq!(3, count(&html, "value=\"4\""));
}

#[test]
fn pass_override_applies_to_that_iteration_only() {
    let html = render_o("#repeat 3\n#UI x = 1\n#loop", true, &[("x:1:2", "4")]);
    assert_eq!(1, count(&html, "value=\"4\""));
    assert_eq!(2, count(&html, "value=\"1\""));
}

#[test]
fn control_numbering_is_stable_across_conditional_branches() {
    for b in ["1", "0"] {
        let source = format!("#if {}\n#UI a = 1\n#end if\n#UI b = 2", b);
        contains("data-ui-var=\"b:1\"", &render(&source, true));
    }
}

#[test]
fn ui_state_does_not_leak_to_the_following_line() {
    let html = render("#UI L = 10m\nW = 5m", true);
    contains("data-ui-var=\"L:1\"", &html);
    not_contains("data-ui-var=\"W:1\"", &html);
    assert_eq!(1, count(&html, "calcpad-ui-input"));
}

#[test]
fn error_line_is_hidden_in_input_mode_and_shown_in_report_mode() {
    let source = "#UI x = y";
    let ui_html = render(source, true);
    contains("Error in", &ui_html);
    not_contains("on line", &ui_html);
    not_contains("data-text=", &ui_html);
    let report_html = render(source, false);
    contains("Error in", &report_html);
    contains("on line", &report_html);
    contains("data-text=", &report_html);
}

#[test]
fn datagrid_size_limit_explicitly_declared_size_too_large() {
    let html = render(
        "#UI {\"type\": \"datagrid\", \"rows\": 100001, \"columns\": 1} x = vector(100001)",
        true,
    );
    contains("Error", &html);
    contains("datagrid size cannot exceed", &html);
    contains("100000", &html);
    not_contains("calcpad-ui-datagrid", &html);
}

#[test]
fn datagrid_size_limit_auto_detected_literal_too_large() {
    let html = render("#UI M = matrix(1000; 1000)", true);
    contains("Error", &html);
    contains("datagrid size cannot exceed", &html);
    contains("#Read", &html);
    not_contains("calcpad-ui-datagrid", &html);
}

#[test]
fn datagrid_size_limit_computed_size_too_large() {
    let html = render("#UI M = matrix(500; 300)", true);
    contains("Error", &html);
    contains("datagrid size cannot exceed", &html);
    not_contains("calcpad-ui-datagrid", &html);
}

#[test]
fn datagrid_size_limit_under_limit_is_allowed() {
    let html = render("#UI M = matrix(300; 300)", true);
    not_contains("Error", &html);
    contains("data-ui-rows=\"300\" data-ui-columns=\"300\"", &html);
}
