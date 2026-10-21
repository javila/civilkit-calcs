//! Lints every .cpd under a folder (includes from disk) and prints the diagnostics.
//!
//!   cargo run -p calcpad-highlighter --example lint_dir -- <folder> [--list]
use calcpad_highlighter::content_resolution::{ContentResolver, NativeDisk};
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("cpd") {
            out.push(p);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = PathBuf::from(&args[1]).canonicalize().unwrap();
    let list = args.iter().any(|a| a == "--list");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut by_code: BTreeMap<String, (String, usize, usize)> = BTreeMap::new();
    let mut files_with: BTreeMap<String, usize> = BTreeMap::new();
    for f in &files {
        let content = std::fs::read_to_string(f).unwrap();
        let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content).to_string();
        let resolver = ContentResolver::with_provider(Box::new(NativeDisk));
        let staged = resolver.get_staged_content(&content, None, None, f.to_str());
        let regions = LintIgnoreRegionParser::extract_regions(&content);
        let result = CalcpadLinter::new().lint(&staged, &regions);
        let mut seen = std::collections::BTreeSet::new();
        for d in &result.diagnostics {
            let e = by_code
                .entry(d.code.clone())
                .or_insert((format!("{:?}", d.severity), 0, 0));
            e.1 += 1;
            if seen.insert(d.code.clone()) {
                *files_with.entry(d.code.clone()).or_default() += 1;
            }
            if list {
                let rel = f.strip_prefix(&root).unwrap().display();
                println!("{rel}:{}:{} [{}] {}", d.line + 1, d.column + 1, d.code, d.message);
            }
        }
    }
    println!("{} files", files.len());
    for (code, (sev, n, _)) in &by_code {
        println!("{code} {sev}: {n} diagnostics in {} files", files_with[code]);
    }
}
