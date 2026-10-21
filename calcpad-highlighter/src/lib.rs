//! Rust port of Calcpad.Highlighter (CalcpadCE). Module names follow the C# folders.
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::collapsible_match,
    clippy::len_zero,
    clippy::nonminimal_bool,
    clippy::new_without_default,
    clippy::should_implement_trait,
    clippy::field_reassign_with_default
)]
pub mod api;
pub mod content_resolution;
pub mod core_dto;
pub mod html_comment;
pub mod linter;
pub mod parsing;
pub mod path_roots;
pub mod prettifier;
pub mod snippets;
pub mod text;
pub mod tokenizer;
pub mod utf16;
