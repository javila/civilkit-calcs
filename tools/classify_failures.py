"""Classifies the FAIL worksheets of the last tools/compare_stubs.py run.

The comparison rules of compare_stubs.py (upstream compare_renderings.py) are not changed;
this tool only labels the failures so that numeric noise and PNG encoding differences are
reported separately from real failures:

  NOISE        every changed line differs only in numbers, and every differing number
               agrees with the stub to 1e-12 relative, or the stub prints an integer that
               the rendered value rounds to within 1e-12.
  PNG-PIXEL    the only changed lines are embedded $Map PNG images and their decoded
               pixels are equal (SkiaSharp's deflate stream is not reproduced).
  TIMEOUT      the CLI did not finish within the harness time limit.
  FAIL         anything else.

Run tools/compare_stubs.py --save first. Usage:
  python -I tools/classify_failures.py --ref ../calcpad-ref
"""
import argparse
import base64
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
NUM_RE = re.compile(r"[-−]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?")
PNG_RE = re.compile(r"data:image/png;base64,([A-Za-z0-9+/=]+)")


def to_float(s: str) -> float:
    return float(s.replace("−", "-"))


def numbers_noise(old: str, new: str):
    """Returns (is_noise, max_abs, max_rel) for a pair of changed lines, or None when the
    lines differ in more than numbers."""
    if NUM_RE.sub("#", old) != NUM_RE.sub("#", new):
        return None
    a = NUM_RE.findall(old)
    b = NUM_RE.findall(new)
    max_abs = 0.0
    max_rel = 0.0
    ok = True
    for x, y in zip(a, b):
        if x == y:
            continue
        fx, fy = to_float(x), to_float(y)
        d = abs(fx - fy)
        rel = d / max(abs(fx), abs(fy)) if max(abs(fx), abs(fy)) > 0 else 0.0
        max_abs = max(max_abs, d)
        max_rel = max(max_rel, rel)
        is_int = re.fullmatch(r"[-−]?\d+", x) is not None
        if rel <= 1e-12:
            continue
        if is_int and abs(fy - fx) <= 1e-12 * max(1.0, abs(fx)):
            continue
        ok = False
    return ok, max_abs, max_rel


def png_pixels_equal(old: str, new: str, cp) -> bool:
    ma, mb = PNG_RE.search(old), PNG_RE.search(new)
    if not (ma and mb):
        return False
    if PNG_RE.sub("", old) != PNG_RE.sub("", new):
        return False
    try:
        return cp.pixels(base64.b64decode(ma.group(1))) == cp.pixels(base64.b64decode(mb.group(1)))
    except Exception:
        return False


def classify(diff_text: str, cp):
    removed, added = [], []
    for line in diff_text.splitlines():
        if line.startswith("---") or line.startswith("+++"):
            continue
        if line.startswith("-"):
            removed.append(line[1:])
        elif line.startswith("+"):
            added.append(line[1:])
    if len(removed) != len(added) or not removed:
        return "FAIL", None
    kinds = set()
    max_abs = max_rel = 0.0
    for old, new in zip(removed, added):
        if "data:image/png" in old or "data:image/png" in new:
            if png_pixels_equal(old, new, cp):
                kinds.add("PNG-PIXEL")
                continue
            return "FAIL", None
        r = numbers_noise(old, new)
        if r is None or not r[0]:
            return "FAIL", None
        kinds.add("NOISE")
        max_abs = max(max_abs, r[1])
        max_rel = max(max_rel, r[2])
    if kinds == {"PNG-PIXEL"}:
        return "PNG-PIXEL", None
    if "NOISE" in kinds:
        label = "NOISE" if kinds == {"NOISE"} else "NOISE+PNG-PIXEL"
        return label, (max_abs, max_rel)
    return "FAIL", None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ref", default=str(HERE.parent.parent / "calcpad-ref"))
    ap.add_argument("--out", default=str(HERE / "out"))
    args = ap.parse_args()
    sys.path.insert(0, str(HERE))
    import compare_png as cp  # noqa: E402

    out = Path(args.out)
    results = json.loads((out / "results.json").read_text(encoding="utf-8"))
    counts = {}
    rows = []
    for r in results:
        status = r["status"]
        detail = ""
        if status == "FAIL":
            if r["detail"] == "timeout":
                status = "TIMEOUT"
            else:
                d = out / "diffs" / (r["file"].replace("/", "__") + ".diff")
                text = d.read_text(encoding="utf-8") if d.exists() else ""
                status, noise = classify(text, cp)
                if noise:
                    detail = f"max abs {noise[0]:.3g}, max rel {noise[1]:.3g}"
            rows.append((status, r["file"], detail))
        counts[status] = counts.get(status, 0) + 1
    for status, f, detail in sorted(rows):
        print(f"{status:16} {f}  {detail}")
    print("counts:", json.dumps(counts))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
