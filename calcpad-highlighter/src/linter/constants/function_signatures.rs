//! FunctionSignatures.cs: built-in function signatures derived from the snippet registry.
use crate::linter::models::{CalcpadType, FunctionSignature};
use crate::snippets::{self, SnippetItem, SnippetRegistry};
use crate::text::StrMap;
use std::sync::OnceLock;

fn to_linter_type(t: snippets::CalcpadType) -> CalcpadType {
    use snippets::CalcpadType as S;
    match t {
        S::Unknown => CalcpadType::Unknown,
        S::Value => CalcpadType::Value,
        S::Vector => CalcpadType::Vector,
        S::Matrix => CalcpadType::Matrix,
        S::CustomUnit => CalcpadType::CustomUnit,
        S::Function => CalcpadType::Function,
        S::InlineMacro => CalcpadType::InlineMacro,
        S::MultilineMacro => CalcpadType::MultilineMacro,
        S::Various => CalcpadType::Various,
    }
}

fn snippet_to_signature(name: &str, s: &SnippetItem) -> FunctionSignature {
    FunctionSignature {
        name: name.to_string(),
        min_params: s.min_params(),
        max_params: s.max_params(),
        parameter_types: s
            .parameters
            .as_ref()
            .map(|ps| ps.iter().map(|p| p.param_type).collect())
            .unwrap_or_default(),
        return_type: s.return_type.map(to_linter_type).unwrap_or(CalcpadType::Value),
        description: s.description.clone(),
        is_element_wise: s.is_element_wise,
        accepts_any_count: s.accepts_any_count,
    }
}

/// Signatures keyed ignore case. Overloaded functions get the most permissive signature.
pub fn signatures() -> &'static StrMap<FunctionSignature> {
    static M: OnceLock<StrMap<FunctionSignature>> = OnceLock::new();
    M.get_or_init(|| {
        let mut m = StrMap::new_ci();
        for (k, v) in SnippetRegistry::get_function_snippets_by_name().iter() {
            m.insert(k, snippet_to_signature(k, v));
        }
        m
    })
}

/// All overloads keyed ignore case.
pub fn all_overloads() -> &'static StrMap<Vec<FunctionSignature>> {
    static M: OnceLock<StrMap<Vec<FunctionSignature>>> = OnceLock::new();
    M.get_or_init(|| {
        let mut m = StrMap::new_ci();
        for (k, v) in SnippetRegistry::get_function_overloads().iter() {
            m.insert(k, v.iter().map(|s| snippet_to_signature(k, s)).collect());
        }
        m
    })
}

pub fn get_signature(function_name: &str) -> Option<&'static FunctionSignature> {
    signatures().get(function_name)
}

pub fn get_all_overloads(function_name: &str) -> &'static [FunctionSignature] {
    all_overloads().get(function_name).map_or(&[], Vec::as_slice)
}

pub fn has_signature(function_name: &str) -> bool {
    signatures().contains_key(function_name)
}
