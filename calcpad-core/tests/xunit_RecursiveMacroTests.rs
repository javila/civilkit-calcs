//! Transcribed from Calcpad.Tests/Macros/RecursiveMacroTests.cs by tools/transcribe_xunit.py.
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

#[test]
fn DirectSelfReference_ReportsCircularError_DoesNotOverflow() {
    let mut macroParser = TestMacroParser::new();
    let mut source = "#def selfref$ = selfref$\nselfref$\n";
    let (mut hasErrors, mut expanded) = macroParser.Parse(source);
    assert!(hasErrors);
    assert_contains("ircular", &expanded);
}
#[test]
fn MutualRecursion_ReportsCircularError_DoesNotOverflow() {
    let mut macroParser = TestMacroParser::new();
    let mut source = "#def alpha$ = beta$\n#def beta$ = alpha$\nalpha$\n";
    let (mut hasErrors, mut expanded) = macroParser.Parse(source);
    assert!(hasErrors);
    assert_contains("ircular", &expanded);
}
#[test]
fn ParameterizedSelfReference_ReportsCircularError_DoesNotOverflow() {
    let mut macroParser = TestMacroParser::new();
    let mut source = "#def grow$(x$) = grow$(x$)\ngrow$(1)\n";
    let (mut hasErrors, mut expanded) = macroParser.Parse(source);
    assert!(hasErrors);
    assert_contains("ircular", &expanded);
}
#[test]
fn NonCyclicNesting_StillExpandsFully() {
    let mut macroParser = TestMacroParser::new();
    let mut source = "#def one$ = 1\n#def two$ = one$ + one$\n#def three$ = two$ + one$\nthree$\n";
    let (mut hasErrors, mut expanded) = macroParser.Parse(source);
    assert!(!(hasErrors));
    assert_contains("1 + 1 + 1", &expanded);
}
#[test]
fn ConcurrentParsers_DoNotShareMacroTable() {
    // C#: 32 Task.Run macro parsers, awaited together.
    let handles: Vec<_> = (0..32)
        .map(|i| {
            std::thread::spawn(move || {
                let macro_parser = TestMacroParser::new();
                let source = format!("#def only{i}$ = {i}\nonly{i}$\n");
                let (has_errors, expanded) = macro_parser.Parse(&source);
                assert!(!has_errors);
                assert_contains(&i.to_string(), &expanded);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}
