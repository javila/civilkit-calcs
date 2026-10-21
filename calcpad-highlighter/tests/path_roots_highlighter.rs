//! PathRootsHighlighterTests.cs
//! The include tests that C# runs against a real temp folder use `NativeDisk` here for the folder
//! checks; the include file contents still come from the in-memory map keyed by resolved path.
use calcpad_highlighter::content_resolution::{ContentResolver, NativeDisk};
use calcpad_highlighter::text::StrMap;
use calcpad_highlighter::tokenizer::models::TokenType;
use calcpad_highlighter::tokenizer::CalcpadTokenizer;
use std::path::{Path, PathBuf};

fn source_file_path() -> String {
    std::env::temp_dir()
        .join("project")
        .join("main.cpd")
        .to_string_lossy()
        .into_owned()
}

/// A fresh empty folder under the temp dir, removed on drop (Path.GetRandomFileName + Delete).
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("calcpad-hl-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn full(base: &Path, rel: &str) -> String {
    base.join(rel).to_string_lossy().into_owned()
}

#[test]
fn declaration_tokenizes_value_as_single_token() {
    let result = CalcpadTokenizer::new().tokenize("#ProjectPath C:/Jobs/1042");
    let path_root_tokens: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.token_type == TokenType::FilePath)
        .collect();
    assert_eq!(path_root_tokens.len(), 1);
    assert_eq!(path_root_tokens[0].text, "C:/Jobs/1042");
}

#[test]
fn declaration_is_recognized_as_keyword() {
    let result = CalcpadTokenizer::new().tokenize("#LibraryPath C:/Lib");
    let keyword_tokens: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Keyword)
        .collect();
    assert_eq!(keyword_tokens.len(), 1);
    assert_eq!(keyword_tokens[0].text, "#LibraryPath");
}

#[test]
fn include_with_library_token_resolves_against_declared_root() {
    let temp = TempDir::new("lib");
    let lib_dir = temp.path().join("lib");
    std::fs::create_dir_all(&lib_dir).unwrap();
    let resolved_library_file = full(temp.path(), "lib/steel.cpd");
    let content = format!(
        "#LibraryPath {}\n#include {{library}}/steel.cpd\nx = 1",
        lib_dir.display()
    );
    let mut include_files: StrMap<String> = StrMap::new();
    include_files.insert(&resolved_library_file, "y = 2".to_string());

    let staged = ContentResolver::with_provider(Box::new(NativeDisk)).get_staged_content(
        &content,
        Some(&include_files),
        None,
        Some(&full(temp.path(), "main.cpd")),
    );
    let joined = staged.stage2.lines.join("\n");

    assert!(!joined.contains("Error: Include file not provided"), "{joined}");
    assert!(joined.contains("y = 2"), "{joined}");
}

#[test]
fn include_with_undeclared_token_falls_back_to_not_found() {
    let content = "#include {library}/steel.cpd\nx = 1";
    let staged = ContentResolver::new().get_staged_content(content, None, None, Some(&source_file_path()));
    let joined = staged.stage2.lines.join("\n");

    assert!(joined.contains("Error: Include file not provided"), "{joined}");
}

#[test]
fn stage2_path_roots_declared_inside_include_is_populated_and_usable_by_parent() {
    let temp = TempDir::new("decl");
    let lib_dir = temp.path().join("lib");
    std::fs::create_dir_all(&lib_dir).unwrap();
    let resolved_decl_file = full(temp.path(), "decl.cpd");
    let resolved_steel_file = full(&lib_dir, "steel.cpd");
    let content = "#include decl.cpd\n#include {library}/steel.cpd\nx = 1";
    let mut include_files: StrMap<String> = StrMap::new();
    include_files.insert(&resolved_decl_file, format!("#LibraryPath {}", lib_dir.display()));
    include_files.insert(&resolved_steel_file, "y = 2".to_string());

    let staged = ContentResolver::with_provider(Box::new(NativeDisk)).get_staged_content(
        content,
        Some(&include_files),
        None,
        Some(&full(temp.path(), "main.cpd")),
    );
    let joined = staged.stage2.lines.join("\n");

    assert!(!joined.contains("Error: Include file not provided"), "{joined}");
    assert!(joined.contains("y = 2"), "{joined}");
    assert_eq!(
        staged.stage2.path_roots.library(),
        Some(lib_dir.to_string_lossy().as_ref())
    );
}

#[test]
fn include_with_user_token_resolves_with_no_declaration() {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap();
    let resolved_file = full(Path::new(&home), "lib/steel.cpd");
    let content = "#include {user}/lib/steel.cpd\nx = 1";
    let mut include_files: StrMap<String> = StrMap::new();
    include_files.insert(&resolved_file, "y = 2".to_string());

    let staged = ContentResolver::with_provider(Box::new(NativeDisk)).get_staged_content(
        content,
        Some(&include_files),
        None,
        Some(&source_file_path()),
    );
    let joined = staged.stage2.lines.join("\n");

    assert!(!joined.contains("Error: Include file not provided"), "{joined}");
    assert!(joined.contains("y = 2"), "{joined}");
}
