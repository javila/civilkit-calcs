//! The format-neutral intermediate form between a reader (a "front door") and the translator.
//!
//! A reader turns one file format into a [`Sheet`]; the translator turns a [`Sheet`] into
//! Calcpad text. Nothing in this module knows about XML. A reader for another container of the
//! same worksheet language (for instance a zipped one) only has to produce a [`Sheet`].

use crate::report::Code;

/// A whole worksheet: the array origin of its settings and its regions in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    /// The array origin setting (0 or 1 in practice). Indexes in the sheet are relative to it.
    pub origin: i64,
    /// Regions, nested ones (math inside a text region) already flattened out. Not yet in
    /// reading order: the translator sorts them.
    pub regions: Vec<Region>,
}

/// A picture region: a vector metafile (WMF or EMF bytes) and/or a PNG.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Picture {
    pub metafile: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
}

/// One region of the page. Position is kept only so that the translator can order the regions.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub id: String,
    pub left: f64,
    pub top: f64,
    /// The baseline the region sits on, when the file gives one; used to group regions into rows.
    pub baseline: Option<f64>,
    /// 1-based line in the source file (0 when the reader has no line information).
    pub line: u32,
    /// Size on the page in points (0 when the file gives none).
    pub width: f64,
    pub height: f64,
    /// The tool's own saved picture of the region (PNG bytes), when the file carries one.
    pub rendering: Option<Vec<u8>>,
    pub kind: RegionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegionKind {
    Text(Vec<Paragraph>),
    Math(MathRegion),
    Plot,
    Picture(Picture),
    /// An embedded object of another application (an OLE object).
    Ole,
    /// A scripted or data component (data table, spreadsheet, ...).
    Component,
    /// A reference to another worksheet.
    Reference,
    /// Layout only, carries no content.
    PageBreak,
    /// Anything else, with the element name the reader saw.
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// The paragraph style name as the file spells it ("Heading 1", "Normal", ...).
    pub style: String,
    pub pieces: Vec<Piece>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    Run(Run),
    Break,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub sub: bool,
    pub sup: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathRegion {
    pub stmt: Stmt,
    /// The region is switched off in the source (it is not evaluated there either).
    pub disabled: bool,
    /// The last calculated result the file stored for the region, when it is a single value.
    pub saved_result: Option<SavedResult>,
}

/// A stored scalar result: the number (or string) as written, and its unit in the source's spelling.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedResult {
    pub value: String,
    pub unit: Option<String>,
}

/// What a math region does.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `target := value` (or `target == value` for a global definition).
    Define { target: Target, value: Expr, global: bool },
    /// `expr =`, with an optional unit the result is shown in.
    Eval { expr: Expr, unit: Option<Expr> },
    /// An expression with no definition or evaluation operator.
    Bare(Expr),
    /// A construct the reader recognises by name but does not read further.
    Unsupported { code: Code, detail: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Var(Name),
    Func(Name, Vec<Name>),
    /// `v[i] := ...`, with the array's name when the target is a plain indexed name.
    Element(Option<Name>),
    /// `[a, b] := ...`: several names defined at once by a matrix of names.
    Multi(Vec<Name>),
}

/// An identifier with its literal subscript (`x_i` is base `x`, sub `i`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub base: String,
    pub sub: Option<String>,
}

impl Name {
    pub fn new(base: &str) -> Name {
        Name {
            base: base.to_string(),
            sub: None,
        }
    }
    /// The Calcpad spelling: `base_sub`.
    pub fn flat(&self) -> String {
        match &self.sub {
            Some(s) if !s.is_empty() => format!("{}_{}", self.base, s),
            _ => self.base.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    Abs,
    Sqrt,
    Factorial,
    Transpose,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal, kept as the file spells it.
    Num(String),
    Str(String),
    Name(Name),
    Un(UnOp, Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// A function call.
    Call(Name, Vec<Expr>),
    /// A unit written after its value (`30 degF`).
    Postfix(Name, Box<Expr>),
    /// `base[i, j]` (array indexing, not a literal subscript).
    Index(Box<Expr>, Vec<Expr>),
    /// Column-major, as the file stores it.
    Matrix {
        rows: usize,
        cols: usize,
        items: Vec<Expr>,
    },
    /// `first, second .. last`.
    Range {
        first: Box<Expr>,
        second: Option<Box<Expr>>,
        last: Box<Expr>,
    },
    /// A construct the reader recognises by name but does not read further.
    Unsupported {
        code: Code,
        detail: Option<String>,
    },
}
