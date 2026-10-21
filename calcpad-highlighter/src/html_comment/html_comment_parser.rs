//! HtmlCommentParser.cs
use crate::linter::helpers::directive_json_reporter::parse_json_document;
use crate::text::is_white_space;
use crate::tokenizer::models::{Token, TokenType, TokenizerResult};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HtmlCommentParseStatus {
    Success,
    InvalidJson,
}

#[derive(Clone, Debug)]
pub struct HtmlCommentBlock {
    /// Zero-based start line in the tokenizer result.
    pub start_line: i32,
    /// Zero-based end line (inclusive).
    pub end_line: i32,
    pub start_column: i32,
    /// Column just past the comment's extent on `start_line`.
    pub end_column: i32,
    /// Raw text between `<!--` and `-->`, trimmed.
    pub raw_json: Option<String>,
    /// Parsed JSON; None if parsing failed.
    pub data: Option<Value>,
    /// JSON parse error message (serde_json's wording, not .NET's); None on success.
    pub parse_error: Option<String>,
    pub status: HtmlCommentParseStatus,
}

const OPEN_MARKER: &str = "<!--";
const CLOSE_MARKER: &str = "-->";

#[derive(Default)]
pub struct HtmlCommentParser;

#[derive(PartialEq)]
enum ParseState {
    Normal,
    InHtmlComment,
}

impl HtmlCommentParser {
    pub fn new() -> Self {
        HtmlCommentParser
    }

    pub fn parse_result(&self, tokenizer_result: &TokenizerResult) -> Vec<HtmlCommentBlock> {
        self.parse(&tokenizer_result.tokens)
    }

    pub fn parse(&self, tokens: &[Token]) -> Vec<HtmlCommentBlock> {
        let mut results = Vec::new();
        let mut state = ParseState::Normal;
        let mut buffer: Vec<String> = Vec::new();
        let mut last_line: i32 = -1;
        let mut opener: Option<&Token> = None;

        // One buffer entry per source line: several HtmlComment tokens on the same line are concatenated.
        fn add_fragment(buffer: &mut Vec<String>, last_line: &mut i32, line: i32, text: &str) {
            if !buffer.is_empty() && line == *last_line {
                buffer.last_mut().unwrap().push_str(text);
            } else {
                buffer.push(text.to_string());
            }
            *last_line = line;
        }

        for token in tokens {
            if token.token_type != TokenType::HtmlComment {
                continue;
            }
            let content = token.text.as_str();

            if state == ParseState::Normal {
                let Some(open_idx) = content.find(OPEN_MARKER) else {
                    continue;
                };
                let after_open = &content[open_idx + OPEN_MARKER.len()..];
                if let Some(close_idx) = after_open.find(CLOSE_MARKER) {
                    if let Some(b) = build_block(token, token.line, &after_open[..close_idx]) {
                        results.push(b);
                    }
                } else {
                    buffer.clear();
                    add_fragment(&mut buffer, &mut last_line, token.line, after_open);
                    opener = Some(token);
                    state = ParseState::InHtmlComment;
                }
            } else {
                if token.line > last_line + 1 {
                    // Non-consecutive line: block broken, discard and re-evaluate as a new opener.
                    buffer.clear();
                    state = ParseState::Normal;
                    let Some(open_idx) = content.find(OPEN_MARKER) else {
                        continue;
                    };
                    let after_open = &content[open_idx + OPEN_MARKER.len()..];
                    if let Some(close_idx) = after_open.find(CLOSE_MARKER) {
                        if let Some(b) = build_block(token, token.line, &after_open[..close_idx]) {
                            results.push(b);
                        }
                    } else {
                        add_fragment(&mut buffer, &mut last_line, token.line, after_open);
                        opener = Some(token);
                        state = ParseState::InHtmlComment;
                    }
                    continue;
                }

                if let Some(closing_idx) = content.find(CLOSE_MARKER) {
                    add_fragment(&mut buffer, &mut last_line, token.line, &content[..closing_idx]);
                    if let Some(op) = opener {
                        if let Some(b) = build_block(op, token.line, &buffer.join("\n")) {
                            results.push(b);
                        }
                    }
                    buffer.clear();
                    state = ParseState::Normal;
                } else {
                    add_fragment(&mut buffer, &mut last_line, token.line, content);
                }
            }
        }
        // An open block at the end of the stream is silently discarded (unterminated).
        results
    }
}

fn build_block(opener: &Token, end_line: i32, raw_json: &str) -> Option<HtmlCommentBlock> {
    let raw = raw_json.trim_matches(is_white_space);
    if raw.is_empty() {
        return None;
    }
    let mut block = HtmlCommentBlock {
        start_line: opener.line,
        end_line,
        start_column: opener.column,
        end_column: opener.column + opener.length,
        raw_json: Some(raw.to_string()),
        data: None,
        parse_error: None,
        status: HtmlCommentParseStatus::Success,
    };
    match parse_json_document(raw) {
        Some(v) => block.data = Some(v),
        None => {
            block.status = HtmlCommentParseStatus::InvalidJson;
            block.parse_error = Some(
                serde_json::from_str::<Value>(raw)
                    .err()
                    .map_or_else(|| "Invalid JSON".to_string(), |e| e.to_string()),
            );
        }
    }
    Some(block)
}
