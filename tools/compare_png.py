"""Pixel comparison of the $Map rasters for worksheets that differ from their stubs.

CalcpadCE embeds the colour map of $Map as a PNG encoded by SkiaSharp (libpng with
Chromium's zlib). calcpad-rs computes the same pixels but its deflate stream differs,
so the base64 text cannot match the stub. This tool replaces every embedded PNG with a
digest of its decoded pixels in both the stub and the rendering, and then applies the
normal stub comparison: a worksheet is PIXEL-EQUAL when it matches after that.

Run tools/compare_stubs.py --save first. Usage:
  python -I tools/compare_png.py --ref ../calcpad-ref
"""
import argparse
import base64
import hashlib
import json
import re
import struct
import sys
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
PNG_RE = re.compile(rb"data:image/png;base64,([A-Za-z0-9+/=]+)")


def pixels(data: bytes) -> bytes:
    i = 8
    idat = b""
    w = h = 0
    while i < len(data):
        ln = struct.unpack(">I", data[i : i + 4])[0]
        kind = data[i + 4 : i + 8]
        body = data[i + 8 : i + 8 + ln]
        if kind == b"IHDR":
            w, h = struct.unpack(">II", body[:8])
            if body[8] != 8 or body[9] != 6:
                raise ValueError("not 8 bit RGBA")
        elif kind == b"IDAT":
            idat += body
        i += 12 + ln
    raw = zlib.decompress(idat)
    stride = 4 * w + 1
    rows = []
    for r in range(h):
        if raw[r * stride] != 0:
            raise ValueError("filtered PNG rows are not supported")
        rows.append(raw[r * stride + 1 : (r + 1) * stride])
    return struct.pack(">II", w, h) + b"".join(rows)


def digest(m: re.Match) -> bytes:
    px = pixels(base64.b64decode(m.group(1)))
    return b"data:image/png;pixels," + hashlib.sha256(px).hexdigest().encode()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ref", default=str(HERE.parent.parent / "calcpad-ref"))
    ap.add_argument("--out", default=str(HERE / "out"))
    args = ap.parse_args()
    ref = Path(args.ref).resolve()
    sys.path.insert(0, str(ref / ".github" / "scripts"))
    import compare_renderings as cr  # noqa: E402

    out = Path(args.out)
    results = json.loads((out / "results.json").read_text(encoding="utf-8"))
    for r in results:
        if r["status"] != "FAIL":
            continue
        rendered_path = out / "rendered" / (r["file"].replace("/", "__") + ".html")
        if not rendered_path.exists():
            continue
        existing = cr._read_normalized(cr._stub_path(ref / r["file"]))
        if b"data:image/png" not in existing:
            continue
        html = rendered_path.read_text(encoding="utf-8")
        html = re.sub(r'(src|href)\s*=\s*"\.\./\.\./\s*', r'\1="', html).replace("\r\n", "\n")
        rendered = html.encode("utf-8")
        try:
            existing_d = PNG_RE.sub(digest, existing)
            rendered_d = PNG_RE.sub(digest, rendered)
        except Exception as e:  # noqa: BLE001
            print(f"ERROR        {r['file']}: {e}")
            continue
        rendered_d = cr._reconcile_numbers(rendered_d, existing_d)
        rendered_d = cr._reconcile_file_url(rendered_d, existing_d)
        pretty = cr._prettify(rendered_d)
        status = "PIXEL-EQUAL" if pretty == existing_d else "PIXEL-DIFF"
        print(f"{status:12} {r['file']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
