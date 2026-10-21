//! The per-category snippet tables (`Snippets/Data/*.cs`), loaded once from `snippets.json`.
//!
//! The JSON was generated from the C# object initializers with item order preserved. The
//! only computed part is `SymbolSnippets.BuildDiacriticItems`, which is reproduced here.

use super::calcpad_type::CalcpadType;
use super::models::{ParameterType, SnippetItem, SnippetParameter};
use serde::Deserialize;
use std::sync::OnceLock;

const SNIPPETS_JSON: &str = include_str!("snippets.json");

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct RawParameter {
    name: String,
    #[serde(default, rename = "Type")]
    param_type: ParameterType,
    type_description: Option<String>,
    description: Option<String>,
    #[serde(default)]
    is_optional: bool,
    #[serde(default)]
    is_variadic: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct RawItem {
    insert: String,
    keyword_type: Option<String>,
    description: String,
    documentation: Option<String>,
    example: Option<String>,
    return_type_description: Option<String>,
    label: Option<String>,
    category: String,
    #[serde(default)]
    internal_only: bool,
    quick_type: Option<String>,
    parameters: Option<Vec<RawParameter>>,
    #[serde(default)]
    accepts_any_count: bool,
    return_type: Option<String>,
    #[serde(default)]
    is_element_wise: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct RawData {
    constant: Vec<RawItem>,
    operator: Vec<RawItem>,
    function: Vec<RawItem>,
    vector_function: Vec<RawItem>,
    matrix_function: Vec<RawItem>,
    keyword: Vec<RawItem>,
    command: Vec<RawItem>,
    setting: Vec<RawItem>,
    unit: Vec<RawItem>,
    format_string: Vec<RawItem>,
    html: Vec<RawItem>,
    css: Vec<RawItem>,
    markdown: Vec<RawItem>,
    symbol: Vec<RawItem>,
}

impl From<RawParameter> for SnippetParameter {
    fn from(p: RawParameter) -> Self {
        SnippetParameter {
            name: p.name,
            param_type: p.param_type,
            type_description: p.type_description,
            description: p.description,
            is_optional: p.is_optional,
            is_variadic: p.is_variadic,
        }
    }
}

impl From<RawItem> for SnippetItem {
    fn from(r: RawItem) -> Self {
        SnippetItem {
            insert: r.insert,
            keyword_type: r.keyword_type,
            description: r.description,
            documentation: r.documentation,
            example: r.example,
            return_type_description: r.return_type_description,
            label: r.label,
            category: r.category,
            internal_only: r.internal_only,
            quick_type: r.quick_type,
            parameters: r.parameters.map(|ps| ps.into_iter().map(Into::into).collect()),
            accepts_any_count: r.accepts_any_count,
            return_type: r
                .return_type
                .map(|n| CalcpadType::from_name(&n).unwrap_or_else(|| panic!("snippets.json: unknown ReturnType {n}"))),
            is_element_wise: r.is_element_wise,
        }
    }
}

fn convert(items: Vec<RawItem>) -> Vec<SnippetItem> {
    items.into_iter().map(Into::into).collect()
}

pub(crate) struct SnippetData {
    constant: Vec<SnippetItem>,
    operator: Vec<SnippetItem>,
    function: Vec<SnippetItem>,
    vector_function: Vec<SnippetItem>,
    matrix_function: Vec<SnippetItem>,
    keyword: Vec<SnippetItem>,
    command: Vec<SnippetItem>,
    setting: Vec<SnippetItem>,
    unit: Vec<SnippetItem>,
    format_string: Vec<SnippetItem>,
    html: Vec<SnippetItem>,
    css: Vec<SnippetItem>,
    markdown: Vec<SnippetItem>,
    symbol: Vec<SnippetItem>,
}

impl SnippetData {
    /// All snippets in `SnippetRegistry`'s static constructor order.
    pub(crate) fn all(&self) -> impl Iterator<Item = &SnippetItem> {
        [
            &self.constant,
            &self.operator,
            &self.function,
            &self.vector_function,
            &self.matrix_function,
            &self.keyword,
            &self.command,
            &self.setting,
            &self.unit,
            &self.format_string,
            &self.html,
            &self.css,
            &self.markdown,
            &self.symbol,
        ]
        .into_iter()
        .flatten()
    }
}

/// `SymbolSnippets.BuildDiacriticItems`: every letter combined with each diacritic mark
/// (decomposed form: base letter followed by the combining mark).
fn build_diacritic_items() -> Vec<SnippetItem> {
    let diacritics: [(char, &str, &str); 4] = [
        ('\u{0304}', "Bar", "bar"),
        ('\u{0307}', "Dot", "dot"),
        ('\u{0308}', "Double Dot", "ddot"),
        ('\u{0301}', "Acute Accent", "acute"),
    ];
    let mut items = Vec::with_capacity(diacritics.len() * 52);
    for (mark, name, tag) in diacritics {
        for c in 'a'..='z' {
            items.push(make_diacritic_item(c, mark, name, tag, "Lowercase"));
        }
        for c in 'A'..='Z' {
            items.push(make_diacritic_item(c, mark, name, tag, "Uppercase"));
        }
    }
    items
}

fn make_diacritic_item(base: char, mark: char, name: &str, tag: &str, case_label: &str) -> SnippetItem {
    let glyph: String = [base, mark].iter().collect();
    SnippetItem {
        description: format!("{base} with {}", name.to_lowercase()),
        label: Some(format!("{glyph} {base}{tag}")),
        category: format!("Symbols/{name} {case_label}"),
        insert: glyph,
        keyword_type: None,
        documentation: None,
        example: None,
        return_type_description: None,
        internal_only: false,
        quick_type: None,
        parameters: None,
        accepts_any_count: false,
        return_type: None,
        is_element_wise: false,
    }
}

pub(crate) fn data() -> &'static SnippetData {
    static DATA: OnceLock<SnippetData> = OnceLock::new();
    DATA.get_or_init(|| {
        let raw: RawData = serde_json::from_str(SNIPPETS_JSON).expect("snippets.json is valid");
        let mut symbol = convert(raw.symbol);
        symbol.extend(build_diacritic_items());
        SnippetData {
            constant: convert(raw.constant),
            operator: convert(raw.operator),
            function: convert(raw.function),
            vector_function: convert(raw.vector_function),
            matrix_function: convert(raw.matrix_function),
            keyword: convert(raw.keyword),
            command: convert(raw.command),
            setting: convert(raw.setting),
            unit: convert(raw.unit),
            format_string: convert(raw.format_string),
            html: convert(raw.html),
            css: convert(raw.css),
            markdown: convert(raw.markdown),
            symbol,
        }
    })
}

macro_rules! category_accessors {
    ($($(#[$doc:meta])* $fn_name:ident => $field:ident),* $(,)?) => {
        $(
            $(#[$doc])*
            pub fn $fn_name() -> &'static [SnippetItem] {
                &data().$field
            }
        )*
    };
}

category_accessors! {
    /// `ConstantSnippets.Items`.
    constant_snippets => constant,
    /// `OperatorSnippets.Items`.
    operator_snippets => operator,
    /// `FunctionSnippets.Items`.
    function_snippets => function,
    /// `VectorFunctionSnippets.Items`.
    vector_function_snippets => vector_function,
    /// `MatrixFunctionSnippets.Items`.
    matrix_function_snippets => matrix_function,
    /// `KeywordSnippets.Items`.
    keyword_snippets => keyword,
    /// `CommandSnippets.Items`.
    command_snippets => command,
    /// `SettingSnippets.Items`.
    setting_snippets => setting,
    /// `UnitSnippets.Items`.
    unit_snippets => unit,
    /// `FormatStringSnippets.Items`.
    format_string_snippets => format_string,
    /// `HtmlSnippets.Items`.
    html_snippets => html,
    /// `CssSnippets.Items`.
    css_snippets => css,
    /// `MarkdownSnippets.Items`.
    markdown_snippets => markdown,
    /// `SymbolSnippets.Items` (base symbols followed by the generated diacritic combinations).
    symbol_snippets => symbol,
}
