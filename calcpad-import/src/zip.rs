//! A small read-only zip reader for the mcdx importer.
//!
//! The archive is untrusted, so every size is capped before anything is allocated: the entry
//! count, the declared and the actual size of each entry, and the total decoded bytes. Stored and
//! deflated entries are read (through `miniz_oxide`, already a dependency); encryption, zip64 and
//! other compression methods are refused. The central directory is the only index that is trusted,
//! and an entry is looked up by name, never extracted to disk, so a hostile name cannot reach a
//! file system. The CRC is not checked: a damaged entry shows up as a parse error further on.

use std::cell::Cell;

/// Why an archive could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZipError {
    /// Not a zip archive, or its directory is damaged.
    Bad(&'static str),
    /// A feature the reader refuses (encryption, zip64, an unknown compression method).
    Unsupported(&'static str),
    /// A cap was exceeded.
    TooLarge,
}

#[derive(Debug, Clone)]
struct Entry {
    name: String,
    method: u16,
    comp_size: usize,
    size: usize,
    /// Offset of the local header.
    local: usize,
}

/// An opened archive: the central directory plus the bytes.
pub struct Archive<'a> {
    bytes: &'a [u8],
    entries: Vec<Entry>,
    /// Decoded bytes still allowed over the life of this archive.
    budget: Cell<usize>,
}

/// Most entries read from one archive.
const MAX_ENTRIES: usize = 20_000;

fn u16_at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i.checked_add(2)?)?.try_into().ok()?))
}

fn u32_at(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i.checked_add(4)?)?.try_into().ok()?))
}

impl<'a> Archive<'a> {
    /// Opens an archive. `total_budget` caps the decoded bytes handed out by [`Archive::read`].
    pub fn open(bytes: &'a [u8], total_budget: usize) -> Result<Archive<'a>, ZipError> {
        const BAD: ZipError = ZipError::Bad("not a zip archive or its directory is damaged");
        if bytes.len() < 22 {
            return Err(BAD);
        }
        // End of central directory: the last signature within the trailing 64 KiB + 22 bytes.
        let min = bytes.len().saturating_sub(65_557);
        let eocd = (min..=bytes.len() - 22)
            .rev()
            .find(|&i| bytes[i..i + 4] == [0x50, 0x4b, 0x05, 0x06])
            .ok_or(BAD)?;
        let count = u16_at(bytes, eocd + 10).ok_or(BAD)? as usize;
        let cd_size = u32_at(bytes, eocd + 12).ok_or(BAD)?;
        let cd_offset = u32_at(bytes, eocd + 16).ok_or(BAD)?;
        if count == 0xFFFF || cd_size == 0xFFFF_FFFF || cd_offset == 0xFFFF_FFFF {
            return Err(ZipError::Unsupported("zip64 archive"));
        }
        if count > MAX_ENTRIES {
            return Err(ZipError::TooLarge);
        }
        let mut p = cd_offset as usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            if u32_at(bytes, p) != Some(0x0201_4b50) {
                return Err(BAD);
            }
            let flags = u16_at(bytes, p + 8).ok_or(BAD)?;
            let method = u16_at(bytes, p + 10).ok_or(BAD)?;
            let comp_size = u32_at(bytes, p + 20).ok_or(BAD)?;
            let size = u32_at(bytes, p + 24).ok_or(BAD)?;
            let name_len = u16_at(bytes, p + 28).ok_or(BAD)? as usize;
            let extra_len = u16_at(bytes, p + 30).ok_or(BAD)? as usize;
            let comment_len = u16_at(bytes, p + 32).ok_or(BAD)? as usize;
            let local = u32_at(bytes, p + 42).ok_or(BAD)?;
            if flags & 1 != 0 {
                return Err(ZipError::Unsupported("encrypted entry"));
            }
            if comp_size == 0xFFFF_FFFF || size == 0xFFFF_FFFF || local == 0xFFFF_FFFF {
                return Err(ZipError::Unsupported("zip64 entry"));
            }
            let name_bytes = bytes.get(p + 46..p + 46 + name_len).ok_or(BAD)?;
            let name = String::from_utf8_lossy(name_bytes).replace('\\', "/");
            p = p.checked_add(46 + name_len + extra_len + comment_len).ok_or(BAD)?;
            entries.push(Entry {
                name,
                method,
                comp_size: comp_size as usize,
                size: size as usize,
                local: local as usize,
            });
        }
        Ok(Archive {
            bytes,
            entries,
            budget: Cell::new(total_budget),
        })
    }

    /// The entry names, in directory order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.name.as_str())
    }

    /// The name of the first entry matching `pred`.
    pub fn find(&self, pred: impl Fn(&str) -> bool) -> Option<&str> {
        self.names().find(|n| pred(n))
    }

    /// True when an entry has exactly this name (ASCII case-insensitive, leading `/` ignored).
    pub fn contains(&self, name: &str) -> bool {
        self.entry(name).is_some()
    }

    fn entry(&self, name: &str) -> Option<&Entry> {
        let want = name.trim_start_matches('/');
        self.entries
            .iter()
            .find(|e| e.name.trim_start_matches('/').eq_ignore_ascii_case(want))
    }

    /// The decoded bytes of an entry, or `Ok(None)` when there is no such entry. An entry larger
    /// than `max` bytes (declared or actual) or past the archive's total budget is `TooLarge`.
    pub fn read(&self, name: &str, max: usize) -> Result<Option<Vec<u8>>, ZipError> {
        const BAD: ZipError = ZipError::Bad("entry data is damaged");
        let Some(e) = self.entry(name) else {
            return Ok(None);
        };
        if e.size > max || e.size > self.budget.get() {
            return Err(ZipError::TooLarge);
        }
        let b = self.bytes;
        if u32_at(b, e.local) != Some(0x0403_4b50) {
            return Err(BAD);
        }
        let lname = u16_at(b, e.local + 26).ok_or(BAD)? as usize;
        let lextra = u16_at(b, e.local + 28).ok_or(BAD)? as usize;
        let start = e.local.checked_add(30 + lname + lextra).ok_or(BAD)?;
        let end = start.checked_add(e.comp_size).ok_or(BAD)?;
        let raw = b.get(start..end).ok_or(BAD)?;
        let data = match e.method {
            0 => {
                if raw.len() != e.size {
                    return Err(BAD);
                }
                raw.to_vec()
            }
            8 => {
                // The limit is the declared size, so a lying header cannot inflate past it.
                let out = miniz_oxide::inflate::decompress_to_vec_with_limit(raw, e.size).map_err(|_| BAD)?;
                if out.len() != e.size {
                    return Err(BAD);
                }
                out
            }
            _ => return Err(ZipError::Unsupported("compression method")),
        };
        self.budget.set(self.budget.get() - data.len());
        Ok(Some(data))
    }
}
