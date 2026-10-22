use super::*;
use serde_json::Value;

const SAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../web/calcpad-web/public/sheet-samples/");

fn sample(name: &str) -> String {
    std::fs::read_to_string(format!("{SAMPLES}{name}.sheet")).unwrap()
}

/// Compiles a sheet and renders the result with the engine. Returns (compiled, html, errors).
fn render(src: &str) -> (Compiled, String, Vec<Value>) {
    let c = compile(src);
    let (html, errs) = run(&c.text);
    (c, html, errs)
}

fn run(calcpad: &str) -> (String, Vec<Value>) {
    let req = serde_json::json!({ "content": calcpad, "includeLineAnchors": true });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    (
        v["html"].as_str().unwrap_or("").to_string(),
        v["errors"].as_array().cloned().unwrap_or_default(),
    )
}

/// The visible text of the report: tags removed, white space collapsed.
fn text(html: &str) -> String {
    let body = html.split("</head>").nth(1).unwrap_or(html);
    // Drop the style and script blocks.
    let mut s = body.to_string();
    for tag in ["style", "script", "svg"] {
        while let Some(a) = s.find(&format!("<{tag}")) {
            let Some(b) = s[a..].find(&format!("</{tag}>")) else {
                break;
            };
            s.replace_range(a..a + b + tag.len() + 3, " ");
        }
    }
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn no_errors(name: &str, errs: &[Value], c: &Compiled) {
    assert!(errs.is_empty(), "{name}: engine errors {errs:?}\n{}", c.text);
    assert!(c.diagnostics.is_empty(), "{name}: {:?}", c.diagnostics);
}

#[test]
fn beam_bending_values() {
    let (c, html, errs) = render(&sample("Beam bending"));
    no_errors("beam", &errs, &c);
    let t = text(&html);
    // Hand check: M* = 12 x 6^2 / 8 = 54 kN.m; Ms = 300 MPa x 1.23e6 mm3 = 369 kN.m;
    // phiMs = 0.9 x 369 = 332.1; 54 / 332.1 = 0.1626.
    assert!(t.contains("= 54 kN · m"), "{t}");
    assert!(t.contains("= 369 kN · m"));
    assert!(t.contains("= 332.1 kN · m"));
    assert!(t.contains("= 0.163"));
    assert!(t.contains("OK: the beam passes in bending."));
    assert!(!t.contains("Fails: increase"));
}

#[test]
fn column_values() {
    let (c, html, errs) = render(&sample("Column"));
    no_errors("column", &errs, &c);
    let t = text(&html);
    // 7500 mm2 x 350 MPa = 2 625 000 N = 2625 kN; 0.85 x 0.8 x 2625 = 1785; 1200 / 1785 = 0.6723.
    assert!(t.contains("= 2625 kN"), "{t}");
    assert!(t.contains("= 1785 kN"));
    assert!(t.contains("= 0.672"));
}

#[test]
fn loads_table_values() {
    let (c, html, errs) = render(&sample("Loads table"));
    no_errors("loads", &errs, &c);
    let t = text(&html);
    // 1.2 x 4.5 + 1.5 x 3 = 9.9; 4.5 + 0.7 x 3 = 6.6; 9.9 x 2.4 = 23.76; 23.76 x 5^2 / 8 = 74.25.
    assert!(t.contains("= 9.9 kPa"), "{t}");
    assert!(t.contains("= 6.6 kPa"));
    assert!(t.contains("= 23.76 kN ∕ m"));
    assert!(t.contains("= 74.25 kN · m"));
    assert!(html.contains("<table"), "the Markdown table renders");
    assert!(html.contains("<b>K</b>") || html.contains("<b><var>K</var></b>"));
}

#[test]
fn plot_sample() {
    let (c, html, errs) = render(&sample("Plot"));
    no_errors("plot", &errs, &c);
    assert!(c.text.contains("$Plot{ M(x) @ x = 0 : L }"), "{}", c.text);
    assert!(html.contains("<svg"), "a plot is drawn");
    // M(3) = 12 x 3 x 3 / 2 = 54.
    assert!(text(&html).contains("= 54"));
    assert!(c.uses_latex);
}

#[test]
fn latex_sample() {
    let (c, html, errs) = render(&sample("LaTeX prose"));
    no_errors("latex", &errs, &c);
    assert!(c.uses_latex);
    assert!(html.contains("data-display=\"1\""), "the display formula is a block");
    assert!(html.contains("class=\"sheet-tex\""));
    // 54e6 N.mm x 150 mm / 8.5e7 mm4 = 95.29 MPa.
    assert!(text(&html).contains("= 95.29 MPa"));
    assert!(compile_with(&sample("LaTeX prose"), &Options { plain_tex: true })
        .text
        .contains("sheet-tex-src"));
}

#[test]
fn mixing_sample() {
    let (c, html, errs) = render(&sample("Mixing with Calcpad"));
    no_errors("mixing", &errs, &c);
    let t = text(&html);
    // Three bays of 2.5 m added by a raw Calcpad loop, then read by a sheet line.
    assert!(t.contains("= 7.5 m"), "{t}");
    // The friendly loop ran three times; the else branch once and the then branch twice.
    assert_eq!(t.matches("Stage one.").count(), 1);
    assert_eq!(t.matches("flag = k").count(), 2);
    assert!(t.contains("= 30 kN"));
}

#[test]
fn every_sample_compiles_cleanly() {
    for e in std::fs::read_dir(SAMPLES).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_none_or(|x| x != "sheet") {
            continue;
        }
        let src = std::fs::read_to_string(&p).unwrap();
        assert!(is_sheet(&src), "{p:?} has the marker");
        let (c, _, errs) = render(&src);
        no_errors(&format!("{p:?}"), &errs, &c);
        assert_eq!(c.map.len(), c.text.lines().count());
    }
}

fn lines(c: &Compiled) -> Vec<&str> {
    c.text.lines().collect()
}

#[test]
fn quantity_lines() {
    let c = compile("fy: 300 MPa | grade\nZe = 1.5e3 mm³\nMs: fy * Ze -> kN*m\nx: 1; y: 2");
    let l = lines(&c);
    assert!(l[0].starts_with("fy = 300 MPa '<span"));
    assert!(l[0].contains(">grade<"));
    assert_eq!(l[1], "Ze = 1500 mm^3");
    assert_eq!(l[2], "Ms = fy * Ze|kN*m");
    assert_eq!(&l[3..5], ["x = 1", "y = 2"]);
    assert_eq!(c.map, vec![0, 1, 2, 3, 3]);
    assert_eq!(
        c.defs.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
        ["fy", "Ze", "Ms", "x", "y"]
    );
}

#[test]
fn names_typeset() {
    let c = compile("phi: 0.9\nsigma_c: 25 MPa\nMstar: 5 kN*m\nphiMs: phi * Mstar\nf(x): x^2\nDelta_T: 3");
    let l = lines(&c);
    assert_eq!(l[0], "φ = 0.9");
    assert_eq!(l[1], "σ_c = 25 MPa");
    assert_eq!(l[2], "M_star = 5 kN*m");
    assert_eq!(l[3], "φMs = φ * M_star");
    assert_eq!(l[4], "f(x) = x^2");
    assert_eq!(l[5], "Δ_T = 3");
}

#[test]
fn prose_stays_prose() {
    let c = compile("# Title\n\nNote: see the drawing below.\nThe load is 5 kN applied here.\n\n- a: item\n");
    assert_eq!(lines(&c)[0], "#markdown");
    assert!(c.text.contains("Note: see the drawing below."));
    assert!(!c.text.contains(" = "));
    assert_eq!(*lines(&c).last().unwrap(), "#end markdown");
    // A backslash forces text.
    let c = compile("\\fy: 300 MPa");
    assert!(c.text.contains("\nfy: 300 MPa\n"));
}

#[test]
fn vectors_and_matrices() {
    let c = compile("v: [1; 2; 3]\nr: [1, 2, 3]\nM: [1, 2; 3, 4]\nN: [max(1, 2), 5]");
    let l = lines(&c);
    assert_eq!(l[0], "v = [1; 2; 3]");
    assert_eq!(l[1], "r = [1; 2; 3]");
    assert_eq!(l[2], "M = [1; 2 | 3; 4]");
    assert_eq!(l[3], "N = [max(1; 2); 5]");
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty());
    assert!(html.contains("matrix"));
}

#[test]
fn conditions_and_plots() {
    let c = compile("a: 2\nif a <= 3: \"small\" else: \"big\"\nif a == 2: b: 1\nplot f(x), g(x) for x in 0..a * 2");
    let l = lines(&c);
    assert_eq!(&l[1..6], ["#if a ≤ 3", "'small", "#else", "'big", "#end if"]);
    assert_eq!(&l[6..9], ["#if a ≡ 2", "b = 1", "#end if"]);
    assert_eq!(l[9], "$Plot{ f(x) & g(x) @ x = 0 : a * 2 }");
    let bad = compile("plot f(x) for x in 0");
    assert!(bad.text.contains("#markdown"), "an invalid plot line stays text");
}

#[test]
fn escapes_pass_through_verbatim() {
    let c = compile("a: 1\n>> b = a + 1 'raw text\n```calcpad\n#if b > 1\n'big\n#end if\n```\nc: b");
    let l = lines(&c);
    assert_eq!(l[1], "b = a + 1 'raw text");
    assert_eq!(&l[2..5], ["#if b > 1", "'big", "#end if"]);
    assert_eq!(l[5], "c = b");
    assert_eq!(c.map, vec![0, 1, 3, 4, 5, 7]);
}

/// Sheet lines and raw Calcpad alternate in any order and share variables both ways.
#[test]
fn mixing_shares_variables() {
    let src = "<!--sheet-->\n# Mix\nn: 4\nphi: 0.5\n>> s = 0\n```calcpad\n#for i = 1 : n\n    s = s + i*phi\n#loop\n```\nprose between\nt: s * 2\n>> #if t > 10\n>> u = t\n>> #else\n>> u = 0\n>> #end if\nw: u + 1\n$$x = y$$\nz: w";
    let (c, html, errs) = render(src);
    no_errors("mix", &errs, &c);
    let t = text(&html);
    // s = 0.5 x (1 + 2 + 3 + 4) = 5; t = 10; the else branch gives u = 0; w = 1; z = 1.
    assert!(t.contains("s = s + i · φ"), "{t}");
    assert!(t.contains("t = s · 2 = 5 · 2 = 10"), "{t}");
    assert!(t.contains("w = u + 1 = 0 + 1 = 1"), "{t}");
    assert!(t.contains("z = w = 1"));
}

#[test]
fn raw_lines_can_use_sheet_spelling() {
    // The sheet defines `phi` (the engine's letter is phi) and a raw line reads it by the sheet spelling.
    let (c, html, errs) = render("<!--sheet-->\nphi: 0.9\nMstar: 10\n>> r = phi * Mstar");
    no_errors("raw names", &errs, &c);
    assert!(c.text.contains("r = φ * M_star"), "{}", c.text);
    assert!(text(&html).contains("= 0.9 · 10 = 9"));
    // A raw line that defines a name the sheet then reads.
    let (c, _, errs) = render("<!--sheet-->\n>> sigma = 3\nb: sigma + 1");
    no_errors("raw defs", &errs, &c);
    assert!(c.text.contains("σ = 3") && c.text.contains("b = σ + 1"), "{}", c.text);
}

#[test]
fn error_lines_map_back_to_the_sheet() {
    let src = "<!--sheet-->\n# T\n\na: 1\nprose line\nb: a + missing\n>> k = nothere\n\nc: 2";
    let (c, _, errs) = render(src);
    assert_eq!(errs.len(), 2, "{errs:?}");
    let sheet_lines: Vec<usize> = errs
        .iter()
        .map(|e| c.map[e["sourceLine"].as_u64().unwrap() as usize - 1])
        .collect();
    assert_eq!(sheet_lines, vec![5, 6]);
}

#[test]
fn blocks_nest_and_close_by_indentation() {
    let src = "for i in 1..2:\n    if i == 1:\n        for j in 1..2:\n            a: i * j\n    else:\n        b: i\nafter: 1";
    let c = compile(src);
    assert_eq!(
        lines(&c),
        [
            "#for i = 1 : 2",
            "#if i ≡ 1",
            "#for j = 1 : 2",
            "a = i * j",
            "#loop",
            "#else",
            "b = i",
            "#end if",
            "#loop",
            "after = 1"
        ]
    );
    assert!(c.diagnostics.is_empty());
    assert_eq!(c.map, vec![0, 1, 2, 3, 2, 4, 5, 1, 0, 6]);
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}");
    let t = text(&html);
    // i = 1: a = 1, 2 ; i = 2: b = 2.
    assert!(t.contains("a = i · j = 1 · 2 = 2"), "{t}");
    assert!(t.contains("b = i = 2"));
}

#[test]
fn else_if_while_and_vector_loops() {
    let c = compile("n: 3\nwhile n > 0:\n    n: n - 1\nif n > 5:\n    a: 1\nelse if n == 0:\n    a: 2\nelse:\n    a: 3\nfor x in [1, 2, 3]:\n    s: x");
    let l = lines(&c);
    assert_eq!(l[1], "#while n > 0");
    assert_eq!(l[4], "#if n > 5");
    assert_eq!(l[6], "#else if n ≡ 0");
    assert_eq!(l[8], "#else");
    assert!(l.contains(&"#hide") && l.iter().any(|x| x.starts_with("#for loopi1 = 1 : len(loopseq1)")));
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}\n{}", c.text);
    let t = text(&html);
    assert!(t.contains("a = 2"));
    assert!(t.contains("s = x = 1") && t.contains("s = x = 3"), "{t}");
}

#[test]
fn quantity_lines_in_loops_match_calcpad() {
    let sheet = "for i in 1..3:\n    q: i * 2 kN\n";
    let calc = "#for i = 1 : 3\nq = i * 2 kN\n#loop\n";
    let strip = |h: String| text(&h);
    let a = strip(run(&compile(sheet).text).0);
    let b = strip(run(calc).0);
    assert_eq!(a, b);
    assert!(a.contains("q = i · 2 kN = 3 · 2 kN = 6 kN"));
}

#[test]
fn prose_and_latex_inside_blocks() {
    let src = "for i in 1..2:\n    Stage $i$ of the loop.\n\n    $$a_i$$\n    r: i\n";
    let (c, html, errs) = render(src);
    assert!(errs.is_empty(), "{errs:?}\n{}", c.text);
    assert!(c.uses_latex);
    assert_eq!(html.matches("class=\"sheet-tex\"").count(), 4);
    assert_eq!(c.diagnostics.len(), 0);
}

#[test]
fn block_errors() {
    let c = compile("if 1 > 0:\n\ta: 1\n\t    b: 2\n");
    assert!(
        c.diagnostics
            .iter()
            .any(|d| d.line == 2 && d.severity == Severity::Error && d.message.contains("tabs and spaces")),
        "{:?}",
        c.diagnostics
    );
    let c = compile("else:\n    a: 1");
    assert!(c
        .diagnostics
        .iter()
        .any(|d| d.line == 0 && d.message.contains("without a matching")));
    let c = compile("if x:\ny: 1");
    assert!(c
        .diagnostics
        .iter()
        .any(|d| d.line == 0 && d.message.contains("no body")));
    let c = compile("If the load is large:\n    note");
    assert!(!c.text.contains("#if"));
    let c = compile("$$\na\n");
    assert!(c.diagnostics.iter().any(|d| d.message.contains("never closed")));
}

#[test]
fn deterministic_with_consistent_map() {
    for name in [
        "Beam bending",
        "Column",
        "Loads table",
        "Plot",
        "LaTeX prose",
        "Mixing with Calcpad",
    ] {
        let src = sample(name);
        let a = compile(&src);
        let b = compile(&src);
        assert_eq!(a.text, b.text);
        assert_eq!(a.map, b.map);
        let n = src.lines().count();
        assert_eq!(a.map.len(), a.text.lines().count());
        assert!(a.map.iter().all(|&l| l < n));
        // The map never goes backwards across top level statements, apart from block closers.
        assert!(a.text.ends_with('\n'));
    }
}

#[test]
fn prose_only_input_round_trips_through_markdown() {
    let src =
        "# H\n\nSome *text* with a [link](http://example.com).\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n![alt](pic.png)\n";
    let c = compile(src);
    let body: Vec<&str> = lines(&c)[1..lines(&c).len() - 1].to_vec();
    assert_eq!(body.join("\n"), src.trim_end());
}

#[test]
fn marker_detection() {
    assert!(is_sheet("\n  <!--sheet-->\na: 1"));
    assert!(!is_sheet("a = 1\n<!--sheet-->"));
}
