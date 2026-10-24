//! Prints the Calcpad text and the report for one `.xmcd` or `.mcdx` file.
//!
//! ```text
//! cargo run -p civilkit-calcs-import-xmcd --example import -- <file>
//! ```
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: import <file>");
        return ExitCode::from(2);
    };
    let bytes = match std::fs::read(&path) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("cannot read {}: {}", path, e);
            return ExitCode::from(1);
        }
    };
    let r = calcpad_import_xmcd::import_xmcd_bytes(&bytes, &calcpad_import_xmcd::ImportOptions::default());
    print!("{}", r.calcpad);
    for a in &r.assets {
        eprintln!("asset {} ({}, {} bytes)", a.path, a.mime, a.bytes.len());
    }
    eprintln!("---- report ----");
    eprint!("{}", r.report.to_text());
    if r.report.is_fatal() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
