//! Port of Parsers/MapParser.cs ($Map).
use super::math_parser::MathParser;
use super::plot_parser::{is_null_or_white_space, split_command, PARTS};
use crate::base_types::unit::Unit;
use crate::base_types::variable::Parameter;
use crate::exceptions;
use crate::extension_methods::{almost_equals, trim};
use crate::math_parser_exception::CResult;
use crate::messages;
use crate::netfmt::fmt;
use crate::plotter::map_plotter::MapPlotter;
use crate::settings::{ColorScales, LightDirections, PlotSettings};

/// `MapParser.Parse(script, calculate)`; `settings` is updated from the plot variables as
/// in C# (`GetHtmlImage`).
pub fn parse(parser: &MathParser, settings: &mut PlotSettings, script: &str, calculate: bool) -> CResult<String> {
    const N: usize = 7;
    let delimiters = ['{', '@', '=', ':', '&', '=', ':', '}'];
    let (input, index) = split_command(script, &delimiters, N);
    if index <= N {
        return Ok(fmt(
            messages::Missing_delimiter_0_in_surface_map_command_1,
            &[&delimiters[index].to_string(), script],
        ));
    }
    for (i, s) in input.iter().enumerate() {
        if is_null_or_white_space(s) {
            let part = if i > 3 { i - 3 } else { i };
            return Err(exceptions::MissingMapItem(PARTS[part]));
        }
    }
    let mut input: Vec<String> = input.into_iter().map(|s| s.unwrap_or_default()).collect();
    input[1] = trim(&input[1]).to_string();
    input[4] = trim(&input[4]).to_string();
    if !calculate {
        return get_html_text(parser, &mut input);
    }
    parser.parse(&input[2], true)?;
    let start_x = parser.calculate_real()?;
    let x_units = parser.units();
    parser.parse(&input[3], true)?;
    let mut end_x = parser.calculate_real()?;
    let mut end_units = parser.units();
    if !Unit::is_consistent_opt(&x_units, &end_units) {
        return Err(exceptions::InconsistentUnits(
            &Unit::get_text_of(&x_units),
            &Unit::get_text_of(&end_units),
        ));
    }
    if let Some(eu) = &end_units {
        end_x *= eu.convert_to(x_units.as_ref().unwrap());
    }
    parser.parse(&input[5], true)?;
    let start_y = parser.calculate_real()?;
    let y_units = parser.units();
    parser.parse(&input[6], true)?;
    let mut end_y = parser.calculate_real()?;
    end_units = parser.units();
    if !Unit::is_consistent_opt(&y_units, &end_units) {
        return Err(exceptions::InconsistentUnits(
            &Unit::get_text_of(&y_units),
            &Unit::get_text_of(&end_units),
        ));
    }
    if let Some(eu) = &end_units {
        end_y *= eu.convert_to(y_units.as_ref().unwrap());
    }
    if almost_equals(start_x, end_x) || almost_equals(start_y, end_y) {
        return Err(exceptions::PlotLimitsIdentical());
    }
    let parameters = [Parameter::new(&input[1]), Parameter::new(&input[4])];
    let function = parser.compile(&input[0], &parameters)?;
    // GetHtmlImage
    let color_scale = parser.get_settings_variable("PlotPalette", settings.color_scale as i32 as f64)? as i32;
    settings.color_scale = ColorScales::ALL[color_scale.clamp(0, ColorScales::ALL.len() as i32 - 1) as usize];
    let shadows = parser.get_settings_variable("PlotShadows", if settings.shadows() { 1.0 } else { 0.0 })? as i32;
    settings.set_shadows(shadows != 0);
    let smooth = parser.get_settings_variable("PlotSmooth", if settings.smooth_scale { 1.0 } else { 0.0 })? as i32;
    settings.smooth_scale = smooth != 0;
    let light = parser.get_settings_variable("PlotLightDir", settings.light_direction as i32 as f64)? as i32;
    settings.light_direction = LightDirections::ALL[light.clamp(0, LightDirections::ALL.len() as i32 - 1) as usize];
    let settings: &PlotSettings = settings;
    MapPlotter::new(parser, settings).plot(
        &function,
        &parameters[0].variable,
        &parameters[1].variable,
        start_x,
        end_x,
        start_y,
        end_y,
        &x_units,
        &y_units,
    )
}

fn get_html_text(parser: &MathParser, input: &mut [String]) -> CResult<String> {
    for s in input.iter_mut() {
        parser.parse(s, true)?;
        *s = parser.to_html()?;
    }
    Ok(format!(
        "<span class = \"eq\"><span class=\"cond\">$Map</span>{{{} @ {} \u{2208} [{}; {}]}}</span>",
        input[0], input[1], input[2], input[3]
    ))
}
