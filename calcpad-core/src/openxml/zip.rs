//! The zip container of an OpenXml package: reads stored and deflated entries and writes
//! deflated ones. Uses miniz_oxide (already a dependency, pure Rust) for deflate.

/// A package part: its name inside the zip and its bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub data: Vec<u8>,
}

fn u16_at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?))
}

/// Reads every entry through the central directory.
pub fn read(bytes: &[u8]) -> Result<Vec<Entry>, String> {
    const BAD: &str = "The file is not a valid Excel workbook (zip).";
    // End of central directory: the last "PK\x05\x06" within the trailing 64 KiB.
    let min = bytes.len().saturating_sub(65_557);
    let eocd = (min..bytes.len().saturating_sub(21))
        .rev()
        .find(|&i| bytes[i..i + 4] == [0x50, 0x4b, 0x05, 0x06])
        .ok_or(BAD)?;
    let count = u16_at(bytes, eocd + 10).ok_or(BAD)? as usize;
    let mut p = u32_at(bytes, eocd + 16).ok_or(BAD)? as usize;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        if u32_at(bytes, p) != Some(0x0201_4b50) {
            return Err(BAD.into());
        }
        let method = u16_at(bytes, p + 10).ok_or(BAD)?;
        let comp_size = u32_at(bytes, p + 20).ok_or(BAD)? as usize;
        let size = u32_at(bytes, p + 24).ok_or(BAD)? as usize;
        let name_len = u16_at(bytes, p + 28).ok_or(BAD)? as usize;
        let extra_len = u16_at(bytes, p + 30).ok_or(BAD)? as usize;
        let comment_len = u16_at(bytes, p + 32).ok_or(BAD)? as usize;
        let local = u32_at(bytes, p + 42).ok_or(BAD)? as usize;
        let name = String::from_utf8_lossy(bytes.get(p + 46..p + 46 + name_len).ok_or(BAD)?).to_string();
        p += 46 + name_len + extra_len + comment_len;
        if u32_at(bytes, local) != Some(0x0403_4b50) {
            return Err(BAD.into());
        }
        let lname = u16_at(bytes, local + 26).ok_or(BAD)? as usize;
        let lextra = u16_at(bytes, local + 28).ok_or(BAD)? as usize;
        let start = local + 30 + lname + lextra;
        let raw = bytes.get(start..start + comp_size).ok_or(BAD)?;
        let data = match method {
            0 => raw.to_vec(),
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(raw, size.max(1) * 2 + 1024)
                .map_err(|_| BAD.to_string())?,
            _ => return Err(BAD.into()),
        };
        entries.push(Entry { name, data });
    }
    Ok(entries)
}

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = table[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

/// Writes the entries as a zip with deflated data (no timestamps, so output is stable).
pub fn write(entries: &[Entry]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for e in entries {
        let compressed = miniz_oxide::deflate::compress_to_vec(&e.data, 6);
        let crc = crc32(&e.data);
        let offset = out.len() as u32;
        let name = e.name.as_bytes();
        let header = |sig: u32, central_dir: bool| {
            let mut h = Vec::new();
            h.extend_from_slice(&sig.to_le_bytes());
            if central_dir {
                h.extend_from_slice(&20u16.to_le_bytes()); // version made by
            }
            h.extend_from_slice(&20u16.to_le_bytes()); // version needed
            h.extend_from_slice(&0x0800u16.to_le_bytes()); // UTF-8 names
            h.extend_from_slice(&8u16.to_le_bytes()); // deflate
            h.extend_from_slice(&0u16.to_le_bytes()); // time
            h.extend_from_slice(&0x0021u16.to_le_bytes()); // date 1980-01-01
            h.extend_from_slice(&crc.to_le_bytes());
            h.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
            h.extend_from_slice(&(e.data.len() as u32).to_le_bytes());
            h.extend_from_slice(&(name.len() as u16).to_le_bytes());
            h.extend_from_slice(&0u16.to_le_bytes()); // extra
            if central_dir {
                h.extend_from_slice(&0u16.to_le_bytes()); // comment
                h.extend_from_slice(&0u16.to_le_bytes()); // disk
                h.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
                h.extend_from_slice(&0u32.to_le_bytes()); // external attributes
                h.extend_from_slice(&offset.to_le_bytes());
            }
            h.extend_from_slice(name);
            h
        };
        out.extend_from_slice(&header(0x0403_4b50, false));
        out.extend_from_slice(&compressed);
        central.extend_from_slice(&header(0x0201_4b50, true));
    }
    let cd_offset = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let entries = vec![
            Entry {
                name: "a.xml".into(),
                data: b"<a>hello hello hello</a>".to_vec(),
            },
            Entry {
                name: "dir/b.bin".into(),
                data: (0..2000u32).map(|i| (i % 7) as u8).collect(),
            },
        ];
        assert_eq!(read(&write(&entries)).unwrap(), entries);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
