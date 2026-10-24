//! Imports one `.xmcd` or `.mcdx` file, runs the Calcpad text through the engine and writes the
//! text and the engine's HTML next to each other, for checking converted lines against saved results.
//!
//! ```text
//! cargo run --release -p civilkit-calcs-import-xmcd --example engine_check -- <file> <out-prefix> [probe-file]
//! ```
//! A probe file holds extra Calcpad lines appended to the converted text before the engine runs.
//!
//! For an `.mcdx` file the sheet is first extended so that every math region with a saved scalar
//! result is announced by a marker line (`@@<region id>@@`) and shown in SI base units, and a
//! definition is followed by a line that evaluates the defined name. `<out-prefix>.pairs` lists
//! `id`, saved value and the unit expression, so a script can compare the engine's number with the
//! saved one. The engine prints 12 decimals.
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: engine_check <file> <out-prefix> [probe-file]");
        return ExitCode::from(2);
    }
    let bytes = match std::fs::read(&args[1]) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read {}: {}", args[1], e);
            return ExitCode::from(1);
        }
    };
    let opts = calcpad_import_xmcd::ImportOptions::default();
    let mut pairs = String::new();
    let r = if bytes.starts_with(b"PK") {
        match calcpad_import_xmcd::mcdx::read(&bytes, opts.limits) {
            Ok(mut sheet) => {
                calcpad_import_xmcd::translate::reading_order(&mut sheet.regions);
                sheet.regions = with_probes(sheet.regions, &mut pairs);
                calcpad_import_xmcd::import_sheet_opts(sheet, &opts)
            }
            Err(_) => calcpad_import_xmcd::import_xmcd_bytes(&bytes, &opts),
        }
    } else {
        calcpad_import_xmcd::import_xmcd_bytes(&bytes, &opts)
    };
    let mut code = format!("#round 12\n{}", r.calcpad);
    let _ = std::fs::write(format!("{}.pairs", args[2]), &pairs);
    if let Some(p) = args.get(3) {
        code.push_str(&std::fs::read_to_string(p).unwrap_or_default());
    }
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&code, true);
    let _ = std::fs::write(format!("{}.cpd", args[2]), &code);
    let _ = std::fs::write(format!("{}.html", args[2]), &parser.html_result);
    ExitCode::SUCCESS
}

use calcpad_import_xmcd::ir::{BinOp, Expr, Name, Paragraph, Piece, Region, RegionKind, Run, Stmt, Target};

/// The unit of a saved result (file spelling: `kilogram/(meter^3)`) as an expression over Calcpad names.
fn unit_expr(u: &str) -> Option<Expr> {
    let base = |n: &str| -> Option<&'static str> {
        Some(match n {
            "kilogram" => "kg",
            "meter" => "m",
            "second" => "s",
            "ampere" => "A",
            "kelvin" => "K",
            "mole" => "mol",
            "candela" => "cd",
            _ => return None,
        })
    };
    let term = |t: &str| -> Option<Expr> {
        let t = t.trim().trim_matches(|c| c == '(' || c == ')');
        match t.split_once('^') {
            Some((n, p)) => {
                let p = p.trim_matches(|c| c == '(' || c == ')');
                Some(Expr::Bin(
                    BinOp::Pow,
                    Box::new(Expr::Name(Name::new(base(n)?))),
                    Box::new(Expr::Num(p.to_string())),
                ))
            }
            None => Some(Expr::Name(Name::new(base(t)?))),
        }
    };
    let product = |s: &str| -> Option<Expr> {
        let mut it = s.split('*').map(&term);
        let mut acc = it.next()??;
        for t in it {
            acc = Expr::Bin(BinOp::Mul, Box::new(acc), Box::new(t?));
        }
        Some(acc)
    };
    match u.split_once('/') {
        None => product(u),
        Some((a, b)) => {
            let num = if a.trim() == "1" {
                Expr::Num("1".into())
            } else {
                product(a)?
            };
            Some(Expr::Bin(BinOp::Div, Box::new(num), Box::new(product(b)?)))
        }
    }
}

fn marker(r: &Region, text: String) -> Region {
    Region {
        kind: RegionKind::Text(vec![Paragraph {
            style: "Normal".into(),
            pieces: vec![Piece::Run(Run { text, ..Run::default() })],
        }]),
        rendering: None,
        ..r.clone()
    }
}

fn with_probes(regions: Vec<Region>, pairs: &mut String) -> Vec<Region> {
    let mut out = Vec::new();
    for r in regions {
        let RegionKind::Math(m) = &r.kind else {
            out.push(r);
            continue;
        };
        let (Some(sr), false) = (&m.saved_result, m.disabled) else {
            out.push(r);
            continue;
        };
        let unit = match &sr.unit {
            Some(u) => match unit_expr(u) {
                Some(e) => Some(e),
                None => {
                    out.push(r);
                    continue;
                }
            },
            None => None,
        };
        let mut m2 = m.clone();
        match &m.stmt {
            Stmt::Eval { expr, .. } => {
                m2.stmt = Stmt::Eval {
                    expr: expr.clone(),
                    unit: unit.clone(),
                };
                pairs.push_str(&format!(
                    "{}\t{}\t{}\teval\n",
                    r.id,
                    sr.value,
                    sr.unit.clone().unwrap_or_default()
                ));
                out.push(marker(&r, format!("@@{}@@", r.id)));
                out.push(Region {
                    kind: RegionKind::Math(m2),
                    ..r
                });
            }
            Stmt::Define {
                target: Target::Var(n), ..
            } => {
                pairs.push_str(&format!(
                    "{}\t{}\t{}\tdefine\n",
                    r.id,
                    sr.value,
                    sr.unit.clone().unwrap_or_default()
                ));
                let probe = Region {
                    kind: RegionKind::Math(calcpad_import_xmcd::ir::MathRegion {
                        stmt: Stmt::Eval {
                            expr: Expr::Name(n.clone()),
                            unit,
                        },
                        disabled: false,
                        saved_result: None,
                    }),
                    ..r.clone()
                };
                out.push(r.clone());
                out.push(marker(&r, format!("@@{}@@", r.id)));
                out.push(probe);
            }
            _ => out.push(r),
        }
    }
    out
}
