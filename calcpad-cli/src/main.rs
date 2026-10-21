//! Command line front end, compatible with the batch conversion mode of Calcpad.Cli:
//!
//! ```text
//! Cli <input.cpd> [output.html | output.docx | output.xlsx | html | docx | xlsx] [-b] [-s]
//! ```
//!
//! `-b` writes only `<div class="calcpad-output">...</div>` (used by the conformance
//! harness), `-s` is silent (the converted file is not opened). Settings are read from
//! `$CALCPAD_SETTINGS` or `~/Documents/.config/calcpad/Settings.xml`, like Calcpad.Cli on
//! Linux and macOS. If `$CALCPAD_RS_SCOPE_FILE` is set, the first out of scope feature hit
//! while rendering is written to that file (used to classify worksheets).
mod calcpad_reader;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use calcpad_core::parsers::expression_parser::ExpressionParser;
use calcpad_core::parsers::macro_parser::MacroParser;
use calcpad_core::settings::{ColorScales, LightDirections, MathSettings, PlotSettings};

struct CliSettings {
    math: MathSettings,
    plot: PlotSettings,
    is_us: bool,
}

fn settings_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CALCPAD_SETTINGS") {
        return Some(PathBuf::from(p));
    }
    let home = std::env::var("HOME").ok()?;
    Some(
        Path::new(&home)
            .join("Documents")
            .join(".config")
            .join("calcpad")
            .join("Settings.xml"),
    )
}

fn xml_value<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let i = xml.find(&open)? + open.len();
    let j = xml[i..].find(&close)? + i;
    Some(xml[i..j].trim())
}

fn get_settings() -> CliSettings {
    let mut math = MathSettings::default();
    math.set_decimals(6);
    let mut is_us = true;
    let mut plot = PlotSettings::default();
    if let Some(path) = settings_path() {
        if let Ok(xml) = std::fs::read_to_string(&path) {
            // XmlSerializer creates a new Settings object first (Decimals = 2) and then applies the file.
            math = MathSettings::default();
            if let Some(math_xml) = xml_value(&xml, "Math") {
                if let Some(v) = xml_value(math_xml, "Decimals").and_then(|s| s.parse().ok()) {
                    math.set_decimals(v);
                }
                if let Some(v) = xml_value(math_xml, "Degrees").and_then(|s| s.parse().ok()) {
                    math.degrees = v;
                }
                let b = |t: &str| xml_value(math_xml, t).map(|s| s.eq_ignore_ascii_case("true"));
                if let Some(v) = b("IsComplex") {
                    math.is_complex = v;
                }
                if let Some(v) = b("Substitute") {
                    math.substitute = v;
                }
                if let Some(v) = b("FormatEquations") {
                    math.format_equations = v;
                }
                if let Some(v) = b("ZeroSmallMatrixElements") {
                    math.zero_small_matrix_elements = v;
                }
                if let Some(v) = b("ShowHiddenOutput") {
                    math.show_hidden_output = v;
                }
                if let Some(v) = xml_value(math_xml, "MaxOutputCount").and_then(|s| s.parse().ok()) {
                    math.set_max_output_count(v);
                }
            }
            if let Some(plot_xml) = xml_value(&xml, "Plot") {
                let b = |t: &str| xml_value(plot_xml, t).map(|s| s.eq_ignore_ascii_case("true"));
                if let Some(v) = b("IsAdaptive") {
                    plot.is_adaptive = v;
                }
                if let Some(v) = xml_value(plot_xml, "ScreenScaleFactor").and_then(|s| s.parse().ok()) {
                    plot.screen_scale_factor = v;
                }
                if let Some(v) = xml_value(plot_xml, "Width").and_then(|s| s.parse().ok()) {
                    plot.width = v;
                }
                if let Some(v) = xml_value(plot_xml, "Height").and_then(|s| s.parse().ok()) {
                    plot.height = v;
                }
                if let Some(v) = xml_value(plot_xml, "Step").and_then(|s| s.parse().ok()) {
                    plot.step = v;
                }
                if let Some(v) = xml_value(plot_xml, "ImagePath") {
                    plot.image_path = v.to_string();
                }
                if let Some(v) = xml_value(plot_xml, "ImageUri") {
                    plot.image_uri = v.to_string();
                }
                if let Some(v) = b("VectorGraphics") {
                    plot.vector_graphics = v;
                }
                if let Some(v) = xml_value(plot_xml, "ColorScale").and_then(ColorScales::parse) {
                    plot.color_scale = v;
                }
                if let Some(v) = b("SmoothScale") {
                    plot.smooth_scale = v;
                }
                if let Some(v) = b("Shadows") {
                    plot.set_shadows(v);
                }
                if let Some(v) = xml_value(plot_xml, "LightDirection").and_then(LightDirections::parse) {
                    plot.light_direction = v;
                }
            }
            if let Some(v) = xml_value(&xml, "IsUs") {
                is_us = v.eq_ignore_ascii_case("true");
            }
        }
    }
    CliSettings { math, plot, is_us }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: Cli <input.cpd> [output.html] [-b] [-s]");
        return ExitCode::from(2);
    }
    // Calcpad.Cli joins the arguments with spaces and splits at the input extension.
    let joined = args.join(" ");
    let joined = joined.trim();
    let ext_pos = joined.find(".cpd").or_else(|| joined.find(".txt"));
    let i = match ext_pos {
        Some(i) => i + 4,
        None => {
            eprintln!("Invalid input file extension. Must be .cpd or .txt: {}", joined);
            return ExitCode::from(2);
        }
    };
    let mut out_file = joined[i..].trim().to_string();
    let mut is_body_only = false;
    loop {
        if out_file.ends_with(" -s") || out_file == "-s" {
            out_file = out_file[..out_file.len() - 2].trim_end().to_string();
        } else if out_file.ends_with(" -b") || out_file == "-b" {
            is_body_only = true;
            out_file = out_file[..out_file.len() - 2].trim_end().to_string();
        } else {
            break;
        }
    }
    let file_name = joined[..i].trim().to_string();
    let input = Path::new(&file_name);
    if !input.exists() {
        eprintln!("Input file does not exist.");
        return ExitCode::from(1);
    }
    let out_path: PathBuf = if out_file.is_empty() {
        input.with_extension("html")
    } else if Path::new(&out_file).is_dir() {
        PathBuf::from(format!(
            "{}{}.html",
            out_file,
            input.file_stem().unwrap().to_string_lossy()
        ))
    } else if ["html", "htm", "docx", "xlsx"].contains(&out_file.as_str()) {
        input.with_extension(&out_file)
    } else {
        PathBuf::from(&out_file)
    };
    let ext = out_path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let is_docx = ext == "docx";
    let is_xlsx = ext == "xlsx";
    if ext != "html" && ext != "htm" && !is_docx && !is_xlsx {
        eprintln!("Invalid output extension. Must be .html, .htm, .docx or .xlsx (PDF output is not supported).");
        return ExitCode::from(1);
    }
    let full_input = std::fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    if let Some(dir) = full_input.parent() {
        let _ = std::env::set_current_dir(dir);
    }
    let file_name_abs = full_input.to_string_lossy().to_string();
    let settings = get_settings();
    let code = match calcpad_reader::read(&file_name_abs) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::from(1);
        }
    };
    let files = calcpad_reader::DiskFiles;
    let mut macro_parser = MacroParser::new(&files);
    macro_parser.source_file_path = Some(file_name_abs.clone());
    let (has_macro_errors, unwrapped) = macro_parser.parse(&code, true);
    let mut out_of_scope: Option<String> = None;
    let mut expressions: Vec<String> = Vec::new();
    let mut captured = Vec::new();
    let html = if has_macro_errors {
        calcpad_reader::code_to_html(&unwrapped)
    } else {
        let mut parser = ExpressionParser::new();
        *parser.math_settings.borrow_mut() = settings.math.clone();
        parser.is_us = settings.is_us;
        parser.plot_settings = settings.plot.clone();
        // Converter.ToOpenXml: the equations as Office Math too.
        parser.capture_open_xml = is_docx;
        // The Excel export: the variables and their values (calcpad-rs's own feature).
        parser.capture_values = is_xlsx;
        parser.parse(&unwrapped, true);
        out_of_scope = parser.out_of_scope.clone();
        expressions = std::mem::take(&mut parser.open_xml_expressions);
        captured = std::mem::take(&mut parser.captured_values);
        parser.html_result.clone()
    };
    if let Ok(scope_file) = std::env::var("CALCPAD_RS_SCOPE_FILE") {
        let _ = std::fs::write(scope_file, out_of_scope.unwrap_or_default());
    }
    if is_xlsx {
        let bytes = match calcpad_core::openxml::variables_export::workbook(&captured) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{}", e);
                return ExitCode::from(1);
            }
        };
        if let Err(e) = std::fs::write(&out_path, bytes) {
            eprintln!("{}", e);
            return ExitCode::from(1);
        }
        return ExitCode::SUCCESS;
    }
    if is_docx {
        calcpad_docx::fonts::load_system_fonts();
        let base = full_input.parent().map(|d| d.to_path_buf()).unwrap_or_default();
        let resolver = move |src: &str| -> Option<Vec<u8>> {
            let path = src.strip_prefix("file:///").unwrap_or(src);
            let p = Path::new(path);
            let p = if p.is_absolute() { p.to_path_buf() } else { base.join(p) };
            std::fs::read(p).ok()
        };
        let page = format!("<html><body>{}</body></html>", html);
        let bytes = calcpad_docx::html_to_docx(&page, &expressions, Some(&resolver));
        if let Err(e) = std::fs::write(&out_path, bytes) {
            eprintln!("{}", e);
            return ExitCode::from(1);
        }
        return ExitCode::SUCCESS;
    }
    let content = if is_body_only {
        format!("<div class=\"calcpad-output\">{}</div>", html)
    } else {
        format!(
            "<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\"><title>Calcpad</title></head><body>{} </body></html>",
            html
        )
    };
    if let Err(e) = std::fs::write(&out_path, content) {
        eprintln!("{}", e);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
