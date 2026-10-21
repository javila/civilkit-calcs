//! Where `#read`, `#write` and `#append` get and put their files.
//!
//! CalcpadCE uses System.IO directly. Here the expression parser goes through `DataFiles`, so
//! a host without a file system (the browser) can serve files the user picked and collect the
//! written ones. Native builds default to `StdFiles`, which behaves like System.IO.
use std::rc::Rc;

pub trait DataFiles {
    /// `Path.GetFullPath(path, baseDir)`, or relative to the current directory without one.
    fn full_path(&self, path: &str, base_dir: Option<&str>) -> String;
    /// `Path.GetDirectoryName`
    fn directory_name(&self, path: &str) -> Option<String>;
    /// `Environment.ExpandEnvironmentVariables`
    fn expand_environment_variables(&self, s: &str) -> String {
        s.to_string()
    }
    /// The file's bytes, or None when it does not exist.
    fn read(&self, full_path: &str) -> Option<Vec<u8>>;
    fn dir_exists(&self, dir: &str) -> bool;
    /// Creates or replaces the file.
    fn write(&self, full_path: &str, data: &[u8]) -> Result<(), String>;
    /// Adds to the end of the file, creating it when missing.
    fn append(&self, full_path: &str, data: &[u8]) -> Result<(), String> {
        let mut all = self.read(full_path).unwrap_or_default();
        all.extend_from_slice(data);
        self.write(full_path, &all)
    }
}

pub type DataFilesRef = Rc<dyn DataFiles>;

/// No files at all: every read reports the file as missing and writes fail.
pub struct NoFiles;

impl DataFiles for NoFiles {
    fn full_path(&self, path: &str, base_dir: Option<&str>) -> String {
        normalize_join(path, base_dir)
    }
    fn directory_name(&self, path: &str) -> Option<String> {
        posix_directory_name(path)
    }
    fn read(&self, _full_path: &str) -> Option<Vec<u8>> {
        None
    }
    fn dir_exists(&self, _dir: &str) -> bool {
        false
    }
    fn write(&self, full_path: &str, _data: &[u8]) -> Result<(), String> {
        Err(format!("Could not find a part of the path '{}'.", full_path))
    }
}

/// Joins and normalizes forward slash paths (a virtual file system rooted at "/").
pub fn normalize_join(path: &str, base_dir: Option<&str>) -> String {
    let p = path.replace('\\', "/");
    let joined = if p.starts_with('/') {
        p
    } else {
        format!("{}/{}", base_dir.unwrap_or("/").trim_end_matches('/'), p)
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("/{}", parts.join("/"))
}

pub fn posix_directory_name(path: &str) -> Option<String> {
    let p = path.replace('\\', "/");
    let i = p.rfind('/')?;
    Some(if i == 0 { "/".to_string() } else { p[..i].to_string() })
}

/// System.IO on the local disk.
#[cfg(not(target_arch = "wasm32"))]
pub struct StdFiles;

#[cfg(not(target_arch = "wasm32"))]
impl DataFiles for StdFiles {
    fn full_path(&self, path: &str, base_dir: Option<&str>) -> String {
        use std::path::{Component, Path, PathBuf};
        let p = Path::new(path);
        let joined = if p.is_absolute() {
            p.to_path_buf()
        } else {
            match base_dir {
                Some(b) => Path::new(b).join(p),
                None => std::env::current_dir().unwrap_or_default().join(p),
            }
        };
        let mut out = PathBuf::new();
        for c in joined.components() {
            match c {
                Component::ParentDir => {
                    out.pop();
                }
                Component::CurDir => {}
                c => out.push(c.as_os_str()),
            }
        }
        out.to_string_lossy().to_string()
    }

    fn directory_name(&self, path: &str) -> Option<String> {
        std::path::Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
    }

    fn expand_environment_variables(&self, s: &str) -> String {
        let mut out = String::new();
        let mut rest = s;
        while let Some(i) = rest.find('%') {
            let after = &rest[i + 1..];
            match after.find('%') {
                Some(j) if j > 0 => match std::env::var(&after[..j]) {
                    Ok(v) => {
                        out.push_str(&rest[..i]);
                        out.push_str(&v);
                        rest = &after[j + 1..];
                    }
                    Err(_) => {
                        out.push_str(&rest[..=i]);
                        rest = after;
                    }
                },
                _ => {
                    out.push_str(&rest[..=i]);
                    rest = after;
                }
            }
        }
        out.push_str(rest);
        out
    }

    fn read(&self, full_path: &str) -> Option<Vec<u8>> {
        let p = std::path::Path::new(full_path);
        if !p.is_file() {
            return None;
        }
        std::fs::read(p).ok()
    }

    fn dir_exists(&self, dir: &str) -> bool {
        std::path::Path::new(dir).is_dir()
    }

    fn write(&self, full_path: &str, data: &[u8]) -> Result<(), String> {
        std::fs::write(full_path, data).map_err(|e| e.to_string())
    }

    fn append(&self, full_path: &str, data: &[u8]) -> Result<(), String> {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(full_path)
            .map_err(|e| e.to_string())?;
        f.write_all(data).map_err(|e| e.to_string())
    }
}

/// The default for a new parser: the disk natively, nothing in WebAssembly.
pub fn default_files() -> DataFilesRef {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(StdFiles)
    }
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(NoFiles)
    }
}
