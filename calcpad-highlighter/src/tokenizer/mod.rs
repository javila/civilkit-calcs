//! Tokenizer/: tokenizes Calcpad source for highlighting and analysis (partial class
//! `CalcpadTokenizer` upstream, split over the same files here).
//!
//!   mod.rs               core state, public API, main loop, token emission
//!   comments.rs          comment and HTML tag parsing
//!   macros.rs            macro state, definitions, parameters, arguments
//!   parsing.rs           brackets, operators, delimiters, units, line breaks
//!   type_resolution.rs   definition tracking, type resolution, lookups
//!   helpers.rs           static utility methods
//!   definitions.rs       lint-mode definition extraction, include map
//!   macro_collection.rs  macro collection mode
pub mod models;
pub mod parse_mode_tracker;

mod comments;
mod definitions;
mod helpers;
mod macro_collection;
mod macros;
mod parsing;
mod type_resolution;

use crate::content_resolution::result::SourceInfo;
use crate::linter::constants::builtins;
use crate::linter::helpers::character_helpers::is_letter_for_tokenizer;
use crate::linter::models::{DefinitionMetadata, ParseMode};
use crate::parsing::char_classifier::{is_bracket, is_delimiter, is_line_extension};
use crate::parsing::split_lines;
use crate::text::{is_white_space, is_white_space_slice, trim, StrMap, StrSet};
use helpers::{is_closing_special_content_tag, is_file_path_end_marker, is_macro_ident_char, is_plot_line};
use models::{Token, TokenType, TokenizerMode, TokenizerResult};
use parse_mode_tracker::ParseModeTracker;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TagState {
    None,
    Starting,
    Closing,
    SelfClosing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpecialContentType {
    None,
    Script,
    Style,
    Svg,
}

#[derive(Clone)]
pub(crate) struct TokenizerState {
    pub line: i32,
    pub token_start_column: i32,
    pub text: Rc<Vec<char>>,
    pub text_comment: char,
    pub tag_comment: char,
    pub is_subscript: bool,
    pub is_leading: bool,
    pub is_units: bool,
    pub is_plot: bool,
    pub is_tag: bool,
    pub is_tag_comment: bool,
    pub is_macro: bool,
    pub is_function: bool,
    pub keyword: String,
    pub has_macro: bool,
    pub macro_args: i32,
    pub command_count: i32,
    pub bracket_count: i32,
    /// Bit per bracket depth: call to a function with optional params
    pub function_call_mask: u32,
    pub matrix_count: i32,
    pub current_type: TokenType,
    pub previous_type: TokenType,
    pub allow_unary_minus: bool,
    pub is_function_definition: bool,
    pub is_in_function_params: bool,
    pub is_data_exchange_keyword: bool,
    pub expecting_file_path: bool,
    pub expecting_ui_json: bool,
    pub ui_json_block: bool,
    pub is_after_at_or_amp: bool,
    pub is_in_command_block: bool,
    pub in_special_content: SpecialContentType,
    pub has_html_content: bool,
    pub current_macro_call: Option<String>,
    pub current_macro_arg_index: i32,
}

impl Default for TokenizerState {
    fn default() -> Self {
        TokenizerState {
            line: 0,
            token_start_column: 0,
            text: Rc::new(Vec::new()),
            text_comment: '\0',
            tag_comment: '\0',
            is_subscript: false,
            is_leading: false,
            is_units: false,
            is_plot: false,
            is_tag: false,
            is_tag_comment: false,
            is_macro: false,
            is_function: false,
            keyword: String::new(),
            has_macro: false,
            macro_args: 0,
            command_count: 0,
            bracket_count: 0,
            function_call_mask: 0,
            matrix_count: 0,
            current_type: TokenType::None,
            previous_type: TokenType::None,
            allow_unary_minus: false,
            is_function_definition: false,
            is_in_function_params: false,
            is_data_exchange_keyword: false,
            expecting_file_path: false,
            expecting_ui_json: false,
            ui_json_block: false,
            is_after_at_or_amp: false,
            is_in_command_block: false,
            in_special_content: SpecialContentType::None,
            has_html_content: false,
            current_macro_call: None,
            current_macro_arg_index: 0,
        }
    }
}

impl TokenizerState {
    pub fn retain_type(&mut self) {
        if self.current_type != TokenType::None {
            self.previous_type = self.current_type;
        }
    }

    pub fn current_type_or_previous(&self) -> TokenType {
        if self.current_type == TokenType::None {
            self.previous_type
        } else {
            self.current_type
        }
    }
}

pub struct CalcpadTokenizer {
    mode: TokenizerMode,
    result: TokenizerResult,
    builder: Vec<char>,
    state: TokenizerState,
    // Carries structural state across line continuations
    explicit_line_break: bool,
    continue_line: bool,

    // Comments
    tag_state: TagState,
    current_tag_name: String,
    continue_text_comment: char,
    continue_tag_comment: char,
    in_html_comment: bool,

    // Type resolution
    defined_variables: StrSet,
    defined_functions: StrSet,
    defined_macros: StrSet,
    defined_units: StrSet,
    local_variables: StrSet,
    functions_with_defaults: StrSet,
    expecting_read_variable: bool,
    before_first_code_token: bool,
    pending_variable_name: Option<String>,
    pending_variable_line: i32,
    pending_function_name: Option<String>,
    pending_has_default: bool,
    pending_function_line: i32,
    pending_function_paren_depth: i32,

    // Macros
    in_macro_definition: bool,
    macro_parameters: StrSet,
    macro_comment_parameters: StrMap<StrSet>,
    macro_parameter_order: StrMap<Vec<String>>,
    macro_bodies: StrMap<String>,
    macro_call_pre_tokenized: Option<Vec<Vec<Token>>>,
    macro_arg_pre_emitted: bool,
    pending_macro_def_name: Option<String>,
    last_macro_call_name: Option<String>,
    has_external_macro_comment_info: bool,
    resolving_macros: Option<Rc<RefCell<StrSet>>>,

    // Macro collection
    macro_curr_name: Option<String>,
    macro_curr_params: Vec<String>,
    macro_curr_has_params: bool,
    macro_curr_defaults: Option<Vec<Option<String>>>,
    macro_curr_start_line: i32,
    macro_curr_is_inline: bool,
    macro_curr_inline_content: Option<String>,
    macro_curr_content_lines: Vec<String>,
    macro_curr_collecting_body: bool,
    macro_first_definition_lines: StrMap<i32>,
    macro_pending_metadata: Option<DefinitionMetadata>,

    // Lint definitions
    lint_capturing_expression: bool,
    lint_expression_start_column: i32,
    lint_def_line_text: Vec<char>,
    lint_def_line: i32,
    lint_def_name: Option<String>,
    lint_def_name_line: i32,
    lint_def_is_function: bool,
    lint_def_is_custom_unit: bool,
    lint_def_function_params: Option<Vec<String>>,
    lint_def_is_const: bool,
    lint_pending_function_params: Vec<String>,
    lint_pending_is_const: bool,
    lint_expecting_read_variable: bool,
    lint_pending_read_variable_name: Option<String>,
    lint_pending_metadata: Option<DefinitionMetadata>,
    include_map: Option<BTreeMap<i32, SourceInfo>>,
}

impl Default for CalcpadTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CalcpadTokenizer {
    pub fn new() -> Self {
        CalcpadTokenizer {
            mode: TokenizerMode::Highlight,
            result: TokenizerResult::new(),
            builder: Vec::with_capacity(100),
            state: TokenizerState::default(),
            explicit_line_break: false,
            continue_line: false,
            tag_state: TagState::None,
            current_tag_name: String::new(),
            continue_text_comment: '\0',
            continue_tag_comment: '\0',
            in_html_comment: false,
            defined_variables: StrSet::new(),
            defined_functions: StrSet::new(),
            defined_macros: StrSet::new_ci(),
            defined_units: StrSet::new_ci(),
            local_variables: StrSet::new(),
            functions_with_defaults: StrSet::new(),
            expecting_read_variable: false,
            before_first_code_token: false,
            pending_variable_name: None,
            pending_variable_line: -1,
            pending_function_name: None,
            pending_has_default: false,
            pending_function_line: -1,
            pending_function_paren_depth: 0,
            in_macro_definition: false,
            macro_parameters: StrSet::new(),
            macro_comment_parameters: StrMap::new_ci(),
            macro_parameter_order: StrMap::new_ci(),
            macro_bodies: StrMap::new_ci(),
            macro_call_pre_tokenized: None,
            macro_arg_pre_emitted: false,
            pending_macro_def_name: None,
            last_macro_call_name: None,
            has_external_macro_comment_info: false,
            resolving_macros: None,
            macro_curr_name: None,
            macro_curr_params: Vec::new(),
            macro_curr_has_params: false,
            macro_curr_defaults: None,
            macro_curr_start_line: -1,
            macro_curr_is_inline: false,
            macro_curr_inline_content: None,
            macro_curr_content_lines: Vec::new(),
            macro_curr_collecting_body: false,
            macro_first_definition_lines: StrMap::new_ci(),
            macro_pending_metadata: None,
            lint_capturing_expression: false,
            lint_expression_start_column: 0,
            lint_def_line_text: Vec::new(),
            lint_def_line: -1,
            lint_def_name: None,
            lint_def_name_line: -1,
            lint_def_is_function: false,
            lint_def_is_custom_unit: false,
            lint_def_function_params: None,
            lint_def_is_const: false,
            lint_pending_function_params: Vec::new(),
            lint_pending_is_const: false,
            lint_expecting_read_variable: false,
            lint_pending_read_variable_name: None,
            lint_pending_metadata: None,
            include_map: None,
        }
    }

    /// Tokenize source code (Highlight mode).
    pub fn tokenize(&mut self, source: &str) -> TokenizerResult {
        self.tokenize_mode(source, TokenizerMode::Highlight)
    }

    /// Tokenize with the given mode. In Lint mode also extracts the definition metadata, in
    /// Macro mode the macro definitions.
    pub fn tokenize_mode(&mut self, source: &str, mode: TokenizerMode) -> TokenizerResult {
        if source.is_empty() {
            return TokenizerResult::new();
        }

        self.mode = mode;
        self.result = TokenizerResult::new();
        self.builder = Vec::with_capacity(100);

        self.reset_comment_state();
        self.reset_type_resolution_state();
        self.reset_macro_state();
        self.explicit_line_break = false;
        self.continue_line = false;

        if self.mode == TokenizerMode::Lint {
            self.init_lint_state();
        }
        if self.mode == TokenizerMode::Macro {
            self.init_macro_collection_state();
        }

        let src: Vec<char> = source.chars().collect();
        let mut line_num: i32 = 0;
        let mut parse_modes = ParseModeTracker::new();
        for line in split_lines(&src) {
            let trimmed = trim(line);
            let in_macro_body = self.in_macro_definition;
            if parse_modes.mode() != ParseMode::Cpd {
                self.result.line_modes.insert(line_num, parse_modes.mode());
                if self.mode != TokenizerMode::Macro
                    && !in_macro_body
                    && !ParseModeTracker::is_directive(trimmed, parse_modes.mode())
                {
                    self.result.raw_lines.insert(line_num);
                    self.add_raw_line_macro_tokens(line, line_num);
                    line_num += 1;
                    self.continue_line = false;
                    continue;
                }
            }
            self.tokenize_line_internal(Rc::new(line.to_vec()), line_num);
            line_num += 1;
            if !in_macro_body {
                parse_modes.apply(trimmed);
            }
        }

        std::mem::take(&mut self.result)
    }

    /// HTML/markdown content lines only highlight the macro calls that expand inside them.
    fn add_raw_line_macro_tokens(&mut self, text: &[char], line_number: i32) {
        if self.defined_macros.is_empty() {
            return;
        }
        let mut end = text.iter().position(|c| *c == '$');
        while let Some(e) = end {
            let mut start = e;
            while start > 0 && is_macro_ident_char(text[start - 1], 1) && text[start - 1] != '$' {
                start -= 1;
            }
            while start < e && !is_macro_ident_char(text[start], 0) {
                start += 1;
            }
            for i in start..e {
                let candidate: String = text[i..=e].iter().collect();
                if self.defined_macros.contains(&candidate) {
                    let len = (e + 1 - i) as i32;
                    self.result
                        .add_token(Token::new(line_number, i as i32, len, TokenType::Macro, candidate));
                    break;
                }
            }
            end = text[e + 1..].iter().position(|c| *c == '$').map(|n| e + 1 + n);
        }
    }

    /// Tokenize a single line (for incremental updates).
    pub fn tokenize_single_line(&mut self, line: &str, line_number: i32) -> TokenizerResult {
        self.result = TokenizerResult::new();
        self.builder = Vec::with_capacity(100);
        self.in_html_comment = false;
        self.explicit_line_break = false;
        self.continue_line = false;
        self.tokenize_line_internal(Rc::new(line.chars().collect()), line_number);
        std::mem::take(&mut self.result)
    }

    fn tokenize_line_internal(&mut self, text_rc: Rc<Vec<char>>, line_number: i32) {
        self.init_state(text_rc.clone(), line_number);
        let text: &[char] = &text_rc;
        self.builder.clear();
        self.tag_state = TagState::None;

        let len = text.len();
        let mut i = 0usize;
        while i < len {
            let this_i = i;
            i += 1;
            let mut c = text[this_i];
            let i = this_i;

            // Normalize whitespace (except in comments)
            if self.state.current_type != TokenType::Comment {
                if is_white_space(c) {
                    c = ' ';
                } else if c == '·' {
                    c = '*';
                }

                if i > 0 && self.state.is_leading && !is_white_space(c) {
                    self.append(TokenType::None);
                }
            }

            // Line continuation check: works in both code and comments
            if c == '_' && (i == len - 1 || is_white_space_slice(&text[i + 1..])) && i > 0 && text[i - 1] == ' ' {
                self.parse_line_break(i as i32);
                break;
            }

            // Format string handling: accumulate format specifier characters
            if self.state.current_type == TokenType::Format && c != '\'' && c != '"' && c != '$' && !is_white_space(c) {
                self.builder.push(c);
                continue;
            }

            // Units mode tracking
            if !self.state.is_plot
                && self.state.matrix_count == 0
                && self.state.current_type != TokenType::Comment
                && self.state.current_type != TokenType::SettingsJson
                && c == '|'
            {
                self.state.is_units = true;
            }

            // Comment parsing
            if self.state.macro_args == 0 {
                self.parse_comment(c);
            }

            // #UI's block is optional, so only a '{' opens the mode
            if self.state.expecting_ui_json && c != ' ' {
                self.state.expecting_ui_json = false;
                if c == '{' {
                    self.state.current_type = TokenType::SettingsJson;
                    self.state.ui_json_block = true;
                }
            }

            // Main parsing logic
            if self.state.macro_args > 0 {
                self.parse_macro_args(c, i as i32);
            } else if self.state.text_comment != '\0' {
                // Inside script/style/svg, skip tag parsing for all < except the matching closing tag
                if c == '<'
                    && self.state.in_special_content != SpecialContentType::None
                    && !is_closing_special_content_tag(text, i, self.state.in_special_content)
                {
                    self.builder.push(c);
                } else {
                    self.parse_tag_in_comment(c);
                }
            } else if self.state.current_type == TokenType::Include {
                // #include filename: can have spaces, ends at comment or newline
                if c == '#' {
                    self.append(TokenType::Include);
                    self.state.current_type = TokenType::Bracket;
                    self.builder.push(c);
                } else if c == '\'' || c == '"' {
                    let blank = self.builder.is_empty() || is_white_space_slice(&self.builder);
                    if blank {
                        // Start of a quoted filename
                        self.builder.push(c);
                    } else {
                        // Trailing comment: end the include token
                        self.append(TokenType::Include);
                        self.parse_comment(c);
                    }
                } else {
                    self.builder.push(c);
                }
            } else if self.state.current_type == TokenType::FilePath {
                // File paths can contain spaces; they end at @ (range), a quote (comment), or
                // keywords like sep/type
                if c == '\'' || c == '"' {
                    self.append(TokenType::FilePath);
                    self.state.expecting_file_path = false;
                    self.parse_comment(c);
                } else if c == '@' {
                    self.append(TokenType::FilePath);
                    self.state.expecting_file_path = false;
                    self.state.current_type = TokenType::None;
                    self.builder.push(c);
                    self.append(TokenType::Operator);
                } else if c == ' ' && is_file_path_end_marker(text, i) {
                    self.append(TokenType::FilePath);
                    self.state.expecting_file_path = false;
                    self.state.current_type = TokenType::None;
                    self.builder.push(c);
                    self.append(TokenType::None);
                } else {
                    self.builder.push(c);
                }
            } else if self.state.current_type == TokenType::SettingsJson {
                // #settings {json}: the whole payload is one token
                self.builder.push(c);
                // #UI's block ends at the first '}'
                if self.state.ui_json_block && c == '}' {
                    self.append(TokenType::SettingsJson);
                    self.state.ui_json_block = false;
                    self.state.current_type = TokenType::None;
                }
            } else if c == '$' && !self.builder.is_empty() {
                self.parse_macro();
            } else if self.is_double_op(c, '%') {
                self.append_double_operator_shortcut('⦼');
            } else if self.is_double_op(c, '<') {
                self.append_double_operator_shortcut('∠');
            } else if is_white_space(c) {
                self.parse_space(c, i as i32);
            } else if is_bracket(c) {
                self.parse_brackets(c, i as i32);
            } else if builtins::operators().contains(&c) {
                if c == '<' && i + 1 < len && text[i + 1] == '<' {
                    self.builder.push('<');
                } else {
                    self.parse_operator(c);
                    if self.parse_macro_content(c, i, len) {
                        break;
                    }
                }
            } else if is_delimiter(c) {
                self.parse_delimiter(c);
            } else if c == '.'
                && self.builder.is_empty()
                && (self.state.current_type == TokenType::MacroParameter || self.state.current_type == TokenType::Macro)
            {
                // Element access dot after a macro parameter/call: the macro token was already emitted
                self.state.token_start_column = i as i32;
                self.builder.push(c);
                self.state.current_type = TokenType::Operator;
                self.append(TokenType::Operator);
            } else if c == '.' && self.builder.is_empty() && self.state.is_leading {
                // Custom unit prefix dot at the start of a line (.USD = 1)
                self.state.token_start_column = i as i32;
                self.builder.push(c);
                self.state.current_type = TokenType::Operator;
                self.append(TokenType::Operator);
            } else if self.builder.is_empty() {
                self.state.current_type = self.init_type(c, self.state.current_type);
                // Characters that resolve to Comment here are dropped rather than buffered. A
                // quote was already emitted by parse_comment, so the cursor is past it.
                if self.state.current_type != TokenType::Comment {
                    self.state.token_start_column = i as i32;
                    self.builder.push(c);
                } else if c != '\'' && c != '"' {
                    self.state.token_start_column = i as i32 + 1;
                }

                if self.state.current_type == TokenType::Input {
                    self.append(TokenType::Input);
                }
            } else if self.state.current_type == TokenType::Const && c == 'i' {
                self.parse_imaginary(c, i, len, text);
            } else if self.state.current_type == TokenType::Const && is_letter_for_tokenizer(c) {
                self.parse_units(c);
            } else if self.state.current_type == TokenType::Variable && c == '.' {
                // Dots after an identifier are always element access
                self.append(TokenType::Variable);
                self.state.token_start_column = i as i32;
                self.builder.push(c);
                self.state.current_type = TokenType::Operator;
                self.append(TokenType::Operator);
            } else {
                if self.state.current_type == TokenType::None && !self.builder.is_empty() {
                    self.state.current_type = self.state.previous_type;
                    self.append(self.state.current_type);
                    self.state.token_start_column = i as i32;
                    self.state.current_type = self.init_type(c, self.state.current_type);
                }
                self.builder.push(c);
            }

            self.state.retain_type();

            if !is_white_space(c) {
                self.state.is_leading = false;
            }

            if self.state.current_type == TokenType::Units || self.state.current_type == TokenType::Variable {
                if c == '_' {
                    self.state.is_subscript = true;
                }
            } else {
                self.state.is_subscript = false;
            }
        }

        // Finalize the remaining buffer
        self.state.current_type = self.state.current_type_or_previous();
        if self.state.current_type == TokenType::Comment || self.in_html_comment {
            self.check_html_comment();
        }

        self.append(self.state.current_type);

        if self.mode == TokenizerMode::Lint {
            self.finalize_line_lint();
        }
        if self.mode == TokenizerMode::Macro {
            self.finalize_line_macro_collection();
        }

        // Multiline macro definition: #def macro$(...) with no = on this line
        if self.state.has_macro && !self.in_macro_definition && !self.macro_parameters.is_empty() {
            self.in_macro_definition = true;
        }

        // Inline macro definitions: clear the parameters at the end of the line
        if self.state.has_macro && self.in_macro_definition && !self.state.is_macro && self.state.is_function {
            self.in_macro_definition = false;
            self.macro_parameters.clear();
        }

        self.continue_line = self.explicit_line_break || self.has_implicit_continuation(text);
        self.explicit_line_break = false;
    }

    /// True when the line ends with a line extension character outside a comment and brackets,
    /// matrices or command blocks are still open, so the next line continues it.
    fn has_implicit_continuation(&self, text: &[char]) -> bool {
        if self.state.text_comment != '\0'
            || (self.state.bracket_count <= 0 && self.state.matrix_count <= 0 && self.state.command_count <= 0)
        {
            return false;
        }
        let mut i = text.len() as isize - 1;
        while i >= 0 && is_white_space(text[i as usize]) {
            i -= 1;
        }
        i >= 0 && is_line_extension(text[i as usize])
    }

    fn init_state(&mut self, text: Rc<Vec<char>>, line_number: i32) {
        let prev_special = self.state.in_special_content;
        let prev_html = self.state.has_html_content;
        let previous = self.state.clone();
        let is_continuation = self.continue_line;
        self.continue_line = false;

        if !is_continuation {
            self.local_variables.clear();
        }

        self.pending_variable_name = None;
        self.pending_variable_line = -1;
        self.pending_function_name = None;
        self.pending_function_line = -1;
        self.pending_function_paren_depth = 0;
        self.pending_has_default = false;
        self.before_first_code_token = true;
        let is_plot = is_plot_line(&text);
        self.state = TokenizerState {
            line: line_number,
            text,
            is_leading: true,
            is_plot,
            allow_unary_minus: true,
            keyword: String::new(),
            in_special_content: prev_special,
            has_html_content: prev_html,
            ..Default::default()
        };

        if is_continuation {
            self.state.is_plot |= previous.is_plot;
            self.state.is_units = previous.is_units;
            self.state.bracket_count = previous.bracket_count;
            self.state.function_call_mask = previous.function_call_mask;
            self.state.matrix_count = previous.matrix_count;
            self.state.command_count = previous.command_count;
            self.state.is_in_command_block = previous.is_in_command_block;
            self.state.is_after_at_or_amp = previous.is_after_at_or_amp;
            self.state.is_function = previous.is_function;
            self.state.is_function_definition = previous.is_function_definition;
            self.state.is_in_function_params = previous.is_in_function_params;
            self.state.is_macro = previous.is_macro;
            self.state.has_macro = previous.has_macro;
            self.state.macro_args = previous.macro_args;
            self.state.is_data_exchange_keyword = previous.is_data_exchange_keyword;
            self.state.keyword = previous.keyword.clone();
            self.state.current_macro_call = previous.current_macro_call.clone();
            self.state.current_macro_arg_index = previous.current_macro_arg_index;
        }

        // Restore the comment state when the previous line ended with " _" inside a comment
        if self.continue_text_comment != '\0' {
            self.state.text_comment = self.continue_text_comment;
            self.state.tag_comment = self.continue_tag_comment;
            self.state.current_type = match self.state.in_special_content {
                SpecialContentType::Script => TokenType::JavaScript,
                SpecialContentType::Style => TokenType::Css,
                SpecialContentType::Svg => TokenType::Svg,
                SpecialContentType::None => {
                    if self.state.has_html_content {
                        TokenType::HtmlContent
                    } else {
                        TokenType::Comment
                    }
                }
            };
            self.continue_text_comment = '\0';
            self.continue_tag_comment = '\0';
        }
    }

    pub(crate) fn append(&mut self, ty: TokenType) {
        if self.builder.is_empty() {
            return;
        }
        let text: String = self.builder.iter().collect();
        self.builder.clear();

        if ty == TokenType::Input {
            self.add_token(ty, &text);
            return;
        }

        // Resolve types based on context (no error detection here)
        let ty = self.resolve_type(ty, &text);

        // Macro parameters inside comment content within a macro definition
        if self.in_macro_definition
            && !self.macro_parameters.is_empty()
            && matches!(
                ty,
                TokenType::Comment | TokenType::HtmlContent | TokenType::Css | TokenType::JavaScript | TokenType::Svg
            )
        {
            self.append_comment_with_macro_params(ty, &text);
            self.state.allow_unary_minus = true;
            return;
        }

        if !text.is_empty() {
            self.add_token(ty, &text);
        }

        self.state.allow_unary_minus =
            ty == TokenType::Comment || ty == TokenType::Operator || text == "(" || text == "{" || text == "[";
    }

    pub(crate) fn add_token(&mut self, ty: TokenType, text: &str) {
        let len = text.chars().count() as i32;
        // None tokens need no highlighting
        if ty == TokenType::None {
            self.state.token_start_column += len;
            return;
        }

        // A token that can occupy a value position ends the "before first code token" window
        if self.before_first_code_token
            && !matches!(
                ty,
                TokenType::Comment
                    | TokenType::Tag
                    | TokenType::HtmlComment
                    | TokenType::HtmlContent
                    | TokenType::Css
                    | TokenType::JavaScript
                    | TokenType::Svg
                    | TokenType::Keyword
                    | TokenType::ControlBlockKeyword
                    | TokenType::Bracket
                    | TokenType::Operator
                    | TokenType::SettingsJson
            )
        {
            self.before_first_code_token = false;
        }

        self.track_definitions(ty, text);

        let token = Token::new(self.state.line, self.state.token_start_column, len, ty, text);
        self.result.add_token(token);
        self.state.token_start_column += len;
    }
}
