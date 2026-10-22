//! Pictures, fallbacks, saved results, the function-code split and the guards on hostile metafiles.
//! All bytes are made by hand here; no real worksheet content is used.

use std::time::{Duration, Instant};

use calcpad_import_xmcd::picture::{base64_decode, metafile_to_svg, SvgError, SvgLimits};
use calcpad_import_xmcd::{import_xmcd_opts, Code, ImportOptions, ImportResult, GAP_MARKER};

// ---- tiny files made by hand -------------------------------------------------------------------------

fn b64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::new();
    for ch in bytes.chunks(3) {
        let n =
            (u32::from(ch[0]) << 16) | (u32::from(*ch.get(1).unwrap_or(&0)) << 8) | u32::from(*ch.get(2).unwrap_or(&0));
        o.push(T[(n >> 18) as usize & 63] as char);
        o.push(T[(n >> 12) as usize & 63] as char);
        o.push(if ch.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        o.push(if ch.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    o
}

/// A PNG signature plus filler: the importer checks the signature only and never decodes pixels.
fn fake_png(tag: u8) -> Vec<u8> {
    let mut v = b"\x89PNG\r\n\x1a\n".to_vec();
    v.extend_from_slice(&[
        0, 0, 0, 13, b'I', b'H', b'D', b'R', tag, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0,
    ]);
    v
}

fn words(w: &[u16]) -> Vec<u8> {
    w.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// A minimal placeable-less WMF: header, window extents, one rectangle, end of file.
fn tiny_wmf() -> Vec<u8> {
    let rec_ext = [5u16, 0, 0x020C, 40, 60]; // SetWindowExt (3 words + header = 5 words; size is a u32)
    let _ = rec_ext;
    let mut body: Vec<u8> = Vec::new();
    // SetWindowExt: size 5 words, function 0x020C, y, x
    body.extend(words(&[5, 0, 0x020C, 40, 60]));
    // SetWindowOrg: size 5 words, function 0x020B, y, x
    body.extend(words(&[5, 0, 0x020B, 0, 0]));
    // Rectangle: size 7 words, function 0x041B, bottom, right, top, left
    body.extend(words(&[7, 0, 0x041B, 30, 50, 5, 5]));
    // EOF
    body.extend(words(&[3, 0, 0x0000]));
    let total_words = 9 + body.len() / 2;
    let mut v = words(&[1, 9, 0x0300]); // type memory, header size, version
    v.extend(words(&[(total_words & 0xFFFF) as u16, (total_words >> 16) as u16])); // size in words
    v.extend(words(&[0])); // objects
    v.extend(words(&[7, 0])); // max record size (words)
    v.extend(words(&[0])); // members
    v.extend(body);
    v
}

fn sheet(regions: &str, items: &str) -> String {
    format!(
        r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30">
<regions>{regions}</regions><binaryContent>{items}</binaryContent></worksheet>"#
    )
}

fn item(id: u32, bytes: &[u8]) -> String {
    format!(r#"<item item-id="{id}">{}</item>"#, b64(bytes))
}

fn picture_region(id: u32, top: u32, inner: &str, rendering: Option<u32>) -> String {
    let r = rendering
        .map(|i| format!(r#"<rendering item-idref="{i}"/>"#))
        .unwrap_or_default();
    format!(
        r#"<region region-id="{id}" left="0" top="{top}" width="120" height="80"><picture>{inner}</picture>{r}</region>"#
    )
}

fn run(xml: &str) -> ImportResult {
    import_xmcd_opts(xml, &ImportOptions::default())
}

// ---- tiny files are valid inputs for the converter ------------------------------------------------------------

#[test]
fn the_hand_made_metafile_converts() {
    let svg = metafile_to_svg(&tiny_wmf(), SvgLimits::default()).expect("tiny WMF converts");
    assert!(svg.contains("<svg"), "{svg}");
}

#[test]
fn base64_helper_agrees_with_the_test_encoder() {
    let data: Vec<u8> = (0..=255).collect();
    assert_eq!(base64_decode(&b64(&data)).unwrap(), data);
}

// ---- picture regions ---------------------------------------------------------------------------------------------

#[test]
fn metafile_becomes_an_svg_asset_and_an_image_line() {
    let xml = sheet(
        &picture_region(7, 0, r#"<metafile item-idref="1"/>"#, Some(2)),
        &(item(1, &tiny_wmf()) + &item(2, &fake_png(1))),
    );
    let res = import_xmcd_opts(
        &xml,
        &ImportOptions {
            asset_prefix: "sheet one".into(),
            ..ImportOptions::default()
        },
    );
    assert_eq!(res.assets.len(), 1);
    let a = &res.assets[0];
    assert_eq!((a.path.as_str(), a.mime), ("sheetone/R7.svg", "image/svg+xml"));
    assert!(std::str::from_utf8(&a.bytes).unwrap().contains("<svg"));
    let body: Vec<&str> = res.calcpad.lines().skip(1).collect();
    assert_eq!(body.len(), 1);
    assert!(body[0].starts_with("'<img style=\"width:120.0pt;"), "{}", body[0]);
    assert!(body[0].contains("src=\"sheetone/R7.svg\""));
    assert_eq!(res.report.kinds["image"].converted, 1);
    assert_eq!(res.report.pictures.svg_ok, 1);
    assert_eq!(res.report.regions_needing_attention(), 0);
}

#[test]
fn gzip_encoded_items_are_inflated() {
    // gzip of the tiny WMF using a stored (uncompressed) deflate block, made by hand.
    let raw = tiny_wmf();
    let mut gz = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff];
    gz.push(1); // final block, stored
    gz.extend((raw.len() as u16).to_le_bytes());
    gz.extend((!(raw.len() as u16)).to_le_bytes());
    gz.extend(&raw);
    gz.extend([0u8; 8]); // crc and size are not checked
    let xml = sheet(
        &picture_region(1, 0, r#"<metafile item-idref="1"/>"#, None),
        &format!(r#"<item item-id="1" content-encoding="gzip">{}</item>"#, b64(&gz)),
    );
    let res = run(&xml);
    assert_eq!(res.report.pictures.svg_ok, 1, "{:?}", res.report.issues);
}

#[test]
fn bad_metafile_falls_back_to_the_png_item() {
    let xml = sheet(
        &picture_region(3, 0, r#"<metafile item-idref="1"/><png item-idref="2"/>"#, None),
        &(item(1, b"this is not a metafile at all, not even close") + &item(2, &fake_png(2))),
    );
    let res = run(&xml);
    assert_eq!(res.assets.len(), 1);
    assert_eq!(res.assets[0].path, "imported/R3.png");
    assert_eq!((res.report.pictures.svg_failed, res.report.pictures.png_used), (1, 1));
    assert!(res.report.notes().any(|i| i.code == Code::SvgFallback));
    assert_eq!(res.report.regions_needing_attention(), 0);
}

#[test]
fn bad_metafile_falls_back_to_the_rendering() {
    let xml = sheet(
        &picture_region(3, 0, r#"<metafile item-idref="1"/>"#, Some(2)),
        &(item(1, &[1, 2, 3]) + &item(2, &fake_png(3))),
    );
    let res = run(&xml);
    assert_eq!(res.assets.len(), 1);
    assert!(res.assets[0].path.ends_with(".png"));
}

#[test]
fn an_oversized_svg_falls_back_to_the_png() {
    let xml = sheet(
        &picture_region(3, 0, r#"<metafile item-idref="1"/>"#, Some(2)),
        &(item(1, &tiny_wmf()) + &item(2, &fake_png(4))),
    );
    let opts = ImportOptions {
        svg_limits: SvgLimits {
            max_output: 10,
            ..SvgLimits::default()
        },
        ..ImportOptions::default()
    };
    let res = import_xmcd_opts(&xml, &opts);
    assert_eq!(res.assets.len(), 1);
    assert!(res.assets[0].path.ends_with(".png"));
    assert_eq!(res.report.pictures.svg_failed, 1);
}

#[test]
fn a_picture_with_nothing_usable_keeps_the_placeholder() {
    let xml = sheet(
        &picture_region(5, 0, r#"<metafile item-idref="1"/>"#, None),
        &item(1, b"junk"),
    );
    let res = run(&xml);
    assert!(res.assets.is_empty());
    assert!(res
        .calcpad
        .lines()
        .any(|l| l.starts_with(&format!("'{GAP_MARKER} R5 image]"))));
    assert_eq!(res.report.regions_needing_attention(), 1);
    // A picture with no items at all, or a dangling reference, is the same.
    let res = run(&sheet(
        &picture_region(6, 0, r#"<metafile item-idref="99"/>"#, None),
        "",
    ));
    assert!(res.calcpad.contains("R6 image]"));
}

#[test]
fn pictures_off_gives_placeholders_and_no_assets() {
    let xml = sheet(
        &picture_region(7, 0, r#"<metafile item-idref="1"/>"#, Some(2)),
        &(item(1, &tiny_wmf()) + &item(2, &fake_png(1))),
    );
    let res = import_xmcd_opts(
        &xml,
        &ImportOptions {
            pictures: false,
            ..ImportOptions::default()
        },
    );
    assert!(res.assets.is_empty());
    assert!(res.calcpad.contains("R7 image]"));
    assert_eq!(res.report.regions_needing_attention(), 1);
}

#[test]
fn an_asset_budget_is_enforced() {
    let xml = sheet(
        &picture_region(7, 0, r#"<png item-idref="1"/>"#, None),
        &item(1, &fake_png(1)),
    );
    let res = import_xmcd_opts(
        &xml,
        &ImportOptions {
            max_total_asset_bytes: 10,
            ..ImportOptions::default()
        },
    );
    assert!(res.assets.is_empty());
    assert!(res.calcpad.contains("R7 image]"));
}

#[test]
fn duplicate_region_ids_get_distinct_asset_paths() {
    let regions = picture_region(1, 0, r#"<png item-idref="1"/>"#, None)
        + &picture_region(1, 100, r#"<png item-idref="1"/>"#, None);
    let res = run(&sheet(&regions, &item(1, &fake_png(1))));
    let mut paths: Vec<_> = res.assets.iter().map(|a| a.path.clone()).collect();
    paths.sort();
    assert_eq!(paths, ["imported/R1-2.png", "imported/R1.png"]);
}

#[test]
fn asset_prefix_cannot_escape_the_library() {
    let xml = sheet(
        &picture_region(1, 0, r#"<png item-idref="1"/>"#, None),
        &item(1, &fake_png(1)),
    );
    for (given, want) in [
        ("../../etc", "etc"),
        ("/abs/path", "abs/path"),
        ("a b/c\"d", "ab/cd"),
        ("", "imported"),
        ("..", "imported"),
    ] {
        let res = import_xmcd_opts(
            &xml,
            &ImportOptions {
                asset_prefix: given.into(),
                ..ImportOptions::default()
            },
        );
        assert_eq!(res.assets[0].path, format!("{want}/R1.png"), "{given:?}");
    }
}

// ---- fallback pictures for unconverted regions ------------------------------------------------------------------------

fn ole_region(id: u32, top: u32, rendering: Option<u32>) -> String {
    let r = rendering
        .map(|i| format!(r#"<rendering item-idref="{i}"/>"#))
        .unwrap_or_default();
    format!(r#"<region region-id="{id}" left="0" top="{top}" width="90" height="20"><ole clsid="x"/>{r}</region>"#)
}

#[test]
fn unconverted_regions_show_their_saved_picture_with_a_caption() {
    let math = r#"<region region-id="9" left="0" top="40" width="60" height="15"><math><ml:eval><ml:apply><ml:id>vendor_fn</ml:id><ml:real>1</ml:real></ml:apply></ml:eval></math><rendering item-idref="2"/></region>"#;
    let xml = sheet(
        &(ole_region(4, 0, Some(1)) + math),
        &(item(1, &fake_png(1)) + &item(2, &fake_png(2))),
    );
    let res = run(&xml);
    let l: Vec<&str> = res.calcpad.lines().skip(1).collect();
    assert_eq!(l.len(), 2, "{l:?}");
    assert!(
        l[0].starts_with("'<img ") && l[0].contains("src=\"imported/R4.png\""),
        "{}",
        l[0]
    );
    assert!(
        l[0].ends_with("<br/><i>static picture of the original, not live (R4 ole-object)</i>"),
        "{}",
        l[0]
    );
    assert!(l[1].contains("src=\"imported/R9.png\""));
    assert!(
        l[1].ends_with("<br/><i>static picture of the original, not live (R9 addin-function)</i>"),
        "{}",
        l[1]
    );
    // No grey placeholder in the output; the report still counts both as needing attention.
    assert!(!res.calcpad.contains(&format!("'{GAP_MARKER}")));
    assert_eq!(res.report.regions_needing_attention(), 2);
    assert_eq!(res.report.regions_showing(), 2);
    assert_eq!(res.report.pictures.fallback_shown, 2);
    assert!(res
        .report
        .gaps()
        .any(|i| i.code == Code::OleObject && i.region_id.as_deref() == Some("4")));
    assert!(res.report.notes().any(|i| i.code == Code::StaticPicture));
    assert_eq!(res.assets.len(), 2);
}

#[test]
fn without_a_rendering_the_placeholder_stays() {
    let res = run(&sheet(&ole_region(4, 0, None), ""));
    assert!(res.calcpad.contains(&format!("'{GAP_MARKER} R4 ole-object]")));
    assert!(res.assets.is_empty());
    assert_eq!(res.report.regions_showing(), 0);
}

#[test]
fn a_rendering_that_is_not_a_png_is_ignored() {
    let res = run(&sheet(
        &ole_region(4, 0, Some(1)),
        &item(1, b"GIF89a not a png, long enough to pass any size check"),
    ));
    assert!(res.assets.is_empty());
    assert!(res.calcpad.contains("R4 ole-object]"));
}

#[test]
fn picture_switch_off_restores_placeholders() {
    let xml = sheet(&ole_region(4, 0, Some(1)), &item(1, &fake_png(1)));
    let res = import_xmcd_opts(
        &xml,
        &ImportOptions {
            pictures: false,
            ..ImportOptions::default()
        },
    );
    assert!(res.calcpad.contains(&format!("'{GAP_MARKER} R4 ole-object]")));
    assert!(res.assets.is_empty());
    assert!(!res.calcpad.contains("<img"));
}

// ---- hostile items -----------------------------------------------------------------------------------------------------------

#[test]
fn hostile_items_do_not_panic_or_blow_up() {
    // Bad base64, a gzip bomb (long run of zeros), a truncated gzip, an enormous reference list.
    let bomb = {
        // Stored-block-free: deflate fixed block of many zero bytes is awkward by hand, so ask for a
        // huge declared stored block instead and let the cap reject it.
        let mut gz = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff, 1, 0xff, 0xff, 0, 0];
        gz.extend(vec![0u8; 65535]);
        gz.extend([0u8; 8]);
        gz
    };
    let items = r#"<item item-id="1">@@@@</item>"#.to_string()
        + &format!(r#"<item item-id="2" content-encoding="gzip">{}</item>"#, b64(&bomb))
        + r#"<item item-id="3" content-encoding="gzip">H4sIAAAA</item>"#
        + r#"<item item-id="4"/>"#;
    let regions: String = (1..=4)
        .map(|i| {
            picture_region(
                i,
                i * 100,
                &format!(r#"<png item-idref="{i}"/><metafile item-idref="{i}"/>"#),
                Some(i),
            )
        })
        .collect();
    let mut opts = ImportOptions::default();
    opts.limits.max_item_bytes = 1000; // the stored block above would decode to 65535 bytes
    let res = import_xmcd_opts(&sheet(&regions, &items), &opts);
    assert!(res.assets.is_empty());
    assert_eq!(res.report.regions_needing_attention(), 4);
}

#[test]
fn total_item_budget_is_enforced() {
    let png = fake_png(1);
    let regions: String = (1..=5)
        .map(|i| picture_region(i, i * 100, r#"<png item-idref="1"/>"#, None))
        .collect();
    let mut opts = ImportOptions::default();
    opts.limits.max_total_item_bytes = png.len() * 2 + 1;
    let res = import_xmcd_opts(&sheet(&regions, &item(1, &png)), &opts);
    assert_eq!(res.assets.len(), 2);
}

#[test]
fn truncated_and_random_metafiles_never_panic_and_finish_quickly() {
    let good = tiny_wmf();
    let limits = SvgLimits {
        timeout: Duration::from_secs(5),
        ..SvgLimits::default()
    };
    let t = Instant::now();
    // Every truncation of a valid file.
    for n in 0..good.len() {
        let _ = metafile_to_svg(&good[..n], limits);
    }
    // Every single-byte corruption.
    for i in 0..good.len() {
        let mut b = good.clone();
        b[i] ^= 0xFF;
        let _ = metafile_to_svg(&b, limits);
    }
    // Pseudo-random buffers, some starting like a metafile header.
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    for round in 0..300 {
        let len = (next() % 600) as usize;
        let mut b: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        if round % 2 == 0 && b.len() > 18 {
            b[..18].copy_from_slice(&good[..18]);
        }
        let _ = metafile_to_svg(&b, limits);
    }
    assert!(t.elapsed() < Duration::from_secs(60), "took {:?}", t.elapsed());
    assert_eq!(
        metafile_to_svg(&vec![0u8; 5 * 1024 * 1024], limits),
        Err(SvgError::InputTooLarge)
    );
}

// ---- the function-code split --------------------------------------------------------------------------------------------------

fn call_sheet(extra_regions: &str) -> String {
    sheet(
        &(r#"<region region-id="1" left="0" top="0"><math><ml:eval><ml:apply><ml:id>vendor_fn</ml:id><ml:real>1</ml:real></ml:apply></ml:eval></math></region>"#.to_string()
            + r#"<region region-id="2" left="0" top="20"><math><ml:eval><ml:apply><ml:id>stdev</ml:id><ml:real>1</ml:real></ml:apply></ml:eval></math></region>"#
            + extra_regions),
        "",
    )
}

#[test]
fn undefined_calls_split_into_addin_reference_and_builtin_missing() {
    let res = run(&call_sheet(""));
    let codes: Vec<(Code, Option<&str>)> = res.report.gaps().map(|i| (i.code, i.detail.as_deref())).collect();
    assert_eq!(
        codes,
        vec![
            (Code::AddinFunction, Some("vendor_fn")),
            (Code::BuiltinMissing, Some("stdev"))
        ]
    );
    // With a worksheet reference in the sheet, the unknown name may come from there.
    let res = run(&call_sheet(
        r#"<region region-id="3" left="0" top="40"><reference/></region>"#,
    ));
    let codes: Vec<(Code, Option<&str>)> = res
        .report
        .gaps()
        .map(|i| (i.code, i.detail.as_deref()))
        .filter(|(c, _)| *c != Code::WorksheetReference)
        .collect();
    assert_eq!(
        codes,
        vec![
            (Code::ReferenceFunction, Some("vendor_fn")),
            (Code::BuiltinMissing, Some("stdev"))
        ]
    );
    // All three are ranked as separate rows.
    let ranked = res.report.ranked_gaps();
    for c in [Code::ReferenceFunction, Code::BuiltinMissing] {
        assert!(ranked.iter().any(|r| r.code == c));
    }
}

#[test]
fn builtin_tables_do_not_overlap() {
    use calcpad_import_xmcd::tables::{is_builtin_missing, FUNCTIONS};
    for (n, ..) in FUNCTIONS {
        assert!(!is_builtin_missing(n), "{n}");
    }
}

// ---- saved results --------------------------------------------------------------------------------------------------------------

fn result_region() -> String {
    r#"<region region-id="12" left="0" top="0"><math><ml:eval><ml:apply><ml:id>vendor_fn</ml:id><ml:real>1</ml:real></ml:apply>
       <result xmlns="http://example.invalid/math30"><unitedValue><ml:real>571.9381</ml:real>
       <unitMonomial xmlns="http://example.invalid/units10"><unitReference unit="International_BTU"/><unitReference unit="pound" power-numerator="-1"/></unitMonomial></unitedValue></result>
       </ml:eval></math></region>"#
        .to_string()
}

#[test]
fn saved_results_are_counted_and_written_only_on_request() {
    let xml = sheet(&result_region(), "");
    let res = run(&xml);
    assert_eq!(res.report.pictures.saved_result_available, 1);
    assert_eq!(res.report.pictures.saved_result_emitted, 0);
    assert!(!res.calcpad.contains("saved result"));

    let res = import_xmcd_opts(
        &xml,
        &ImportOptions {
            saved_results: true,
            ..ImportOptions::default()
        },
    );
    assert_eq!(res.report.pictures.saved_result_emitted, 1);
    let line = res.calcpad.lines().find(|l| l.contains("saved result")).unwrap();
    assert_eq!(
        line,
        "'<i>saved result from the original, not live (R12): 571.9381 International_BTU/(pound)</i>"
    );
    // The placeholder is still there: the saved result is an addition, not a conversion.
    assert!(res.calcpad.contains(&format!("'{GAP_MARKER} R12 addin-function]")));
    assert_eq!(res.report.regions_needing_attention(), 1);
}

#[test]
fn a_picture_takes_precedence_over_a_saved_result() {
    let with_pic = result_region().replace("</math></region>", "</math><rendering item-idref=\"1\"/></region>");
    let res = import_xmcd_opts(
        &sheet(&with_pic, &item(1, &fake_png(1))),
        &ImportOptions {
            saved_results: true,
            ..ImportOptions::default()
        },
    );
    assert_eq!(res.report.pictures.saved_result_available, 1);
    assert_eq!(res.report.pictures.saved_result_emitted, 0);
    assert!(res.calcpad.contains("<img"));
}

#[test]
fn saved_results_that_are_not_single_numbers_are_ignored() {
    for body in [
        r#"<ml:str>text</ml:str>"#,
        r#"<ml:matrix rows="1" cols="1"><ml:real>1</ml:real></ml:matrix>"#,
        r#"<ml:real>not a number</ml:real>"#,
        r#"<ml:real>NaN</ml:real>"#,
    ] {
        let xml = sheet(
            &format!(
                r#"<region region-id="1" left="0" top="0"><math><ml:eval><ml:apply><ml:id>vendor_fn</ml:id><ml:real>1</ml:real></ml:apply><result xmlns="http://example.invalid/math30">{body}</result></ml:eval></math></region>"#
            ),
            "",
        );
        assert_eq!(run(&xml).report.pictures.saved_result_available, 0, "{body}");
    }
}
