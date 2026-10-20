//! Transcribed from Calcpad.Tests/ExtensionMethodsTests.cs by tools/transcribe_xunit.py.
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

// ToLineArray is inlined into the transcribed theory.

#[test]
fn EnumerateLines_TrimsSingleTerminalLineBreak_Symmetrically() {
    // [MemberData(nameof(EnumerateLinesCases))]
    let cases: [(&str, &[&str]); 17] = [
        ("", &[]),
        ("a", &["a"]),
        ("a\n", &["a"]),
        ("a\r\n", &["a"]),
        ("a\r", &["a"]),
        ("a\nb", &["a", "b"]),
        ("a\rb", &["a", "b"]),
        ("a\nb\n", &["a", "b"]),
        ("a\r\nb\r\n", &["a", "b"]),
        ("a\rb\r", &["a", "b"]),
        ("a\n\nb\n", &["a", "", "b"]),
        ("a\nb\n\n", &["a", "b", ""]),
        ("a\r\nb\r\n\r\n", &["a", "b", ""]),
        ("a\rb\r\r", &["a", "b", ""]),
        ("\n", &[""]),
        ("\r\n", &[""]),
        ("\r", &[""]),
    ];
    for (source, expected) in cases {
        let lines = calcpad_core::extension_methods::enumerate_lines(source);
        assert_eq!(expected, lines.as_slice(), "source {source:?}");
    }
}
