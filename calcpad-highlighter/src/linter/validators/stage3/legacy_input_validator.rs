//! LegacyInputValidator.cs: warns on '?{value}' input fields, the legacy way of storing an input
//! value in the document that #UI replaces.
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use calcpad_core::extension_methods::enumerate_comments;
use calcpad_core::ui_syntax;

const CODE: &str = "CPD-3419";

#[derive(Default)]
pub struct LegacyInputValidator;

impl LegacyInputValidator {
    pub fn new() -> Self {
        LegacyInputValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        for (idx, line) in stage3.lines.iter().enumerate() {
            let i = idx as i32;
            if !tp.is_cpd_mode(i) {
                continue;
            }
            let mut offset = 0i32;
            for segment in enumerate_comments(line) {
                let seg: Vec<char> = segment.chars().collect();
                if ui_syntax::is_code(segment) {
                    report_input_fields(i, &seg, offset, result);
                }
                offset += seg.len() as i32;
            }
        }
    }
}

fn report_input_fields(line_index: i32, segment: &[char], offset: i32, result: &mut LinterResult) {
    let mut i = 0usize;
    while i < segment.len() {
        if segment[i] != '?' {
            i += 1;
            continue;
        }
        let mut brace = i + 1;
        while brace < segment.len() && segment[brace] == ' ' {
            brace += 1;
        }
        if brace >= segment.len() || segment[brace] != '{' {
            i += 1;
            continue;
        }
        let end = match segment[brace..].iter().position(|c| *c == '}') {
            None => segment.len(),
            Some(c) => brace + c + 1,
        };
        let text: String = segment[i..end].iter().collect();
        result.add_warning(
            line_index,
            offset + i as i32,
            offset + end as i32,
            CODE,
            &format!("'{text}' - declare the input with #UI instead"),
        );
        i = end;
    }
}
