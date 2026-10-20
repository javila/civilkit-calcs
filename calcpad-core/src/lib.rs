//! calcpad-core: a Rust port of the CalcpadCE calculation engine (Calcpad.Core).
//!
//! Module names mirror the folders and files of Calcpad.Core so that the port can be
//! followed file by file. The core performs no I/O; file access for `#include` goes
//! through the [`parsers::macro_parser::FileProvider`] trait.
// Style lints that would force the port away from the structure of the C# source.
#![allow(
    clippy::excessive_precision,
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::needless_range_loop,
    clippy::type_complexity,
    clippy::manual_range_contains,
    clippy::collapsible_match,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::manual_is_multiple_of,
    clippy::wrong_self_convention,
    clippy::should_implement_trait,
    clippy::result_unit_err,
    clippy::needless_late_init,
    clippy::manual_clamp,
    clippy::manual_ignore_case_cmp,
    clippy::comparison_chain,
    clippy::len_zero,
    clippy::nonminimal_bool
)]

pub mod base_types;
pub mod calcpad_error;
pub mod calculator;
pub mod clock;
pub mod exceptions;
pub mod extension_methods;
pub mod markdig;
pub mod math_parser_exception;
pub mod matrix;
pub mod messages;
pub mod netfmt;
pub mod output;
pub mod parsers;
pub mod plotter;
pub mod settings;
pub mod solver;
pub mod sort;
pub mod validator;
pub mod vector;

pub use math_parser_exception::{CResult, MathParserException};
pub use parsers::math_parser::MathParser;
pub use settings::{MathSettings, Settings};
