//! Port of `Calcpad.Highlighter/Snippets`: snippet models, the category data and the
//! [`SnippetRegistry`] lookups used by the linter and the frontend.
//!
//! The data tables live in `snippets.json` (generated once from the C# object
//! initializers, order preserved) and are parsed on first use.

mod calcpad_type;
mod data;
mod folded;
mod json;
mod models;
mod registry;

pub use calcpad_type::CalcpadType;
pub use data::{
    command_snippets, constant_snippets, css_snippets, format_string_snippets, function_snippets, html_snippets,
    keyword_snippets, markdown_snippets, matrix_function_snippets, operator_snippets, setting_snippets,
    symbol_snippets, unit_snippets, vector_function_snippets,
};
pub use folded::{FoldedMap, NameSet};
pub use json::snippets_json;
pub use models::{ParameterType, SnippetItem, SnippetParameter};
pub use registry::SnippetRegistry;

#[cfg(test)]
mod tests;
