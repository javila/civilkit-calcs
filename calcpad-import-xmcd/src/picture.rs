//! Binary items of a worksheet file: base64 and gzip decoding with output caps, PNG detection, and
//! the metafile to SVG conversion, all bounded because every byte here is untrusted.

use std::sync::mpsc;
use std::time::Duration;

/// The label under a picture that stands in for a region that did not convert.
pub const PICTURE_CAPTION: &str = "static picture of the original, not live";

/// Longest base64 text decoded for one item (about 24 MiB of data).
const MAX_BASE64_CHARS: usize = 32 * 1024 * 1024;

/// Decodes base64, ignoring ASCII whitespace. `None` on a bad character or a bad length.
pub fn base64_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() > MAX_BASE64_CHARS {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut acc = 0u32;
    let mut bits = 0u32;
    let mut pad = 0usize;
    for &c in text.as_bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => {
                pad += 1;
                continue;
            }
            b' ' | b'\t' | b'\r' | b'\n' => continue,
            _ => return None,
        };
        if pad > 0 {
            return None; // data after padding
        }
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    if pad > 2 {
        return None;
    }
    Some(out)
}

/// Inflates a gzip member, refusing to produce more than `limit` bytes.
pub fn gunzip(data: &[u8], limit: usize) -> Option<Vec<u8>> {
    if data.len() < 18 || data[0] != 0x1f || data[1] != 0x8b || data[2] != 8 {
        return None;
    }
    let flags = data[3];
    let mut pos = 10usize;
    if flags & 0x04 != 0 {
        let n = usize::from(*data.get(pos)?) | (usize::from(*data.get(pos + 1)?) << 8);
        pos = pos.checked_add(2 + n)?;
    }
    for bit in [0x08u8, 0x10] {
        if flags & bit != 0 {
            let end = data.get(pos..)?.iter().position(|&b| b == 0)?;
            pos += end + 1;
        }
    }
    if flags & 0x02 != 0 {
        pos = pos.checked_add(2)?;
    }
    let body = data.get(pos..data.len().checked_sub(8)?)?;
    miniz_oxide::inflate::decompress_to_vec_with_limit(body, limit).ok()
}

pub fn is_png(b: &[u8]) -> bool {
    b.len() > 24 && b.starts_with(b"\x89PNG\r\n\x1a\n")
}

/// Why a metafile did not become an SVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgError {
    /// The input is larger than the allowed size.
    InputTooLarge,
    /// The converter failed (bad data, unsupported record).
    Failed,
    /// The converter panicked or ran past the time limit.
    Aborted,
    /// The SVG is larger than the allowed size.
    OutputTooLarge,
    /// The SVG contains something an image should not (script, foreign content).
    Unsafe,
}

/// Limits for [`metafile_to_svg`].
#[derive(Debug, Clone, Copy)]
pub struct SvgLimits {
    pub max_input: usize,
    pub max_output: usize,
    pub timeout: Duration,
}

impl Default for SvgLimits {
    fn default() -> Self {
        SvgLimits {
            max_input: 4 * 1024 * 1024,
            max_output: 3 * 1024 * 1024,
            timeout: Duration::from_secs(10),
        }
    }
}

/// Converts a WMF or EMF metafile to SVG text.
///
/// The conversion runs on its own thread with a large stack: a panic is caught, and a run that
/// takes longer than the time limit is abandoned (the thread is left to finish on its own, so a
/// hostile input costs one stuck thread, not the caller).
pub fn metafile_to_svg(bytes: &[u8], lim: SvgLimits) -> Result<String, SvgError> {
    if bytes.len() > lim.max_input {
        return Err(SvgError::InputTooLarge);
    }
    let data = bytes.to_vec();
    let (tx, rx) = mpsc::channel();
    let spawned = std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(move || {
        let r = std::panic::catch_unwind(|| emf_core::converter::convert_to_svg(data.as_slice()));
        let _ = tx.send(match r {
            Ok(Ok(svg)) => String::from_utf8(svg).map_err(|_| SvgError::Failed),
            Ok(Err(_)) => Err(SvgError::Failed),
            Err(_) => Err(SvgError::Aborted),
        });
    });
    if spawned.is_err() {
        return Err(SvgError::Aborted);
    }
    let svg = match rx.recv_timeout(lim.timeout) {
        Ok(r) => r?,
        Err(_) => return Err(SvgError::Aborted),
    };
    if svg.len() > lim.max_output {
        return Err(SvgError::OutputTooLarge);
    }
    let lower = svg.to_ascii_lowercase();
    if !lower.trim_start().starts_with("<svg") && !lower.trim_start().starts_with("<?xml") {
        return Err(SvgError::Failed);
    }
    for bad in [
        "<script",
        "javascript:",
        "<foreignobject",
        "<iframe",
        "<!entity",
        "<!doctype",
        "onload=",
        "onerror=",
    ] {
        if lower.contains(bad) {
            return Err(SvgError::Unsafe);
        }
    }
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_and_rejects_junk() {
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64_decode("aGVs\nbG8=\n").unwrap(), b"hello");
        assert_eq!(base64_decode("aGVsbG8").unwrap(), b"hello");
        assert!(base64_decode("aGV$bG8=").is_none());
        assert!(base64_decode("aG=Vs").is_none());
        assert_eq!(base64_decode("").unwrap(), b"");
    }

    #[test]
    fn gunzip_caps_output_and_rejects_bad_headers() {
        // gzip of 10 zero bytes, made by hand: header, stored-less fixed block is awkward, so use a
        // known small stream (python: gzip.compress(b"hello", mtime=0)).
        let gz: [u8; 25] = [
            0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0x02, 0x13, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x07, 0x00, 0x86, 0xa6, 0x10,
            0x36, 0x05, 0, 0, 0,
        ];
        assert_eq!(gunzip(&gz, 100).unwrap(), b"hello");
        assert!(gunzip(&gz, 3).is_none(), "output cap");
        assert!(gunzip(&gz[..12], 100).is_none());
        assert!(gunzip(b"not gzip at all, not gzip at all", 100).is_none());
    }

    #[test]
    fn png_detection() {
        let mut p = b"\x89PNG\r\n\x1a\n".to_vec();
        p.extend_from_slice(&[0; 20]);
        assert!(is_png(&p));
        assert!(!is_png(b"GIF89a................................"));
    }
}
