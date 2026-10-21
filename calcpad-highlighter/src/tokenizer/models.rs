//! Tokenizer/Models: Token, TokenType, TokenizerMode, TokenizerResult.
use crate::content_resolution::result::{
    CommandBlockInfo, CustomUnitDefinition, DuplicateMacro, FunctionDefinition, MacroDefinition, MacroInfo,
    VariableDefinition,
};
use crate::linter::models::ParseMode;
use crate::text::StrMap;
use std::collections::{BTreeMap, BTreeSet};

/// Token types for syntax highlighting. The discriminants are the wire values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum TokenType {
    None = 0,
    Const = 1,
    Operator = 2,
    Bracket = 3,
    LineContinuation = 4,
    Variable = 5,
    LocalVariable = 6,
    Function = 7,
    Macro = 8,
    MacroParameter = 9,
    Units = 10,
    Setting = 11,
    Keyword = 12,
    ControlBlockKeyword = 13,
    EndKeyword = 14,
    Command = 15,
    Include = 16,
    FilePath = 17,
    DataExchangeKeyword = 18,
    Comment = 19,
    HtmlComment = 20,
    Tag = 21,
    HtmlContent = 22,
    JavaScript = 23,
    Css = 24,
    Svg = 25,
    Input = 26,
    Format = 27,
    SettingsJson = 28,
    FutureReserved29 = 29,
    FutureReserved30 = 30,
}

impl TokenType {
    /// `Enum.GetNames(typeof(TokenType))`, in declaration order.
    pub const NAMES: [&'static str; 31] = [
        "None",
        "Const",
        "Operator",
        "Bracket",
        "LineContinuation",
        "Variable",
        "LocalVariable",
        "Function",
        "Macro",
        "MacroParameter",
        "Units",
        "Setting",
        "Keyword",
        "ControlBlockKeyword",
        "EndKeyword",
        "Command",
        "Include",
        "FilePath",
        "DataExchangeKeyword",
        "Comment",
        "HtmlComment",
        "Tag",
        "HtmlContent",
        "JavaScript",
        "Css",
        "Svg",
        "Input",
        "Format",
        "SettingsJson",
        "FutureReserved29",
        "FutureReserved30",
    ];

    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    pub fn from_i32(v: i32) -> Option<TokenType> {
        use TokenType::*;
        const ALL: [TokenType; 31] = [
            None,
            Const,
            Operator,
            Bracket,
            LineContinuation,
            Variable,
            LocalVariable,
            Function,
            Macro,
            MacroParameter,
            Units,
            Setting,
            Keyword,
            ControlBlockKeyword,
            EndKeyword,
            Command,
            Include,
            FilePath,
            DataExchangeKeyword,
            Comment,
            HtmlComment,
            Tag,
            HtmlContent,
            JavaScript,
            Css,
            Svg,
            Input,
            Format,
            SettingsJson,
            FutureReserved29,
            FutureReserved30,
        ];
        ALL.get(usize::try_from(v).ok()?).copied()
    }

    /// Case-insensitive `Enum.TryParse` by name.
    pub fn parse_name(s: &str) -> Option<TokenType> {
        Self::NAMES
            .iter()
            .position(|n| crate::text::eq_ignore_case(n, s))
            .and_then(|i| Self::from_i32(i as i32))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenizerMode {
    Highlight,
    Macro,
    Lint,
}

/// A token with position information. Line and column are zero based; the column counts
/// characters (see `crate::utf16` for the UTF-16 conversion used by the JSON layer).
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub line: i32,
    pub column: i32,
    pub length: i32,
    pub token_type: TokenType,
    pub text: String,
}

impl Token {
    pub fn new(line: i32, column: i32, length: i32, token_type: TokenType, text: impl Into<String>) -> Token {
        Token {
            line,
            column,
            length,
            token_type,
            text: text.into(),
        }
    }
    pub fn end_column(&self) -> i32 {
        self.column + self.length
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}:{}-{}] {:?}: \"{}\"",
            self.line,
            self.column,
            self.end_column(),
            self.token_type,
            self.text
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct TokenizerResult {
    pub tokens: Vec<Token>,
    pub tokens_by_line: BTreeMap<i32, Vec<Token>>,
    /// Variables defined in the source (name -> line number), first definition only.
    pub defined_variables: StrMap<i32>,
    /// Functions defined in the source (name -> line number), case sensitive.
    pub defined_functions: StrMap<i32>,
    pub variable_reassignments: Vec<(String, i32, i32)>,
    pub outer_scope_assignments: Vec<(String, i32, i32)>,
    /// Parse mode of every line inside an #html or #markdown block. Absent lines are Calcpad.
    pub line_modes: BTreeMap<i32, ParseMode>,
    pub raw_lines: BTreeSet<i32>,
    // Lint mode
    pub variable_definitions: Vec<VariableDefinition>,
    pub function_definitions: Vec<FunctionDefinition>,
    pub custom_unit_definitions: Vec<CustomUnitDefinition>,
    pub command_block_functions: StrMap<CommandBlockInfo>,
    // Macro mode
    pub macro_definitions: Vec<MacroDefinition>,
    pub duplicate_macros: Vec<DuplicateMacro>,
    /// Keyed by macro name, ignore case.
    pub user_defined_macros: StrMap<MacroInfo>,
}

impl TokenizerResult {
    pub fn new() -> Self {
        TokenizerResult {
            defined_variables: StrMap::new(),
            defined_functions: StrMap::new(),
            command_block_functions: StrMap::new(),
            user_defined_macros: StrMap::new_ci(),
            ..Default::default()
        }
    }

    pub fn get_tokens_for_line(&self, line: i32) -> Vec<Token> {
        self.tokens_by_line.get(&line).cloned().unwrap_or_default()
    }

    pub fn add_token(&mut self, token: Token) {
        self.tokens_by_line.entry(token.line).or_default().push(token.clone());
        self.tokens.push(token);
    }

    pub fn add_variable_definition(&mut self, name: &str, line: i32) {
        if !self.defined_variables.contains_key(name) {
            self.defined_variables.insert(name, line);
        }
    }

    pub fn add_function_definition(&mut self, name: &str, line: i32) {
        if !self.defined_functions.contains_key(name) {
            self.defined_functions.insert(name, line);
        }
    }
}
