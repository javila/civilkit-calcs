//! HighlighterLinterFixture.cs
#![allow(dead_code)]
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};
use calcpad_highlighter::text::StrMap;
use std::path::{Path, PathBuf};

pub fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("cpd") {
            out.push(p);
        }
    }
}

pub fn cpd_files(sub: &str) -> Vec<PathBuf> {
    let mut v = Vec::new();
    collect(&base_dir().join(sub), &mut v);
    v
}

/// Every .cpd under the data folder by file name (ignore case), as the fixture builds it.
pub fn include_files() -> StrMap<String> {
    let mut m = StrMap::new_ci();
    for p in cpd_files("") {
        m.insert(
            p.file_name().unwrap().to_str().unwrap(),
            std::fs::read_to_string(&p).unwrap(),
        );
    }
    m
}

pub fn lint_file(path: &Path) -> LinterResult {
    let content = std::fs::read_to_string(path).unwrap();
    lint_source(&content, &include_files())
}

pub fn lint_source(content: &str, includes: &StrMap<String>) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, Some(includes), None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}
