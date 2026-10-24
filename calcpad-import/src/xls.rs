//! Reads the numbers in a range of a spreadsheet component's embedded workbook.
//!
//! The workbook is an old binary spreadsheet in a compound file (the format of Excel 97 to 2003):
//! a small file system of sectors holding a `Workbook` stream of records. Only what a range of
//! numbers needs is read: the sheet names, and the numeric cells (`NUMBER`, `RK`, `MULRK` and the
//! cached numeric result of a `FORMULA`). A range that holds anything else (text, an empty cell, a
//! formula whose result is not a number) is not read at all: the caller gets `None`, never a
//! guess. The bytes are untrusted: every length is checked and every chain is bounded.

#![allow(clippy::chunks_exact_to_as_chunks)]

/// Largest workbook read, in bytes.
pub const MAX_BYTES: usize = 16 * 1024 * 1024;

fn u16_at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i.checked_add(2)?)?.try_into().ok()?))
}
fn u32_at(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i.checked_add(4)?)?.try_into().ok()?))
}
fn f64_at(b: &[u8], i: usize) -> Option<f64> {
    Some(f64::from_le_bytes(b.get(i..i.checked_add(8)?)?.try_into().ok()?))
}

/// The bytes of a stream of a compound file, by name.
fn cfb_stream(file: &[u8], wanted: &[&str]) -> Option<Vec<u8>> {
    if file.len() < 512 || file.len() > MAX_BYTES || file[..8] != [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
        return None;
    }
    let shift = u16_at(file, 0x1E)? as u32;
    let mini_shift = u16_at(file, 0x20)? as u32;
    if !(7..=12).contains(&shift) || mini_shift > shift {
        return None;
    }
    let ssize = 1usize << shift;
    let msize = 1usize << mini_shift;
    let sector = |id: u32| -> Option<&[u8]> {
        let start = (id as usize).checked_add(1)?.checked_mul(ssize)?;
        file.get(start..start.checked_add(ssize)?)
    };
    let nsectors = file.len() / ssize;
    // The allocation table: sector ids listed in the header, then in the extra table sectors.
    let mut fat_ids: Vec<u32> = Vec::new();
    for k in 0..109 {
        let v = u32_at(file, 0x4C + 4 * k)?;
        if v < 0xFFFF_FFFA {
            fat_ids.push(v);
        }
    }
    let mut difat = u32_at(file, 0x44)?;
    let mut guard = 0;
    while difat < 0xFFFF_FFFA && guard < nsectors {
        let s = sector(difat)?;
        for k in 0..(ssize / 4 - 1) {
            let v = u32_at(s, 4 * k)?;
            if v < 0xFFFF_FFFA {
                fat_ids.push(v);
            }
        }
        difat = u32_at(s, ssize - 4)?;
        guard += 1;
    }
    let mut fat: Vec<u32> = Vec::new();
    for id in fat_ids {
        let s = sector(id)?;
        for k in 0..ssize / 4 {
            fat.push(u32_at(s, 4 * k)?);
        }
    }
    let chain = |start: u32, fat: &[u32]| -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut id = start;
        let mut steps = 0;
        while id < 0xFFFF_FFFA {
            out.extend_from_slice(sector(id)?);
            id = *fat.get(id as usize)?;
            steps += 1;
            if steps > nsectors + 1 || out.len() > MAX_BYTES {
                return None;
            }
        }
        Some(out)
    };
    // The directory: 128-byte entries.
    let dir = chain(u32_at(file, 0x30)?, &fat)?;
    let mut root_start = None;
    let mut found: Option<(u32, usize)> = None;
    for e in dir.chunks_exact(128) {
        let name_len = u16_at(e, 0x40)? as usize;
        if !(2..=64).contains(&name_len) {
            continue;
        }
        let units: Vec<u16> = e[..name_len - 2]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let name = String::from_utf16_lossy(&units);
        let kind = e[0x42];
        let start = u32_at(e, 0x74)?;
        let size = u32_at(e, 0x78)? as usize;
        if kind == 5 {
            root_start = Some(start);
        } else if kind == 2 && found.is_none() && wanted.iter().any(|w| w.eq_ignore_ascii_case(&name)) {
            found = Some((start, size));
        }
    }
    let (start, size) = found?;
    if size > MAX_BYTES {
        return None;
    }
    let cutoff = u32_at(file, 0x38)? as usize;
    let data = if size < cutoff {
        // A small stream lives in the mini stream, which is the root entry's own chain.
        let mini_fat_bytes = chain(u32_at(file, 0x3C)?, &fat)?;
        let mini_fat: Vec<u32> = mini_fat_bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        let container = chain(root_start?, &fat)?;
        let mut out = Vec::new();
        let mut id = start;
        let mut steps = 0;
        while id < 0xFFFF_FFFA {
            let a = (id as usize).checked_mul(msize)?;
            out.extend_from_slice(container.get(a..a.checked_add(msize)?)?);
            id = *mini_fat.get(id as usize)?;
            steps += 1;
            if steps > mini_fat.len() + 1 {
                return None;
            }
        }
        out
    } else {
        chain(start, &fat)?
    };
    Some(data.get(..size)?.to_vec())
}

/// One cell the reader knows: a number, or something that is not (text, a non-numeric result).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell {
    Number(f64),
    Other,
}

fn rk(v: u32) -> f64 {
    let x = if v & 2 != 0 {
        ((v as i32) >> 2) as f64
    } else {
        f64::from_bits(((v & 0xFFFF_FFFC) as u64) << 32)
    };
    if v & 1 != 0 {
        x / 100.0
    } else {
        x
    }
}

/// A sheet's name and the offset of its substream, from the workbook globals.
fn sheets(wb: &[u8]) -> Option<Vec<(String, usize)>> {
    let mut out = Vec::new();
    let mut p = 0usize;
    let mut first = true;
    while p + 4 <= wb.len() {
        let id = u16_at(wb, p)?;
        let len = u16_at(wb, p + 2)? as usize;
        let d = wb.get(p + 4..p + 4 + len)?;
        if first && id != 0x0809 {
            return None;
        }
        first = false;
        match id {
            0x0085 => {
                let off = u32_at(d, 0)? as usize;
                let cch = *d.get(6)? as usize;
                let flags = *d.get(7)?;
                let name = if flags & 1 == 0 {
                    d.get(8..8 + cch)?.iter().map(|&b| b as char).collect::<String>()
                } else {
                    let units: Vec<u16> = d
                        .get(8..8 + cch * 2)?
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect();
                    String::from_utf16_lossy(&units)
                };
                out.push((name, off));
            }
            0x000A => break,
            _ => {}
        }
        p += 4 + len;
    }
    Some(out)
}

/// The cells of the sheet substream that starts at `off`, inside the rectangle given (zero-based,
/// inclusive).
fn cells_in(
    wb: &[u8],
    off: usize,
    r0: u16,
    r1: u16,
    c0: u16,
    c1: u16,
) -> Option<std::collections::HashMap<(u16, u16), Cell>> {
    let mut map = std::collections::HashMap::new();
    let mut p = off;
    let mut depth = 0;
    let mut started = false;
    let inside = |r: u16, c: u16| r >= r0 && r <= r1 && c >= c0 && c <= c1;
    while p + 4 <= wb.len() {
        let id = u16_at(wb, p)?;
        let len = u16_at(wb, p + 2)? as usize;
        let d = wb.get(p + 4..p + 4 + len)?;
        p += 4 + len;
        match id {
            0x0809 => {
                // A nested substream (a chart, say) is skipped whole.
                if started {
                    depth += 1;
                }
                started = true;
            }
            0x000A => {
                if depth == 0 {
                    return Some(map);
                }
                depth -= 1;
            }
            _ if depth > 0 => {}
            0x0203 => {
                let (r, c) = (u16_at(d, 0)?, u16_at(d, 2)?);
                if inside(r, c) {
                    map.insert((r, c), Cell::Number(f64_at(d, 6)?));
                }
            }
            0x027E => {
                let (r, c) = (u16_at(d, 0)?, u16_at(d, 2)?);
                if inside(r, c) {
                    map.insert((r, c), Cell::Number(rk(u32_at(d, 6)?)));
                }
            }
            0x00BD => {
                let r = u16_at(d, 0)?;
                let first = u16_at(d, 2)?;
                let last = u16_at(d, d.len().checked_sub(2)?)?;
                for (k, c) in (first..=last).enumerate() {
                    let rec = d.get(4 + 6 * k..4 + 6 * k + 6)?;
                    if inside(r, c) {
                        map.insert((r, c), Cell::Number(rk(u32_at(rec, 2)?)));
                    }
                }
            }
            0x0006 => {
                let (r, c) = (u16_at(d, 0)?, u16_at(d, 2)?);
                if inside(r, c) {
                    // A cached result with 0xFFFF in its top bytes is a string, boolean or error.
                    let numeric = u16_at(d, 12)? != 0xFFFF;
                    map.insert(
                        (r, c),
                        if numeric {
                            Cell::Number(f64_at(d, 6)?)
                        } else {
                            Cell::Other
                        },
                    );
                }
            }
            0x00FD | 0x0204 | 0x00D6 => {
                let (r, c) = (u16_at(d, 0)?, u16_at(d, 2)?);
                if inside(r, c) {
                    map.insert((r, c), Cell::Other);
                }
            }
            _ => {}
        }
    }
    None
}

fn column_number(s: &str) -> Option<u16> {
    if s.is_empty() || s.len() > 3 {
        return None;
    }
    let mut n = 0u32;
    for c in s.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        n = n * 26 + (c.to_ascii_uppercase() as u32 - 'A' as u32 + 1);
    }
    u16::try_from(n - 1).ok()
}

fn corner(s: &str) -> Option<(u16, u16)> {
    let s = s.trim().replace('$', "");
    let split = s.find(|c: char| c.is_ascii_digit())?;
    let col = column_number(&s[..split])?;
    let row: u32 = s[split..].parse().ok()?;
    Some((u16::try_from(row.checked_sub(1)?).ok()?, col))
}

/// A range such as `Sheet1!C6:H16` (or `'Sheet 1'!B2`) as sheet name and zero-based corners.
/// A sheet name and the two corners (row, column) of a range, zero-based.
type RangeSpec = (String, (u16, u16), (u16, u16));

fn parse_range(spec: &str) -> Option<RangeSpec> {
    let (sheet, cells) = spec.rsplit_once('!')?;
    let sheet = sheet.trim().trim_matches('\'').replace("''", "'");
    let (a, b) = match cells.split_once(':') {
        Some((a, b)) => (corner(a)?, corner(b)?),
        None => (corner(cells)?, corner(cells)?),
    };
    Some((sheet, (a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1))))
}

/// The numbers of `range` in the workbook `file`, as `(rows, cols, values column by column)`, or
/// `None` if the file or range cannot be read or any cell in it is text or not a number. A cell
/// that was never written reads as 0, so a range of only such cells is refused as well.
pub fn read_range(file: &[u8], range: &str) -> Option<(usize, usize, Vec<f64>)> {
    let (sheet, (r0, c0), (r1, c1)) = parse_range(range)?;
    let rows = (r1 - r0) as usize + 1;
    let cols = (c1 - c0) as usize + 1;
    if rows * cols > 4000 {
        return None;
    }
    let wb = cfb_stream(file, &["Workbook", "Book"])?;
    let list = sheets(&wb)?;
    let (_, off) = list.iter().find(|(n, _)| n.eq_ignore_ascii_case(&sheet))?;
    let cells = cells_in(&wb, *off, r0, r1, c0, c1)?;
    if cells.is_empty() {
        return None;
    }
    let mut values = Vec::with_capacity(rows * cols);
    for c in c0..=c1 {
        for r in r0..=r1 {
            match cells.get(&(r, c)) {
                Some(Cell::Number(v)) if v.is_finite() => values.push(*v),
                // A cell nothing was written to reads as 0, as in the spreadsheet component.
                None => values.push(0.0),
                _ => return None,
            }
        }
    }
    Some((rows, cols, values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_and_numbers() {
        assert_eq!(parse_range("Sheet1!C6:H16"), Some(("Sheet1".into(), (5, 2), (15, 7))));
        assert_eq!(parse_range("'My Sheet'!B2"), Some(("My Sheet".into(), (1, 1), (1, 1))));
        assert_eq!(parse_range("nothing"), None);
        assert_eq!(rk(0x0000_000A), 2.0);
        assert_eq!(rk(0x0000_000B), 0.02);
    }

    #[test]
    fn garbage_is_none() {
        assert!(read_range(&[0u8; 100], "A!A1").is_none());
        let mut f = vec![0u8; 2048];
        f[..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
        assert!(read_range(&f, "A!A1").is_none());
    }
}
