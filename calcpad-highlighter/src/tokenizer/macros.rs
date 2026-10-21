//! CalcpadTokenizer.Macros.cs: macro state, setup, definitions, parameters, arguments.
use super::helpers::is_macro_ident_char;
use super::models::{Token, TokenType};
use super::CalcpadTokenizer;
use crate::linter::constants::builtins;
use crate::linter::helpers::parsing_helpers::{find_matching_close, parse_param_list, skip_whitespace};
use crate::text::{eq_ignore_case, index_of, is_letter, is_white_space, to_string, trim_start, StrMap, StrSet};
use std::cell::RefCell;
use std::rc::Rc;

pub type ParamDefaults = Option<Vec<Option<String>>>;

impl CalcpadTokenizer {
    /// Shares the "currently-resolving macros" set with a nested tokenizer.
    pub(super) fn set_resolving_macros(&mut self, resolving: Option<Rc<RefCell<StrSet>>>) {
        self.resolving_macros = resolving;
    }

    /// Sets macro comment parameter information from ContentResolver Stage 2, enabling the
    /// tokenization of macro call arguments as Comment vs expression.
    pub fn set_macro_comment_parameters(
        &mut self,
        comment_params: Option<&StrMap<StrSet>>,
        param_order: Option<&StrMap<Vec<String>>>,
        macro_bodies: Option<&StrMap<String>>,
    ) {
        self.macro_comment_parameters.clear();
        self.macro_parameter_order.clear();
        self.macro_bodies.clear();

        if let Some(cp) = comment_params {
            for (k, v) in cp.iter() {
                let mut set = StrSet::new();
                for x in v.iter() {
                    set.insert(x);
                }
                self.macro_comment_parameters.insert(k, set);
            }
        }
        if let Some(po) = param_order {
            for (k, v) in po.iter() {
                self.macro_parameter_order.insert(k, v.clone());
                self.defined_macros.insert(k);
            }
        }
        if let Some(mb) = macro_bodies {
            for (k, v) in mb.iter() {
                self.macro_bodies.insert(k, v.clone());
            }
        }
        self.has_external_macro_comment_info = true;
    }

    /// Resets all macro state for a new tokenization pass (after `reset_type_resolution_state`).
    pub(super) fn reset_macro_state(&mut self) {
        self.in_macro_definition = false;
        self.macro_parameters.clear();
        if !self.has_external_macro_comment_info {
            self.macro_comment_parameters.clear();
            self.macro_parameter_order.clear();
            self.macro_bodies.clear();
        } else {
            let keys: Vec<String> = self.macro_parameter_order.keys().cloned().collect();
            for k in keys {
                self.defined_macros.insert(&k);
            }
        }
        self.macro_call_pre_tokenized = None;
        self.macro_arg_pre_emitted = false;
        self.pending_macro_def_name = None;
        self.last_macro_call_name = None;
    }

    /// Appends comment text while extracting macro parameters as separate tokens.
    pub(super) fn append_comment_with_macro_params(&mut self, comment_type: TokenType, text: &str) {
        if text.is_empty() {
            return;
        }
        let mut sorted: Vec<Vec<char>> = self.macro_parameters.iter().map(|p| p.chars().collect()).collect();
        // Longer parameters first (stable)
        sorted.sort_by_key(|p| std::cmp::Reverse(p.len()));

        if sorted.is_empty() {
            self.add_token(comment_type, text);
            return;
        }

        let t: Vec<char> = text.chars().collect();
        let mut segment_start = 0;
        let mut i = 0;
        while i < t.len() {
            let mut matched = false;
            for param in &sorted {
                if i + param.len() <= t.len() && t[i..i + param.len()] == param[..] {
                    if i > segment_start {
                        let s = to_string(&t[segment_start..i]);
                        self.add_token(comment_type, &s);
                    }
                    let p = to_string(param);
                    self.add_token(TokenType::MacroParameter, &p);
                    i += param.len();
                    segment_start = i;
                    matched = true;
                    break;
                }
            }
            if !matched {
                i += 1;
            }
        }
        if segment_start < t.len() {
            let s = to_string(&t[segment_start..]);
            self.add_token(comment_type, &s);
        }
    }

    pub(super) fn parse_macro(&mut self) {
        self.builder.push('$');
        if self.state.is_macro {
            self.state.current_type = TokenType::Macro;
            // Capture the macro definition name before append clears the builder
            let name = to_string(&self.builder);
            self.defined_macros.insert(&name);
            self.pending_macro_def_name = Some(name);
        } else if self.state.is_in_function_params && self.state.has_macro {
            // Inside macro definition parameters: identifiers (with or without $) are parameters
            let p = to_string(&self.builder);
            self.macro_parameters.insert(&p);
            self.state.current_type = TokenType::MacroParameter;
        } else if self.in_macro_definition && self.macro_parameters.contains(&to_string(&self.builder)) {
            self.state.current_type = TokenType::MacroParameter;
        } else if self.in_macro_definition && self.try_split_macro_parameter_suffix() {
            // prefix emitted; the builder holds the parameter suffix
        } else if self.try_split_defined_macro_suffix() {
            // prefix emitted; the builder holds the macro name
        } else if self.state.current_type == TokenType::Variable || self.state.current_type == TokenType::Units {
            let name = to_string(&self.builder);
            self.state.current_type = TokenType::Macro;
            // Capture the call-site macro name before append clears the builder
            self.last_macro_call_name = Some(name);
        }

        self.append(self.state.current_type);
        if self.state.is_macro {
            self.state.has_macro = true;
        }
        self.state.is_macro = false;
    }

    /// True if the next non-whitespace character after the builder is '('.
    #[allow(dead_code)]
    pub(super) fn is_followed_by_open_paren(&self) -> bool {
        let text = &self.state.text;
        let mut pos = (self.state.token_start_column.max(0) as usize) + self.builder.len();
        skip_whitespace(text, &mut pos);
        pos < text.len() && text[pos] == '('
    }

    /// When a defined macro matches a suffix of the builder (longest first), emits the prefix as
    /// the current type and leaves the macro name in the builder as Macro.
    fn try_split_defined_macro_suffix(&mut self) -> bool {
        if self.defined_macros.is_empty() {
            return false;
        }
        let text: Vec<char> = self.builder.clone();
        let mut best: Option<Vec<char>> = None;
        for m in self.defined_macros.iter() {
            let mc: Vec<char> = m.chars().collect();
            if mc.len() <= text.len() {
                let tail = to_string(&text[text.len() - mc.len()..]);
                if eq_ignore_case(&tail, m) && best.as_ref().is_none_or(|b| mc.len() > b.len()) {
                    best = Some(mc);
                }
            }
        }
        let Some(best) = best else { return false };
        let prefix_len = text.len() - best.len();
        if prefix_len > 0 {
            self.builder = text[..prefix_len].to_vec();
            self.append(self.state.current_type);
        }
        self.builder = best.clone();
        self.state.current_type = TokenType::Macro;
        self.last_macro_call_name = Some(to_string(&best));
        true
    }

    /// Inside a macro body, a macro parameter matching a suffix of the builder (longest first):
    /// emits the prefix as the current type and leaves the parameter as MacroParameter.
    fn try_split_macro_parameter_suffix(&mut self) -> bool {
        if self.macro_parameters.is_empty() {
            return false;
        }
        let text: Vec<char> = self.builder.clone();
        let mut best: Option<Vec<char>> = None;
        for p in self.macro_parameters.iter() {
            let pc: Vec<char> = p.chars().collect();
            if pc.len() < text.len()
                && text[text.len() - pc.len()..] == pc[..]
                && best.as_ref().is_none_or(|b| pc.len() > b.len())
            {
                best = Some(pc);
            }
        }
        let Some(best) = best else { return false };
        let prefix_len = text.len() - best.len();
        self.builder = text[..prefix_len].to_vec();
        self.append(self.state.current_type);
        self.builder.extend_from_slice(&best);
        self.state.current_type = TokenType::MacroParameter;
        true
    }

    pub(super) fn parse_macro_in_comment(&mut self, ty: TokenType) {
        let len = self.builder.len();
        if len < 2 {
            return;
        }
        // Leftmost extent of identifier characters before the trailing $
        let mut ident_start = len - 1;
        let mut i = len as isize - 2;
        while i >= 0 {
            if !is_macro_ident_char(self.builder[i as usize], i as usize) {
                break;
            }
            ident_start = i as usize;
            i -= 1;
        }

        let s: Vec<char> = self.builder.clone();
        for j in ident_start..len - 1 {
            let candidate = to_string(&s[j..]);
            if self.in_macro_definition && self.macro_parameters.contains(&candidate) {
                self.builder.truncate(j);
                self.append(ty);
                self.builder.extend(candidate.chars());
                self.append(TokenType::MacroParameter);
                self.state.current_type = ty;
                return;
            }
            if self.defined_macros.contains(&candidate) {
                self.builder.truncate(j);
                self.append(ty);
                self.builder.extend(candidate.chars());
                self.append(TokenType::Macro);
                self.state.current_type = ty;
                return;
            }
        }

        // Fallback: highlight a $-suffixed identifier as a potential macro
        for j in ident_start..len - 1 {
            if is_letter(s[j]) || s[j] == '_' {
                self.builder.truncate(j);
                self.append(ty);
                self.builder.extend(s[j..].iter());
                self.append(TokenType::Macro);
                self.state.current_type = ty;
                return;
            }
        }
    }

    pub(super) fn parse_macro_args(&mut self, c: char, position: i32) {
        if self.macro_call_pre_tokenized.is_some() {
            self.parse_macro_args_pre_tokenized(c, position);
            return;
        }

        let is_comment_arg = self.is_current_macro_arg_comment();

        if !is_comment_arg && self.state.text_comment == '\0' && (c == '\'' || c == '"') {
            self.append(self.state.current_type_or_previous());
            self.state.text_comment = c;
            self.builder.push(c);
            self.state.current_type = TokenType::Comment;
            self.state.previous_type = self.state.current_type;
            return;
        }

        if !is_comment_arg && self.state.text_comment != '\0' {
            if c == self.state.text_comment {
                self.builder.push(c);
                self.append(TokenType::Comment);
                self.state.text_comment = '\0';
            } else if c == '$' && !self.builder.is_empty() {
                self.builder.push('$');
                self.parse_macro_in_comment(TokenType::Comment);
            } else {
                self.builder.push(c);
                self.state.current_type = TokenType::Comment;
            }
            self.state.previous_type = self.state.current_type;
            return;
        }

        if c == '$' && !self.builder.is_empty() {
            self.builder.push('$');
            let t = if is_comment_arg {
                TokenType::Comment
            } else {
                self.state.current_type
            };
            self.parse_macro_in_comment(t);
        } else if c == '(' || c == ')' {
            if c == '(' {
                self.state.bracket_count += 1;
            } else {
                if self.state.bracket_count <= self.state.macro_args {
                    self.state.macro_args = 0;
                    self.state.current_macro_call = None;
                    self.state.current_macro_arg_index = 0;
                    self.state.text_comment = '\0';
                }
                self.state.bracket_count -= 1;
            }

            if is_comment_arg {
                self.append(TokenType::Comment);
            } else {
                self.append(self.state.current_type_or_previous());
            }

            self.builder.push(c);
            self.state.current_type = TokenType::Bracket;
            self.append(self.state.current_type);
            if self.state.text_comment != '\0' && c == ')' && self.state.bracket_count == 0 {
                self.state.current_type = TokenType::Comment;
            }
        } else if c == ';' {
            if is_comment_arg {
                self.append(TokenType::Comment);
            } else {
                self.append(self.state.current_type_or_previous());
            }
            self.builder.push(c);
            self.state.current_type = TokenType::Operator;
            self.append(self.state.current_type);
            self.state.current_macro_arg_index += 1;
        } else if is_comment_arg {
            self.builder.push(c);
            self.state.current_type = TokenType::Comment;
        } else if builtins::operators().contains(&c) {
            self.append(self.state.current_type_or_previous());
            self.builder.push(c);
            self.state.current_type = TokenType::Operator;
            self.append(self.state.current_type);
        } else if is_white_space(c) {
            self.append(self.state.current_type_or_previous());
            self.builder.push(c);
            self.append(TokenType::None);
            self.state.current_type = TokenType::None;
        } else if self.builder.is_empty() {
            self.state.current_type = self.init_type(c, self.state.current_type);
            self.builder.push(c);
        } else {
            self.builder.push(c);
        }

        self.state.previous_type = self.state.current_type;
    }

    fn is_current_macro_arg_comment(&self) -> bool {
        let Some(call) = self.state.current_macro_call.as_deref() else {
            return false;
        };
        if call.is_empty() {
            return false;
        }
        let Some(param_order) = self.macro_parameter_order.get(call) else {
            return false;
        };
        let idx = self.state.current_macro_arg_index;
        if idx < 0 || idx as usize >= param_order.len() {
            return false;
        }
        let param_name = &param_order[idx as usize];
        match self.macro_comment_parameters.get(call) {
            Some(cp) => cp.contains(param_name),
            None => false,
        }
    }

    /// Raw argument strings and start columns of a macro call, scanning from `start_pos` (just
    /// after the opening '(') to the matching ')'.
    fn extract_macro_call_args_with_positions(text: &[char], start_pos: usize) -> Vec<(String, usize)> {
        let mut args = Vec::new();
        let mut depth = 1;
        let mut arg_start = start_pos;
        let mut i = start_pos;
        while i < text.len() && depth > 0 {
            let c = text[i];
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    args.push((to_string(&text[arg_start..i]), arg_start));
                    break;
                }
            } else if c == ';' && depth == 1 {
                args.push((to_string(&text[arg_start..i]), arg_start));
                arg_start = i + 1;
            }
            i += 1;
        }
        args
    }

    /// Resolves macro call argument tokens by substituting each argument into the macro body,
    /// tokenizing the expanded body and extracting the tokens over the argument region.
    pub(super) fn resolve_macro_call_arg_tokens(&mut self, macro_name: &str, open_paren_col: i32) {
        self.macro_call_pre_tokenized = None;
        self.macro_arg_pre_emitted = false;

        let Some(body) = self.macro_bodies.get(macro_name).cloned() else {
            return;
        };
        let Some(param_order) = self.macro_parameter_order.get(macro_name).cloned() else {
            return;
        };
        if param_order.is_empty() {
            return;
        }
        // Cycle guard for macros whose body re-invokes itself
        if let Some(r) = &self.resolving_macros {
            if r.borrow().contains(macro_name) {
                return;
            }
        }

        let text = self.state.text.clone();
        let args = Self::extract_macro_call_args_with_positions(&text, (open_paren_col.max(0) as usize) + 1);
        if args.is_empty() {
            return;
        }

        let resolving = self
            .resolving_macros
            .get_or_insert_with(|| Rc::new(RefCell::new(StrSet::new_ci())))
            .clone();
        resolving.borrow_mut().insert(macro_name);

        let body_chars: Vec<char> = body.chars().collect();
        let mut pre: Vec<Vec<Token>> = Vec::new();

        for i in 0..args.len().min(param_order.len()) {
            let (arg_text, arg_start_col) = (&args[i].0, args[i].1 as i32);
            let param_name = &param_order[i];
            let pn: Vec<char> = param_name.chars().collect();

            // Find the parameter in the body, line by line for multi-line bodies
            let mut target_line: Option<Vec<char>> = None;
            let mut pos_in_line: i32 = -1;
            if body_chars.contains(&'\n') {
                let mut line_start = 0;
                while line_start < body_chars.len() {
                    let line_end =
                        crate::text::index_of_char(&body_chars, '\n', line_start).unwrap_or(body_chars.len());
                    let line = &body_chars[line_start..line_end];
                    if let Some(idx) = index_of(line, &pn, 0) {
                        target_line = Some(line.to_vec());
                        pos_in_line = idx as i32;
                        break;
                    }
                    line_start = line_end + 1;
                }
            } else if let Some(idx) = index_of(&body_chars, &pn, 0) {
                target_line = Some(body_chars.clone());
                pos_in_line = idx as i32;
            }

            let Some(target_line) = target_line else {
                pre.push(Vec::new());
                continue;
            };
            if pos_in_line < 0 {
                pre.push(Vec::new());
                continue;
            }
            let p = pos_in_line as usize;
            let arg_chars: Vec<char> = arg_text.chars().collect();
            let mut expanded: Vec<char> = target_line[..p].to_vec();
            expanded.extend_from_slice(&arg_chars);
            expanded.extend_from_slice(&target_line[p + pn.len()..]);

            // A fresh tokenizer with macro context resolves nested macro calls
            let mut tokenizer = CalcpadTokenizer::new();
            if self.has_external_macro_comment_info {
                tokenizer.set_macro_comment_parameters(
                    Some(&self.macro_comment_parameters),
                    Some(&self.macro_parameter_order),
                    Some(&self.macro_bodies),
                );
            }
            tokenizer.set_resolving_macros(Some(resolving.clone()));
            let result = tokenizer.tokenize_single_line(&to_string(&expanded), self.state.line);

            let arg_start_in_expanded = p as i32;
            let arg_end_in_expanded = p as i32 + arg_chars.len() as i32;
            let mut arg_tokens: Vec<Token> = Vec::new();

            for token in &result.tokens {
                let tok_start = token.column;
                let tok_end = token.column + token.length;
                if tok_end <= arg_start_in_expanded || tok_start >= arg_end_in_expanded {
                    continue;
                }
                let clip_start = tok_start.max(arg_start_in_expanded);
                let clip_end = tok_end.min(arg_end_in_expanded);
                let clip_len = clip_end - clip_start;
                if clip_len <= 0 {
                    continue;
                }
                let original_col = clip_start - arg_start_in_expanded + arg_start_col;
                let text_offset = (clip_start - tok_start) as usize;
                let clip_text = if text_offset == 0 && clip_len == token.length {
                    token.text.clone()
                } else {
                    token.text.chars().skip(text_offset).take(clip_len as usize).collect()
                };
                arg_tokens.push(Token::new(
                    self.state.line,
                    original_col,
                    clip_len,
                    token.token_type,
                    clip_text,
                ));
            }

            // A keyword argument names a parameter of the called macro
            if arg_tokens.len() > 1 && arg_tokens[1].text == "=" && param_order.contains(&arg_tokens[0].text) {
                let kw = arg_tokens[0].clone();
                arg_tokens[0] = Token::new(kw.line, kw.column, kw.length, TokenType::MacroParameter, kw.text);
            }

            // Inside a macro definition, tokens matching outer macro parameters become MacroParameter
            if self.in_macro_definition && !self.macro_parameters.is_empty() {
                for j in 0..arg_tokens.len() {
                    let t = arg_tokens[j].clone();
                    if t.token_type == TokenType::Macro && self.macro_parameters.contains(&t.text) {
                        arg_tokens[j] = Token::new(t.line, t.column, t.length, TokenType::MacroParameter, t.text);
                    }
                }
            }

            pre.push(arg_tokens);
        }
        self.macro_call_pre_tokenized = Some(pre);
        resolving.borrow_mut().remove(macro_name);
    }

    /// Emits the pre-computed tokens for each argument plus the call's own separators and
    /// closing paren; nested parens and semicolons are only counted.
    fn parse_macro_args_pre_tokenized(&mut self, c: char, position: i32) {
        if !self.macro_arg_pre_emitted {
            let idx = self.state.current_macro_arg_index;
            let tokens = self
                .macro_call_pre_tokenized
                .as_ref()
                .and_then(|p| if idx >= 0 { p.get(idx as usize) } else { None })
                .cloned();
            if let Some(tokens) = tokens {
                for t in tokens {
                    self.result.add_token(t);
                }
                self.macro_arg_pre_emitted = true;
            }
        }

        let at_call_depth = self.state.bracket_count <= self.state.macro_args;

        if c == '(' || c == ')' {
            if c == '(' {
                self.state.bracket_count += 1;
            } else {
                if at_call_depth {
                    self.state.macro_args = 0;
                    self.state.current_macro_call = None;
                    self.state.current_macro_arg_index = 0;
                    self.macro_call_pre_tokenized = None;
                }
                self.state.bracket_count -= 1;
            }

            if c == ')' && at_call_depth {
                self.builder.clear();
                self.builder.push(c);
                self.state.token_start_column = position;
                self.state.current_type = TokenType::Bracket;
                self.append(TokenType::Bracket);
            }

            if self.state.text_comment != '\0' && c == ')' && self.state.bracket_count == 0 {
                self.state.current_type = TokenType::Comment;
            }
        } else if c == ';' && at_call_depth {
            self.builder.clear();
            self.builder.push(c);
            self.state.token_start_column = position;
            self.state.current_type = TokenType::Operator;
            self.append(TokenType::Operator);
            self.state.current_macro_arg_index += 1;
            self.macro_arg_pre_emitted = false;
        }

        self.state.previous_type = self.state.current_type;
    }

    pub(super) fn parse_macro_content(&mut self, c: char, i: usize, len: usize) -> bool {
        // Inside the parameter list '=' introduces a default, not the macro body
        if c == '=' && self.state.has_macro && !self.state.is_in_function_params && self.state.bracket_count == 0 {
            let text = self.state.text.clone();
            let after_equals = if i + 1 < len {
                to_string(trim_start(&text[i + 1..]))
            } else {
                String::new()
            };

            if let Some(name) = self.pending_macro_def_name.take() {
                self.macro_bodies.insert(&name, after_equals.clone());
                if !self.macro_parameter_order.contains_key(&name) {
                    self.macro_parameter_order.insert(&name, extract_macro_params(&text));
                }
            }

            // In Macro mode, capture the inline content before tokenizing the rest
            if self.mode == super::models::TokenizerMode::Macro {
                if let Some(curr) = self.macro_curr_name.clone() {
                    self.macro_curr_inline_content = Some(after_equals);
                    self.macro_curr_is_inline = true;
                    let (names, defaults) = extract_macro_params_with_defaults(&text);
                    self.macro_curr_params = names.clone();
                    self.macro_curr_defaults = defaults;
                    if !self.macro_parameter_order.contains_key(&curr) {
                        self.macro_parameter_order.insert(&curr, names);
                    }
                }
            }
            return false;
        }
        false
    }

    /// Finds which parameters appear in comment sections of a line.
    pub fn find_comment_params_in_line(line: &str, parameters: &[String]) -> StrSet {
        let mut result = StrSet::new();
        if parameters.is_empty() {
            return result;
        }
        let l: Vec<char> = line.chars().collect();
        let mut in_comment = false;
        let mut comment_char = '\0';
        let mut comment_start = 0usize;

        let check = |section: &[char], result: &mut StrSet| {
            for p in parameters {
                let pc: Vec<char> = p.chars().collect();
                if index_of(section, &pc, 0).is_some() {
                    result.insert(p);
                }
            }
        };

        for i in 0..l.len() {
            let c = l[i];
            if !in_comment && (c == '\'' || c == '"') {
                in_comment = true;
                comment_char = c;
                comment_start = i;
            } else if in_comment && c == comment_char {
                check(&l[comment_start..=i], &mut result);
                in_comment = false;
                comment_char = '\0';
            }
        }
        if in_comment {
            check(&l[comment_start..], &mut result);
        }
        result
    }
}

pub(super) fn extract_macro_params(line: &[char]) -> Vec<String> {
    extract_macro_params_with_defaults(line).0
}

/// Macro parameter names and defaults from a `#def` line (inline or multiline form).
pub(super) fn extract_macro_params_with_defaults(line: &[char]) -> (Vec<String>, ParamDefaults) {
    let Some(dollar) = line.iter().position(|c| *c == '$') else {
        return (Vec::new(), None);
    };
    let after = trim_start(&line[dollar + 1..]);
    if after.is_empty() || after[0] != '(' {
        return (Vec::new(), None);
    }
    let open_paren = line.len() - after.len();
    let close = find_matching_close(line, open_paren, '(', ')');
    if close < 0 {
        return (Vec::new(), None);
    }
    parse_param_list(&to_string(&line[open_paren + 1..close as usize]))
}
