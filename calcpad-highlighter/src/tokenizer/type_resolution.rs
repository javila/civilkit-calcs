//! CalcpadTokenizer.TypeResolution.cs: definition tracking, type resolution, lookups.
use super::helpers::is_macro_ident_char;
use super::models::{TokenType, TokenizerMode};
use super::CalcpadTokenizer;
use crate::linter::constants::builtins;
use crate::linter::helpers::character_helpers::{is_identifier_start_char, is_letter_for_tokenizer};
use crate::text::{eq_ignore_case, is_white_space};

fn trim_end_ws(s: &str) -> &str {
    s.trim_end_matches(is_white_space)
}

impl CalcpadTokenizer {
    pub(super) fn reset_type_resolution_state(&mut self) {
        self.defined_variables.clear();
        self.defined_functions.clear();
        self.defined_macros.clear();
        self.defined_units.clear();
        self.functions_with_defaults.clear();
        self.expecting_read_variable = false;
    }

    /// Resolve the token type from context. Does not detect errors; that is the linter's job.
    pub(super) fn resolve_type(&mut self, ty: TokenType, text: &str) -> TokenType {
        match ty {
            TokenType::Function => {
                // Unknown function calls become Variable so the linter can flag them
                if !self.is_known_function(text) && !self.state.is_function_definition {
                    return TokenType::Variable;
                }
            }
            TokenType::Variable => {
                if self.state.is_data_exchange_keyword {
                    if eq_ignore_case(text, "to") || eq_ignore_case(text, "from") {
                        self.state.expecting_file_path = true;
                        return TokenType::DataExchangeKeyword;
                    }
                    if eq_ignore_case(text, "sep") || eq_ignore_case(text, "type") {
                        return TokenType::DataExchangeKeyword;
                    }
                }
                // A macro parameter inside #def macro$(...) parentheses
                if self.state.is_in_function_params && self.state.has_macro {
                    self.macro_parameters.insert(text);
                    return TokenType::MacroParameter;
                }
                // A known macro parameter in the macro body
                if self.in_macro_definition && self.macro_parameters.contains(text) {
                    return TokenType::MacroParameter;
                }
                // A keyword argument names a parameter of the called function
                if self.is_keyword_argument_name(text) {
                    return TokenType::LocalVariable;
                }
                // A local variable (function param, command scope var, etc.)
                if self.state.is_in_function_params || self.state.is_after_at_or_amp {
                    self.local_variables.insert(text);
                    self.state.is_after_at_or_amp = false;
                    return TokenType::LocalVariable;
                }
                if self.local_variables.contains(text) {
                    return TokenType::LocalVariable;
                }
                if self.is_known_variable(text) {
                    return TokenType::Variable;
                }
                // Fall back to a unit when valid ("ft" in "x = 5*ft"), except at the first
                // identifier position on a line, which is a variable definition
                if self.is_known_unit(text) && !self.before_first_code_token && !self.state.is_data_exchange_keyword {
                    return TokenType::Units;
                }
            }
            TokenType::Keyword => {
                let trimmed = trim_end_ws(text);
                if builtins::keywords().contains(trimmed) {
                    self.state.keyword = trimmed.to_string();
                    self.state.is_data_exchange_keyword = is_data_exchange_keyword(trimmed);
                } else if builtins::control_block_keywords().contains(trimmed)
                    || builtins::end_keywords().contains(trimmed)
                {
                    // So that ':' in "#for i = 1 : 5" is an Operator, not a format delimiter
                    self.state.keyword = trimmed.to_string();
                }
                if eq_ignore_case(trimmed, "#end def") {
                    self.in_macro_definition = false;
                    self.macro_parameters.clear();
                }
            }
            TokenType::Command | TokenType::Macro => {}
            TokenType::Units => {
                if self.local_variables.contains(text) {
                    return TokenType::LocalVariable;
                }
                if self.is_known_variable(text) {
                    return TokenType::Variable;
                }
            }
            _ => {}
        }
        ty
    }

    pub(super) fn init_type(&self, c: char, current: TokenType) -> TokenType {
        if c == '$' {
            return TokenType::Command;
        }
        if c == '#' {
            return if self.state.is_leading {
                TokenType::Keyword
            } else {
                TokenType::Bracket
            };
        }
        if c.is_ascii_digit() {
            return TokenType::Const;
        }
        if self.state.is_macro && is_macro_ident_char(c, 0) {
            return TokenType::Macro;
        }
        if is_letter_for_tokenizer(c) {
            if is_identifier_start_char(c) {
                if self.state.is_units || self.state.previous_type == TokenType::Const {
                    return TokenType::Units;
                }
                return TokenType::Variable;
            }
            // Valid inside an identifier but cannot start one (_, digits, subscripts)
            return TokenType::Variable;
        }
        if c == '?' {
            return TokenType::Input;
        }
        current
    }

    /// Track variable and function definitions during tokenization.
    pub(super) fn track_definitions(&mut self, ty: TokenType, text: &str) {
        // Lint mode: pre-process all tokens except "=" (which needs is_first_def)
        if self.mode == TokenizerMode::Lint && !(ty == TokenType::Operator && text == "=") {
            self.track_definitions_lint(ty, text);
        }
        if self.mode == TokenizerMode::Macro {
            self.track_definitions_macro(ty, text);
        }

        match ty {
            TokenType::Variable | TokenType::Function if self.is_in_default_value() => {}
            TokenType::Variable => {
                self.pending_variable_name = Some(text.to_string());
                self.pending_variable_line = self.state.line;
                self.pending_function_name = None;
                self.pending_function_paren_depth = 0;
                self.pending_has_default = false;

                // #read variables are defined for the lines that follow
                if self.expecting_read_variable {
                    if !self.defined_variables.contains(text) {
                        self.defined_variables.insert(text);
                        self.result.add_variable_definition(text, self.state.line);
                    }
                    self.expecting_read_variable = false;
                }
            }
            TokenType::Function => {
                if self.state.is_function_definition {
                    self.pending_function_name = Some(text.to_string());
                    self.pending_function_line = self.state.line;
                    self.pending_has_default = false;
                    self.pending_variable_name = None;
                    self.pending_variable_line = -1;
                } else if self.pending_variable_name.is_some() {
                    self.pending_function_name = self.pending_variable_name.take();
                    self.pending_function_line = self.pending_variable_line;
                    self.pending_has_default = false;
                    self.pending_variable_line = -1;
                }
            }
            TokenType::Bracket => {
                if text == "(" {
                    if self.pending_variable_name.is_some() {
                        self.pending_function_name = self.pending_variable_name.take();
                        self.pending_function_line = self.pending_variable_line;
                        self.pending_function_paren_depth = 1;
                        self.pending_has_default = false;
                        self.pending_variable_line = -1;
                    } else if self.pending_function_name.is_some() {
                        self.pending_function_paren_depth += 1;
                    }
                } else if text == ")" {
                    if self.pending_function_name.is_some() && self.pending_function_paren_depth > 0 {
                        self.pending_function_paren_depth -= 1;
                    }
                } else if self.pending_function_name.is_none() || self.pending_function_paren_depth == 0 {
                    self.clear_pending_definitions();
                }
            }
            TokenType::Keyword => {
                if eq_ignore_case(trim_end_ws(text), "#read") {
                    self.expecting_read_variable = true;
                }
                self.clear_pending_definitions();
            }
            TokenType::Operator => {
                if text == "=" {
                    // Inside function definition parens, = is a default value separator
                    if self.pending_function_name.is_some() && self.pending_function_paren_depth > 0 {
                        if self.state.is_function_definition {
                            if let Some(n) = self.pending_function_name.clone() {
                                self.functions_with_defaults.insert(&n);
                            }
                            self.pending_has_default = true;
                        }
                        return;
                    }

                    let mut is_first_def = false;
                    if let (Some(name), 0) = (self.pending_function_name.clone(), self.pending_function_paren_depth) {
                        if self.defined_functions.contains(&name) {
                            self.result
                                .variable_reassignments
                                .push((name, self.pending_function_line, 0));
                        } else {
                            self.defined_functions.insert(&name);
                            self.result.add_function_definition(&name, self.pending_function_line);
                            is_first_def = true;
                        }
                    } else if let Some(name) = self.pending_variable_name.clone() {
                        if self.defined_variables.contains(&name) {
                            self.result
                                .variable_reassignments
                                .push((name, self.pending_variable_line, 0));
                        } else {
                            self.defined_variables.insert(&name);
                            self.result.add_variable_definition(&name, self.pending_variable_line);
                            is_first_def = true;
                        }
                    }

                    if self.mode == TokenizerMode::Lint {
                        self.on_equals_seen_lint(is_first_def);
                    }
                    self.clear_pending_definitions();
                } else if text == "←" {
                    if let (Some(name), 0) = (self.pending_function_name.clone(), self.pending_function_paren_depth) {
                        self.result
                            .outer_scope_assignments
                            .push((name, self.pending_function_line, 0));
                    } else if let Some(name) = self.pending_variable_name.clone() {
                        self.result
                            .outer_scope_assignments
                            .push((name, self.pending_variable_line, 0));
                    }
                    self.clear_pending_definitions();
                } else if text != ";" && text != "," {
                    if self.pending_function_name.is_none() || self.pending_function_paren_depth == 0 {
                        self.pending_variable_name = None;
                        self.pending_variable_line = -1;
                        if self.pending_function_paren_depth == 0 {
                            self.pending_function_name = None;
                            self.pending_function_line = -1;
                        }
                    }
                }
            }
            TokenType::LocalVariable
            | TokenType::Const
            | TokenType::Units
            | TokenType::Comment
            | TokenType::Tag
            | TokenType::HtmlComment => {}
            _ => self.clear_pending_definitions(),
        }
    }

    fn is_in_default_value(&self) -> bool {
        self.pending_has_default && self.pending_function_name.is_some() && self.pending_function_paren_depth > 0
    }

    pub(super) fn clear_pending_definitions(&mut self) {
        self.pending_variable_name = None;
        self.pending_variable_line = -1;
        self.pending_function_name = None;
        self.pending_function_line = -1;
        self.pending_function_paren_depth = 0;
        self.pending_has_default = false;
    }

    /// True for an identifier followed by '=' inside a call that takes keyword arguments.
    fn is_keyword_argument_name(&self, text: &str) -> bool {
        if (self
            .state
            .function_call_mask
            .wrapping_shr(self.state.bracket_count as u32)
            & 1)
            == 0
        {
            return false;
        }
        let span = &self.state.text;
        let mut i = (self.state.token_start_column.max(0) as usize) + text.chars().count();
        while i < span.len() && is_white_space(span[i]) {
            i += 1;
        }
        i < span.len() && span[i] == '=' && (i + 1 >= span.len() || span[i + 1] != '=')
    }

    pub(super) fn is_known_function(&self, name: &str) -> bool {
        builtins::functions().contains(name) || self.defined_functions.contains(name)
    }

    pub(super) fn is_known_variable(&self, name: &str) -> bool {
        self.local_variables.contains(name)
            || self.defined_variables.contains(name)
            || builtins::common_constants().contains(name)
    }

    pub(super) fn is_known_unit(&self, name: &str) -> bool {
        self.defined_units.contains(name) || builtins::units().contains(name)
    }

    #[allow(dead_code)]
    pub(super) fn is_known_macro(&self, name: &str) -> bool {
        self.defined_macros.contains(name)
    }
}

pub(super) fn is_data_exchange_keyword(keyword: &str) -> bool {
    eq_ignore_case(keyword, "#read") || eq_ignore_case(keyword, "#write") || eq_ignore_case(keyword, "#append")
}
