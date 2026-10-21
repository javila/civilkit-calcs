//! KeywordArgumentLinterTests.cs
mod common;
use calcpad_highlighter::content_resolution::result::StagedResolvedContent;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};
use calcpad_highlighter::text::StrMap;

fn lint(content: &str) -> (LinterResult, StagedResolvedContent) {
    let files: StrMap<String> = StrMap::new();
    let staged = ContentResolver::new().get_staged_content(content, Some(&files), None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    (CalcpadLinter::new().lint(&staged, &regions), staged)
}

fn codes(r: &LinterResult) -> Vec<&str> {
    r.diagnostics.iter().map(|d| d.code.as_str()).collect()
}

#[test]
fn function_defaults_no_parameter_count_error() {
    let (result, _) =
        lint("f(x; y = 2; z = 3) = x + y + z\nr = f(1)\ns = f(1; z = 9)\nt = f(z = 9; x = 1)\nu = r + s + t\n");
    assert!(!codes(&result).contains(&"CPD-3302"));
}

#[test]
fn function_call_too_few_arguments_still_reported() {
    let (result, _) = lint("f(x; y; z = 3) = x + y + z\nr = f(1)\n");
    assert!(codes(&result).contains(&"CPD-3302"));
}

#[test]
fn function_call_unknown_keyword_argument_is_reported() {
    let (result, _) = lint("f(x; y = 2) = x + y\nr = f(1; q = 3)\n");
    assert!(codes(&result).contains(&"CPD-3315"));
}

#[test]
fn function_definition_required_after_optional_is_reported() {
    let (result, _) = lint("f(x = 1; y) = x + y\n");
    assert!(codes(&result).contains(&"CPD-3207"));
}

#[test]
fn macro_defaults_expand_using_defaults() {
    let (result, staged) = lint("#def m$(a$; b$ = 10; c$ = 5) = a$ + b$ + c$\nr = m$(1)\ns = m$(1; c$ = 7)\n");
    assert!(
        staged.stage3.lines.iter().any(|l| l == "r = 1 + 10 + 5"),
        "{:?}",
        staged.stage3.lines
    );
    assert!(
        staged.stage3.lines.iter().any(|l| l == "s = 1 + 10 + 7"),
        "{:?}",
        staged.stage3.lines
    );
    assert!(!codes(&result).contains(&"CPD-3304"));
    assert!(!codes(&result).contains(&"CPD-2211"));
}

#[test]
fn macro_call_unknown_keyword_argument_is_reported() {
    let (result, _) = lint("#def m$(a$; b$ = 1) = a$ + b$\nr = m$(1; z$ = 2)\n");
    assert!(codes(&result).contains(&"CPD-3314"));
    assert!(!codes(&result).contains(&"CPD-3303"));
}

#[test]
fn macro_argument_naming_an_existing_macro_stays_positional() {
    let (result, staged) = lint("#def y$ = q\n#def emit$(lhs$) = [lhs$]\nr = emit$(y$ = 5)\n");
    assert!(
        staged.stage3.lines.iter().any(|l| l == "r = [q = 5]"),
        "{:?}",
        staged.stage3.lines
    );
    assert!(!codes(&result).contains(&"CPD-3314"));
}

#[test]
fn macro_definition_required_after_optional_is_reported() {
    let (result, _) = lint("#def m$(a$ = 1; b$) = a$ + b$\nr = m$(1; 2)\n");
    assert!(codes(&result).contains(&"CPD-2213"));
}

#[test]
fn macro_call_too_few_arguments_still_reported() {
    let (result, _) = lint("#def m$(a$; b$; c$ = 1) = a$ + b$ + c$\nr = m$(1)\n");
    assert!(codes(&result).contains(&"CPD-3304"));
}

#[test]
fn function_defaults_are_tracked_for_hover() {
    let (_, staged) = lint("f(x; y = 2kg) = x + y\nr = f(1kg)\n");
    let defs: Vec<_> = staged
        .stage3
        .functions_with_params
        .iter()
        .filter(|f| f.name == "f")
        .collect();
    assert_eq!(1, defs.len());
    let def = defs[0];
    assert_eq!(vec!["x".to_string(), "y".to_string()], def.params);
    assert_eq!(vec![None, Some("2kg".to_string())], def.defaults);
    assert_eq!(
        1,
        staged
            .stage3
            .user_defined_functions
            .get("f")
            .unwrap()
            .required_param_count
    );
}

#[test]
fn function_call_equality_comparison_is_not_keyword_argument() {
    let (result, _) = lint("f(x; y = 2) = x + y\na = 1\nb = 1\nr = f(a == b)\n");
    assert!(!codes(&result).contains(&"CPD-3315"));
}

#[test]
fn command_block_function_call_using_defaults_validates_body() {
    for call in ["r = f(1)", "r = f(1; z = 9)", "r = f(z = 9; x = 1)", "r = f(1; 2; 3)"] {
        let (result, _) = lint(&format!("f(x; y = 2; z = 3) = $Inline{{a = x + y + z + q}}\n{call}\n"));
        let n = result
            .diagnostics
            .iter()
            .filter(|d| d.code == "CPD-3301" && d.line == 1)
            .count();
        assert_eq!(1, n, "{call}: {:?}", codes(&result));
    }
}

#[test]
fn command_block_function_too_few_arguments_skips_body_validation() {
    let (result, _) = lint("f(x; y; z = 3) = $Block{a = x + y + z + q}\nr = f(1)\n");
    assert!(codes(&result).contains(&"CPD-3302"));
    assert!(!codes(&result).contains(&"CPD-3301"));
}

#[test]
fn default_referencing_variable_keeps_function_defined() {
    for content in [
        "b = 1\nf(x; y = b) = x + y\nr = f(1)\n",
        "f(x; y = b) = x + y\nb = 7\nr = f(1)\n",
        "b = 1\nf(x; y = sqr(b)) = x + y\nr = f(1)\n",
        "g(t) = t\nb = 1\nf(x; y = g(b)) = x + y\nr = f(1)\n",
        "f(b; h = b) = b*h\nr = f(3)\n",
    ] {
        let (result, _) = lint(content);
        assert!(!codes(&result).contains(&"CPD-3301"), "{content}");
        assert!(!codes(&result).contains(&"CPD-3305"), "{content}");
    }
}
