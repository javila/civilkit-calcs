//! Ports of ImageWriter.cs and ImageProcessor.cs. Bitmaps (data: URLs, or files a resolver
//! supplies) go in as they are, with their size read from their headers. An SVG drawing is
//! rasterised to PNG with resvg and the SVG kept beside it as an `svgBlip`, so Word versions
//! that show SVG use the vector (upstream embeds the SVG alone).
use std::cell::Cell;

use crate::package::Package;
use crate::writer::{PPr, RPr, Spacing, N};
use calcpad_core::openxml::xml::escape;
use calcpad_core::parsers::expression_parser::base64_decode;

/// Supplies the bytes of an image that is not a data: URL (a library file, a file next to the
/// worksheet).
pub type ImageResolver<'a> = dyn Fn(&str) -> Option<Vec<u8>> + 'a;

thread_local! {
    static DOC_PR_ID: Cell<u32> = const { Cell::new(0) };
}

fn next_doc_pr_id() -> u32 {
    DOC_PR_ID.with(|c| {
        c.set(c.get() + 1);
        c.get()
    })
}

/// (width, height) in pixels from a bitmap's header.
pub fn bitmap_size(data: &[u8]) -> Option<(u32, u32)> {
    let be32 = |i: usize| Some(u32::from_be_bytes(data.get(i..i + 4)?.try_into().ok()?));
    let le16 = |i: usize| Some(u16::from_le_bytes(data.get(i..i + 2)?.try_into().ok()?) as u32);
    let le32 = |i: usize| Some(i32::from_le_bytes(data.get(i..i + 4)?.try_into().ok()?));
    if data.starts_with(b"\x89PNG") {
        return Some((be32(16)?, be32(20)?));
    }
    if data.starts_with(b"GIF8") {
        return Some((le16(6)?, le16(8)?));
    }
    if data.starts_with(b"BM") {
        return Some((le32(18)?.unsigned_abs(), le32(22)?.unsigned_abs()));
    }
    if data.starts_with(&[0xFF, 0xD8]) {
        let mut i = 2;
        while i + 9 < data.len() {
            if data[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = data[i + 1];
            let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            if (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
                let h = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                let w = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                return Some((w, h));
            }
            i += 2 + len;
        }
    }
    None
}

/// The size an SVG states for itself: width and height, or its viewBox.
pub fn svg_size(svg: &str) -> Option<(f64, f64)> {
    let head_end = svg.find('>')?;
    let head = &svg[..head_end];
    let attr = |name: &str| -> Option<String> {
        let lower = head.to_ascii_lowercase();
        let mut from = 0;
        while let Some(i) = lower[from..].find(name).map(|i| i + from) {
            let before = lower[..i].chars().last();
            let after = lower[i + name.len()..].trim_start();
            if matches!(before, Some(' ' | '\t' | '\n' | '\r')) && after.starts_with('=') {
                let v = after[1..].trim_start();
                let q = v.chars().next()?;
                let v = if q == '"' || q == '\'' {
                    &v[1..v[1..].find(q)? + 1]
                } else {
                    v.split_whitespace().next()?
                };
                return Some(v.to_string());
            }
            from = i + name.len();
        }
        None
    };
    let w = attr("width")
        .map(|v| crate::css::parse_size_px(&v))
        .filter(|v| *v > 0.0);
    let h = attr("height")
        .map(|v| crate::css::parse_size_px(&v))
        .filter(|v| *v > 0.0);
    if let (Some(w), Some(h)) = (w, h) {
        return Some((w, h));
    }
    let vb = attr("viewbox")?;
    let n: Vec<f64> = vb
        .split([' ', ','])
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    (n.len() == 4).then(|| (n[2], n[3]))
}

/// Calcpad's plots align their labels with `.plot text.end` and `.plot text.mid` style rules,
/// which neither resvg nor the SVG renderers of word processors apply; the same alignment as
/// attributes, in the raster and in the SVG kept beside it.
pub fn anchor_plot_text(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len() + 256);
    let mut rest = svg;
    while let Some(i) = rest.find("<text ") {
        let end = rest[i..].find('>').map(|e| i + e).unwrap_or(rest.len());
        let tag = &rest[i..end];
        out.push_str(&rest[..end]);
        if !tag.contains("text-anchor") {
            let class = tag.find("class=\"").and_then(|c| {
                let v = &tag[c + 7..];
                v.find('"').map(|q| &v[..q])
            });
            let anchor = class.and_then(|c| {
                c.split_ascii_whitespace().find_map(|x| match x {
                    "end" => Some("end"),
                    "mid" => Some("middle"),
                    "left" => Some("start"),
                    _ => None,
                })
            });
            if let Some(a) = anchor {
                let self_closing = out.ends_with('/');
                if self_closing {
                    out.pop();
                }
                out.push_str(&format!(" text-anchor=\"{}\"", a));
                if self_closing {
                    out.push('/');
                }
            }
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// The SVG as a PNG at twice its size, for print. Fonts come from `fonts`.
#[cfg(feature = "svg")]
pub fn rasterize_svg(svg: &str, width: f64, height: f64) -> Result<Vec<u8>, String> {
    use resvg::{tiny_skia, usvg};
    let mut opt = usvg::Options::default();
    crate::fonts::with_database(|db| opt.fontdb = std::sync::Arc::new(db.clone()));
    opt.font_family = "DejaVu Sans".into();
    let tree = usvg::Tree::from_str(svg, &opt).map_err(|e| e.to_string())?;
    let size = tree.size();
    let (w, h) = if width > 0.0 && height > 0.0 {
        (width, height)
    } else {
        (size.width() as f64, size.height() as f64)
    };
    const SCALE: f64 = 2.0;
    let pw = (w * SCALE).round().max(1.0) as u32;
    let ph = (h * SCALE).round().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(pw, ph).ok_or("The drawing is too large to rasterise.")?;
    pixmap.fill(tiny_skia::Color::WHITE);
    let t = tiny_skia::Transform::from_scale(pw as f32 / size.width(), ph as f32 / size.height());
    resvg::render(&tree, t, &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| e.to_string())
}

#[cfg(not(feature = "svg"))]
pub fn rasterize_svg(_svg: &str, _width: f64, _height: f64) -> Result<Vec<u8>, String> {
    Err("SVG images need the svg feature.".into())
}

fn extension_of(path: &str) -> String {
    let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let file = file.split(['?', '#']).next().unwrap_or(file);
    file.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default()
}

fn check_type(ext: &str) -> Result<String, String> {
    let e = if ext == "jpg" { "jpeg" } else { ext };
    match e {
        "png" | "gif" | "jpeg" | "bmp" | "tif" | "tiff" | "wmf" | "emf" | "ico" | "pcx" | "svg" => Ok(e.to_string()),
        "svg+xml" => Ok("svg".into()),
        _ => Err(format!("Unsupported image type: {}.", ext)),
    }
}

/// `ImageWriter.AddImage`: the image as a paragraph with an inline (or floating) drawing.
pub fn add_image(
    pkg: &mut Package,
    src: &str,
    alt: &str,
    width: i64,
    height: i64,
    align: Option<&str>,
    resolver: Option<&ImageResolver>,
) -> Result<N, String> {
    let mut name = alt.to_string();
    // (bytes, extension) of the main picture, and the SVG when the picture came from one.
    let (data, ext, svg): (Vec<u8>, String, Option<String>);
    let mut size: Option<(f64, f64)> = None;
    if src.trim_start().to_ascii_lowercase().starts_with("<svg") {
        let s = src.to_string();
        let (w, h) = if width > 0 && height > 0 {
            (width as f64, height as f64)
        } else {
            svg_size(&s).unwrap_or((0.0, 0.0))
        };
        size = Some((w, h));
        data = rasterize_svg(&anchor_plot_text(&s), w, h)?;
        ext = "png".into();
        svg = Some(s);
    } else if src.to_ascii_lowercase().starts_with("data:image/") {
        let start = src.find('/').ok_or("Invalid image data.")? + 1;
        let end = src.find(';').ok_or("Invalid image data.")?;
        let comma = src.find(',').ok_or("Invalid image data.")?;
        let t = check_type(&src[start..end].to_ascii_lowercase())?;
        let bytes = base64_decode(&src[comma + 1..]).ok_or("Invalid image data.")?;
        if t == "svg" {
            let s = String::from_utf8_lossy(&bytes).to_string();
            let wh = svg_size(&s).unwrap_or((width as f64, height as f64));
            size = Some(wh);
            data = rasterize_svg(&anchor_plot_text(&s), wh.0, wh.1)?;
            ext = "png".into();
            svg = Some(s);
        } else {
            data = bytes;
            ext = t;
            svg = None;
        }
    } else {
        let bytes = resolver
            .and_then(|r| r(src))
            .ok_or_else(|| format!("Could not find image file {}.", src))?;
        if name.trim().is_empty() {
            name = src.rsplit(['/', '\\']).next().unwrap_or(src).to_string();
        }
        let t = check_type(&extension_of(src))?;
        if t == "svg" {
            let s = String::from_utf8_lossy(&bytes).to_string();
            let wh = svg_size(&s).unwrap_or((width as f64, height as f64));
            size = Some(wh);
            data = rasterize_svg(&anchor_plot_text(&s), wh.0, wh.1)?;
            ext = "png".into();
            svg = Some(s);
        } else {
            data = bytes;
            ext = t;
            svg = None;
        }
    }
    let (mut width, mut height) = (width, height);
    if width == 0 || height == 0 {
        let (w, h) = match size {
            Some((w, h)) => (w as i64, h as i64),
            None => bitmap_size(&data).map(|(w, h)| (w as i64, h as i64)).unwrap_or((0, 0)),
        };
        if width == 0 {
            width = w;
        }
        if height == 0 {
            height = h;
        }
        if height != h && h != 0 {
            width = (width as f64 * height as f64 / h as f64) as i64;
        }
        if width != w && w != 0 {
            height = (height as f64 * width as f64 / w as f64) as i64;
        }
    }
    let svg = svg.map(|s| anchor_plot_text(&s));
    let image_id = pkg.add_image(&ext, data);
    let svg_id = svg.map(|s| pkg.add_image("svg", s.into_bytes()));
    Ok(image_reference(
        &name,
        width,
        height,
        &image_id,
        svg_id.as_deref(),
        align,
    ))
}

/// `AddPicture`
fn picture(name: &str, w: i64, h: i64, image_id: &str, svg_id: Option<&str>) -> String {
    let svg_ext = svg_id
        .map(|id| {
            format!(
                "<a:ext uri=\"{{96DAC541-7B7A-43D3-8B79-37D633B846F1}}\"><asvg:svgBlip r:embed=\"{}\"/></a:ext>",
                id
            )
        })
        .unwrap_or_default();
    format!(
        "<a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic>\
         <pic:nvPicPr><pic:cNvPr id=\"0\" name=\"{name}\"/><pic:cNvPicPr/></pic:nvPicPr>\
         <pic:blipFill><a:blip r:embed=\"{image_id}\" cstate=\"print\"><a:extLst><a:ext uri=\"{{28A0092B-C50C-407E-A947-70E740481C1C}}\"/>{svg_ext}</a:extLst></a:blip>\
         <a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
         <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{w}\" cy=\"{h}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr>\
         </pic:pic></a:graphicData></a:graphic>",
        name = escape(name, true),
    )
}

/// `AddImageReference` and `AddFloatingImage`
fn image_reference(
    name: &str,
    width: i64,
    height: i64,
    image_id: &str,
    svg_id: Option<&str>,
    align: Option<&str>,
) -> N {
    const PX2EMU: i64 = 9525;
    let (w, h) = (width.max(0) * PX2EMU, height.max(0) * PX2EMU);
    let id = next_doc_pr_id();
    let frame_locks = "<wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr>";
    let doc_pr = format!("<wp:docPr id=\"{}\" name=\"{}\"/>", id, escape(name, true));
    match align {
        Some(a @ ("left" | "right")) => {
            let drawing = format!(
                "<w:drawing><wp:anchor distT=\"0\" distB=\"0\" distL=\"114300\" distR=\"114300\" simplePos=\"0\" relativeHeight=\"251658240\" behindDoc=\"1\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
                 <wp:simplePos x=\"0\" y=\"0\"/><wp:positionH relativeFrom=\"margin\"><wp:align>{a}</wp:align></wp:positionH>\
                 <wp:positionV relativeFrom=\"paragraph\"><wp:posOffset>0</wp:posOffset></wp:positionV>\
                 <wp:extent cx=\"{w}\" cy=\"{h}\"/><wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/><wp:wrapSquare wrapText=\"largest\"/>\
                 {doc_pr}{frame_locks}{pic}</wp:anchor></w:drawing>",
                pic = picture(name, w, h, image_id, svg_id)
            );
            N::P(
                Some(PPr {
                    spacing: Some(Spacing {
                        before: Some("0"),
                        after: Some("0"),
                        line: Some("0"),
                        exact: true,
                    }),
                    ..Default::default()
                }),
                vec![N::R(
                    Some(RPr {
                        size: Some("2".into()),
                        ..Default::default()
                    }),
                    vec![N::T("\u{2693}".into()), N::Raw(drawing)],
                )],
            )
        }
        _ => {
            let drawing = format!(
                "<w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{w}\" cy=\"{h}\"/>\
                 <wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>{doc_pr}{frame_locks}{pic}</wp:inline></w:drawing>",
                pic = picture(name, w, h, image_id, svg_id)
            );
            N::P(None, vec![N::R(None, vec![N::Raw(drawing)])])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_plot_labels_from_their_classes() {
        let s =
            anchor_plot_text("<svg><text x=\"1\" class=\"sm end\">a</text><text class=\"mid\"/><text>b</text></svg>");
        assert_eq!(
            s,
            "<svg><text x=\"1\" class=\"sm end\" text-anchor=\"end\">a</text><text class=\"mid\" text-anchor=\"middle\"/><text>b</text></svg>"
        );
    }

    #[test]
    fn reads_bitmap_and_svg_sizes() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&300u32.to_be_bytes());
        png.extend_from_slice(&120u32.to_be_bytes());
        assert_eq!(bitmap_size(&png), Some((300, 120)));
        assert_eq!(
            svg_size("<svg class=\"plot\" width=\"400\" height=\"250\">"),
            Some((400.0, 250.0))
        );
        assert_eq!(svg_size("<svg viewBox=\"0 0 120 80\">"), Some((120.0, 80.0)));
    }
}
