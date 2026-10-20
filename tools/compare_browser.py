"""Compare the worksheets rendered in the browser (web/e2e/browser-check.mjs) with the
CalcpadCE *.html.stub golden files.

The browser output is the body of the page the UI saves with Export > Preview > Save HTML,
wrapped in <div class="calcpad-output"> as the CLI's -b mode does. It then goes through the
same steps as tools/compare_stubs.py: upstream's compare_renderings.py number and file URL
reconciliation and prettifying, unchanged.

Usage: python3 -I tools/compare_browser.py [--ref ../calcpad-ref] [--diff]
"""
import argparse
import difflib
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ref", default=str(HERE.parent.parent / "calcpad-ref"))
    ap.add_argument("--dir", default=str(HERE / "out" / "browser"))
    ap.add_argument("--diff", action="store_true")
    args = ap.parse_args()

    ref = Path(args.ref).resolve()
    sys.path.insert(0, str(ref / ".github" / "scripts"))
    import compare_renderings as cr  # noqa: E402

    rendered_dir = Path(args.dir)
    index = json.loads((rendered_dir / "index.json").read_text(encoding="utf-8"))
    counts = {}
    for item in index:
        stub = cr._stub_path(ref / item["file"])
        html = (rendered_dir / item["rendered"]).read_text(encoding="utf-8")
        html = re.sub(r'(src|href)\s*=\s*"\.\./\.\./\s*', r'\1="', html).replace("\r\n", "\n")
        existing = cr._read_normalized(stub)
        rendered = cr._reconcile_numbers(html.encode("utf-8"), existing)
        rendered = cr._reconcile_file_url(rendered, existing)
        pretty = cr._prettify(rendered)
        status = "PASS" if pretty == existing else "FAIL"
        counts[status] = counts.get(status, 0) + 1
        print(f"{status}  {item['file']}  ({item['ms']} ms)")
        if status == "FAIL" and args.diff:
            diff = difflib.unified_diff(
                existing.decode("utf-8", "replace").splitlines(),
                pretty.decode("utf-8", "replace").splitlines(),
                "stub", "browser", lineterm="",
            )
            print("\n".join(list(diff)[:80]))
    print("counts:", json.dumps(counts))
    return 0 if counts.get("FAIL", 0) == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
