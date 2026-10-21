//! Calcpad.Core `PathRoots`: the `#ProjectPath` / `#LibraryPath` declarations and the
//! `{project}`, `{library}` and `{user}` tokens of a file path.
//!
//! The C# class talks to `System.IO` and `System.Environment` directly. The browser build has no
//! file system, so every such call goes through [`IncludeProvider`]; [`NoDisk`] (the default
//! everywhere) answers "nothing exists" and [`NativeDisk`] reads the real disk for native tests
//! and tools. `Path.GetFullPath`, `Path.Combine` and `Path.GetDirectoryName` are reimplemented
//! on strings (see [`get_full_path`]).
use crate::text::{is_white_space, trim_start};

const PROJECT_TOKEN: &str = "{project}";
const LIBRARY_TOKEN: &str = "{library}";
const USER_TOKEN: &str = "{user}";
const PROJECT_KEYWORD: &str = "#projectpath";
const LIBRARY_KEYWORD: &str = "#librarypath";

/// The part of the file system and process environment the content resolver and `PathRoots` use.
/// Every method has a "nothing there" default, which is what the browser build gets.
pub trait IncludeProvider {
    /// `File.Exists(path)` followed by `File.ReadAllText(path)`; `None` when either fails.
    fn read_file(&self, _full_path: &str) -> Option<String> {
        None
    }
    /// `Directory.Exists(path)`.
    fn directory_exists(&self, _full_path: &str) -> bool {
        false
    }
    /// `Environment.GetFolderPath(SpecialFolder.UserProfile)`.
    fn user_profile(&self) -> String {
        String::new()
    }
    /// `Directory.GetCurrentDirectory()`; `None` when the host has no working directory, in which
    /// case a relative path without a base directory cannot be resolved.
    fn current_directory(&self) -> Option<String> {
        None
    }
    /// `Environment.GetEnvironmentVariable(name)`.
    fn environment_variable(&self, _name: &str) -> Option<String> {
        None
    }
    /// `Environment.ExpandEnvironmentVariables(s)` (`%NAME%` syntax on every platform).
    fn expand_environment_variables(&self, s: &str) -> String {
        expand_environment_variables(s, &|name| self.environment_variable(name))
    }
}

/// No file system: no file or folder exists, there is no home or working directory.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoDisk;

impl IncludeProvider for NoDisk {}

/// The real disk and process environment (not for the browser build).
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeDisk;

impl IncludeProvider for NativeDisk {
    fn read_file(&self, full_path: &str) -> Option<String> {
        let path = std::path::Path::new(full_path);
        if !path.is_file() {
            return None;
        }
        let bytes = std::fs::read(path).ok()?;
        let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(&bytes);
        Some(String::from_utf8_lossy(bytes).into_owned())
    }
    fn directory_exists(&self, full_path: &str) -> bool {
        std::path::Path::new(full_path).is_dir()
    }
    fn user_profile(&self) -> String {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default()
    }
    fn current_directory(&self) -> Option<String> {
        std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned())
    }
    fn environment_variable(&self, name: &str) -> Option<String> {
        if name.is_empty() || name.contains('=') || name.contains('\0') {
            return None;
        }
        std::env::var(name).ok()
    }
}

/// `Environment.ExpandEnvironmentVariables`: replaces each `%NAME%` whose variable is defined and
/// leaves the others (the closing `%` of an unknown name can open the next one).
pub fn expand_environment_variables(s: &str, lookup: &dyn Fn(&str) -> Option<String>) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut pos = 0;
    while pos < chars.len() {
        let Some(first) = crate::text::index_of_char(&chars, '%', pos) else {
            break;
        };
        let Some(last) = crate::text::index_of_char(&chars, '%', first + 1) else {
            break;
        };
        let key: String = chars[first + 1..last].iter().collect();
        match lookup(&key) {
            Some(value) => {
                out.extend(&chars[pos..first]);
                out.push_str(&value);
                pos = last + 1;
            }
            None => {
                out.extend(&chars[pos..last]);
                pos = last;
            }
        }
    }
    out.extend(&chars[pos..]);
    out
}

/// A drive-letter path (`C:\x`, `C:/x`, `C:`) or a UNC path: the signal that the Windows rules
/// apply (both separators, `\` as the output separator).
pub fn is_windows_rooted(p: &str) -> bool {
    let c: Vec<char> = p.chars().take(3).collect();
    if c.len() >= 2 && c[0].is_ascii_alphabetic() && c[1] == ':' {
        return true;
    }
    c.len() >= 2 && c[0] == '\\' && c[1] == '\\'
}

fn is_sep(c: char, win: bool) -> bool {
    c == '/' || (win && c == '\\')
}

/// `Path.IsPathFullyQualified`.
fn is_fully_qualified(p: &str, win: bool) -> bool {
    let c: Vec<char> = p.chars().take(3).collect();
    if win {
        if c.len() >= 3 && c[0].is_ascii_alphabetic() && c[1] == ':' && is_sep(c[2], true) {
            return true;
        }
        return c.len() >= 2 && c[0] == '\\' && c[1] == '\\';
    }
    c.first() == Some(&'/')
}

/// `Path.IsPathRooted`.
fn is_rooted(p: &str, win: bool) -> bool {
    let c: Vec<char> = p.chars().take(2).collect();
    if win && c.len() >= 2 && c[0].is_ascii_alphabetic() && c[1] == ':' {
        return true;
    }
    c.first().is_some_and(|ch| is_sep(*ch, win))
}

/// `Path.Combine(a, b)`. The separator follows the style of the arguments (Windows when either
/// is a drive-letter or UNC path, otherwise `/`).
pub fn path_combine(a: &str, b: &str) -> String {
    if b.is_empty() {
        return a.to_string();
    }
    if a.is_empty() {
        return b.to_string();
    }
    let win = is_windows_rooted(a) || is_windows_rooted(b);
    if is_rooted(b, win) {
        return b.to_string();
    }
    let last = a.chars().last().unwrap_or('/');
    if is_sep(last, win) || (win && last == ':') {
        format!("{a}{b}")
    } else {
        format!("{a}{}{b}", if win { '\\' } else { '/' })
    }
}

/// `Path.GetFullPath(path)` / `Path.GetFullPath(path, basePath)`: resolves `.` and `..`, collapses
/// repeated separators and keeps a trailing separator. `Err` carries what the C# exception would
/// say (an empty path, a NUL character, a base that is not fully qualified, a relative path with
/// no base and no working directory).
pub fn get_full_path(path: &str, base_path: Option<&str>, current_dir: Option<&str>) -> Result<String, String> {
    if path.is_empty() {
        return Err("The value cannot be an empty string. (Parameter 'path')".to_string());
    }
    if path.contains('\0') {
        return Err("Null character in path.".to_string());
    }
    let win = is_windows_rooted(path) || base_path.is_some_and(is_windows_rooted);
    let combined = if is_fully_qualified(path, win) {
        path.to_string()
    } else {
        let base = match base_path {
            Some(b) => {
                if !is_fully_qualified(b, win) {
                    return Err("Basepath argument is not fully qualified. (Parameter 'basePath')".to_string());
                }
                b.to_string()
            }
            None => match current_dir {
                Some(d) if is_fully_qualified(d, win) => d.to_string(),
                _ => return Err("The current directory is not available.".to_string()),
            },
        };
        if is_rooted(path, win) {
            // Drive-less rooted path: take the base's volume.
            let vol = if win {
                base.chars().take(2).collect::<String>()
            } else {
                String::new()
            };
            format!("{vol}{path}")
        } else {
            path_combine(&base, path)
        }
    };
    Ok(normalize(&combined, win))
}

fn normalize(p: &str, win: bool) -> String {
    let chars: Vec<char> = p.chars().collect();
    let sep = if win { '\\' } else { '/' };
    let (prefix, mut i) = if win && chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
        (format!("{}:{}", chars[0], sep), 2)
    } else if win && chars.len() >= 2 && chars[0] == '\\' && chars[1] == '\\' {
        (format!("{sep}{sep}"), 2)
    } else {
        (sep.to_string(), 0)
    };
    let trailing = chars.last().is_some_and(|c| is_sep(*c, win));
    let mut segments: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut ends_with_dot = false;
    while i <= chars.len() {
        if i == chars.len() || is_sep(chars[i], win) {
            match cur.as_str() {
                "" => {}
                "." => ends_with_dot = true,
                ".." => {
                    segments.pop();
                    ends_with_dot = true;
                }
                _ => {
                    segments.push(std::mem::take(&mut cur));
                    ends_with_dot = false;
                }
            }
            cur.clear();
        } else {
            cur.push(chars[i]);
        }
        i += 1;
    }
    let mut out = prefix;
    out.push_str(&segments.join(&sep.to_string()));
    if trailing && !ends_with_dot && !segments.is_empty() {
        out.push(sep);
    }
    out
}

/// `Path.GetDirectoryName(path)`: `None` for a root (or an empty path), `""` for a bare file name.
pub fn get_directory_name(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let win = is_windows_rooted(path);
    let chars: Vec<char> = path.chars().collect();
    let root_len = if win && chars.len() >= 3 && chars[1] == ':' && is_sep(chars[2], true) {
        3
    } else if win && chars.len() >= 2 && chars[1] == ':' {
        2
    } else if !win && chars[0] == '/' {
        1
    } else {
        0
    };
    if root_len > 0 && chars.len() <= root_len {
        return None;
    }
    let idx = chars.iter().rposition(|c| is_sep(*c, win));
    let Some(idx) = idx else {
        return Some(String::new());
    };
    let mut end = idx;
    while end > root_len && is_sep(chars[end - 1], win) {
        end -= 1;
    }
    if end < root_len {
        end = root_len;
    }
    if end == 0 {
        return Some(String::new());
    }
    Some(chars[..end].iter().collect())
}

fn up(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

/// `span.StartsWith(prefix, StringComparison.OrdinalIgnoreCase)`.
pub fn starts_with_ignore_case(s: &[char], prefix: &str) -> bool {
    let mut it = s.iter();
    prefix.chars().all(|p| it.next().is_some_and(|c| up(*c) == up(p)))
}

/// Result of [`PathRoots::try_expand`]: the (possibly expanded) text, and the error when the
/// expansion failed (the text is then the input, untouched).
pub type Expansion = (String, Option<String>);

/// The `#ProjectPath` and `#LibraryPath` roots declared so far.
#[derive(Clone, Debug, Default)]
pub struct PathRoots {
    project: Option<String>,
    library: Option<String>,
}

impl PathRoots {
    pub fn new() -> PathRoots {
        PathRoots::default()
    }

    /// `PathRoots.Project`.
    pub fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }

    /// `PathRoots.Library`.
    pub fn library(&self) -> Option<&str> {
        self.library.as_deref()
    }

    /// `HasToken(path)`.
    pub fn has_token(path: &[char]) -> bool {
        starts_with_ignore_case(path, PROJECT_TOKEN) || starts_with_ignore_case(path, LIBRARY_TOKEN)
    }

    /// `IsUserToken(path)`.
    pub fn is_user_token(path: &[char]) -> bool {
        starts_with_ignore_case(path, USER_TOKEN)
    }

    fn expand_user_token(raw: &str, fs: &dyn IncludeProvider) -> String {
        let mut rest: Vec<char> = raw.chars().skip(USER_TOKEN.len()).collect();
        if !rest.is_empty() && (rest[0] == '/' || rest[0] == '\\') {
            rest.remove(0);
        }
        let home = fs.user_profile();
        if rest.is_empty() {
            home
        } else {
            path_combine(&home, &rest.iter().collect::<String>())
        }
    }

    /// `TryGetTokenKind(path, out isProject)`: `Some(true)` for `{project}`, `Some(false)` for
    /// `{library}`, `None` when the path starts with neither.
    pub fn try_get_token_kind(path: &[char]) -> Option<bool> {
        if starts_with_ignore_case(path, PROJECT_TOKEN) {
            Some(true)
        } else if starts_with_ignore_case(path, LIBRARY_TOKEN) {
            Some(false)
        } else {
            None
        }
    }

    /// `IsDeclaration(line, out isProject, out start, out length)`: `Some((is_project, start,
    /// length))` where `start`/`length` locate the value inside `line` (a zero length when the
    /// directive has no value).
    pub fn is_declaration(line: &[char]) -> Option<(bool, usize, usize)> {
        let is_project = if starts_with_ignore_case(line, PROJECT_KEYWORD) {
            true
        } else if starts_with_ignore_case(line, LIBRARY_KEYWORD) {
            false
        } else {
            return None;
        };
        let keyword_length = if is_project {
            PROJECT_KEYWORD.len()
        } else {
            LIBRARY_KEYWORD.len()
        };
        let rest = &line[keyword_length..];
        let trimmed_start = rest.len() - trim_start(rest).len();
        let mut value = &rest[trimmed_start..];
        if let Some(ci) = value.iter().position(|c| *c == '\'' || *c == '"') {
            value = &value[..ci];
        }
        let value = crate::text::trim_end(value);
        Some((is_project, keyword_length + trimmed_start, value.len()))
    }

    fn keyword_name(is_project: bool) -> &'static str {
        if is_project {
            "#ProjectPath"
        } else {
            "#LibraryPath"
        }
    }

    /// `TryDeclare(isProject, rawValue, declaringDirectory, out error)`.
    pub fn try_declare(
        &mut self,
        is_project: bool,
        raw_value: &str,
        declaring_directory: Option<&str>,
        fs: &dyn IncludeProvider,
    ) -> Result<(), String> {
        let name = Self::keyword_name(is_project);
        if raw_value.chars().all(is_white_space) {
            return Err(format!("{name} requires a path."));
        }
        let raw_chars: Vec<char> = raw_value.chars().collect();
        let expanded = if Self::is_user_token(&raw_chars) {
            Self::expand_user_token(raw_value, fs)
        } else {
            raw_value.to_string()
        };
        let expanded = fs.expand_environment_variables(&expanded);
        let cwd = fs.current_directory();
        let resolved = match declaring_directory {
            Some(d) if !d.is_empty() => get_full_path(&expanded, Some(d), cwd.as_deref()),
            _ => get_full_path(&expanded, None, cwd.as_deref()),
        }?;
        if !fs.directory_exists(&resolved) {
            return Err(format!("{name} folder not found: {resolved}."));
        }
        let existing = if is_project { &self.project } else { &self.library };
        if let Some(existing) = existing {
            let same = if is_windows_rooted(&resolved) {
                crate::text::eq_ignore_case(existing, &resolved)
            } else {
                *existing == resolved
            };
            return if same {
                Ok(())
            } else {
                Err(format!("{name} is already declared in this document."))
            };
        }
        if is_project {
            self.project = Some(resolved);
        } else {
            self.library = Some(resolved);
        }
        Ok(())
    }

    /// `TryExpand(raw, out expanded, out error)`: the expanded text and the error, if any.
    /// `{user}` needs no declaration; `{project}` / `{library}` fail when undeclared and the text
    /// is then returned as it was.
    pub fn try_expand(&self, raw: &str, fs: &dyn IncludeProvider) -> Expansion {
        let chars: Vec<char> = raw.chars().collect();
        if Self::is_user_token(&chars) {
            return (Self::expand_user_token(raw, fs), None);
        }
        let Some(is_project) = Self::try_get_token_kind(&chars) else {
            return (raw.to_string(), None);
        };
        let token = if is_project { PROJECT_TOKEN } else { LIBRARY_TOKEN };
        let root = if is_project { &self.project } else { &self.library };
        let Some(root) = root else {
            let keyword = Self::keyword_name(is_project);
            return (
                raw.to_string(),
                Some(format!("{token} is not declared. Add {keyword} before this line.")),
            );
        };
        let mut rest: Vec<char> = chars[token.len()..].to_vec();
        if !rest.is_empty() && (rest[0] == '/' || rest[0] == '\\') {
            rest.remove(0);
        }
        let expanded = if rest.is_empty() {
            root.clone()
        } else {
            path_combine(root, &rest.iter().collect::<String>())
        };
        (expanded, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::chars;
    use std::path::PathBuf;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> TempDir {
            let n = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let p = std::env::temp_dir().join(format!("cpd_pr_{tag}_{}_{n}", std::process::id()));
            std::fs::create_dir_all(&p).unwrap();
            // Canonical form so the expectations do not depend on symlinked temp roots.
            TempDir(p.canonicalize().unwrap())
        }
        fn path(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn declaring_dir() -> String {
        std::env::temp_dir().to_string_lossy().into_owned()
    }

    #[test]
    fn try_declare_with_an_empty_value_fails() {
        let mut roots = PathRoots::new();
        let e = roots
            .try_declare(true, "", Some(&declaring_dir()), &NativeDisk)
            .unwrap_err();
        assert!(e.contains("ProjectPath"));
    }

    #[test]
    fn try_declare_twice_with_the_same_value_is_a_no_op() {
        let t = TempDir::new("same");
        let mut roots = PathRoots::new();
        assert!(roots
            .try_declare(true, &t.path(), Some(&declaring_dir()), &NativeDisk)
            .is_ok());
        assert!(roots
            .try_declare(true, &t.path(), Some(&declaring_dir()), &NativeDisk)
            .is_ok());
    }

    #[test]
    fn try_declare_twice_with_a_different_value_fails() {
        let (t1, t2) = (TempDir::new("d1"), TempDir::new("d2"));
        let mut roots = PathRoots::new();
        assert!(roots
            .try_declare(true, &t1.path(), Some(&declaring_dir()), &NativeDisk)
            .is_ok());
        let e = roots
            .try_declare(true, &t2.path(), Some(&declaring_dir()), &NativeDisk)
            .unwrap_err();
        assert!(e.contains("ProjectPath"));
    }

    #[test]
    fn try_declare_with_a_missing_folder_fails() {
        let mut roots = PathRoots::new();
        let e = roots
            .try_declare(true, "no/such/folder", Some(&declaring_dir()), &NativeDisk)
            .unwrap_err();
        assert!(e.contains("ProjectPath"));
    }

    #[test]
    fn try_declare_without_a_disk_never_finds_the_folder() {
        let mut roots = PathRoots::new();
        let e = roots.try_declare(false, "/lib", Some("/project"), &NoDisk).unwrap_err();
        assert_eq!(e, "#LibraryPath folder not found: /lib.");
    }

    #[test]
    fn try_expand_with_an_undeclared_root_fails_but_leaves_the_token_in_place() {
        let roots = PathRoots::new();
        let (expanded, error) = roots.try_expand("{library}/steel.cpd", &NoDisk);
        assert_eq!(expanded, "{library}/steel.cpd");
        assert!(error.unwrap().contains("LibraryPath"));
    }

    #[test]
    fn try_expand_with_a_declared_root_joins_the_remainder() {
        let t = TempDir::new("exp");
        let mut roots = PathRoots::new();
        roots
            .try_declare(false, &t.path(), Some(&declaring_dir()), &NativeDisk)
            .unwrap();
        let (expanded, error) = roots.try_expand("{library}/steel.cpd", &NativeDisk);
        assert!(error.is_none());
        assert_eq!(expanded, path_combine(&t.path(), "steel.cpd"));
        assert_eq!(roots.library(), Some(t.path().as_str()));
    }

    #[test]
    fn try_expand_a_non_token_path_is_left_untouched() {
        let roots = PathRoots::new();
        let (expanded, error) = roots.try_expand("./data/loads.csv", &NoDisk);
        assert_eq!(expanded, "./data/loads.csv");
        assert!(error.is_none());
    }

    #[test]
    fn is_declaration_locates_the_value() {
        for (line, project, value) in [
            ("#ProjectPath C:/Jobs/1042", true, "C:/Jobs/1042"),
            ("#LibraryPath C:/Lib", false, "C:/Lib"),
            ("#projectpath ./rel", true, "./rel"),
        ] {
            let l = chars(line);
            let (is_project, start, length) = PathRoots::is_declaration(&l).unwrap();
            assert_eq!(is_project, project);
            assert_eq!(crate::text::sub(&l, start, length), value);
        }
    }

    #[test]
    fn is_declaration_with_no_value_reports_a_zero_length_value() {
        let (_, _, length) = PathRoots::is_declaration(&chars("#ProjectPath")).unwrap();
        assert_eq!(length, 0);
        assert!(PathRoots::is_declaration(&chars("x = 1")).is_none());
    }

    #[test]
    fn is_declaration_with_a_trailing_comment_stops_before_it() {
        let l = chars("#LibraryPath /lib 'the shared library");
        let (_, start, length) = PathRoots::is_declaration(&l).unwrap();
        assert_eq!(crate::text::sub(&l, start, length), "/lib");
    }

    #[test]
    fn user_token_joins_the_home_directory_and_needs_no_declaration() {
        struct Home;
        impl IncludeProvider for Home {
            fn user_profile(&self) -> String {
                "/home/u".to_string()
            }
        }
        let roots = PathRoots::new();
        assert_eq!(
            roots.try_expand("{user}/lib/steel.cpd", &Home),
            ("/home/u/lib/steel.cpd".to_string(), None)
        );
        assert_eq!(roots.try_expand("{USER}", &Home), ("/home/u".to_string(), None));
    }

    #[test]
    fn token_helpers() {
        assert!(PathRoots::has_token(&chars("{Project}/x")));
        assert!(!PathRoots::has_token(&chars("{user}/x")));
        assert!(PathRoots::is_user_token(&chars("{user}")));
        assert_eq!(PathRoots::try_get_token_kind(&chars("{library}")), Some(false));
        assert_eq!(PathRoots::try_get_token_kind(&chars("lib")), None);
    }

    #[test]
    fn full_path_resolution() {
        assert_eq!(
            get_full_path("a/../b/./c.cpd", Some("/x/y"), None).unwrap(),
            "/x/y/b/c.cpd"
        );
        assert_eq!(get_full_path("/a//b/", None, None).unwrap(), "/a/b/");
        assert_eq!(get_full_path("/a/b/..", None, None).unwrap(), "/a");
        assert_eq!(get_full_path("../../..", Some("/a"), None).unwrap(), "/");
        assert!(get_full_path("a", Some("rel"), None).is_err());
        assert!(get_full_path("a", Some(""), None).is_err());
        assert!(get_full_path("a", None, None).is_err());
        assert_eq!(get_full_path("a", None, Some("/cwd")).unwrap(), "/cwd/a");
        assert_eq!(
            get_full_path("sub\\f.cpd", Some("C:\\proj"), None).unwrap(),
            "C:\\proj\\sub\\f.cpd"
        );
        assert_eq!(get_full_path("C:/Jobs/../x", None, None).unwrap(), "C:\\x");
    }

    #[test]
    fn directory_name_and_combine() {
        assert_eq!(get_directory_name("/a/b/c.cpd").as_deref(), Some("/a/b"));
        assert_eq!(get_directory_name("/c.cpd").as_deref(), Some("/"));
        assert_eq!(get_directory_name("c.cpd").as_deref(), Some(""));
        assert_eq!(get_directory_name("/"), None);
        assert_eq!(get_directory_name("C:\\a\\b.cpd").as_deref(), Some("C:\\a"));
        assert_eq!(path_combine("/a", "b"), "/a/b");
        assert_eq!(path_combine("/a/", "b"), "/a/b");
        assert_eq!(path_combine("/a", "/b"), "/b");
        assert_eq!(path_combine("", "b"), "b");
    }

    #[test]
    fn environment_variables_expand_like_dotnet() {
        let lookup = |n: &str| if n == "DIR" { Some("/d".to_string()) } else { None };
        assert_eq!(expand_environment_variables("%DIR%/x", &lookup), "/d/x");
        assert_eq!(expand_environment_variables("%NOPE%/x", &lookup), "%NOPE%/x");
        assert_eq!(expand_environment_variables("%NOPE%DIR%/x", &lookup), "%NOPE/d/x");
        assert_eq!(expand_environment_variables("100%", &lookup), "100%");
    }
}
