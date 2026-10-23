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
    // Plain Calcpad (Show compiled, Convert, Word and Excel): the TeX source in italics, readable.
    let plain = compile_with(&sample("LaTeX prose"), &Options { plain_tex: true }).text;
    assert!(plain.contains("<i>\\sigma = \\frac{M\\,y}{I}</i>"), "{plain}");
    assert!(!plain.contains("&#"), "no character references in plain output");
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
        // The engine alone cannot run a python fence: the page runs it first (web/e2e/python-check.mjs runs these samples).
        if src.contains("```python") {
            let c = compile(&src);
            assert!(c.diagnostics.is_empty(), "{p:?}: {:?}", c.diagnostics);
            assert!(c.text.contains("#python\n") && c.text.contains("\n#end python"));
            assert_eq!(c.map.len(), c.text.lines().count());
            continue;
        }
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
    // A heading and a plain paragraph are written as a person would; the list needs Markdown.
    assert_eq!(
        lines(&c),
        [
            "'<h1>Title</h1>",
            "",
            "'Note: see the drawing below. The load is 5 kN applied here.",
            "",
            "#markdown",
            "- a: item",
            "#end markdown"
        ]
    );
    assert_eq!(c.map, vec![0, 1, 2, 4, 5, 5, 5]);
    // A backslash forces text.
    let c = compile("\\fy: 300 MPa");
    assert_eq!(c.text, "'fy: 300 MPa\n");
}

#[test]
fn headings_and_paragraphs_are_hand_written_calcpad() {
    let c = compile("# One\n## Two\n### Three\n#### Four\n\nPlain words.\nMore plain words.\n\nSome *emphasis* here.\n\nMore plain.\n");
    assert_eq!(
        lines(&c),
        [
            "'<h1>One</h1>",
            "'<h2>Two</h2>",
            "\"Three",
            "'<h4>Four</h4>",
            "",
            "'Plain words. More plain words.",
            "",
            "#markdown",
            "Some *emphasis* here.",
            "#end markdown",
            "",
            "'More plain."
        ]
    );
    // Words with underscores (f_y) and a lone asterisk are not Markdown emphasis in prose with spaces around.
    let c = compile("Use f_y for the yield stress.\n");
    assert_eq!(c.text, "'Use f_y for the yield stress.\n");
}

#[test]
fn notes_are_plain_comments_in_plain_calcpad() {
    let src = "x: 5 kN | the load\ny: 2 m\n";
    let page = compile(src).text;
    assert_eq!(page, "x = 5 kN '<span class=\"sheet-note\">the load</span>\ny = 2 m\n");
    let plain = compile_with(src, &Options { plain_tex: true }).text;
    assert_eq!(plain, "x = 5 kN 'the load\ny = 2 m\n");
}

#[test]
fn blank_lines_are_kept_between_sections() {
    let c = compile("a: 1\n\n\nb: 2\nif b > 1:\n    c: 3\n\n    d: 4\ne: 5");
    assert_eq!(
        lines(&c),
        [
            "a = 1",
            "",
            "b = 2",
            "#if b > 1",
            "c = 3",
            "",
            "d = 4",
            "#end if",
            "e = 5"
        ]
    );
}

#[test]
fn step_break_and_continue() {
    let c = compile("for i in 0..10 step 2:\n    if i > 6:\n        break\n    s: i\n");
    assert_eq!(
        lines(&c),
        [
            "#for loopi1 = 1 : floor((10 - 0)/(2) + 0.000000001) + 1",
            "#hide",
            "i = 0 + (loopi1 - 1)*(2)",
            "#show",
            "#if i > 6",
            "#break",
            "#end if",
            "s = i",
            "#loop"
        ]
    );
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}");
    // 0, 2, 4, 6 then i = 8 breaks: the loop ran for i = 0, 2, 4, 6.
    let t = text(&html);
    assert!(t.contains("s = i = 6") && !t.contains("s = i = 8"), "{t}");
    // A step down, and continue.
    let c = compile("for i in 5..1 step -2:\n    if i == 3:\n        continue\n    s: i\n");
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}");
    let t = text(&html);
    assert!(
        t.contains("s = i = 5") && t.contains("s = i = 1") && !t.contains("s = i = 3"),
        "{t}"
    );
    // One-line form and outside a loop.
    let c = compile("while n > 0:\n    if n == 2: break\n    n: n - 1\n");
    assert!(c.text.contains("#if n ≡ 2\n#break\n#end if"));
    let c = compile("break\n");
    assert!(c.diagnostics.iter().any(|d| d.message.contains("outside a loop")));
}

#[test]
fn line_kinds() {
    let c = compile("<!--sheet-->\n# T\na: 1\n>> b = 2\nplot a for x in 0..1\nif a > 0:\n    c: 3\n\ntext");
    assert_eq!(c.kinds.len(), 9);
    use LineKind::*;
    assert_eq!(
        c.kinds,
        vec![Blank, Prose, Quantity, Raw, Plot, Block, Quantity, Blank, Prose]
    );
}

#[test]
fn format_is_idempotent_and_leaves_prose() {
    let src = "<!--sheet-->\n# T\n## Inputs\nL:6 m|span\nfy :  300 MPa   |  yield\nMs: fy*Ze->kN*m\n\n\nprose  with   spaces  \n>> raw   line\nx=1 ;y : 2\n";
    let once = format(src);
    assert_eq!(once, format(&once));
    assert!(once.contains(">> raw   line") && once.contains("prose  with   spaces\n"));
    // The prettified sheet compiles to the same Calcpad.
    let flat = |t: String| t.replace(' ', "").replace("\n\n", "\n");
    assert_eq!(flat(compile(src).text), flat(compile(&once).text));
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
    assert!(
        bad.text.contains("'plot f(x) for x in 0"),
        "an invalid plot line stays text"
    );
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
    let src = "Some *text* with a [link](http://example.com).\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n![alt](pic.png)\n";
    let c = compile(src);
    // One markdown block, the text untouched.
    let l = lines(&c);
    assert_eq!(l[0], "#markdown");
    assert_eq!(l[1..l.len() - 1].join("\n"), src.trim_end());
}

#[test]
fn marker_detection() {
    assert!(is_sheet("\n  <!--sheet-->\na: 1"));
    assert!(!is_sheet("a = 1\n<!--sheet-->"));
}

#[test]
fn deflection_samples_values() {
    let (c, html, errs) = render(&sample("Beam deflection"));
    no_errors("deflection", &errs, &c);
    let t = text(&html);
    // EI = 30 GPa x 160000 cm4 = 48000 kN.m2; d = 5 x 30 x 6^4 / (384 x 48000) = 10.55 mm; phi = 30 x 6^3 / (24 x 48000) = 0.005625 (shown as 0.00562).
    assert!(t.contains("= 48000 kN · m 2"), "{t}");
    assert!(t.contains("= 10.55 mm"), "{t}");
    assert!(t.contains("= 0.00562"), "{t}");
    assert!(html.contains("<svg"), "the deflection curve is drawn");
    let (c, html, errs) = render(&sample("Cantilever tip load"));
    no_errors("cantilever", &errs, &c);
    let t = text(&html);
    // 20 x 3^3 / (3 x 48000) = 3.75 mm; M = 20 x 3 = 60 kN.m; for a load of 3 P: 11.25 mm.
    assert!(
        t.contains("= 3.75 mm") && t.contains("= 60 kN · m") && t.contains("= 11.25 mm"),
        "{t}"
    );
}

#[test]
fn python_fence_becomes_a_python_block() {
    let src = "<!--sheet-->\n# T\n\n```python\nimport numpy as np\nfor i in range(2):\n    export(a=i)\n```\n\nx: 1";
    let c = compile(src);
    let lines: Vec<&str> = c.text.split('\n').collect();
    let at = lines.iter().position(|l| *l == "#python").expect("opens a block");
    assert_eq!(
        &lines[at..at + 5],
        [
            "#python",
            "import numpy as np",
            "for i in range(2):",
            "    export(a=i)",
            "#end python"
        ]
    );
    // every line of the block maps to its own sheet line
    assert_eq!(c.map[at], 3);
    assert_eq!(c.map[at + 1], 4);
    assert_eq!(c.map[at + 3], 6);
    assert_eq!(c.map[at + 4], 7);
    assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics.len());
}

#[test]
fn python_fence_keeps_indentation_relative_to_its_block() {
    let src = "<!--sheet-->\nif 1 > 0:\n    ```python\n    if True:\n        print(1)\n    ```\n";
    let c = compile(src);
    assert!(
        c.text.contains("#python\nif True:\n    print(1)\n#end python"),
        "{}",
        c.text
    );
}

#[test]
fn python_fence_is_not_renamed_or_formatted_as_a_sheet() {
    let src = "<!--sheet-->\nphi: 2\n```python\nphi = 3   # not sheet syntax: a | b\n```\n";
    let c = compile(src);
    assert!(c.text.contains("\nphi = 3   # not sheet syntax: a | b\n"), "{}", c.text);
    assert_eq!(crate::format::format(src), src);
}

#[test]
fn an_unclosed_python_fence_is_an_error_and_is_closed() {
    let c = compile("<!--sheet-->\n```python\nx = 1\n");
    assert!(c.text.trim_end().ends_with("#end python"));
    assert!(c.diagnostics.iter().any(|d| matches!(d.severity, Severity::Error)));
}

#[test]
fn a_plain_code_fence_is_still_prose() {
    let c = compile("<!--sheet-->\n```rust\nlet a = 1;\n```\n");
    assert!(!c.text.contains("#python"), "{}", c.text);
}

fn python_body(src: &str) -> String {
    let c = compile(src);
    let t = c.text;
    let a = t.find("#python\n").expect("a python block") + "#python\n".len();
    let b = t.find("\n#end python").expect("closed");
    t[a..b].to_string()
}

#[test]
fn export_keywords_use_the_sheet_spelling() {
    let body = python_body("<!--sheet-->\n```python\nexport(sigma_c=2.0, phi=3.0, Mstar=4.0, y_c=5.0, plain=6.0)\n```\nx: 1");
    assert_eq!(body, "export(σ_c=2.0, φ=3.0, M_star=4.0, y_c=5.0, plain=6.0)");
}

#[test]
fn export_names_follow_across_lines_and_export_data() {
    let body = python_body("<!--sheet-->\n```python\nexport_data(\n    sigma_c=1,\n    phi=2)\nphi = 3\n```\nx: 1");
    assert_eq!(body, "export_data(\n    σ_c=1,\n    φ=2)\nphi = 3");
}

#[test]
fn export_positional_name_string_is_renamed() {
    let body = python_body("<!--sheet-->\n```python\nexport(\"sigma_c\", 2.0)\n```\nx: 1");
    assert_eq!(body, "export(\"σ_c\", 2.0)");
}

#[test]
fn only_the_export_call_keywords_are_renamed() {
    let src = "<!--sheet-->\n```python\nphi = 3\nsigma_c = f(phi=1)  # export(phi=1)\nexport(a=g(phi=2), s=\"phi=3\")\nobj.export(phi=1)\nmyexport(phi=1)\n```\nx: 1";
    let body = python_body(src);
    assert_eq!(
        body,
        "phi = 3\nsigma_c = f(phi=1)  # export(phi=1)\nexport(a=g(phi=2), s=\"phi=3\")\nobj.export(phi=1)\nmyexport(phi=1)"
    );
}

#[test]
fn a_name_exported_from_python_is_defined_for_the_lines_after_it() {
    let c = compile("<!--sheet-->\n```python\nexport(sigma_c=2.0)\n```\na: sigma_c * 2\n");
    assert!(c.text.contains("σ_c=2.0"), "{}", c.text);
    assert!(c.text.contains("σ_c"), "{}", c.text);
}

#[test]
fn a_bare_python_block_in_a_sheet_is_a_python_block() {
    let src = "<!--sheet-->\n#python\nphi = 3   # not sheet syntax: a | b\nexport(sigma_c=phi)\n#end python\nx: sigma_c * 2\n";
    let c = compile(src);
    assert!(c.text.contains("#python\nphi = 3   # not sheet syntax: a | b\nexport(σ_c=phi)\n#end python"), "{}", c.text);
    assert!(!c.text.contains("#markdown"), "{}", c.text);
    assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics);
    assert_eq!(crate::format::format(src), src);
}

#[test]
fn an_unclosed_bare_python_block_is_an_error_and_is_closed() {
    let c = compile("<!--sheet-->\n#python\nx = 1\n");
    assert!(c.text.trim_end().ends_with("#end python"), "{:?}", c.text);
    assert!(!c.diagnostics.is_empty());
}
