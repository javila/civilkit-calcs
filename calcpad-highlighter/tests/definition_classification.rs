//! DefinitionClassificationTests.cs
mod common;
use calcpad_highlighter::content_resolution::result::Stage3Result;
use calcpad_highlighter::content_resolution::ContentResolver;

fn stage3(content: &str) -> Stage3Result {
    ContentResolver::new()
        .get_staged_content(content, None, None, None)
        .stage3
}

#[test]
fn multiple_functions_per_line_classify_as_functions() {
    let src = "am(u; k) = $Root{F(φ; k) = u @ φ = 0 : 10*π}\n\
               sn(u; k) = sin(am(u; k))','cn(u; k) = cos(am(u; k))\n\
               dn(u; k) = sqrt(1 - k*sn(u; k)^2)','cd(u; k) = cn(u; k)/dn(u; k)\n";
    let s3 = stage3(src);
    let funcs: Vec<&str> = s3.functions_with_params.iter().map(|f| f.name.as_str()).collect();
    for n in ["am", "sn", "cn", "dn", "cd"] {
        assert!(funcs.contains(&n), "{n} missing in {funcs:?}");
    }
}

#[test]
fn macro_call_sites_produce_no_variable_definitions() {
    let src = "#def circle$(x$; y$; style$) = '<circle cx=\"'x$'\" cy=\"'y$'\" r=\"12\" style$/>\n\
               R = 10\n\
               circle$(0; 0; R)\n\
               circle$(1; 1; R)\n";
    let vars: Vec<String> = stage3(src)
        .variables_with_definitions
        .iter()
        .map(|v| v.name.clone())
        .collect();
    assert_eq!(vars, ["R"]);
}

#[test]
fn definition_line_numbers_are_original_editor_lines_after_macro_expansion() {
    // A multiline-macro CALL expands to two body lines, shifting every definition after it in Stage3.
    let src = "#def emit$(v$)\n\
               \tv$\n\
               \tv$\n\
               #end def\n\
               p = 1\n\
               emit$(p)\n\
               f(x) = x + p','g(x) = x - p\n\
               q = 2\n";
    let normalized = src.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    let s3 = stage3(src);

    for f in &s3.functions_with_params {
        assert!(
            lines[f.line_number as usize].contains(&format!("{}(", f.name)),
            "{}",
            f.name
        );
    }
    for v in &s3.variables_with_definitions {
        assert!(
            lines[v.line_number as usize]
                .trim_start()
                .starts_with(&format!("{} ", v.name)),
            "{}",
            v.name
        );
    }
    let single = |name: &str| {
        let m: Vec<_> = s3.functions_with_params.iter().filter(|f| f.name == name).collect();
        assert_eq!(1, m.len());
        m[0].line_number
    };
    assert_eq!(6, single("f"));
    assert_eq!(6, single("g"));
    let q: Vec<_> = s3.variables_with_definitions.iter().filter(|v| v.name == "q").collect();
    assert_eq!(1, q.len());
    assert_eq!(7, q[0].line_number);
}
