//! Port of Parsers/MathParser/MathParser.Tokens.cs.
use std::cell::RefCell;
use std::rc::Rc;

use crate::base_types::value::Value;
use crate::base_types::variable::VarRef;
use crate::matrix::Matrix;
use crate::vector::Vector;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ValueTypes {
    None,
    Number,
    Unit,
    NumberWithUnit,
    Vector,
    Matrix,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TokenTypes {
    None,
    Constant,
    Variable,
    Unit,
    Input,
    Vector,
    Matrix,
    Array,
    RowDivisor,
    Operator,
    VectorIndex,
    MatrixIndex,
    ArrayIndex,
    Function,
    Function2,
    Function3,
    MultiFunction,
    Interpolation,
    VectorFunction,
    VectorFunction2,
    VectorFunction3,
    VectorMultiFunction,
    MatrixFunction,
    MatrixFunction2,
    MatrixFunction3,
    MatrixFunction4,
    MatrixFunction5,
    MatrixIterativeFunction,
    MatrixOptionalFunction,
    MatrixMultiFunction,
    CustomFunction,
    BracketLeft,
    BracketRight,
    SquareBracketLeft,
    SquareBracketRight,
    Divisor,
    Solver,
    Error,
}

impl TokenTypes {
    pub fn name(&self) -> String {
        format!("{:?}", self)
    }
}

pub const DEFAULT_ORDER: i8 = -1;

/// The C# Token class hierarchy: plain Token, FunctionToken, ValueToken, VariableToken,
/// VectorToken and MatrixToken. RenderToken lives in output.rs.
#[derive(Clone, Debug)]
pub enum TokenKind {
    Plain,
    Function {
        parameter_count: i32,
        argument_map: Option<Rc<Vec<usize>>>,
        positional_count: i32,
    },
    Value(Value),
    Variable(Option<VarRef>),
    /// `VectorToken.Vector` (set when the literal is evaluated)
    Vector(Option<Vector>),
    /// `MatrixToken.Matrix` (set when the literal is evaluated)
    Matrix(Option<Matrix>),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub content: String,
    pub index: i64,
    pub ttype: TokenTypes,
    pub order: i8,
    pub kind: TokenKind,
}

pub type TokenRef = Rc<RefCell<Token>>;
pub type Rpn = Rc<Vec<TokenRef>>;

impl Token {
    pub fn new(content: impl Into<String>, ttype: TokenTypes) -> Token {
        Token {
            content: content.into(),
            index: -1,
            ttype,
            order: DEFAULT_ORDER,
            kind: TokenKind::Plain,
        }
    }

    pub fn with_order(content: impl Into<String>, ttype: TokenTypes, order: i8) -> Token {
        let mut t = Token::new(content, ttype);
        t.order = order;
        t
    }

    /// `new FunctionToken(content)`: type MultiFunction by default.
    pub fn function(content: impl Into<String>) -> Token {
        let mut t = Token::new(content, TokenTypes::MultiFunction);
        t.kind = TokenKind::Function {
            parameter_count: 0,
            argument_map: None,
            positional_count: 0,
        };
        t
    }

    /// `new ValueToken(value)`: content empty, type Constant.
    pub fn value(value: Value) -> Token {
        let mut t = Token::new("", TokenTypes::Constant);
        t.kind = TokenKind::Value(value);
        t
    }

    /// `new VariableToken(content, v)`
    pub fn variable(content: impl Into<String>, v: Option<VarRef>) -> Token {
        let mut t = Token::new(content, TokenTypes::Variable);
        t.kind = TokenKind::Variable(v);
        t
    }

    pub fn into_ref(self) -> TokenRef {
        Rc::new(RefCell::new(self))
    }

    pub fn is_function_token(&self) -> bool {
        matches!(self.kind, TokenKind::Function { .. })
    }

    pub fn is_value_token(&self) -> bool {
        matches!(self.kind, TokenKind::Value(_))
    }

    pub fn is_variable_token(&self) -> bool {
        matches!(self.kind, TokenKind::Variable(_))
    }

    pub fn variable_ref(&self) -> Option<VarRef> {
        match &self.kind {
            TokenKind::Variable(v) => v.clone(),
            _ => None,
        }
    }

    pub fn set_variable(&mut self, v: Option<VarRef>) {
        if let TokenKind::Variable(slot) = &mut self.kind {
            *slot = v;
        }
    }

    pub fn token_value(&self) -> Option<&Value> {
        match &self.kind {
            TokenKind::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn set_value(&mut self, value: Value) {
        if let TokenKind::Value(v) = &mut self.kind {
            *v = value;
        }
    }

    pub fn parameter_count(&self) -> i32 {
        match &self.kind {
            TokenKind::Function { parameter_count, .. } => *parameter_count,
            _ => 0,
        }
    }

    pub fn set_parameter_count(&mut self, n: i32) {
        if let TokenKind::Function { parameter_count, .. } = &mut self.kind {
            *parameter_count = n;
        }
    }

    pub fn argument_map(&self) -> Option<Rc<Vec<usize>>> {
        match &self.kind {
            TokenKind::Function { argument_map, .. } => argument_map.clone(),
            _ => None,
        }
    }

    pub fn positional_count(&self) -> i32 {
        match &self.kind {
            TokenKind::Function { positional_count, .. } => *positional_count,
            _ => 0,
        }
    }

    pub fn first_char(&self) -> char {
        self.content.chars().next().unwrap_or('\0')
    }
}
