//! WebAssembly bindings of calcpad-core for the CalcpadCE web editor.
//!
//! [`convert`] does what `POST /api/calcpad/convert` of Calcpad.Server does
//! (Calcpad.Web/backend: CalcpadController.ConvertToHtml and CalcpadService.Convert):
//! macro expansion, the expression pass, the error list and the HTML template. `#include`
//! and `#read` files come from an in-memory map that the page fills with [`set_file`] and
//! [`set_file_bytes`]; files that `#write`/`#append` produce go into the same map and are
//! returned with the result, for the page to save.
//!
//! Differences from the server:
//! * the code view (`unwrap` / macro errors) is not syntax highlighted (the server's
//!   fallback when Calcpad.Highlighter fails), since Calcpad.Highlighter is not ported;
//! * the bundled fonts (`BundledFonts`) are not compiled in: the page passes the same
//!   `<style>`/`<script>` markup once with [`set_report_head`];
//! * the datagrid libraries (`BundledUiAssets`) are left as the marker [`UI_ASSETS_MARKER`],
//!   which the page replaces with the libraries, so they load only when a grid is shown.
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use calcpad_core::calcpad_error::{CalcpadError, CalcpadErrorSource};
use calcpad_core::data_files::{normalize_join, posix_directory_name, DataFiles};
use calcpad_core::netfmt::html_encode;
use calcpad_core::parsers::expression_parser::{base64_encode, ExpressionParser};
use calcpad_core::parsers::macro_parser::{FileProvider, MacroParser};
use calcpad_core::settings::{ColorScales, LightDirections, MathSettings, PlotSettings};
use calcpad_core::validator::is_keyword;
use calcpad_core::CResult;
use serde_json::Value as Json;
use wasm_bindgen::prelude::*;

const TEMPLATE: &str = include_str!("../assets/template.html");
/// `UiPreviewScript.GetScriptTag()` (Calcpad.Web/backend, MIT): wires the input form.
const UI_PREVIEW_SCRIPT: &str = include_str!("../assets/ui-preview-script.html");
/// Stands where `BundledUiAssets.GetHeadMarkup()` goes; the page swaps the libraries in.
pub const UI_ASSETS_MARKER: &str = "<!--calcpad-ui-assets-->";

thread_local! {
    static FILES: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
    static WRITTEN: RefCell<Vec<(String, Vec<u8>)>> = const { RefCell::new(Vec::new()) };
    static REPORT_HEAD: RefCell<String> = const { RefCell::new(String::new()) };
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date)]
    fn now() -> f64;
}

#[wasm_bindgen(start)]
pub fn start() {
    calcpad_core::clock::set_clock(now);
}

/// The engine version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Adds or replaces a text file that `#include` and `#read` can read (forward slashes).
#[wasm_bindgen]
pub fn set_file(path: &str, content: &str) {
    FILES.with(|f| f.borrow_mut().insert(normalize(path), content.as_bytes().to_vec()));
}

/// Adds or replaces a binary file (an Excel workbook for `#read`).
#[wasm_bindgen]
pub fn set_file_bytes(path: &str, content: &[u8]) {
    FILES.with(|f| f.borrow_mut().insert(normalize(path), content.to_vec()));
}

/// The markup every page carries before `</head>`: the report fonts (`BundledFonts`).
#[wasm_bindgen]
pub fn set_report_head(markup: &str) {
    REPORT_HEAD.with(|h| *h.borrow_mut() = markup.to_string());
}

#[wasm_bindgen]
pub fn remove_file(path: &str) {
    FILES.with(|f| f.borrow_mut().remove(&normalize(path)));
}

#[wasm_bindgen]
pub fn clear_files() {
    FILES.with(|f| f.borrow_mut().clear());
}

/// `POST /api/calcpad/convert[?unwrap=true]`: takes the request body as JSON and returns
/// `{"html": ..., "errors": [...]}` (the response body and the `X-Calcpad-Errors` header),
/// or `{"status": 400|500, "error": ...}` where the server returns that status.
#[wasm_bindgen]
pub fn convert(request_json: &str, unwrap: bool) -> String {
    let req: Json = match serde_json::from_str(request_json) {
        Ok(v) => v,
        Err(e) => return status_json(400, &e.to_string()),
    };
    let r = ConvertRequest::from_json(&req);
    if r.content.trim().is_empty() {
        return status_json(400, "Content is required");
    }
    WRITTEN.with(|w| w.borrow_mut().clear());
    let (mut html, errors) = convert_inner(&r, unwrap || r.force_unwrapped_code);
    if unwrap {
        html = process_data_text_links(&html);
    }
    let files: Vec<Json> = WRITTEN.with(|w| {
        w.borrow_mut()
            .drain(..)
            .map(|(path, data)| serde_json::json!({ "path": path, "data": base64_encode(&data) }))
            .collect()
    });
    let errors_json: Vec<Json> = errors
        .iter()
        .map(|e| {
            serde_json::json!({
                "sourceLine": e.source_line,
                "outputLine": e.output_line,
                "message": e.message,
                "source": match e.source {
                    CalcpadErrorSource::Macro => "Macro",
                    CalcpadErrorSource::Expression => "Expression",
                },
            })
        })
        .collect();
    serde_json::json!({ "html": html, "errors": errors_json, "files": files }).to_string()
}

fn status_json(status: u16, error: &str) -> String {
    serde_json::json!({ "status": status, "error": error }).to_string()
}

struct ConvertRequest {
    content: String,
    math: MathSettings,
    plot: PlotSettings,
    is_us: bool,
    force_unwrapped_code: bool,
    theme: String,
    source_file_path: Option<String>,
    for_print: bool,
    enable_ui: bool,
    include_line_anchors: Option<bool>,
    hide_error_lines: Option<bool>,
    ui_overrides: Option<HashMap<String, String>>,
    write: bool,
}

/// Case insensitive property lookup, as ASP.NET Core binds JSON.
fn prop<'a>(obj: &'a Json, name: &str) -> Option<&'a Json> {
    obj.as_object()?
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v)
        .filter(|v| !v.is_null())
}

fn prop_bool(obj: &Json, name: &str) -> Option<bool> {
    prop(obj, name).and_then(Json::as_bool)
}

fn prop_f64(obj: &Json, name: &str) -> Option<f64> {
    prop(obj, name).and_then(Json::as_f64)
}

fn prop_str(obj: &Json, name: &str) -> Option<String> {
    prop(obj, name).and_then(Json::as_str).map(str::to_string)
}

impl ConvertRequest {
    fn from_json(req: &Json) -> ConvertRequest {
        let mut math = MathSettings::default();
        let mut plot = PlotSettings::default();
        plot.vector_graphics = true;
        // Calcpad.Core Settings: IsUs defaults to true.
        let mut is_us = true;
        if let Some(settings) = prop(req, "settings") {
            if let Some(m) = prop(settings, "math") {
                if let Some(v) = prop_f64(m, "decimals") {
                    math.set_decimals(v as i32);
                }
                if let Some(v) = prop_f64(m, "degrees") {
                    math.degrees = v as i32;
                }
                if let Some(v) = prop_bool(m, "isComplex") {
                    math.is_complex = v;
                }
                if let Some(v) = prop_bool(m, "substitute") {
                    math.substitute = v;
                }
                if let Some(v) = prop_bool(m, "formatEquations") {
                    math.format_equations = v;
                }
                if let Some(v) = prop_bool(m, "zeroSmallMatrixElements") {
                    math.zero_small_matrix_elements = v;
                }
                if let Some(v) = prop_bool(m, "showHiddenOutput") {
                    math.show_hidden_output = v;
                }
                if let Some(v) = prop_f64(m, "maxOutputCount") {
                    math.set_max_output_count(v as i32);
                }
                if let Some(v) = prop_str(m, "formatString") {
                    math.format_string = if v.is_empty() { None } else { Some(v) };
                }
                if let Some(v) = prop_f64(m, "precision") {
                    math.precision = v;
                }
                if let Some(v) = prop_f64(m, "tol") {
                    math.tol = v;
                }
            }
            if let Some(p) = prop(settings, "plot") {
                if let Some(v) = prop_bool(p, "isAdaptive") {
                    plot.is_adaptive = v;
                }
                if let Some(v) = prop_f64(p, "screenScaleFactor") {
                    plot.screen_scale_factor = v;
                }
                if let Some(v) = prop_str(p, "imagePath") {
                    plot.image_path = v;
                }
                if let Some(v) = prop_str(p, "imageUri") {
                    plot.image_uri = v;
                }
                // Raster $Plot output needs SkiaSharp, which is not ported, so the browser always
                // draws vector (SVG) plots and ignores the editor's "Vector graphics" setting.
                if let Some(v) = prop(p, "colorScale") {
                    let parsed = match v {
                        Json::Number(n) => n.as_u64().and_then(|i| ColorScales::ALL.get(i as usize).copied()),
                        Json::String(s) => ColorScales::parse(s),
                        _ => None,
                    };
                    if let Some(c) = parsed {
                        plot.color_scale = c;
                    }
                }
                if let Some(v) = prop_bool(p, "smoothScale") {
                    plot.smooth_scale = v;
                }
                if let Some(v) = prop_bool(p, "shadows") {
                    plot.set_shadows(v);
                }
                if let Some(v) = prop(p, "lightDirection") {
                    let parsed = match v {
                        Json::Number(n) => n.as_u64().and_then(|i| LightDirections::ALL.get(i as usize).copied()),
                        Json::String(s) => LightDirections::parse(s),
                        _ => None,
                    };
                    if let Some(d) = parsed {
                        plot.light_direction = d;
                    }
                }
                if let Some(v) = prop_f64(p, "width") {
                    plot.width = v as i32;
                }
                if let Some(v) = prop_f64(p, "height") {
                    plot.height = v as i32;
                }
                if let Some(v) = prop_f64(p, "step") {
                    plot.step = v as i32;
                }
            }
            if let Some(v) = prop_bool(settings, "isUs") {
                is_us = v;
            }
        }
        ConvertRequest {
            content: prop_str(req, "content").unwrap_or_default(),
            math,
            plot,
            is_us,
            force_unwrapped_code: prop_bool(req, "forceUnwrappedCode").unwrap_or(false),
            theme: prop_str(req, "theme").unwrap_or_else(|| "light".to_string()),
            source_file_path: prop_str(req, "sourceFilePath"),
            for_print: prop_bool(req, "forPrint").unwrap_or(false),
            enable_ui: prop_bool(req, "enableUi").unwrap_or(false),
            include_line_anchors: prop_bool(req, "includeLineAnchors"),
            hide_error_lines: prop_bool(req, "hideErrorLines"),
            ui_overrides: prop(req, "uiOverrides").and_then(Json::as_object).map(|o| {
                o.iter()
                    .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_string())))
                    .collect()
            }),
            write: prop_bool(req, "write").unwrap_or(false),
        }
    }
}

/// `CalcpadService.Convert`
fn convert_inner(r: &ConvertRequest, force_unwrapped_code: bool) -> (String, Vec<CalcpadError>) {
    let enable_ui = r.enable_ui && !r.for_print;
    let show_error_lines = !(r.hide_error_lines.unwrap_or(enable_ui));
    let files = MemFiles;
    let mut macro_parser = MacroParser::new(&files);
    macro_parser.source_file_path = r.source_file_path.clone();
    let (has_macro_errors, output_text) = macro_parser.parse(&r.content, true);
    let mut errors: Vec<CalcpadError> = macro_parser.errors.clone();
    let make_parser = |debug: bool| {
        let mut parser = ExpressionParser::new();
        *parser.math_settings.borrow_mut() = r.math.clone();
        parser.is_us = r.is_us;
        parser.plot_settings = r.plot.clone();
        parser.debug = debug;
        parser.source_file_path = r.source_file_path.clone();
        parser.data_files = Rc::new(MemDataFiles);
        parser
    };
    let html_result = if has_macro_errors || force_unwrapped_code {
        if force_unwrapped_code && !has_macro_errors {
            // This pass only collects error lines, so it must never repeat the writes.
            let mut silent = make_parser(true);
            silent.allow_data_write = false;
            silent.parse(&output_text, true);
            errors.extend(silent.errors_list.iter().cloned());
        }
        convert_code_to_html(&output_text, &errors)
    } else {
        let mut parser = make_parser(r.include_line_anchors.unwrap_or(!r.for_print));
        parser.for_print = r.for_print;
        parser.enable_ui = enable_ui;
        parser.ui_overrides = r.ui_overrides.clone();
        parser.show_error_lines = show_error_lines;
        parser.allow_data_write = r.write;
        parser.parse(&output_text, true);
        errors.extend(parser.errors_list.iter().cloned());
        parser.html_result.clone()
    };
    (wrap_html_result(&html_result, &r.theme, enable_ui), errors)
}

/// `HttpUtility.HtmlAttributeEncode`
fn html_attribute_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// `HttpUtility.HtmlDecode` for the entities that `HtmlEncode` produces.
fn html_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let end = tail.find(';').filter(|&e| e <= 10);
        let decoded = end.and_then(|e| {
            let name = &tail[1..e];
            let c = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => {
                    if let Some(h) = name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
                        u32::from_str_radix(h, 16).ok().and_then(char::from_u32)
                    } else if let Some(d) = name.strip_prefix('#') {
                        d.parse::<u32>().ok().and_then(char::from_u32)
                    } else {
                        None
                    }
                }
            };
            c.map(|c| (c, e))
        });
        match decoded {
            Some((c, e)) => {
                out.push(c);
                rest = &tail[e + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// `CalcpadService.ConvertCodeToHtml` (the listing is not syntax highlighted: the server's
/// own fallback when tokenizing fails, since Calcpad.Highlighter is not ported).
fn convert_code_to_html(code: &str, error_list: &[CalcpadError]) -> String {
    const ERROR_STRING: &str = "#Error";
    let mut by_output: HashMap<i32, String> = HashMap::new();
    let mut by_source: HashMap<i32, String> = HashMap::new();
    for err in error_list {
        let map = if err.output_line > 0 {
            &mut by_output
        } else {
            &mut by_source
        };
        let key = if err.output_line > 0 {
            err.output_line
        } else {
            err.source_line
        };
        match map.get_mut(&key) {
            Some(prev) => {
                prev.push('\n');
                prev.push_str(&err.message);
            }
            None => {
                map.insert(key, err.message.clone());
            }
        }
    }
    let mut errors: VecDeque<i32> = VecDeque::new();
    let mut sb = String::new();
    let lines: Vec<&str> = code.split(['\r', '\n']).filter(|l| !l.is_empty()).collect();
    sb.push_str("<div class=\"code\">\n");
    let mut line_number = 0;
    for line in &lines {
        line_number += 1;
        let (line_text, source_line) = match line.find('\u{000B}') {
            None => (*line, line_number.to_string()),
            Some(v) => (&line[..v], line[v + 1..].to_string()),
        };
        let line_num_text = line_number.to_string();
        let space_count = 6usize.saturating_sub(line_num_text.len());
        let padded = "&nbsp;".repeat(space_count);
        let mut err_msg = by_output.get(&line_number).cloned();
        if err_msg.is_none() {
            if let Ok(n) = source_line.parse::<i32>() {
                err_msg = by_source.get(&n).cloned();
            }
        }
        let err_class = if err_msg.is_some() { " err-line" } else { "" };
        let err_title = match &err_msg {
            Some(m) => format!(" title=\"{}\"", html_attribute_encode(m)),
            None => String::new(),
        };
        if err_msg.is_some() {
            errors.push_back(line_number);
        }
        sb.push_str(&format!(
            "<p class=\"line-text{}\" id=\"line-{}\"{}><a class=\"line-num\" href=\"#0\" data-text=\"{}\" title=\"Source line {}\">{}{}</a>&emsp;\u{2502}&emsp;",
            err_class, line_number, err_title, source_line, source_line, padded, line_number
        ));
        if let Some(error_text) = line_text.strip_prefix(ERROR_STRING) {
            sb.push_str(&format!("<span class=\"err\">{}</span>", html_decode(error_text)));
        } else if line_text.contains("<a href=\"#0\" data-text=\"") || line_text.starts_with("Error in") {
            sb.push_str(&format!("<span class=\"err\">{}</span>", line_text));
        } else {
            sb.push_str(&html_encode(line_text));
        }
        sb.push_str("</p>\n");
    }
    sb.push_str("</div>\n");
    if !errors.is_empty() {
        sb.push_str(&format!(
            "<div class=\"errorHeader\">Found <b>{}</b> errors in modules and macros:\n",
            errors.len()
        ));
        let mut count = 0;
        while !errors.is_empty() {
            count += 1;
            if count >= 20 {
                break;
            }
            let error_line = errors.pop_front().unwrap();
            sb.push_str(&format!(
                " <span class=\"roundBox\" data-line=\"{}\">{}</span>",
                error_line, error_line
            ));
        }
        if !errors.is_empty() {
            sb.push_str(" ...");
        }
        sb.push_str("</div>\n");
        sb.push_str("<style>body {padding-top:1.1em;} .code p {margin:0; line-height:1.55em;}</style>\n");
    } else {
        sb.push_str("<style>.code p {margin:0; line-height:1.55em;}</style>\n");
    }
    sb
}

/// `CalcpadService.WrapHtmlResult`
fn wrap_html_result(html_content: &str, theme: &str, enable_ui: bool) -> String {
    let theme_class = if theme.eq_ignore_ascii_case("dark") {
        " class=\"dark-theme\""
    } else {
        ""
    };
    let mut template = TEMPLATE.replace("<body>", &format!("<body{}>", theme_class));
    let mut head_markup = REPORT_HEAD.with(|h| h.borrow().clone());
    // The datagrid libraries are large, so they only go in when a grid is present.
    if enable_ui && html_content.contains("calcpad-ui-datagrid") {
        head_markup.push_str(UI_ASSETS_MARKER);
    }
    if !head_markup.is_empty() {
        if let Some(i) = template.to_ascii_lowercase().find("</head>") {
            template.insert_str(i, &head_markup);
        }
    }
    // Report and print output carry no controls, so they get no event script.
    if enable_ui {
        if let Some(i) = template.to_ascii_lowercase().rfind("</body>") {
            template.insert_str(i, UI_PREVIEW_SCRIPT);
        }
    }
    template.replace("{{CONTENT}}", html_content)
}

/// `CalcpadController.ProcessDataTextLinks`
fn process_data_text_links(html: &str) -> String {
    // <a href="#0" data-text="123"  becomes  <a href="#line-123" data-text="123"
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(i) = rest.find("<a") {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let mut j = 2;
        let ws = |s: &str, k: &mut usize| -> bool {
            let start = *k;
            while s[*k..].starts_with(|c: char| c.is_whitespace()) {
                *k += s[*k..].chars().next().unwrap().len_utf8();
            }
            *k > start
        };
        let mut matched = None;
        if ws(tail, &mut j) && tail[j..].starts_with("href=\"#0\"") {
            j += "href=\"#0\"".len();
            if ws(tail, &mut j) && tail[j..].starts_with("data-text=\"") {
                let k = j + "data-text=\"".len();
                let digits: String = tail[k..].chars().take_while(|c| c.is_ascii_digit()).collect();
                if !digits.is_empty() && tail[k + digits.len()..].starts_with('"') {
                    matched = Some((digits.clone(), k + digits.len() + 1));
                }
            }
        }
        match matched {
            Some((d, end)) => {
                out.push_str(&format!("<a href=\"#line-{}\" data-text=\"{}\"", d, d));
                rest = &tail[end..];
            }
            None => {
                out.push_str("<a");
                rest = &tail[2..];
            }
        }
    }
    out.push_str(rest);
    out
}

// ----- in-memory files for #include ---------------------------------------------------------

fn normalize(path: &str) -> String {
    let p = path.replace('\\', "/");
    let absolute = p.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in p.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{}", joined)
    } else {
        joined
    }
}

/// `CalcpadService.ProcessIncludedContent`: drops `#local` sections and saved UI override
/// comments.
fn process_included_content(content: &str) -> String {
    let mut is_local = false;
    let mut is_ui_overrides_comment = false;
    let mut output: Vec<&str> = Vec::new();
    for line in content.split("\r\n").flat_map(|l| l.split(['\r', '\n'])) {
        if is_ui_overrides_comment {
            if line.contains("-->") {
                is_ui_overrides_comment = false;
            }
            continue;
        }
        if is_keyword(line, "#local") {
            is_local = true;
        } else if is_keyword(line, "#global") {
            is_local = false;
        } else if is_ui_overrides_comment_start(line) {
            is_ui_overrides_comment = !line.contains("-->");
        } else if !is_local {
            output.push(line);
        }
    }
    output.join("\n")
}

fn is_ui_overrides_comment_start(line: &str) -> bool {
    let t = line.trim_start();
    (t.starts_with('\'') || t.starts_with('"')) && t.contains("<!--") && t.contains("uiOverrides")
}

struct MemFiles;

impl FileProvider for MemFiles {
    fn full_path(&self, path: &str, base_dir: Option<&str>) -> String {
        let p = path.replace('\\', "/");
        if p.starts_with('/') {
            return normalize(&p);
        }
        match base_dir {
            Some(b) => normalize(&format!("{}/{}", b, p)),
            None => normalize(&format!("/{}", p)),
        }
    }

    fn exists(&self, path: &str) -> bool {
        FILES.with(|f| f.borrow().contains_key(&normalize(path)))
    }

    fn directory_name(&self, path: &str) -> Option<String> {
        let p = normalize(path);
        p.rfind('/')
            .map(|i| if i == 0 { "/".to_string() } else { p[..i].to_string() })
    }

    fn include(&self, path: &str, _fields: Option<VecDeque<String>>) -> CResult<String> {
        let content = FILES.with(|f| f.borrow().get(&normalize(path)).cloned());
        Ok(match content {
            Some(c) => {
                let text = String::from_utf8_lossy(&c);
                process_included_content(text.strip_prefix('\u{FEFF}').unwrap_or(&text))
            }
            None => format!("' Error reading file: {} - file not found", path),
        })
    }
}

/// `#read`/`#write`/`#append` over the in-memory files; writes are also kept in WRITTEN.
struct MemDataFiles;

impl DataFiles for MemDataFiles {
    fn full_path(&self, path: &str, base_dir: Option<&str>) -> String {
        normalize_join(path, base_dir)
    }

    fn directory_name(&self, path: &str) -> Option<String> {
        posix_directory_name(path)
    }

    fn read(&self, full_path: &str) -> Option<Vec<u8>> {
        FILES.with(|f| f.borrow().get(&normalize(full_path)).cloned())
    }

    /// The root always exists; a folder exists while it holds a file.
    fn dir_exists(&self, dir: &str) -> bool {
        let dir = normalize(dir);
        if dir == "/" {
            return true;
        }
        let prefix = format!("{}/", dir.trim_end_matches('/'));
        FILES.with(|f| f.borrow().keys().any(|k| k.starts_with(&prefix)))
    }

    fn write(&self, full_path: &str, data: &[u8]) -> Result<(), String> {
        let path = normalize(full_path);
        FILES.with(|f| f.borrow_mut().insert(path.clone(), data.to_vec()));
        WRITTEN.with(|w| {
            let mut w = w.borrow_mut();
            w.retain(|(p, _)| *p != path);
            w.push((path, data.to_vec()));
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_a_worksheet() {
        let out = convert(
            r#"{"content":"a = 2\nb = a^2","settings":{"math":{"decimals":2}}}"#,
            false,
        );
        let v: Json = serde_json::from_str(&out).unwrap();
        let html = v["html"].as_str().unwrap();
        assert!(html.contains("<var>b</var>"));
        assert!(!html.contains("{{CONTENT}}"));
        assert_eq!(v["errors"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn reports_expression_errors() {
        let out = convert(r#"{"content":"a = b + 1"}"#, false);
        let v: Json = serde_json::from_str(&out).unwrap();
        let e = &v["errors"][0];
        assert_eq!(e["source"], "Expression");
        assert_eq!(e["sourceLine"], 1);
    }

    #[test]
    fn includes_from_memory() {
        set_file("/lib/x.cpd", "c = 7\n#local\nd = 1\n#global\n");
        let out = convert(
            r##"{"content":"#include x.cpd\nc","sourceFilePath":"/lib/main.cpd"}"##,
            false,
        );
        let v: Json = serde_json::from_str(&out).unwrap();
        let html = v["html"].as_str().unwrap();
        assert!(html.contains("= 7"), "{}", html);
        clear_files();
    }

    #[test]
    fn unwrapped_view_links_lines() {
        let out = convert(r#"{"content":"a = 1"}"#, true);
        let v: Json = serde_json::from_str(&out).unwrap();
        let html = v["html"].as_str().unwrap();
        assert!(html.contains("<div class=\"code\">"));
        assert_eq!(
            process_data_text_links(
                "x <a href=\"#0\" data-text=\"12\">12</a> <a class=\"n\" href=\"#0\" data-text=\"3\">"
            ),
            "x <a href=\"#line-12\" data-text=\"12\">12</a> <a class=\"n\" href=\"#0\" data-text=\"3\">"
        );
    }

    #[test]
    fn empty_content_is_a_bad_request() {
        let v: Json = serde_json::from_str(&convert(r#"{"content":"  "}"#, false)).unwrap();
        assert_eq!(v["status"], 400);
    }

    #[test]
    fn reads_and_writes_data_files_in_memory() {
        set_file("/data.csv", "1,2\n3,4\n");
        let out = convert(
            r##"{"content":"#read M from data.csv\nM\n#write M to out.csv","write":true}"##,
            false,
        );
        let v: Json = serde_json::from_str(&out).unwrap();
        let html = v["html"].as_str().unwrap();
        assert!(html.contains("was successfully read from"), "{}", html);
        assert!(html.contains("was successfully written to"), "{}", html);
        assert_eq!(v["files"][0]["path"], "/out.csv");
        assert_eq!(v["files"][0]["data"], base64_encode(b"1,2\n3,4\n"));
        // Without `write` the file is reported but not produced.
        let out = convert(r##"{"content":"M = [1; 2]\n#write M to b.csv"}"##, false);
        let v: Json = serde_json::from_str(&out).unwrap();
        assert!(v["html"].as_str().unwrap().contains("will be written to"));
        assert_eq!(v["files"].as_array().unwrap().len(), 0);
        clear_files();
    }

    #[test]
    fn input_mode_gets_the_script_and_the_grid_marker() {
        let out = convert(
            r##"{"content":"#UI v = [1; 2]\n#UI L = 3m","enableUi":true,"uiOverrides":{"L":"5"}}"##,
            false,
        );
        let v: Json = serde_json::from_str(&out).unwrap();
        let html = v["html"].as_str().unwrap();
        assert!(html.contains("calcpad-ui-datagrid"));
        assert!(html.contains("value=\"5\""));
        assert!(html.contains(UI_ASSETS_MARKER));
        assert!(html.contains("uiValueChange"));
        // A report never carries controls, the event script or the libraries.
        let out = convert(
            r##"{"content":"#UI v = [1; 2]","enableUi":true,"forPrint":true}"##,
            false,
        );
        let html = serde_json::from_str::<Json>(&out).unwrap()["html"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(!html.contains(UI_ASSETS_MARKER) && !html.contains("uiValueChange"));
    }
}
