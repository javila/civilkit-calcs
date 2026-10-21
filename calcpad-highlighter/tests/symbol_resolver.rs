//! SymbolResolverTests.cs
use calcpad_highlighter::content_resolution::symbol_resolver::{SymbolHit, SymbolKind, SymbolResolver};
use calcpad_highlighter::content_resolution::ContentResolver;

fn resolve(content: &str, line: i32, column: i32) -> Option<SymbolHit> {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    SymbolResolver::resolve_symbol_at(Some(&staged.stage3), line, column)
}

#[test]
fn resolves_variable_at_assignment() {
    let src = "x = 5\ny = x + 1\n";
    let hit = resolve(src, 0, 0).unwrap();
    assert_eq!(hit.name, "x");
    assert_eq!(hit.kind, SymbolKind::Variable);
    assert_eq!(hit.locations.len(), 2); // definition + usage
}

#[test]
fn resolves_variable_at_usage() {
    let src = "x = 5\ny = x + 1\n";
    // 'x' usage is on line 1, column 4
    let hit = resolve(src, 1, 4).unwrap();
    assert_eq!(hit.name, "x");
    assert_eq!(hit.kind, SymbolKind::Variable);
}

#[test]
fn resolves_function_at_definition() {
    let src = "f(a) = a + 1\ny = f(2)\n";
    let hit = resolve(src, 0, 0).unwrap();
    assert_eq!(hit.name, "f");
    assert_eq!(hit.kind, SymbolKind::Function);
}

#[test]
fn resolves_function_at_call_site() {
    let src = "f(a) = a + 1\ny = f(2)\n";
    // 'f' call is on line 1 column 4
    let hit = resolve(src, 1, 4).unwrap();
    assert_eq!(hit.name, "f");
    assert_eq!(hit.kind, SymbolKind::Function);
}

#[test]
fn resolves_inline_macro_at_call_site() {
    let src = "#def greet$ = 'hi\ngreet$\n";
    // 'greet$' is on line 1, columns 0..5 inclusive
    let hit = resolve(src, 1, 0).unwrap();
    assert_eq!(hit.name, "greet$");
    assert_eq!(hit.kind, SymbolKind::Macro);
}

#[test]
fn resolves_multiline_macro_cursor_just_past_dollar() {
    // Regression: cursor placed between the `$` and `(` of a macro call
    // (column == start + length) used to fail the client-side overlap check.
    let src = "#def accumulate$(result$; v$)\n\
\tresult$ = 0\n\
\t#for k_acc = 1 : len(v$)\n\
\t\tresult$ = result$ + v$.k_acc\n\
\t#loop\n\
#end def\n\
accumulate$(runningTotal; 2)\n";
    // `accumulate$` on line 6 starts at column 0 and is 11 chars long.
    // Column 11 sits on the `(` and must still resolve to the macro.
    let hit = resolve(src, 6, 11).unwrap();
    assert_eq!(hit.name, "accumulate$");
    assert_eq!(hit.kind, SymbolKind::Macro);
}

#[test]
fn resolves_macro_whose_name_contains_another_macro_name() {
    // Regression: doubleCheck$ calls double$ in its body; the recursive expansion used to record a
    // bogus column-0 call site on the doubleCheck$ line and shadow the real macro.
    let src = "#def double$(n$) = n$*2\n\
#def doubleCheck$(a$) = double$(a$)\n\
doubleCheck$(5)\n";
    // Cursor inside "doubleCheck$" on line 2 must resolve to doubleCheck$.
    let hit = resolve(src, 2, 3).unwrap();
    assert_eq!(hit.name, "doubleCheck$");
    assert_eq!(hit.kind, SymbolKind::Macro);
    // Its definition is on line 1, not double$'s line 0.
    let def = hit.locations.iter().find(|l| l.is_assignment).unwrap();
    assert_eq!(def.line, 1);
}

#[test]
fn resolves_function_definition_with_leading_comment() {
    // Regression: a leading text comment ('text') shifts the function name off column 0. The
    // definition location must still be marked is_assignment so "Go to Definition" finds it.
    let src = "'text'a(x) = x\ny = a(2)\n";
    // Resolve from the call site on line 1.
    let hit = resolve(src, 1, 4).unwrap();
    assert_eq!(hit.name, "a");
    assert_eq!(hit.kind, SymbolKind::Function);

    let def = hit.locations.iter().find(|l| l.is_assignment).unwrap();
    assert_eq!(def.line, 0);
    assert_eq!(def.column, 6); // after 'text'
}

#[test]
fn recursive_function_definition_marks_only_defining_token_as_assignment() {
    // The defining `f` is an assignment; the recursive `f` on the RHS is a usage.
    let src = "f(n) = f(n - 1)\n";
    let hit = resolve(src, 0, 0).unwrap();
    assert_eq!(hit.name, "f");
    assert_eq!(hit.locations.iter().filter(|l| l.is_assignment).count(), 1);
}

#[test]
fn returns_null_when_cursor_outside_any_token() {
    // Column 10 is past the end of the line
    assert!(resolve("x = 5\n", 0, 10).is_none());
}

#[test]
fn returns_null_on_blank_line() {
    assert!(resolve("x = 5\n\ny = 7\n", 1, 0).is_none());
}

#[test]
fn definition_and_usage_both_marked_correctly() {
    let src = "x = 5\ny = x + 1\n";
    let hit = resolve(src, 0, 0).unwrap();
    assert!(hit.locations.iter().any(|l| l.is_assignment && l.line == 0));
    assert!(hit.locations.iter().any(|l| !l.is_assignment && l.line == 1));
}
