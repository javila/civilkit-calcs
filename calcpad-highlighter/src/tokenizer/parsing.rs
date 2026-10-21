//! CalcpadTokenizer.Parsing.cs: brackets, operators, delimiters, units, line breaks.
use super::helpers::{is_continued_condition_builder, is_function_definition_line, is_keyword_builder};
use super::models::TokenType;
use super::CalcpadTokenizer;
use crate::linter::helpers::character_helpers::{is_identifier_char, is_unit_start};
use crate::text::{is_letter, is_white_space};

const UI_KEYWORD: &str = "#ui";

/// Directives whose remainder is a payload rather than an expression. `#UI` is absent: its
/// block is optional, so `parse_space` only arms it.
const PAYLOAD_KEYWORDS: [(&str, TokenType); 5] = [
    ("#include", TokenType::Include),
    ("#format", TokenType::Format),
    ("#settings", TokenType::SettingsJson),
    ("#projectpath", TokenType::FilePath),
    ("#librarypath", TokenType::FilePath),
];

fn get_payload_type(builder: &[char]) -> TokenType {
    for (keyword, payload) in PAYLOAD_KEYWORDS {
        if is_keyword_builder(builder, keyword) {
            return payload;
        }
    }
    TokenType::None
}

impl CalcpadTokenizer {
    pub(super) fn parse_space(&mut self, c: char, position: i32) {
        if self.state.is_leading {
            self.builder.push(c);
        } else if is_continued_condition_builder(&self.builder) {
            self.builder.push(' ');
        } else {
            // Flush any pending content first
            if !self.builder.is_empty() {
                if is_keyword_builder(&self.builder, "#def") {
                    self.state.is_macro = true;
                }

                let is_keyword = self.state.current_type == TokenType::Keyword;
                let payload = if is_keyword {
                    get_payload_type(&self.builder)
                } else {
                    TokenType::None
                };
                let is_ui = is_keyword && is_keyword_builder(&self.builder, UI_KEYWORD);

                // append clears the builder, so both checks above must precede it
                self.append(self.state.current_type);

                // A payload or file path starts AFTER this space, so anchor it there
                if self.state.expecting_file_path && self.state.is_data_exchange_keyword {
                    self.state.current_type = TokenType::FilePath;
                    self.state.token_start_column = position + 1;
                } else if payload != TokenType::None {
                    self.state.current_type = payload;
                    self.state.token_start_column = position + 1;
                } else {
                    // #UI's block is optional, so the main loop opens it only if '{' follows
                    self.state.expecting_ui_json = is_ui;
                    self.state.current_type = TokenType::None;
                }
            }

            // Emit whitespace as None type to preserve column positions, but preserve the
            // FilePath, Include, Format and SettingsJson modes
            let in_file_path = self.state.current_type == TokenType::FilePath;
            let in_include_mode = self.state.current_type == TokenType::Include;
            let in_format_mode = self.state.current_type == TokenType::Format;
            let in_settings_mode = self.state.current_type == TokenType::SettingsJson;
            if in_file_path || in_include_mode || in_settings_mode {
                // Spaces within a file path or include are part of the token; a space as the
                // FIRST char (empty builder) is the leading space after the keyword and is skipped
                if !self.builder.is_empty() {
                    self.builder.push(c);
                }
            } else if in_format_mode && self.builder.is_empty() {
                // Leading space after the #format keyword or between format content and a comment
                self.state.token_start_column = position + 1;
            } else {
                self.state.token_start_column = position;
                self.builder.push(c);
                self.append(TokenType::None);
                self.state.current_type = TokenType::None;
            }
        }
    }

    pub(super) fn parse_brackets(&mut self, c: char, position: i32) {
        let t = self.state.current_type_or_previous();
        if c == '(' {
            self.state.bracket_count += 1;
            let call_bit = 1u32.wrapping_shl(self.state.bracket_count as u32);
            self.state.function_call_mask &= !call_bit;
            if t == TokenType::Variable {
                self.state.current_type = TokenType::Function;
                // Look ahead for "(...) =" to tell a definition from a call. Only definition
                // parameters are marked LocalVariable.
                let text = self.state.text.clone();
                self.state.is_function_definition = is_function_definition_line(&text, self.state.token_start_column);
                self.state.is_in_function_params = self.state.is_function_definition;
            } else if t == TokenType::Function {
                // A call to a known function, not a definition
                self.state.is_function_definition = false;
                self.state.is_in_function_params = false;
            } else if t == TokenType::Macro && self.state.macro_args == 0 {
                self.state.current_type = TokenType::Macro;
                if self.state.has_macro {
                    // Macro definition: #def macro$(param1; param2)
                    self.state.is_in_function_params = true;
                } else {
                    // Macro call: macro$(expr1; expr2)
                    self.state.macro_args = self.state.bracket_count;

                    // The builder was already cleared by parse_macro: use the captured name
                    let macro_name = self.last_macro_call_name.take();

                    if let Some(macro_name) = macro_name {
                        self.state.current_macro_call = Some(macro_name.clone());
                        self.state.current_macro_arg_index = 0;

                        // token_start_column is at the '(' position
                        let col = self.state.token_start_column;
                        self.resolve_macro_call_arg_tokens(&macro_name, col);
                    }
                }
            }

            // Only functions with defaults take keyword arguments
            if !self.state.is_function_definition
                && (t == TokenType::Variable || t == TokenType::Function)
                && !self.builder.is_empty()
            {
                let name: String = self.builder.iter().collect();
                if self.functions_with_defaults.contains(&name) {
                    self.state.function_call_mask |= call_bit;
                }
            }
        } else if c == ')' {
            if self.state.bracket_count == self.state.macro_args {
                self.state.macro_args = 0;
            }
            self.state.bracket_count -= 1;
        }

        // Append BEFORE clearing is_in_function_params so the last param gets marked LocalVariable
        self.append(self.state.current_type);

        // End of function params at the matching close paren (after append)
        if c == ')' && self.state.is_in_function_params && self.state.bracket_count == 0 {
            self.state.is_in_function_params = false;
        }

        if c == '{' {
            if self.state.current_type == TokenType::Command {
                self.state.command_count += 1;
                self.state.is_in_command_block = true;
                if position >= 0 {
                    let text = self.state.text.clone();
                    self.collect_command_scope_variables(&text, position as usize);
                }
            }
            self.state.current_type = TokenType::Bracket;
        } else if c == '}' {
            self.state.command_count -= 1;
            if self.state.command_count <= 0 {
                self.state.is_in_command_block = false;
                self.state.command_count = 0;
            }
            self.state.current_type = TokenType::Bracket;
        } else {
            if c == '[' {
                self.state.matrix_count += 1;
            } else if c == ']' {
                self.state.matrix_count -= 1;
            }
            self.state.current_type = TokenType::Bracket;
        }

        self.builder.push(c);
        self.append(self.state.current_type);
    }

    /// Registers the scope variables a command block declares after its body, e.g. the s in
    /// `$Integral{cos(s^2) @ s = 0 : t}`.
    fn collect_command_scope_variables(&mut self, text: &[char], brace_position: usize) {
        let mut depth = 0;
        let mut i = brace_position;
        while i < text.len() {
            let c = text[i];
            if c == '\'' || c == '"' {
                return;
            }
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    return;
                }
            } else if c == '@' || c == '&' {
                let mut j = i + 1;
                while j < text.len() && is_white_space(text[j]) {
                    j += 1;
                }
                let start = j;
                while j < text.len() && is_identifier_char(text[j]) {
                    j += 1;
                }
                if j > start {
                    self.local_variables.insert(&text[start..j].iter().collect::<String>());
                }
                i = j - 1;
            }
            i += 1;
        }
    }

    pub(super) fn parse_operator(&mut self, c: char) {
        self.append(self.state.current_type_or_previous());
        self.state.token_start_column += self.builder.len() as i32;
        self.builder.push(c);
        let is_percent = c == '%' && self.state.is_units;
        self.append(if is_percent {
            TokenType::Units
        } else {
            TokenType::Operator
        });
        self.state.current_type = TokenType::Operator;

        if c == '=' {
            // Inside parens, = is a default value separator: f(x = 5) = x^2. Stop marking tokens
            // as params but keep is_function_definition so the definition is recognised.
            if self.state.bracket_count == 0 {
                self.state.is_function = true;
                self.state.is_function_definition = false;
            }
            self.state.is_in_function_params = false;

            // Inline macro definitions (#def macro$(params) = body) enter the macro body
            if self.state.has_macro && self.state.bracket_count == 0 {
                self.in_macro_definition = true;
            }
        }
    }

    pub(super) fn parse_delimiter(&mut self, c: char) {
        self.append(self.state.current_type_or_previous());
        self.builder.push(c);

        // @ or & in a command block introduces a scoped variable, even inside a nested command
        if (c == '@' || c == '&') && self.state.is_in_command_block {
            self.state.is_after_at_or_amp = true;
        }

        if self.state.command_count > 0 || c == ';' {
            self.append(TokenType::Operator);
            // Next parameter after a default value
            if c == ';'
                && self.state.bracket_count > 0
                && (self.state.is_function_definition || (self.state.has_macro && !self.in_macro_definition))
            {
                self.state.is_in_function_params = true;
            }
        } else if c == '|' && (self.state.is_units || self.state.matrix_count > 0) {
            self.append(TokenType::Bracket);
        } else if c == ':' {
            if self.state.keyword.is_empty() {
                // Emit ':' as Operator, then enter Format mode for the specifier content
                self.append(TokenType::Operator);
                self.state.current_type = TokenType::Format;
                return;
            }
            self.append(TokenType::Operator);
        } else {
            self.append(TokenType::Operator);
        }

        self.state.current_type = TokenType::Bracket;
    }

    pub(super) fn parse_imaginary(&mut self, c: char, i: usize, len: usize, text: &[char]) {
        let j = i + 1;
        if j < len && text[j] == 'n' {
            self.append(TokenType::Const);
            self.builder.push(c);
            self.state.current_type = TokenType::Units;
        } else {
            self.builder.push('i');
            self.append(TokenType::Const);
        }
    }

    pub(super) fn parse_units(&mut self, c: char) {
        self.append(TokenType::Const);
        self.state.token_start_column += self.builder.len() as i32;
        self.builder.push(c);
        if is_letter(c) || is_unit_start(c) {
            self.state.current_type = TokenType::Units;
        }
    }

    pub(super) fn parse_line_break(&mut self, underscore_position: i32) {
        // Emit the current token if any
        if !self.builder.is_empty() {
            let last = self.builder[self.builder.len() - 1];
            // Remove the trailing space before the line continuation
            if last == ' ' {
                self.builder.pop();
            }

            if !self.builder.is_empty() {
                let mut append_type = self.state.current_type_or_previous();
                // Detect HTML comment markers before emitting, so a <!-- opener on a line ending
                // in " _" is typed as HtmlComment and in_html_comment carries into the next line
                if append_type == TokenType::Comment || self.in_html_comment {
                    self.state.current_type = append_type;
                    self.check_html_comment();
                    append_type = self.state.current_type;
                }
                self.append(append_type);
            }
        }

        // Emit the continuation token spanning the space and the underscore, anchored on the
        // underscore (the space may already have been emitted as a None token)
        self.state.token_start_column = underscore_position - 1;
        self.builder.push(' ');
        self.builder.push('_');
        self.append(TokenType::LineContinuation);

        // Preserve comment state across the continuation
        self.continue_text_comment = self.state.text_comment;
        self.continue_tag_comment = self.state.tag_comment;
        self.explicit_line_break = true;
    }

    pub(super) fn is_double_op(&self, c: char, op: char) -> bool {
        c == op && !self.builder.is_empty() && self.builder[self.builder.len() - 1] == op
    }

    pub(super) fn append_double_operator_shortcut(&mut self, op: char) {
        if self.builder.len() > 1 {
            self.builder.pop();
            self.append(self.state.current_type);
            self.builder.push(op);
        } else if self.builder.len() == 1 {
            self.builder[0] = op;
        } else {
            self.builder.push(op);
        }

        self.state.current_type = TokenType::Operator;
        self.append(self.state.current_type);
    }
}
