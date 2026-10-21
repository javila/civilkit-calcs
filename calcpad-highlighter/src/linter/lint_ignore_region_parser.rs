//! LintIgnoreRegionParser.cs: extracts `LintIgnoreRegion`s from `'<!--{"LintIgnore": [...]}-->`
//! metadata comments in raw source.
//!
//! Rules: codes are case-insensitive; `LintIgnore` with codes opens an independent suppression
//! for each code (re-opening an open code is a no-op); with an empty array it opens a
//! "suppress all" region; `EndLintIgnore` closes the listed codes (all open regions when empty);
//! anything still open at the end of the file suppresses through the end of the file.
use crate::html_comment::{HtmlCommentParseStatus, HtmlCommentParser};
use crate::linter::models::LintIgnoreRegion;
use crate::text::{is_white_space, StrSet};
use crate::tokenizer::CalcpadTokenizer;
use serde_json::Value;

#[derive(Default)]
pub struct LintIgnoreRegionParser;

/// The `Dictionary<string, int>(OrdinalIgnoreCase)` of open regions. .NET reuses the slot of a
/// removed entry (most recently freed first) for the next added key, which changes the
/// enumeration order, so the slots are reproduced here.
#[derive(Default)]
struct OpenByCode {
    slots: Vec<Option<(String, i32)>>,
    free: Vec<usize>,
}

impl OpenByCode {
    fn position(&self, code: &str) -> Option<usize> {
        self.slots
            .iter()
            .position(|s| s.as_ref().is_some_and(|(k, _)| crate::text::eq_ignore_case(k, code)))
    }
    fn try_add(&mut self, code: &str, start: i32) {
        if self.position(code).is_some() {
            return;
        }
        let entry = Some((code.to_string(), start));
        match self.free.pop() {
            Some(i) => self.slots[i] = entry,
            None => self.slots.push(entry),
        }
    }
    fn remove(&mut self, code: &str) -> Option<(String, i32)> {
        let i = self.position(code)?;
        self.free.push(i);
        self.slots[i].take()
    }
    fn clear(&mut self) {
        self.slots.clear();
        self.free.clear();
    }
    fn entries(&self) -> Vec<(String, i32)> {
        self.slots.iter().flatten().cloned().collect()
    }
}

impl LintIgnoreRegionParser {
    pub fn new() -> Self {
        LintIgnoreRegionParser
    }

    /// Tokenizes `content` and returns all lint-ignore regions found in HTML comment blocks.
    pub fn extract_regions(content: &str) -> Vec<LintIgnoreRegion> {
        if content.is_empty() {
            return Vec::new();
        }
        let tokens = CalcpadTokenizer::new().tokenize(content);
        let blocks = HtmlCommentParser::new().parse(&tokens.tokens);

        let mut open_by_code = OpenByCode::default();
        let mut open_all_start: i32 = -1;
        let mut result: Vec<LintIgnoreRegion> = Vec::new();

        for block in blocks {
            if block.status != HtmlCommentParseStatus::Success {
                continue;
            }
            let Some(Value::Object(data)) = &block.data else {
                continue;
            };

            if let Some(Value::Array(open)) = data.get("LintIgnore") {
                let codes = parse_codes(open);
                let region_start = block.end_line + 1;
                if codes.is_empty() {
                    if open_all_start < 0 {
                        open_all_start = region_start;
                    }
                } else {
                    for code in codes.iter() {
                        open_by_code.try_add(code, region_start);
                    }
                }
            } else if let Some(Value::Array(end)) = data.get("EndLintIgnore") {
                let codes = parse_codes(end);
                let region_end = block.start_line - 1;
                if codes.is_empty() {
                    if open_all_start >= 0 {
                        emit_if_non_empty(&mut result, open_all_start, region_end, StrSet::new_ci());
                        open_all_start = -1;
                    }
                    for (code, start) in open_by_code.entries() {
                        emit_if_non_empty(&mut result, start, region_end, single_code_set(&code));
                    }
                    open_by_code.clear();
                } else {
                    for code in codes.iter() {
                        if let Some((key, start)) = open_by_code.remove(code) {
                            emit_if_non_empty(&mut result, start, region_end, single_code_set(&key));
                        }
                    }
                }
            }
        }

        if open_all_start >= 0 {
            result.push(LintIgnoreRegion::new(open_all_start, i32::MAX, StrSet::new_ci()));
        }
        for (code, start) in open_by_code.entries() {
            result.push(LintIgnoreRegion::new(start, i32::MAX, single_code_set(&code)));
        }
        result
    }
}

fn single_code_set(code: &str) -> StrSet {
    let mut s = StrSet::new_ci();
    s.insert(code);
    s
}

fn emit_if_non_empty(result: &mut Vec<LintIgnoreRegion>, start: i32, end: i32, codes: StrSet) {
    if start <= end {
        result.push(LintIgnoreRegion::new(start, end, codes));
    }
}

fn parse_codes(items: &[Value]) -> StrSet {
    let mut set = StrSet::new_ci();
    for item in items {
        if let Value::String(code) = item {
            if !code.chars().all(is_white_space) {
                set.insert(code.trim_matches(is_white_space));
            }
        }
    }
    set
}
