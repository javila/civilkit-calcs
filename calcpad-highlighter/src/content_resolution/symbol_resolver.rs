//! SymbolResolver.cs: "what symbol is at this cursor position?", the single source of truth shared by
//! every editor integration. Lookups go against the Stage 3 indices, which are already mapped back
//! to the original source line and column.
use super::result::{Stage3Result, SymbolLocation};
use crate::text::StrMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function,
    Macro,
}

/// A cursor position resolved to a symbol: its name, kind, and every location where it appears
/// (definitions and usages).
#[derive(Clone, Debug)]
pub struct SymbolHit {
    pub name: String,
    pub kind: SymbolKind,
    pub locations: Vec<SymbolLocation>,
}

pub struct SymbolResolver;

impl SymbolResolver {
    /// The symbol at the given original-source position, or `None` when no user-defined
    /// variable/function/macro covers the cursor. Macros are checked first because their
    /// `$`-suffixed names cannot collide with variable or function names, and function tokens take
    /// the next priority for the same reason.
    pub fn resolve_symbol_at(stage3: Option<&Stage3Result>, line: i32, column: i32) -> Option<SymbolHit> {
        let stage3 = stage3?;

        if let Some(name) = find_in_index(&stage3.macro_index, line, column) {
            let locations = stage3.macro_index.get(name).cloned().unwrap_or_default();
            return Some(SymbolHit {
                name: name.to_string(),
                kind: SymbolKind::Macro,
                locations,
            });
        }
        if let Some(name) = find_in_index(&stage3.function_index, line, column) {
            let locations = stage3.function_index.get(name).cloned().unwrap_or_default();
            return Some(SymbolHit {
                name: name.to_string(),
                kind: SymbolKind::Function,
                locations,
            });
        }
        if let Some(name) = find_in_index(&stage3.variable_index, line, column) {
            let locations = stage3.variable_index.get(name).cloned().unwrap_or_default();
            return Some(SymbolHit {
                name: name.to_string(),
                kind: SymbolKind::Variable,
                locations,
            });
        }
        None
    }
}

/// Scans one index for any local-source location that covers the cursor, inclusive on both edges so
/// an editor reporting the cursor one past the clicked character still resolves to the token it is
/// visually attached to. When several locations match at the edges, an interior match is preferred.
fn find_in_index(index: &StrMap<Vec<SymbolLocation>>, line: i32, column: i32) -> Option<&str> {
    let mut best_name: Option<&str> = None;
    let mut best_is_interior = false;

    for (key, locations) in index.iter() {
        for loc in locations {
            if loc.source != "local" {
                continue;
            }
            if loc.line != line {
                continue;
            }

            let start = loc.column;
            let end = loc.column + loc.length;
            if column < start || column > end {
                continue;
            }

            let is_interior = column > start && column < end;
            if best_name.is_none() || (is_interior && !best_is_interior) {
                best_name = Some(key);
                best_is_interior = is_interior;
            }
        }
    }

    best_name
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc(line: i32, column: i32, length: i32, source: &str) -> SymbolLocation {
        SymbolLocation {
            line,
            column,
            length,
            source: source.to_string(),
            source_file: None,
            is_assignment: false,
        }
    }

    #[test]
    fn priority_edges_and_interior() {
        let mut s = Stage3Result::default();
        s.macro_index.insert("m$", vec![loc(0, 0, 2, "local")]);
        s.variable_index
            .insert("a", vec![loc(0, 2, 1, "local"), loc(1, 0, 1, "include")]);
        s.variable_index.insert("ab", vec![loc(0, 3, 2, "local")]);

        // Macro first
        let hit = SymbolResolver::resolve_symbol_at(Some(&s), 0, 1).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("m$", SymbolKind::Macro));
        // Edge shared by two variables: the first wins unless the other is interior
        let hit = SymbolResolver::resolve_symbol_at(Some(&s), 0, 3).unwrap();
        assert_eq!(hit.name, "a");
        let hit = SymbolResolver::resolve_symbol_at(Some(&s), 0, 4).unwrap();
        assert_eq!(hit.name, "ab");
        // Included locations never resolve; nothing outside
        assert!(SymbolResolver::resolve_symbol_at(Some(&s), 1, 0).is_none());
        assert!(SymbolResolver::resolve_symbol_at(Some(&s), 0, 9).is_none());
        assert!(SymbolResolver::resolve_symbol_at(None, 0, 0).is_none());
    }

    // Ported from SymbolResolverTests.cs (they run the full three-stage pipeline).
    use crate::content_resolution::ContentResolver;

    fn resolve(content: &str, line: i32, column: i32) -> Option<SymbolHit> {
        let staged = ContentResolver::new().get_staged_content(content, None, None, None);
        SymbolResolver::resolve_symbol_at(Some(&staged.stage3), line, column)
    }

    #[test]
    fn resolves_variable_at_assignment_and_usage() {
        let src = "x = 5\ny = x + 1\n";
        let hit = resolve(src, 0, 0).unwrap();
        assert_eq!(
            (hit.name.as_str(), hit.kind, hit.locations.len()),
            ("x", SymbolKind::Variable, 2)
        );
        let hit = resolve(src, 1, 4).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("x", SymbolKind::Variable));
        assert!(hit.locations.iter().any(|l| l.is_assignment && l.line == 0));
        assert!(hit.locations.iter().any(|l| !l.is_assignment && l.line == 1));
    }

    #[test]
    fn resolves_function_at_definition_and_call_site() {
        let src = "f(a) = a + 1\ny = f(2)\n";
        let hit = resolve(src, 0, 0).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("f", SymbolKind::Function));
        let hit = resolve(src, 1, 4).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("f", SymbolKind::Function));
    }

    #[test]
    fn resolves_macros() {
        let hit = resolve("#def greet$ = 'hi\ngreet$\n", 1, 0).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("greet$", SymbolKind::Macro));

        let src = "#def accumulate$(result$; v$)\n\tresult$ = 0\n\t#for k_acc = 1 : len(v$)\n\t\tresult$ = result$ + v$.k_acc\n\t#loop\n#end def\naccumulate$(runningTotal; 2)\n";
        let hit = resolve(src, 6, 11).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("accumulate$", SymbolKind::Macro));

        let src = "#def double$(n$) = n$*2\n#def doubleCheck$(a$) = double$(a$)\ndoubleCheck$(5)\n";
        let hit = resolve(src, 2, 3).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("doubleCheck$", SymbolKind::Macro));
        let def = hit.locations.iter().find(|l| l.is_assignment).unwrap();
        assert_eq!(def.line, 1);
    }

    #[test]
    fn function_definition_with_a_leading_comment_is_still_the_assignment() {
        let hit = resolve("'text'a(x) = x\ny = a(2)\n", 1, 4).unwrap();
        assert_eq!((hit.name.as_str(), hit.kind), ("a", SymbolKind::Function));
        let def = hit.locations.iter().find(|l| l.is_assignment).unwrap();
        assert_eq!((def.line, def.column), (0, 6));
    }

    #[test]
    fn recursive_function_marks_only_the_defining_token() {
        let hit = resolve("f(n) = f(n - 1)\n", 0, 0).unwrap();
        assert_eq!(hit.name, "f");
        assert_eq!(hit.locations.iter().filter(|l| l.is_assignment).count(), 1);
    }

    #[test]
    fn nothing_resolves_outside_a_token() {
        assert!(resolve("x = 5\n", 0, 10).is_none());
        assert!(resolve("x = 5\n\ny = 7\n", 1, 0).is_none());
    }
}
