//! ParseModeHighlighterTests.cs
mod common;
use calcpad_highlighter::content_resolution::source_parse_modes::{ParseModeRange, SourceParseModes};
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::{LinterResult, LinterSeverity, ParseMode};
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};
use calcpad_highlighter::prettifier::CalcpadPrettifier;
use calcpad_highlighter::tokenizer::models::{TokenType, TokenizerResult};
use calcpad_highlighter::tokenizer::CalcpadTokenizer;

fn lint(source: &str) -> LinterResult {
    let includes = common::include_files();
    let staged = ContentResolver::new().get_staged_content(source, Some(&includes), None, None);
    CalcpadLinter::new().lint(&staged, &LintIgnoreRegionParser::extract_regions(source))
}

fn tokenize(source: &str) -> TokenizerResult {
    CalcpadTokenizer::new().tokenize(source)
}

#[test]
fn content_lines_get_no_tokens_and_modes_follow_the_stack() {
    let result = tokenize("#html\n<b>x = 1</b>\n#cpd\ny = 2\n#end cpd\n<i>z</i>\n#end html\nw = 3");
    assert!(result.get_tokens_for_line(1).is_empty());
    assert!(!result.get_tokens_for_line(3).is_empty());
    assert!(result.get_tokens_for_line(5).is_empty());
    let raw: Vec<i32> = result.raw_lines.iter().copied().collect();
    assert_eq!(raw, [1, 5]);
    assert_eq!(result.line_modes[&1], ParseMode::Html);
    assert!(!result.line_modes.contains_key(&3));
    assert_eq!(result.line_modes[&6], ParseMode::Html);
    assert!(!result.line_modes.contains_key(&7));
}

#[test]
fn content_lines_highlight_only_defined_macros() {
    let result = tokenize("#def greet$ = hi\n#html\n<b>greet$</b> costs 5$\n#end html");
    let toks = result.get_tokens_for_line(2);
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].token_type, TokenType::Macro);
    assert_eq!(toks[0].text, "greet$");
    assert_eq!(toks[0].column, 3);
}

#[test]
fn unknown_hash_lines_keep_markdown_colouring() {
    for line in ["# Title", "#hashtag text"] {
        assert!(
            tokenize(&format!("#markdown\n{line}\n#end markdown"))
                .raw_lines
                .contains(&1),
            "{line}"
        );
    }
}

#[test]
fn unrecognized_keywords_are_content() {
    let cases = [
        ("markdown", "#tag"),
        ("markdown", "# Title"),
        ("markdown", "#end html"),
        ("markdown", "#round 2"),
        ("html", "#test"),
        ("html", "#header { color: red; }"),
        ("html", "#end markdown"),
    ];
    for (mode, line) in cases {
        let source = format!("#{mode}\n{line}\n#end {mode}");
        assert!(lint(&source).diagnostics.is_empty(), "{mode} {line}");
        assert!(tokenize(&source).raw_lines.contains(&1), "{mode} {line}");
    }
}

#[test]
fn directive_with_condition_is_tokenized() {
    let result = tokenize("x = 1\n#html\n#html x > 1\n<b>a</b>\n#end html\n#end html");
    assert!(!result.raw_lines.contains(&2));
    assert!(result.raw_lines.contains(&3));
}

#[test]
fn markdown_prose_produces_no_diagnostics() {
    let result = lint("#markdown\na = b + undefinedThing\n\n| x | y |\n|---|---|\n#end markdown");
    assert!(result.diagnostics.is_empty());
}

#[test]
fn mode_directives_are_recognized() {
    let source =
        "#html\n<b>x</b>\n#cpd\nx = 1\n#end cpd\n#markdown\n**y**\n#end markdown\n#end html\n#html x > 0\n<i>z</i>\n#end html";
    assert!(!lint(source)
        .diagnostics
        .iter()
        .any(|d| d.severity == LinterSeverity::Error));
}

#[test]
fn calcpad_directives_in_html_are_content() {
    for line in ["#if 1", "#end if", "#md on"] {
        let source = format!("#html\n{line}\n#end html");
        assert!(lint(&source).diagnostics.is_empty(), "{line}");
        assert!(tokenize(&source).raw_lines.contains(&1), "{line}");
    }
}

#[test]
fn macro_directives_in_html_are_still_directives() {
    let source = "#html\n#def tag$ = <b>t</b>\ntag$\n#end html";
    assert!(lint(source).diagnostics.is_empty());
    assert!(!tokenize(source).raw_lines.contains(&1));
}

#[test]
fn control_blocks_wrap_mode_blocks() {
    let source = "#if x > 3\n#html\n<b>big</b>\n#end html\n#else\n#markdown\n*small*\n#end markdown\n#end if";
    assert!(lint(&format!("x = 1\n{source}")).diagnostics.is_empty());
}

#[test]
fn html_content_does_not_join_continuation_lines() {
    let source = "#html\n<style>\np { color: red;\n}\n</style>\n#end html";
    let includes = common::include_files();
    let staged = ContentResolver::new().get_staged_content(source, Some(&includes), None, None);
    assert_eq!(staged.stage1.lines.len(), 6);
    assert!(lint(source).diagnostics.is_empty());
}

fn source_modes(source: &str) -> Vec<ParseModeRange> {
    let includes = common::include_files();
    let staged = ContentResolver::new().get_staged_content(source, Some(&includes), None, None);
    SourceParseModes::build(&staged, source.split('\n').count() as i32)
}

fn range(start_line: i32, end_line: i32, mode: ParseMode) -> ParseModeRange {
    ParseModeRange {
        start_line,
        end_line,
        mode,
    }
}

#[test]
fn source_modes_follow_directives() {
    let ranges = source_modes("a = 1\n#html\n<b>x</b>\n#cpd\nb = 2\n#end cpd\n<i>y</i>\n#end html\nc = 3");
    assert_eq!(ranges, [range(2, 3, ParseMode::Html), range(6, 7, ParseMode::Html)]);
}

#[test]
fn source_modes_follow_macros_that_switch_mode() {
    let source = "#def open$\n#html\n#end def\n#def close$\n#end html\n#end def\nopen$\n<b>x</b>\n\nclose$\nd = 4";
    assert_eq!(source_modes(source), [range(7, 9, ParseMode::Html)]);
}

#[test]
fn source_modes_ignore_directives_inside_macro_bodies() {
    assert!(source_modes("#def block$\n#markdown\n**b**\n#end markdown\n#end def\ne = 5").is_empty());
}

#[test]
fn prettifier_leaves_content_lines_untouched() {
    let source = "#if 1\n#markdown\n- one\n  - two\n#end markdown\nx = 1\n#end if";
    let expected = "#if 1\n\t#markdown\n- one\n  - two\n\t#end markdown\n\tx = 1\n#end if";
    assert_eq!(CalcpadPrettifier::prettify(source, None), expected);
}
