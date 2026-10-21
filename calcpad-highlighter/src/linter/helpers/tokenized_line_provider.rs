//! TokenizedLineProvider.cs: tokenized lines for linter validators, cached across validators.
use crate::linter::models::ParseMode;
use crate::text::{StrMap, StrSet};
use crate::tokenizer::models::{Token, TokenType, TokenizerResult};
use crate::tokenizer::CalcpadTokenizer;
use std::collections::BTreeMap;

pub struct TokenizedLineProvider {
    tokenizer: CalcpadTokenizer,
    token_cache: BTreeMap<i32, Vec<Token>>,
    line_modes: BTreeMap<i32, ParseMode>,
    full_result: Option<TokenizerResult>,
}

impl Default for TokenizedLineProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenizedLineProvider {
    pub fn new() -> Self {
        TokenizedLineProvider {
            tokenizer: CalcpadTokenizer::new(),
            token_cache: BTreeMap::new(),
            line_modes: BTreeMap::new(),
            full_result: None,
        }
    }

    /// Sets macro comment parameter information from ContentResolver Stage2 (before `tokenize`).
    pub fn set_macro_comment_parameters(
        &mut self,
        comment_params: Option<&StrMap<StrSet>>,
        param_order: Option<&StrMap<Vec<String>>>,
        macro_bodies: Option<&StrMap<String>>,
    ) {
        self.tokenizer
            .set_macro_comment_parameters(comment_params, param_order, macro_bodies);
    }

    /// Tokenize the full source and cache the results.
    pub fn tokenize(&mut self, source: &str) {
        let result = self.tokenizer.tokenize(source);
        self.token_cache.clear();
        self.line_modes.clear();
        for token in &result.tokens {
            self.token_cache.entry(token.line).or_default().push(token.clone());
        }
        for (line, mode) in &result.line_modes {
            self.line_modes.insert(*line, *mode);
        }
        self.full_result = Some(result);
    }

    /// Tokenize from a list of lines (joined with "\n").
    pub fn tokenize_lines(&mut self, lines: &[String]) {
        self.tokenize(&lines.join("\n"));
    }

    pub fn get_tokens_for_line(&self, line_number: i32) -> &[Token] {
        self.token_cache.get(&line_number).map_or(&[], Vec::as_slice)
    }

    pub fn all_tokens(&self) -> &[Token] {
        self.full_result.as_ref().map_or(&[], |r| r.tokens.as_slice())
    }

    pub fn get_tokens_of_type(&self, line_number: i32, ty: TokenType) -> impl Iterator<Item = &Token> {
        self.get_tokens_for_line(line_number)
            .iter()
            .filter(move |t| t.token_type == ty)
    }

    /// Tokens other than comments, tags and whitespace.
    pub fn get_code_tokens_for_line(&self, line_number: i32) -> impl Iterator<Item = &Token> {
        self.get_tokens_for_line(line_number).iter().filter(|t| {
            !matches!(
                t.token_type,
                TokenType::Comment | TokenType::HtmlComment | TokenType::Tag | TokenType::None
            )
        })
    }

    pub fn is_comment_line(&self, line_number: i32) -> bool {
        for t in self.get_tokens_for_line(line_number) {
            if t.token_type == TokenType::None {
                continue;
            }
            return t.token_type == TokenType::Comment || t.token_type == TokenType::HtmlComment;
        }
        false
    }

    pub fn is_directive_line(&self, line_number: i32) -> bool {
        for t in self.get_tokens_for_line(line_number) {
            if t.token_type == TokenType::None {
                continue;
            }
            return t.token_type == TokenType::Keyword;
        }
        false
    }

    pub fn has_code_tokens(&self, line_number: i32) -> bool {
        self.get_code_tokens_for_line(line_number).next().is_some()
    }

    /// Parse mode in effect at the line.
    pub fn get_line_mode(&self, line_number: i32) -> ParseMode {
        self.line_modes.get(&line_number).copied().unwrap_or(ParseMode::Cpd)
    }

    pub fn is_cpd_mode(&self, line_number: i32) -> bool {
        self.get_line_mode(line_number) == ParseMode::Cpd
    }
}
