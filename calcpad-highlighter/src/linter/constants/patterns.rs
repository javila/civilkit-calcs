//! CalcpadPatterns.cs. Only the patterns the highlighter actually applies are compiled; the
//! lookaround patterns of the C# file (`Identifier`, `MacroCall`, `MacroParamInComment`) are
//! unused upstream and are not carried over.
use regex::Regex;
use std::sync::OnceLock;

pub const IDENTIFIER_START_CHARS: &str = "a-zA-Zα-ωΑ-Ω°øØ∡℧";
pub const IDENTIFIER_CHARS: &str = "a-zA-Zα-ωΑ-Ω°øØ∡℧0-9_,′″‴⁗⁰¹²³⁴⁵⁶⁷⁸⁹ⁿ⁺⁻₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎";
pub const MACRO_NAME_START_CHARS: &str = "a-zA-Z_";
pub const MACRO_NAME_CHARS: &str = "a-zA-Z0-9_";

macro_rules! pattern {
    ($name:ident, $build:expr) => {
        pub fn $name() -> &'static Regex {
            static R: OnceLock<Regex> = OnceLock::new();
            R.get_or_init(|| Regex::new(&$build).expect("valid pattern"))
        }
    };
}

// identifier = expression (captures name and expression)
pattern!(
    variable_assignment,
    format!(r"^\s*([{IDENTIFIER_START_CHARS}][{IDENTIFIER_CHARS}]*)\s*=\s*(.+)")
);
// identifier(params) = expression
pattern!(
    function_definition,
    format!(r"^\s*([{IDENTIFIER_START_CHARS}][{IDENTIFIER_CHARS}]*)\s*\(([^)]*)\)\s*=")
);
// #def macroName$(params) = expression
pattern!(
    inline_macro_def,
    format!(r"(?i)#def\s+([{MACRO_NAME_START_CHARS}][{MACRO_NAME_CHARS}]*\$?)(?:\(([^)]*)\))?\s*=\s*(.+)")
);
// #def macroName$(params)
pattern!(
    multiline_macro_def,
    format!(r"(?i)#def\s+([{MACRO_NAME_START_CHARS}][{MACRO_NAME_CHARS}]*\$?)(?:\(([^)]*)\))?\s*$")
);
pattern!(include_statement, r"(?i)^\s*#include\s+(.+)$");
// funcName(params) = $Inline{ | $Block{ | $While{
pattern!(
    function_with_command_block,
    format!(
        r"(?i)^\s*([{IDENTIFIER_START_CHARS}][{IDENTIFIER_CHARS}]*)\s*\(([^)]*)\)\s*=\s*\$(Inline|Block|While)\s*\{{"
    )
);
pattern!(loose_macro_name_extract, r"(?i)#def\s+([^\s(=]+)");
pattern!(
    valid_macro_name,
    format!(r"^[{MACRO_NAME_START_CHARS}][{MACRO_NAME_CHARS}]*\$?$")
);
pattern!(
    valid_macro_param,
    format!(r"^[{MACRO_NAME_START_CHARS}][{MACRO_NAME_CHARS}]*\$?$")
);
