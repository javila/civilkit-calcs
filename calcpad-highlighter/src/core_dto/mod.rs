//! The parts of Calcpad.Core the highlighter needs that calcpad-core does not carry:
//! `DirectiveDto` (shared JSON payload plumbing) and `SettingsDto` (`#settings {...}`).
//! `UiDto` lives in `calcpad_core::ui_dto`.
//!
//! System.Text.Json deserialisation is emulated with serde_json: property names match
//! case insensitively, unknown properties are ignored, the last duplicate wins, and a value of
//! the wrong JSON type (or a non-integer number for an integer property) is an error.
pub mod directive_dto;
pub mod settings_dto;

pub use directive_dto::{enum_try_parse, DirectiveError, JsonError};
pub use settings_dto::{SettingKey, SettingsDto};
