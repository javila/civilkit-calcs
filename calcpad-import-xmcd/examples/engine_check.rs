//! Imports one `.xmcd` or `.mcdx` file, runs the Calcpad text through the engine and writes the
//! text and the engine's HTML next to each other, for checking converted lines against saved results.
//!
//! ```text
//! cargo run --release -p civilkit-calcs-import-xmcd --example engine_check -- <file> <out-prefix> [probe-file]
//! ```
//! A probe file holds extra Calcpad lines appended to the converted text before the engine runs.
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
    let r = calcpad_import_xmcd::import_xmcd_bytes(&bytes, &calcpad_import_xmcd::ImportOptions::default());
    let mut code = r.calcpad.clone();
    if let Some(p) = args.get(3) {
        code.push_str(&std::fs::read_to_string(p).unwrap_or_default());
    }
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&code, true);
    let _ = std::fs::write(format!("{}.cpd", args[2]), &code);
    let _ = std::fs::write(format!("{}.html", args[2]), &parser.html_result);
    ExitCode::SUCCESS
}
