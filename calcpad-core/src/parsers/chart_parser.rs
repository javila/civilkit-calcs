//! Port of Parsers/ChartParser.cs ($Plot).
use super::math_parser::MathParser;
use super::plot_parser::{is_null_or_white_space, split_command, PARTS};
use crate::base_types::unit::Unit;
use crate::base_types::variable::Parameter;
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::messages;
use crate::netfmt::fmt;
use crate::plotter::chart_plotter::{ChartPlotter, PlotFunc};
use crate::settings::PlotSettings;

/// `ChartParser.Parse(script, calculate)`
pub fn parse(parser: &MathParser, settings: &PlotSettings, script: &str, calculate: bool) -> CResult<String> {
    const N: usize = 4;
    let delimiters = ['{', '@', '=', ':', '}'];
    let (input, index) = split_command(script, &delimiters, N);
    if index <= 4 {
        return Ok(fmt(
            messages::Missing_delimiter_0_in_plot_command,
            &[&delimiters[index].to_string(), script],
        ));
    }
    for j in 0..N {
        if is_null_or_white_space(&input[j]) {
            return Ok(fmt(messages::Missing_0_in_plot_command_1, &[PARTS[j], script]));
        }
    }
    let mut input: Vec<String> = input.into_iter().map(|s| s.unwrap_or_default()).collect();
    if calculate {
        let input0 = input[0].clone();
        let charts: Vec<&str> = input0.split('&').collect();
        input[1] = crate::extension_methods::trim(&input[1]).to_string();
        let parameters = [Parameter::new(&input[1])];
        let count = charts.len();
        let mut fx: Vec<PlotFunc> = vec![None; count];
        let mut fy: Vec<PlotFunc> = vec![None; count];
        for i in 0..count {
            let xy: Vec<&str> = charts[i].split('|').collect();
            if xy.len() > 2 {
                return Ok(fmt(messages::More_than_one_delimiter_in_plot_command, &[script]));
            } else if xy.len() == 1 {
                fy[i] = parser.compile(xy[0], &parameters)?;
            } else {
                fx[i] = parser.compile(xy[0], &parameters)?;
                fy[i] = parser.compile(xy[1], &parameters)?;
            }
        }
        parser.parse(&input[2], true)?;
        let left = parser.calculate_real()?;
        let left_units = parser.units();
        parser.parse(&input[3], true)?;
        let mut right = parser.calculate_real()?;
        let right_units = parser.units();
        if !Unit::is_consistent_opt(&left_units, &right_units) {
            return Err(exceptions::InconsistentUnits(
                &Unit::get_text_of(&left_units),
                &Unit::get_text_of(&right_units),
            ));
        }
        if let Some(lu) = &left_units {
            let factor = right_units.as_ref().unwrap().convert_to(lu);
            right *= factor;
        }
        let mut plotter = ChartPlotter::new(parser, settings);
        plotter.plot(&fx, &fy, &parameters[0].variable, left, right, &left_units)
    } else {
        get_html_text(parser, &mut input)
    }
}

fn get_html_text(parser: &MathParser, input: &mut [String]) -> CResult<String> {
    let mut charts: Vec<String> = input[0].split('&').map(|s| s.to_string()).collect();
    input[1] = crate::extension_methods::trim(&input[1]).to_string();
    for chart in charts.iter_mut() {
        let xy: Vec<&str> = chart.split('|').collect();
        if xy.len() > 2 {
            return Ok(fmt(messages::More_than_one_delimiter_in_plot_command, &[&input[0]]));
        }
        let (sx, sy);
        if xy.len() == 1 {
            parser.parse(&input[1], true)?;
            sx = parser.to_html()?;
            parser.parse(xy[0], true)?;
            sy = parser.to_html()?;
        } else {
            parser.parse(xy[0], true)?;
            sx = parser.to_html()?;
            parser.parse(xy[1], true)?;
            sy = parser.to_html()?;
        }
        *chart = format!("{} | {}", sx, sy);
    }
    input[0] = charts.join(" & ");
    for s in input.iter_mut().skip(1) {
        parser.parse(s, true)?;
        *s = parser.to_html()?;
    }
    Ok(format!(
        "<span class = \"eq\"><span class=\"cond\">$Plot</span>{{{} @ {} \u{2208} [{}; {}]}}</span>",
        input[0], input[1], input[2], input[3]
    ))
}
