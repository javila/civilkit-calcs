"""For out-of-scope worksheets, find where the rendering first diverges from the stub and
whether that point is an out-of-scope error. A divergence anywhere else points at a scalar
bug hidden inside a worksheet that is classified as out of scope.

Run tools/compare_stubs.py --save first. Usage:
  python -I tools/first_divergence.py --ref ../calcpad-ref
"""
import argparse
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MARK = "not supported by calcpad-rs yet"


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
    suspicious = []
    for r in results:
        if r["status"] != "OUT-OF-SCOPE":
            continue
        rendered_path = out / "rendered" / (r["file"].replace("/", "__") + ".html")
        if not rendered_path.exists():
            continue
        cpd = ref / r["file"]
        stub = cr._stub_path(cpd)
        existing = cr._read_normalized(stub)
        html = rendered_path.read_text(encoding="utf-8")
        html = re.sub(r'(src|href)\s*=\s*"\.\./\.\./\s*', r'\1="', html).replace("\r\n", "\n")
        rendered = html.encode("utf-8")
        rendered = cr._reconcile_numbers(rendered, existing)
        rendered = cr._reconcile_file_url(rendered, existing)
        pretty = cr._prettify(rendered).decode("utf-8").splitlines()
        stub_lines = existing.decode("utf-8").splitlines()
        n = min(len(pretty), len(stub_lines))
        k = next((i for i in range(n) if pretty[i] != stub_lines[i]), n)
        window = "\n".join(pretty[max(0, k - 3) : k + 12])
        # the out-of-scope error is reported in the paragraph that diverges
        if MARK in window or 'class="err"' in window or k >= n - 1:
            continue
        suspicious.append((r["file"], k, r["scope"], stub_lines[k : k + 6], pretty[k : k + 6]))
    for f, k, scope, a, b in suspicious:
        print(f"== {f} (line {k}, scope {scope})")
        print("  stub:    ", " | ".join(x.strip() for x in a))
        print("  rendered:", " | ".join(x.strip() for x in b))
    print(f"{len(suspicious)} out-of-scope worksheets diverge before their first out-of-scope error")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
