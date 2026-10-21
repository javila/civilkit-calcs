//! The Calcpad.Server endpoints that front Calcpad.Highlighter (`highlight`, `lint`,
//! `definitions`, `symbol-at-position`, `prettify`, `snippets`) as functions from a request JSON
//! to a response JSON, with the same wire shapes (camelCase) and the same status codes.
//!
//! Differences from the server: includes come from an in-memory [`MemoryFiles`] instead of the
//! disk, there is no content cache (every call resolves again), and columns go back to UTF-16
//! code units at this boundary.
use crate::content_resolution::result::{CommandBlockInfo, StagedResolvedContent, SymbolLocation};
use crate::content_resolution::source_parse_modes::SourceParseModes;
use crate::content_resolution::symbol_resolver::SymbolResolver;
use crate::content_resolution::ContentResolver;
use crate::linter::helpers::type_tracker::TypeTracker;
use crate::linter::models::{CalcpadType, LinterSeverity, ParseMode};
use crate::linter::{CalcpadLinter, LintIgnoreRegionParser};
use crate::path_roots::IncludeProvider;
use crate::prettifier::{CalcpadPrettifier, PrettifierOptions};
use crate::text::{is_white_space, StrMap};
use crate::tokenizer::CalcpadTokenizer;
use crate::utf16::{from_utf16, lines_of, span_to_utf16};
use serde_json::{json, Value};
use std::collections::HashMap;

/// An in-memory file system for `#include`: normalized path to text.
#[derive(Default, Clone, Debug)]
pub struct MemoryFiles {
    files: HashMap<String, String>,
}

/// `/a/./b/../c` to `/a/c`, `\` to `/`.
pub fn normalize_path(path: &str) -> String {
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
        format!("/{joined}")
    } else {
        joined
    }
}

impl MemoryFiles {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&mut self, path: &str, content: &str) {
        self.files.insert(normalize_path(path), content.to_string());
    }
    pub fn clear(&mut self) {
        self.files.clear();
    }
}

impl IncludeProvider for MemoryFiles {
    fn read_file(&self, full_path: &str) -> Option<String> {
        self.files.get(&normalize_path(full_path)).cloned()
    }
    fn directory_exists(&self, full_path: &str) -> bool {
        let prefix = format!("{}/", normalize_path(full_path).trim_end_matches('/'));
        self.files.keys().any(|k| k.starts_with(&prefix))
    }
}

/// `{"status": n, "error": "..."}`: where the server answers a status code.
pub fn status_json(status: u16, error: &str) -> String {
    json!({ "status": status, "error": error }).to_string()
}

fn resolve(files: &MemoryFiles, content: &str, source_file_path: Option<&str>) -> StagedResolvedContent {
    let resolver = ContentResolver::with_provider(Box::new(files.clone()));
    let empty: StrMap<String> = StrMap::new_ci();
    resolver.get_staged_content(content, Some(&empty), None, source_file_path)
}

struct Request {
    content: String,
    include_text: bool,
    source_file_path: Option<String>,
    v: Value,
}

fn parse_request(json: &str) -> Result<Request, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| status_json(400, &e.to_string()))?;
    let content = v.get("content").and_then(Value::as_str).unwrap_or("").to_string();
    if content.chars().all(is_white_space) {
        return Err(status_json(400, "Content is required"));
    }
    Ok(Request {
        content,
        include_text: v.get("includeText").and_then(Value::as_bool).unwrap_or(false),
        source_file_path: v.get("sourceFilePath").and_then(Value::as_str).map(str::to_string),
        v,
    })
}

fn severity_name(s: LinterSeverity) -> &'static str {
    match s {
        LinterSeverity::Error => "error",
        LinterSeverity::Information => "information",
        LinterSeverity::Warning => "warning",
    }
}

fn type_id(t: CalcpadType) -> i32 {
    t as i32
}

/// `POST /api/calcpad/highlight`.
pub fn highlight(files: &MemoryFiles, request_json: &str) -> String {
    let req = match parse_request(request_json) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let mut tokenizer = CalcpadTokenizer::new();
    if req.source_file_path.as_deref().is_some_and(|p| !p.is_empty()) {
        let staged = resolve(files, &req.content, req.source_file_path.as_deref());
        tokenizer.set_macro_comment_parameters(
            Some(&staged.stage2.macro_comment_parameters),
            Some(&staged.stage2.macro_parameter_order),
            Some(&staged.stage2.macro_bodies),
        );
    }
    let result = tokenizer.tokenize(&req.content);
    let lines = lines_of(&req.content);
    let tokens: Vec<Value> = result
        .tokens
        .iter()
        .map(|t| {
            let (col, len) = span_to_utf16(&lines, t.line, t.column, t.length);
            json!({
                "line": t.line,
                "column": col,
                "length": len,
                "type": t.token_type.name(),
                "typeId": t.token_type as i32,
                "text": if req.include_text { Value::String(t.text.clone()) } else { Value::Null },
            })
        })
        .collect();
    json!({ "tokens": tokens }).to_string()
}

/// `POST /api/calcpad/lint`.
pub fn lint(files: &MemoryFiles, request_json: &str) -> String {
    let req = match parse_request(request_json) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let staged = resolve(files, &req.content, req.source_file_path.as_deref());
    let regions = LintIgnoreRegionParser::extract_regions(&req.content);
    let result = CalcpadLinter::new().lint(&staged, &regions);
    let lines = lines_of(&req.content);
    let diagnostics: Vec<Value> = result
        .diagnostics
        .iter()
        .map(|d| {
            let (col, len) = span_to_utf16(&lines, d.line, d.column, d.end_column - d.column);
            json!({
                "line": d.line,
                "column": col,
                "endColumn": col + len,
                "code": d.code,
                "message": d.message,
                "severity": severity_name(d.severity),
                "severityId": d.severity as i32,
                "source": d.source,
            })
        })
        .collect();
    json!({
        "errorCount": result.error_count(),
        "warningCount": result.warning_count(),
        "diagnostics": diagnostics,
    })
    .to_string()
}

fn opt_str(s: &Option<String>) -> Value {
    s.as_ref().map(|x| Value::String(x.clone())).unwrap_or(Value::Null)
}

fn opt_list(l: &Option<Vec<String>>) -> Value {
    l.as_ref().map(|x| json!(x)).unwrap_or(Value::Null)
}

fn defaults_json(d: &[Option<String>]) -> Value {
    if d.is_empty() {
        Value::Null
    } else {
        json!(d)
    }
}

fn command_block_part(cb: &Option<CommandBlockInfo>) -> (bool, Value, Value) {
    match cb {
        Some(c) => (true, json!(c.block_type), json!(c.statements)),
        None => (false, Value::Null, Value::Null),
    }
}

fn return_type_of(tracker: &TypeTracker, name: &str) -> (String, i32) {
    match tracker.functions().get(name) {
        Some(f) => (f.return_type.name().to_string(), type_id(f.return_type)),
        None => ("Unknown".to_string(), 0),
    }
}

/// `POST /api/calcpad/definitions`.
pub fn definitions(files: &MemoryFiles, request_json: &str) -> String {
    let req = match parse_request(request_json) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let staged = resolve(files, &req.content, req.source_file_path.as_deref());
    let tracker = &staged.stage3.type_tracker;

    let macros: Vec<Value> = staged
        .stage2
        .macro_definitions
        .iter()
        .map(|m| {
            let is_multiline =
                m.content.len() > 1 || (m.content.len() == 1 && m.content[0].chars().all(is_white_space));
            json!({
                "name": m.name,
                "parameters": m.params,
                "isMultiline": is_multiline,
                "content": m.content,
                "lineNumber": m.line_number,
                "source": if m.source.is_empty() { "local" } else { m.source.as_str() },
                "sourceFile": opt_str(&m.source_file),
                "description": opt_str(&m.description),
                "paramTypes": opt_list(&m.param_types),
                "paramDescriptions": opt_list(&m.param_descriptions),
                "defaults": defaults_json(&m.defaults),
            })
        })
        .collect();

    let functions: Vec<Value> = staged
        .stage3
        .functions_with_params
        .iter()
        .map(|f| {
            let (rt, rt_id) = return_type_of(tracker, &f.name);
            let (has_cb, cb_type, cb_stmts) = command_block_part(&f.command_block);
            json!({
                "name": f.name,
                "parameters": f.params,
                "expression": opt_str(&f.expression),
                "returnType": rt,
                "returnTypeId": rt_id,
                "hasCommandBlock": has_cb,
                "commandBlockType": cb_type,
                "commandBlockStatements": cb_stmts,
                "lineNumber": f.line_number,
                "source": if f.source.is_empty() { "local" } else { f.source.as_str() },
                "sourceFile": opt_str(&f.source_file),
                "description": opt_str(&f.description),
                "paramTypes": opt_list(&f.param_types),
                "paramDescriptions": opt_list(&f.param_descriptions),
                "defaults": defaults_json(&f.defaults),
            })
        })
        .collect();

    let variables: Vec<Value> = staged
        .stage3
        .variables_with_definitions
        .iter()
        .map(|v| {
            let (t, t_id) = match tracker.variables().get(&v.name) {
                Some(i) => (i.type_.name().to_string(), type_id(i.type_)),
                None => ("Unknown".to_string(), 0),
            };
            json!({
                "name": v.name,
                "expression": v.definition,
                "type": t,
                "typeId": t_id,
                "lineNumber": v.line_number,
                "source": if v.source.is_empty() { "local" } else { v.source.as_str() },
                "sourceFile": opt_str(&v.source_file),
                "description": opt_str(&v.description),
            })
        })
        .collect();

    let custom_units: Vec<Value> = staged
        .stage3
        .custom_units
        .iter()
        .map(|u| {
            json!({
                "name": u.name,
                "expression": u.definition,
                "lineNumber": u.line_number,
                "source": if u.source.is_empty() { "local" } else { u.source.as_str() },
                "sourceFile": opt_str(&u.source_file),
                "description": opt_str(&u.description),
            })
        })
        .collect();

    let line_count = req.content.matches('\n').count() as i32 + 1;
    let parse_modes: Vec<Value> = SourceParseModes::build(&staged, line_count)
        .iter()
        .map(|r| {
            json!({
                "startLine": r.start_line,
                "endLine": r.end_line,
                "mode": if r.mode == ParseMode::Html { "html" } else { "markdown" },
            })
        })
        .collect();

    json!({
        "macros": macros,
        "functions": functions,
        "variables": variables,
        "customUnits": custom_units,
        "projectPath": staged.stage2.path_roots.project(),
        "libraryPath": staged.stage2.path_roots.library(),
        "parseModes": parse_modes,
    })
    .to_string()
}

fn location_json(lines: &[Vec<char>], loc: &SymbolLocation) -> Value {
    let (col, len) = span_to_utf16(lines, loc.line, loc.column, loc.length);
    json!({
        "line": loc.line,
        "column": col,
        "length": len,
        "source": loc.source,
        "sourceFile": opt_str(&loc.source_file),
        "isAssignment": loc.is_assignment,
    })
}

/// `POST /api/calcpad/symbol-at-position`. The cursor column of the request is in UTF-16 units.
pub fn symbol_at_position(files: &MemoryFiles, request_json: &str) -> String {
    let req = match parse_request(request_json) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let line = req.v.get("line").and_then(Value::as_i64).unwrap_or(0) as i32;
    let column16 = req.v.get("column").and_then(Value::as_i64).unwrap_or(0) as i32;
    let lines = lines_of(&req.content);
    let column = match usize::try_from(line).ok().and_then(|i| lines.get(i)) {
        Some(l) => from_utf16(l, column16),
        None => column16,
    };
    let staged = resolve(files, &req.content, req.source_file_path.as_deref());
    match SymbolResolver::resolve_symbol_at(Some(&staged.stage3), line, column) {
        None => "null".to_string(),
        Some(hit) => json!({
            "symbolName": hit.name,
            "kind": format!("{:?}", hit.kind).to_lowercase(),
            "locations": hit.locations.iter().map(|l| location_json(&lines, l)).collect::<Vec<_>>(),
        })
        .to_string(),
    }
}

/// `POST /api/calcpad/prettify`.
pub fn prettify(request_json: &str) -> String {
    let v: Value = match serde_json::from_str(request_json) {
        Ok(v) => v,
        Err(e) => return status_json(400, &e.to_string()),
    };
    let Some(content) = v.get("content").and_then(Value::as_str) else {
        return status_json(400, "Content is required");
    };
    let indent = v
        .get("indentUnit")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("\t");
    let options = PrettifierOptions {
        indent_unit: Some(indent.to_string()),
        trim_trailing_whitespace: v.get("trimTrailingWhitespace").and_then(Value::as_bool).unwrap_or(true),
    };
    json!({ "content": CalcpadPrettifier::prettify(content, Some(&options)) }).to_string()
}

/// The tokens of an unwrapped code listing, for `CalcpadService.HighlightLine`: the whole listing
/// is tokenized as one document and the answer maps each line to `[column, length, class]`
/// triples (columns and lengths in Unicode scalar values, not UTF-16: the caller slices its own
/// scalar-value lines). `None` tokens are left out.
pub fn tokenize_listing(text: &str) -> String {
    let result = CalcpadTokenizer::new().tokenize(text);
    let mut by_line = serde_json::Map::new();
    for (line, tokens) in &result.tokens_by_line {
        let items: Vec<Value> = tokens
            .iter()
            .filter(|t| t.token_type != crate::tokenizer::models::TokenType::None)
            .map(|t| json!([t.column, t.length, t.token_type.class_name()]))
            .collect();
        if !items.is_empty() {
            by_line.insert(line.to_string(), Value::Array(items));
        }
    }
    Value::Object(by_line).to_string()
}
