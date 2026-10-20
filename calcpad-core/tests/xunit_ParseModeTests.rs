//! Transcribed from Calcpad.Tests/ExpressionParser/ParseModeTests.cs by tools/transcribe_xunit.py.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::identity_op,
    clippy::needless_borrow,
    clippy::redundant_clone,
    clippy::bool_assert_comparison,
    clippy::useless_format,
    clippy::clone_on_copy,
    unused_imports,
    dead_code,
    suspicious_double_ref_op,
    noop_method_call,
    clippy::needless_borrows_for_generic_args
)]
mod common;
use common::*;

// named-options
// private static string Render(string source, bool debug = false)
fn Render(source: &str, named: &[(&str, bool)]) -> String {
    let debug = named.iter().any(|(n, v)| *n == "debug" && *v);
    let mut parser = new_expression_parser(&[("Debug", debug)]);
    parser.parse(source, true);
    parser.html_result.replace("\r\n", "\n")
}
fn ExpandAndRender(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (has_errors, expanded) = macro_parser.Parse(source);
    assert!(!has_errors, "{expanded}");
    Render(&expanded, &[])
}
// private static int Count(string html, string value) => Regex.Count(html, Regex.Escape(value));
fn Count(html: impl AsRef<str>, value: &str) -> f64 {
    let html = html.as_ref();
    html.matches(value).count() as f64
}

#[test]
fn UnknownHashLine_DoesNotCrashKeywordLookup_case1() {
    let line = "#zeta = 1";
    assert_does_not_contain("Unexpected error", &Render(&line, &[]));
}
#[test]
fn UnknownHashLine_DoesNotCrashKeywordLookup_case2() {
    let line = "#{a}";
    assert_does_not_contain("Unexpected error", &Render(&line, &[]));
}
#[test]
#[ignore = "out of scope: markdown"]
fn Markdown_IsNotMistakenForMd() {}
#[test]
#[ignore = "out of scope: markdown"]
fn MdOn_StillRendersComments() {}
#[test]
fn Html_PassesMultilineScriptThroughUnwrapped() {
    let mut html = Render("#html\n<script>\nlet a = 1;\nlet b = a < 2;\n</script>\n#end html", &[]);
    assert_contains("<script>\nlet a = 1;\nlet b = a < 2;\n</script>", &html);
    assert_does_not_contain("<p>let", &html);
    assert_does_not_contain("class=\"err\"", &html);
}
#[test]
fn Html_KeepsIndentation() {
    assert_contains(
        "<pre>\n    indented\n</pre>",
        &Render("#html\n<pre>\n    indented\n</pre>\n#end html", &[]),
    );
}
#[test]
#[ignore = "out of scope: markdown"]
fn Markdown_RendersTableAsOneBlock() {}
#[test]
#[ignore = "out of scope: markdown"]
fn Markdown_KeepsIndentationForNestedLists() {}
#[test]
#[ignore = "out of scope: markdown"]
fn Markdown_BlankLineSeparatesParagraphs() {}
#[test]
#[ignore = "out of scope: markdown"]
fn Markdown_SupportsEmphasisExtras() {}
#[test]
#[ignore = "out of scope: markdown"]
fn Markdown_TagsEachBlockWithItsSourceLine() {}
#[test]
fn Html_TagsOpeningTagsWithSourceLine() {
    // Assert.Matches("<div[^>]*data-source-line=\"2\"", ...)
    let html = Render("#html\n<div>x</div>\n#end html", &[("debug", true)]);
    let i = html.find("<div").expect("no <div");
    let tag_end = html[i..].find('>').map(|j| i + j).unwrap_or(html.len());
    assert_contains("data-source-line=\"2\"", &html[i..tag_end]);
}
#[test]
fn EndForms_RestorePreviousMode() {
    let mut html = Render(
        "#html\n<i>a</i>\n#cpd\nccc = 3\n#end cpd\n<i>back</i>\n#end html\nddd = 4",
        &[],
    );
    assert_contains("<i>back</i>", &html);
    assert_contains("ccc", &html);
    assert_contains("ddd", &html);
    assert_does_not_contain("class=\"err\"", &html);
}
#[test]
fn SwitchWithoutEnd_LastsUntilNextSwitch() {
    let mut html = Render("#html\n<i>raw</i>\n#cpd\neee = 5", &[]);
    assert_contains("<i>raw</i>", &html);
    assert_contains("eee", &html);
    assert_does_not_contain("class=\"err\"", &html);
}
#[test]
fn InlineCondition_PassesOrSkipsContent() {
    let mut html = Render(
        "x = 5\n#html x > 1\n<b>shown</b>\n#end html\n#html x > 10\n<b>skipped</b>\n#end html",
        &[],
    );
    assert_contains("<b>shown</b>", &html);
    assert_does_not_contain("<b>skipped</b>", &html);
}
#[test]
fn CpdCondition_SkipsCalculationsPerIteration() {
    let html = Render(
        "zzz = 0\n#for i = 1 : 4\n#cpd i > 2\nzzz = zzz + 10\n#end cpd\n#loop\nyyy = zzz",
        &[],
    );
    // Assert.Matches(@"yyy[\s\S]*20", html)
    let i = html.find("yyy").expect("no yyy");
    assert_contains("20", &html[i..]);
    assert_does_not_contain("class=\"err\"", &html);
}
#[test]
fn ModeBlocks_NestInsideIfBranches() {
    let source = "x = 1\n#if x > 0\n#html\n<b>taken</b>\n#end html\n#else\n#markdown\n**skipped**\n#end markdown\n#end if\nwww = 2";
    let mut html = Render(&source, &[]);
    assert_contains("<b>taken</b>", &html);
    assert_does_not_contain("skipped", &html);
    assert_contains("www", &html);
    assert_does_not_contain("class=\"err\"", &html);
}
#[test]
fn HtmlContent_RepeatsInsideLoop() {
    assert_equal(
        3.0,
        Count(Render("#for i = 1 : 3\n#html\n<hr/>\n#end html\n#loop", &[]), "<hr/>"),
    );
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case1() {
    let line = "#if 1";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case2() {
    let line = "#for i = 1 : 2";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case3() {
    let line = "#hide";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case4() {
    let line = "#md on";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case5() {
    let line = "#end if";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case6() {
    let line = "#val";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case7() {
    let line = "#end markdown";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case8() {
    let line = "#test";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
fn UnrecognizedKeywords_AreContentInHtmlMode_case9() {
    let line = "#header { color: red; }";
    assert_contains(line, &Render(&format!("#html\n{}\n#end html", line), &[]));
}
#[test]
#[ignore = "out of scope: markdown"]
fn UnrecognizedKeywords_AreContentInMarkdownMode_case1() {}
#[test]
#[ignore = "out of scope: markdown"]
fn UnrecognizedKeywords_AreContentInMarkdownMode_case2() {}
#[test]
#[ignore = "out of scope: markdown"]
fn UnrecognizedKeywords_AreContentInMarkdownMode_case3() {}
#[test]
fn ControlBlocks_WrapModeBlocks() {
    let mut html = Render(
        "#if 1 > 2\n#html\n<b>no</b>\n#end html\n#else\n#html\n<b>yes</b>\n#end html\n#end if",
        &[],
    );
    assert_contains("<b>yes</b>", &html);
    assert_does_not_contain("<b>no</b>", &html);
}
#[test]
#[ignore = "out of scope: markdown"]
fn MarkdownHeadings_AreContentNotKeywords() {}
#[test]
fn MacroCalls_ExpandInsideHtml() {
    assert_contains(
        "<b>hi</b>",
        &ExpandAndRender("#def greet$ = <b>hi</b>\n#html\ngreet$\n#end html\n"),
    );
}
#[test]
fn MacroWithArguments_DefinedInsideHtml_Expands() {
    let mut html = ExpandAndRender(
        "#html\n#def note$(text$) = <div class=\"note\">text$</div>\nnote$(Check deflection at midspan.)\n#end html\n",
    );
    assert_contains("<div class=\"note\">Check deflection at midspan.</div>", &html);
    assert_does_not_contain("class=\"err\"", &html);
}
#[test]
fn MacrosThatSwitchMode_ApplyAtTheCallSite() {
    let mut html = ExpandAndRender(
        "#def open$\n#html\n#end def\n#def close$\n#end html\n#end def\nopen$\n<b>raw</b>\nclose$\nddd = 4\n",
    );
    assert_contains("<b>raw</b>", &html);
    assert_contains("ddd", &html);
    assert_does_not_contain("class=\"err\"", &html);
}
#[test]
#[ignore = "out of scope: markdown"]
fn Markdown_SupportsStrikethroughAndAutolinks() {}
#[test]
fn TrailingOperatorCharacters_DoNotJoinHtmlLines() {
    let mut html = Render("#html\n<style>\np { color: red;\n}\n</style>\n#end html", &[]);
    assert_contains("p { color: red;\n}", &html);
}
