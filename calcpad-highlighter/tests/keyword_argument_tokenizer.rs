//! KeywordArgumentTokenizerTests.cs and FormatStringMacroTokenizerTests.cs
use calcpad_highlighter::tokenizer::models::{Token, TokenType};
use calcpad_highlighter::tokenizer::CalcpadTokenizer;

fn tokens(source: &str) -> Vec<Token> {
    CalcpadTokenizer::new().tokenize(source).tokens
}

fn type_of(source: &str, text: &str, line: i32) -> TokenType {
    tokens(source)
        .into_iter()
        .find(|t| t.line == line && t.text == text)
        .unwrap()
        .token_type
}

#[test]
fn keyword_argument_name_is_local_variable() {
    for name in ["y", "x", "z"] {
        let source = format!("f(x; y = 2; z = 3) = x + y + z\nr = f({name} = 1)");
        assert_eq!(type_of(&source, name, 1), TokenType::LocalVariable, "{name}");
    }
}

#[test]
fn keyword_argument_name_in_nested_call_is_local_variable() {
    let source = "f(x; y = 2; z = 3) = x + y + z\nn = f(x = f(1; z = 0); y = 0)";
    let got: Vec<String> = tokens(source)
        .into_iter()
        .filter(|t| t.line == 1 && t.token_type == TokenType::LocalVariable)
        .map(|t| t.text)
        .collect();
    assert_eq!(got, ["x", "z", "y"]);
}

#[test]
fn keyword_argument_name_with_unit_valued_default_is_local_variable() {
    let source = "w(L; q = 5kN/m) = q*L^2/8\nM = w(6m; q = 12kN/m)";
    assert_eq!(type_of(source, "q", 1), TokenType::LocalVariable);
}

#[test]
fn equals_in_call_of_function_without_defaults_is_not_keyword_argument() {
    let source = "g(x; y) = x + y\nr = g(1; y = 3)";
    assert_eq!(type_of(source, "y", 1), TokenType::Units);
}

#[test]
fn equals_in_built_in_function_call_stays_comparison() {
    let source = "y = 7\nr = if(y = 2; 1; 0)";
    assert_eq!(type_of(source, "y", 1), TokenType::Variable);
}

#[test]
fn macro_parameter_after_default_is_macro_parameter() {
    let source = "#def calc$(a$; b$ = 10; c$ = 5) = a$ + b$ + c$";
    let got: Vec<TokenType> = tokens(source)
        .into_iter()
        .filter(|t| t.text == "c$")
        .map(|t| t.token_type)
        .collect();
    assert_eq!(got, [TokenType::MacroParameter, TokenType::MacroParameter]);
}

#[test]
fn multiline_macro_body_keeps_parameters_after_default() {
    let source = "#def convert$(v$; u$ = mm)\n\tv$|u$\n#end def";
    let got: Vec<TokenType> = tokens(source)
        .into_iter()
        .filter(|t| t.line == 1)
        .map(|t| t.token_type)
        .collect();
    assert_eq!(
        got,
        [TokenType::MacroParameter, TokenType::Bracket, TokenType::MacroParameter]
    );
}

#[test]
fn macro_keyword_argument_name_is_macro_parameter() {
    let source = "#def calc$(a$; b$ = 10; c$ = 5) = a$ + b$ + c$\nm = calc$(1; c$ = 7)";
    assert_eq!(type_of(source, "c$", 1), TokenType::MacroParameter);
}

#[test]
fn macro_call_tokenizes_every_argument() {
    let source = "#def calc$(a$; b$ = 10; c$ = 5) = a$ + b$ + c$\nm = calc$(1; 2; 3)";
    let got: Vec<String> = tokens(source)
        .into_iter()
        .filter(|t| t.line == 1 && t.token_type == TokenType::Const)
        .map(|t| t.text)
        .collect();
    assert_eq!(got, ["1", "2", "3"]);
}

#[test]
fn name_after_call_resolves_normally() {
    let source = "f(x; y = 2) = x + y\ny = 7\nr = f(1; y = 2) + y";
    let got: Vec<TokenType> = tokens(source)
        .into_iter()
        .filter(|t| t.line == 2 && t.text == "y")
        .map(|t| t.token_type)
        .collect();
    assert_eq!(got, [TokenType::LocalVariable, TokenType::Variable]);
}

#[test]
fn macro_reference_immediately_after_format_string_tokenizes_separately() {
    let t = tokens("#def text$ = 'text'\nf = 5ft|in:f2text$");
    assert!(t.iter().any(|t| t.token_type == TokenType::Format && t.text == "f2"));
    assert!(t.iter().any(|t| t.token_type == TokenType::Macro && t.text == "text$"));
}

#[test]
fn macro_reference_after_various_format_specifiers_tokenizes_separately() {
    for spec in ["e3", "g3", "n3", "c3"] {
        let t = tokens(&format!("#def note$ = 'text'\nx = 123.456:{spec}note$"));
        assert!(
            t.iter().any(|t| t.token_type == TokenType::Format && t.text == spec),
            "{spec}"
        );
        assert!(
            t.iter().any(|t| t.token_type == TokenType::Macro && t.text == "note$"),
            "{spec}"
        );
    }
}

#[test]
fn format_string_without_trailing_macro_is_a_single_format_token() {
    let t = tokens("x = 123.456:f3");
    assert!(t.iter().any(|t| t.token_type == TokenType::Format && t.text == "f3"));
}
