//! CalcpadTokenizer.MacroCollection.cs: macro collection mode (Stage 2 of content resolution).
use super::macros::extract_macro_params_with_defaults;
use super::models::TokenType;
use super::CalcpadTokenizer;
use crate::content_resolution::result::{DuplicateMacro, MacroDefinition, MacroInfo};
use crate::linter::helpers::parsing_helpers::count_required_params;
use crate::linter::models::DefinitionMetadata;
use crate::text::{eq_ignore_case, is_white_space_slice, to_string, trim, StrMap};

impl CalcpadTokenizer {
    pub(super) fn init_macro_collection_state(&mut self) {
        self.macro_curr_name = None;
        self.macro_curr_params = Vec::new();
        self.macro_curr_has_params = false;
        self.macro_curr_defaults = None;
        self.macro_curr_start_line = -1;
        self.macro_curr_is_inline = false;
        self.macro_curr_inline_content = None;
        self.macro_curr_content_lines = Vec::new();
        self.macro_curr_collecting_body = false;
        self.macro_first_definition_lines = StrMap::new_ci();
        self.macro_pending_metadata = None;
    }

    /// Called from `track_definitions` for each token in Macro mode: captures the macro name.
    pub(super) fn track_definitions_macro(&mut self, ty: TokenType, text: &str) {
        // Body lines are accumulated as raw text
        if self.macro_curr_collecting_body {
            return;
        }
        if ty == TokenType::Macro {
            // is_macro (not has_macro): append runs before has_macro is set in parse_macro
            if self.state.is_macro && self.macro_curr_name.is_none() {
                self.macro_curr_name = Some(text.to_string());
                self.macro_curr_start_line = self.state.line;
            }
        }
    }

    /// Called at the end of each line in Macro mode.
    pub(super) fn finalize_line_macro_collection(&mut self) {
        let text = self.state.text.clone();
        if self.macro_curr_collecting_body {
            let trimmed = to_string(trim(&text));
            if eq_ignore_case(&trimmed, "#end def") {
                self.emit_macro_definition(true);
                self.macro_curr_collecting_body = false;
                self.macro_curr_name = None;
                self.macro_curr_params = Vec::new();
                self.macro_curr_has_params = false;
                self.macro_curr_defaults = None;
                self.macro_curr_content_lines = Vec::new();
            } else {
                self.macro_curr_content_lines.push(to_string(&text));
            }
            return;
        }

        let mut emitted = false;

        // An inline macro this line (#def name$(params) = content)
        if self.macro_curr_is_inline && self.macro_curr_name.is_some() {
            self.emit_macro_definition(false);
            self.macro_curr_is_inline = false;
            self.macro_curr_name = None;
            self.macro_curr_params = Vec::new();
            self.macro_curr_has_params = false;
            self.macro_curr_defaults = None;
            self.macro_curr_inline_content = None;
            emitted = true;
        }

        // #def without '=' begins a multiline body
        if !emitted && self.state.has_macro && self.macro_curr_name.is_some() && !self.macro_curr_is_inline {
            let (names, defaults) = extract_macro_params_with_defaults(&text);
            self.macro_curr_params = names;
            self.macro_curr_has_params = true;
            self.macro_curr_defaults = defaults;
            self.macro_curr_collecting_body = true;
            // Metadata is consumed when the macro is emitted
            return;
        }

        if emitted {
            self.macro_pending_metadata = None;
        } else if let Some(m) = DefinitionMetadata::try_parse(&text) {
            self.macro_pending_metadata = Some(m);
        } else if !is_white_space_slice(&text) {
            self.macro_pending_metadata = None;
        }
    }

    fn emit_macro_definition(&mut self, is_multiline: bool) {
        let name = self.macro_curr_name.clone().unwrap_or_default();
        let (source, source_file) = self.get_source_info(self.macro_curr_start_line);
        let meta = self.macro_pending_metadata.clone();

        let params = self.macro_curr_params.clone();
        let def = MacroDefinition {
            name: name.clone(),
            params: params.clone(),
            defaults: self.macro_curr_defaults.clone().unwrap_or_default(),
            content: if is_multiline {
                self.macro_curr_content_lines.clone()
            } else {
                vec![self.macro_curr_inline_content.clone().unwrap_or_default()]
            },
            line_number: self.macro_curr_start_line,
            source,
            source_file,
            description: meta.as_ref().and_then(|m| m.description.clone()),
            param_types: meta.as_ref().and_then(|m| m.param_types.clone()),
            param_descriptions: meta.as_ref().and_then(|m| m.param_descriptions.clone()),
            ..Default::default()
        };

        // Track duplicates
        if let Some(orig) = self.macro_first_definition_lines.get(&name).copied() {
            self.result.duplicate_macros.push(DuplicateMacro {
                name: name.clone(),
                duplicate_line_number: self.macro_curr_start_line,
                original_line_number: orig,
            });
        } else {
            self.macro_first_definition_lines
                .insert(&name, self.macro_curr_start_line);
        }

        self.result.macro_definitions.push(def);

        if !self.result.user_defined_macros.contains_key(&name) {
            self.result.user_defined_macros.insert(
                &name,
                MacroInfo {
                    line_number: self.macro_curr_start_line,
                    param_count: params.len() as i32,
                    required_param_count: count_required_params(params.len() as i32, self.macro_curr_defaults.as_ref()),
                    param_names: params,
                },
            );
        }

        self.macro_pending_metadata = None;
    }
}
