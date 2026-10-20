"""Compare calcpad-rs renderings with the CalcpadCE *.html.stub golden files.

Reuses .github/scripts/compare_renderings.py from the reference checkout (read only) for
discovery, the denylist, number reconciliation, file URL reconciliation and prettifying,
so the comparison rules are identical to upstream CI. Each worksheet is classified as

  PASS          rendering matches the stub after reconciliation
  FAIL          rendering differs
  OUT-OF-SCOPE  the engine reported a phase 1 out of scope feature (vectors, matrices,
                plots, #UI, #read/#write, markdown)
  NOSTUB        no stub committed

Usage: python -I tools/compare_stubs.py --ref ../calcpad-ref [--cli target/release/Cli]
       [--filter substring] [--diff] [--jobs N]
"""
import argparse
import difflib
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ref", default=str(HERE.parent.parent / "calcpad-ref"))
    ap.add_argument("--cli", default=str(HERE.parent / "target" / "release" / "Cli"))
    ap.add_argument("--filter", default=None)
    ap.add_argument("--diff", action="store_true", help="print diffs of failing worksheets")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--out", default=str(HERE / "out"))
    args = ap.parse_args()

    ref = Path(args.ref).resolve()
    sys.path.insert(0, str(ref / ".github" / "scripts"))
    import compare_renderings as cr  # noqa: E402

    settings = ref / ".github" / "scripts" / "CliSettingsComparison.xml"
    cpd_paths = cr._discover()
    if args.filter:
        cpd_paths = [p for p in cpd_paths if args.filter in p.as_posix()]
    out_dir = Path(args.out)
    (out_dir / "diffs").mkdir(parents=True, exist_ok=True)

    def run_one(cpd: Path):
        rel = cr._repo_rel(cpd)
        stub = cr._stub_path(cpd)
        fd, out_path = tempfile.mkstemp(suffix=".html")
        os.close(fd)
        fd2, scope_path = tempfile.mkstemp(suffix=".scope")
        os.close(fd2)
        env = os.environ.copy()
        env["CALCPAD_SETTINGS"] = str(settings)
        env["CALCPAD_RS_SCOPE_FILE"] = scope_path
        started = time.perf_counter()
        status, detail, scope = "FAIL", "", ""
        try:
            r = subprocess.run([args.cli, str(cpd), out_path, "-b", "-s"], capture_output=True, timeout=120, env=env)
            elapsed = time.perf_counter() - started
            scope = Path(scope_path).read_text(encoding="utf-8").strip() if os.path.exists(scope_path) else ""
            if r.returncode != 0 or not os.path.getsize(out_path):
                status = "OUT-OF-SCOPE" if scope else "FAIL"
                detail = f"CLI exit {r.returncode}: {r.stderr.decode(errors='replace')[:300]}"
            else:
                html = Path(out_path).read_text(encoding="utf-8")
                html = re.sub(r'(src|href)\s*=\s*"\.\./\.\./\s*', r'\1="', html)
                html = html.replace("\r\n", "\n")
                rendered = html.encode("utf-8")
                if not stub.exists():
                    status = "NOSTUB"
                else:
                    existing = cr._read_normalized(stub)
                    rendered = cr._reconcile_numbers(rendered, existing)
                    rendered = cr._reconcile_file_url(rendered, existing)
                    pretty = cr._prettify(rendered)
                    if pretty == existing:
                        status = "PASS"
                    elif scope:
                        status = "OUT-OF-SCOPE"
                    else:
                        status = "FAIL"
                        diff = list(difflib.unified_diff(existing.decode("utf-8", "replace").splitlines(), pretty.decode("utf-8", "replace").splitlines(), "stub", "rendered", lineterm=""))
                        name = rel.replace("/", "__")
                        (out_dir / "diffs" / f"{name}.diff").write_text("\n".join(diff), encoding="utf-8")
                        detail = f"{sum(1 for d in diff if d.startswith(('+', '-')) and not d.startswith(('+++', '---')))} changed lines"
        except subprocess.TimeoutExpired:
            elapsed = time.perf_counter() - started
            status, detail = "FAIL", "timeout"
        finally:
            for p in (out_path, scope_path):
                if os.path.exists(p):
                    os.unlink(p)
        return {"file": rel, "status": status, "scope": scope, "detail": detail, "seconds": round(elapsed, 3)}

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        results = list(pool.map(run_one, cpd_paths))

    counts = {}
    for r in results:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    (out_dir / "results.json").write_text(json.dumps(results, indent=1, ensure_ascii=False), encoding="utf-8")
    for r in results:
        if r["status"] == "FAIL" or (args.filter and r["status"] != "PASS"):
            print(f"{r['status']:13} {r['file']}  {r['detail']}")
            if args.diff:
                name = r["file"].replace("/", "__")
                d = out_dir / "diffs" / f"{name}.diff"
                if d.exists():
                    print(d.read_text(encoding="utf-8")[:6000])
    in_scope = counts.get("PASS", 0) + counts.get("FAIL", 0)
    print("counts:", json.dumps(counts))
    if in_scope:
        print(f"PASS rate in scope: {counts.get('PASS', 0)}/{in_scope} = {100 * counts.get('PASS', 0) / in_scope:.1f}%")
    total = len(results)
    print(f"PASS rate overall: {counts.get('PASS', 0)}/{total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
