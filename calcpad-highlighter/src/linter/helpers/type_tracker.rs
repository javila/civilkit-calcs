//! TypeTracker.cs: tracks and infers types for Calcpad variables, functions, and macros.
use crate::linter::constants::function_signatures;
use crate::linter::helpers::character_helpers::{
    is_identifier_char, is_identifier_char_with_dollar, is_identifier_start_char_with_underscore,
};
use crate::linter::models::{CalcpadType, VariableInfo};
use crate::snippets::SnippetRegistry;
use crate::text::{is_digit, is_letter_or_digit, is_white_space, StrMap};
use crate::tokenizer::models::{Token, TokenType};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

fn vector_literal_pattern() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\s*\[[^|]+\]\s*$").expect("valid pattern"))
}

fn matrix_literal_pattern() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\s*\[.+\|.+\]\s*$").expect("valid pattern"))
}

#[derive(Clone, Debug)]
pub struct TypeTracker {
    variables: StrMap<VariableInfo>,
    functions: StrMap<VariableInfo>,
    macros: StrMap<VariableInfo>,
    custom_units: StrMap<VariableInfo>,
    tokens_by_line: Option<Arc<BTreeMap<i32, Vec<Token>>>>,
}

impl Default for TypeTracker {
    fn default() -> Self {
        TypeTracker {
            variables: StrMap::new(),
            functions: StrMap::new(),
            macros: StrMap::new_ci(),
            custom_units: StrMap::new_ci(),
            tokens_by_line: None,
        }
    }
}

fn is_blank(s: &str) -> bool {
    s.chars().all(is_white_space)
}

fn strip_dot(name: &str) -> &str {
    name.strip_suffix('.').unwrap_or(name)
}

fn trim_str(s: &str) -> &str {
    s.trim_matches(is_white_space)
}

impl TypeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Provides per-line tokens from the tokenizer so type inference can use real token boundaries.
    pub fn set_tokens_by_line(&mut self, tokens_by_line: &BTreeMap<i32, Vec<Token>>) {
        self.tokens_by_line = Some(Arc::new(tokens_by_line.clone()));
    }

    pub fn variables(&self) -> &StrMap<VariableInfo> {
        &self.variables
    }
    pub fn functions(&self) -> &StrMap<VariableInfo> {
        &self.functions
    }
    pub fn macros(&self) -> &StrMap<VariableInfo> {
        &self.macros
    }
    pub fn custom_units(&self) -> &StrMap<VariableInfo> {
        &self.custom_units
    }

    /// Registers a variable assignment and infers its type; a different later type makes it Various.
    pub fn register_variable(
        &mut self,
        name: &str,
        expression: &str,
        line_number: i32,
        column: i32,
        source: &str,
        is_const: bool,
    ) -> &mut VariableInfo {
        let new_type = self.infer_type_from_expression(expression, line_number);
        if self.variables.contains_key(name) {
            let existing = self.variables.get_mut(name).unwrap();
            if existing.type_ != new_type && existing.type_ != CalcpadType::Various {
                existing.type_ = CalcpadType::Various;
            }
            return existing;
        }
        let info = VariableInfo {
            name: name.to_string(),
            expression: expression.to_string(),
            line_number,
            column,
            source: source.to_string(),
            type_: new_type,
            is_const,
            ..Default::default()
        };
        self.variables.insert(name, info);
        self.variables.get_mut(name).unwrap()
    }

    pub fn register_function(
        &mut self,
        name: &str,
        parameters: &[String],
        expression: &str,
        line_number: i32,
        column: i32,
        source: &str,
        is_const: bool,
    ) -> &mut VariableInfo {
        let return_type = self.infer_type_from_expression(expression, -1);
        let info = VariableInfo {
            name: name.to_string(),
            parameters: parameters.to_vec(),
            expression: expression.to_string(),
            line_number,
            column,
            source: source.to_string(),
            type_: CalcpadType::Function,
            return_type,
            is_const,
            ..Default::default()
        };
        self.functions.insert(name, info);
        self.functions.get_mut(name).unwrap()
    }

    /// Registers a command block function; the return type comes from the last statement.
    pub fn register_command_block_function(
        &mut self,
        name: &str,
        parameters: &[String],
        statements: &[String],
        line_number: i32,
        column: i32,
        source: &str,
        is_const: bool,
    ) -> &mut VariableInfo {
        let mut return_type = CalcpadType::Unknown;
        if let Some(last) = statements.last() {
            return_type = self.infer_return_type_from_statement(last);
        }
        let info = VariableInfo {
            name: name.to_string(),
            parameters: parameters.to_vec(),
            expression: statements.join("; "),
            line_number,
            column,
            source: source.to_string(),
            type_: CalcpadType::Function,
            return_type,
            is_const,
            ..Default::default()
        };
        self.functions.insert(name, info);
        self.functions.get_mut(name).unwrap()
    }

    fn infer_return_type_from_statement(&self, statement: &str) -> CalcpadType {
        if is_blank(statement) {
            return CalcpadType::Unknown;
        }
        let trimmed: Vec<char> = trim_str(statement).chars().collect();
        let mut assignment_index: i32 = -1;
        let n = trimmed.len();
        let mut i = 0;
        while i + 1 < n {
            if trimmed[i] == '=' && trimmed[i + 1] != '=' {
                if i > 0 && (trimmed[i - 1] == '<' || trimmed[i - 1] == '>' || trimmed[i - 1] == '!') {
                    i += 1;
                    continue;
                }
                assignment_index = i as i32;
                break;
            }
            i += 1;
        }
        if assignment_index > 0 {
            let right: String = trimmed[assignment_index as usize + 1..].iter().collect();
            return self.infer_type_from_expression(trim_str(&right), -1);
        }
        let all: String = trimmed.iter().collect();
        self.infer_type_from_expression(&all, -1)
    }

    pub fn register_inline_macro(
        &mut self,
        name: &str,
        parameters: &[String],
        expression: &str,
        line_number: i32,
        column: i32,
        source: &str,
    ) -> &mut VariableInfo {
        let info = VariableInfo {
            name: name.to_string(),
            parameters: parameters.to_vec(),
            expression: expression.to_string(),
            line_number,
            column,
            source: source.to_string(),
            type_: CalcpadType::InlineMacro,
            ..Default::default()
        };
        self.macros.insert(name, info);
        self.macros.get_mut(name).unwrap()
    }

    pub fn register_multiline_macro(
        &mut self,
        name: &str,
        parameters: &[String],
        line_number: i32,
        column: i32,
        source: &str,
    ) -> &mut VariableInfo {
        let info = VariableInfo {
            name: name.to_string(),
            parameters: parameters.to_vec(),
            line_number,
            column,
            source: source.to_string(),
            type_: CalcpadType::MultilineMacro,
            ..Default::default()
        };
        self.macros.insert(name, info);
        self.macros.get_mut(name).unwrap()
    }

    /// Registers a variable from a #read statement: Vector for TYPE=V, Matrix otherwise.
    pub fn register_read_variable(
        &mut self,
        name: &str,
        is_vector: bool,
        line_number: i32,
        column: i32,
        source: &str,
    ) -> &mut VariableInfo {
        let info = VariableInfo {
            name: name.to_string(),
            expression: "#read".to_string(),
            line_number,
            column,
            source: source.to_string(),
            type_: if is_vector {
                CalcpadType::Vector
            } else {
                CalcpadType::Matrix
            },
            ..Default::default()
        };
        self.variables.insert(name, info);
        self.variables.get_mut(name).unwrap()
    }

    pub fn register_custom_unit(
        &mut self,
        unit_name: &str,
        expression: &str,
        line_number: i32,
        column: i32,
        source: &str,
    ) -> &mut VariableInfo {
        let info = VariableInfo {
            name: format!(".{unit_name}"),
            unit_name: unit_name.to_string(),
            expression: expression.to_string(),
            line_number,
            column,
            source: source.to_string(),
            type_: CalcpadType::CustomUnit,
            ..Default::default()
        };
        self.custom_units.insert(unit_name, info);
        self.custom_units.get_mut(unit_name).unwrap()
    }

    pub fn get_variable_type(&self, name: &str) -> CalcpadType {
        let n = strip_dot(name);
        if let Some(v) = self.variables.get(n) {
            return v.type_;
        }
        if self.functions.contains_key(n) {
            return CalcpadType::Function;
        }
        if let Some(m) = self.macros.get(n) {
            return m.type_;
        }
        if self.custom_units.contains_key(n) {
            return CalcpadType::CustomUnit;
        }
        CalcpadType::Unknown
    }

    pub fn get_function_return_type(&self, name: &str) -> CalcpadType {
        self.functions.get(name).map_or(CalcpadType::Unknown, |f| f.return_type)
    }

    pub fn get_variable_info(&self, name: &str) -> Option<&VariableInfo> {
        let n = strip_dot(name);
        self.variables
            .get(n)
            .or_else(|| self.functions.get(n))
            .or_else(|| self.macros.get(n))
            .or_else(|| self.custom_units.get(n))
    }

    pub fn is_defined(&self, name: &str) -> bool {
        let n = strip_dot(name);
        self.variables.contains_key(n)
            || self.functions.contains_key(n)
            || self.macros.contains_key(n)
            || self.custom_units.contains_key(n)
    }

    pub fn supports_element_access(&self, name: &str) -> bool {
        self.variables
            .get(strip_dot(name))
            .is_some_and(|i| i.supports_element_access())
    }

    /// Infers the type of a value from its expression (`lineNumber` -1 when unknown).
    pub fn infer_type_from_expression(&self, expression: &str, line_number: i32) -> CalcpadType {
        if is_blank(expression) {
            return CalcpadType::Unknown;
        }
        let trimmed = trim_str(expression);
        if matrix_literal_pattern().is_match(trimmed) {
            return CalcpadType::Matrix;
        }
        if vector_literal_pattern().is_match(trimmed) {
            return CalcpadType::Vector;
        }
        let chars: Vec<char> = trimmed.chars().collect();

        let t = self.infer_type_from_element_access(&chars);
        if t != CalcpadType::Unknown {
            return t;
        }
        let t = self.infer_type_from_function_call(&chars);
        if t != CalcpadType::Unknown {
            return t;
        }
        let t = self.infer_type_from_variable_reference(trimmed, &chars);
        if t != CalcpadType::Unknown {
            return t;
        }
        if chars.iter().any(|c| matches!(c, '+' | '-' | '*' | '/' | '^' | '(')) {
            let operand_type = match self.get_expression_tokens(line_number) {
                Some(tokens) => self.infer_type_from_operand_tokens(&tokens),
                None => self.infer_type_from_operand_types(&chars),
            };
            if operand_type != CalcpadType::Unknown {
                return operand_type;
            }
        }
        CalcpadType::Unknown
    }

    fn infer_type_from_function_call(&self, expression: &[char]) -> CalcpadType {
        let paren = match expression.iter().position(|c| *c == '(') {
            Some(p) if p > 0 => p,
            _ => return CalcpadType::Unknown,
        };
        let func_name = extract_last_identifier(&expression[..paren]);
        if func_name.is_empty() {
            return CalcpadType::Unknown;
        }
        if SnippetRegistry::get_vector_returning_functions().contains(&func_name) {
            return CalcpadType::Vector;
        }
        if SnippetRegistry::get_matrix_returning_functions().contains(&func_name) {
            return CalcpadType::Matrix;
        }
        if let Some(f) = self.functions.get(&func_name) {
            if f.return_type != CalcpadType::Unknown {
                return f.return_type;
            }
        }
        if let Some(sig) = function_signatures::get_signature(&func_name) {
            if sig.return_type != CalcpadType::Value {
                return sig.return_type;
            }
        }
        CalcpadType::Unknown
    }

    fn infer_type_from_variable_reference(&self, trimmed: &str, chars: &[char]) -> CalcpadType {
        if chars
            .iter()
            .any(|c| matches!(c, '+' | '-' | '*' | '/' | '^' | '(' | ')'))
        {
            return CalcpadType::Unknown;
        }
        self.variables.get(trimmed).map_or(CalcpadType::Unknown, |i| i.type_)
    }

    fn infer_type_from_element_access(&self, expression: &[char]) -> CalcpadType {
        let dot = match expression.iter().position(|c| *c == '.') {
            Some(d) if d > 0 && d < expression.len() - 1 => d,
            _ => return CalcpadType::Unknown,
        };
        let var_name: String = expression[..dot].iter().collect();
        let Some(info) = self.variables.get(&var_name) else {
            return CalcpadType::Unknown;
        };
        if !matches!(
            info.type_,
            CalcpadType::Vector | CalcpadType::Matrix | CalcpadType::Various
        ) {
            return CalcpadType::Unknown;
        }
        let after = expression[dot + 1];
        if is_letter_or_digit(after) || after == '(' {
            return CalcpadType::Value;
        }
        CalcpadType::Unknown
    }

    fn infer_type_from_operand_types(&self, expression: &[char]) -> CalcpadType {
        let mut atoms: Vec<CalcpadType> = Vec::new();
        let mut ops: Vec<char> = Vec::new();
        let mut pending = '\0';
        let mut i = 0;
        let len = expression.len();
        while i < len {
            let c = expression[i];
            if is_identifier_start_char_with_underscore(c) {
                let start = i;
                i += 1;
                while i < len && is_identifier_char(expression[i]) {
                    i += 1;
                }
                let name: String = expression[start..i].iter().collect();
                let atom;
                if i < len && expression[i] == '(' {
                    atom = self.get_function_call_return_type(&name);
                    i = skip_balanced_parentheses(expression, i);
                } else {
                    atom = self.variables.get(&name).map_or(CalcpadType::Unknown, |v| v.type_);
                }
                atoms.push(atom);
                ops.push(pending);
                pending = '\0';
            } else if is_digit(c) {
                i += 1;
                while i < len && (is_digit(expression[i]) || expression[i] == '.') {
                    i += 1;
                }
                atoms.push(CalcpadType::Value);
                ops.push(pending);
                pending = '\0';
            } else {
                if matches!(c, '+' | '-' | '*' | '/' | '^') && pending == '\0' {
                    pending = c;
                }
                i += 1;
            }
        }
        fold_operand_types(&atoms, &ops)
    }

    /// Tokens after the '=' operator of a definition line, or None.
    fn get_expression_tokens(&self, line_number: i32) -> Option<Vec<Token>> {
        if line_number < 0 {
            return None;
        }
        let line_tokens = self.tokens_by_line.as_ref()?.get(&line_number)?;
        for (i, t) in line_tokens.iter().enumerate() {
            if t.token_type == TokenType::Operator && t.text == "=" {
                return Some(line_tokens[i + 1..].to_vec());
            }
        }
        None
    }

    fn infer_type_from_operand_tokens(&self, tokens: &[Token]) -> CalcpadType {
        let mut atoms: Vec<CalcpadType> = Vec::new();
        let mut ops: Vec<char> = Vec::new();
        let mut pending = '\0';
        let mut depth = 0i32;
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];
            if token.token_type == TokenType::Bracket {
                match token.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => depth -= 1,
                    _ => {}
                }
                i += 1;
                continue;
            }
            if depth > 0 {
                i += 1;
                continue;
            }
            if token.token_type == TokenType::Operator {
                if token.text != "." && pending == '\0' {
                    // token.Text[0]: an empty operator text would throw upstream
                    pending = token.text.chars().next().unwrap_or('\0');
                }
                i += 1;
                continue;
            }
            let atom;
            if token.token_type == TokenType::Function {
                atom = self.get_function_call_return_type(&token.text);
            } else if token.token_type == TokenType::Variable || token.token_type == TokenType::LocalVariable {
                if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Operator && tokens[i + 1].text == "."
                {
                    i += 1;
                    if i + 1 < tokens.len() {
                        let idx = &tokens[i + 1];
                        if matches!(
                            idx.token_type,
                            TokenType::Variable | TokenType::LocalVariable | TokenType::Const
                        ) {
                            i += 1;
                        }
                    }
                    atom = CalcpadType::Value;
                } else {
                    atom = self
                        .variables
                        .get(&token.text)
                        .map_or(CalcpadType::Unknown, |v| v.type_);
                }
            } else if token.token_type == TokenType::Const {
                atom = CalcpadType::Value;
            } else {
                i += 1;
                continue;
            }
            atoms.push(atom);
            ops.push(pending);
            pending = '\0';
            i += 1;
        }
        fold_operand_types(&atoms, &ops)
    }

    fn get_function_call_return_type(&self, func_name: &str) -> CalcpadType {
        if let Some(sig) = function_signatures::get_signature(func_name) {
            return sig.return_type;
        }
        if let Some(f) = self.functions.get(func_name) {
            return f.return_type;
        }
        CalcpadType::Unknown
    }

    pub fn clear(&mut self) {
        self.variables.clear();
        self.functions.clear();
        self.macros.clear();
        self.custom_units.clear();
    }
}

fn fold_operand_types(atoms: &[CalcpadType], ops: &[char]) -> CalcpadType {
    if atoms.is_empty() {
        return CalcpadType::Unknown;
    }
    let mut terms = vec![atoms[0]];
    for i in 1..atoms.len() {
        if ops[i] == '*' {
            let last = terms.len() - 1;
            terms[last] = combine_multiply(terms[last], atoms[i]);
        } else {
            terms.push(atoms[i]);
        }
    }
    let mut result = CalcpadType::Unknown;
    for t in terms {
        if t == CalcpadType::Matrix {
            return CalcpadType::Matrix;
        }
        if t == CalcpadType::Vector {
            result = CalcpadType::Vector;
        }
    }
    result
}

fn combine_multiply(left: CalcpadType, right: CalcpadType) -> CalcpadType {
    if left == CalcpadType::Matrix && right == CalcpadType::Vector {
        return CalcpadType::Vector;
    }
    if left == CalcpadType::Matrix || right == CalcpadType::Matrix {
        return CalcpadType::Matrix;
    }
    if left == CalcpadType::Vector || right == CalcpadType::Vector {
        return CalcpadType::Vector;
    }
    if left != CalcpadType::Unknown {
        left
    } else {
        right
    }
}

fn skip_balanced_parentheses(expression: &[char], start: usize) -> usize {
    let mut depth = 0;
    let mut i = start;
    while i < expression.len() {
        if expression[i] == '(' {
            depth += 1;
        } else if expression[i] == ')' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
        i += 1;
    }
    i
}

fn extract_last_identifier(text: &[char]) -> String {
    let mut end = text.len() as i64 - 1;
    while end >= 0 && is_white_space(text[end as usize]) {
        end -= 1;
    }
    if end < 0 {
        return String::new();
    }
    let end = end as usize;
    let mut start = end;
    while start > 0 && is_identifier_char_with_dollar(text[start - 1]) {
        start -= 1;
    }
    let ident: String = text[start..=end].iter().collect();
    if is_identifier_start_char_with_underscore(text[start]) {
        return ident;
    }
    String::new()
}
