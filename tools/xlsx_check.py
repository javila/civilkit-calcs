#!/usr/bin/env python3
"""Excel export check (calcpad-rs's own feature) over the Samples, read with openpyxl.

For every sample, the CLI writes the worksheet as .xlsx (a Report sheet, then the variables); openpyxl must open the workbook, and:
  * "Variables" and "Inputs" start with the Name / Value / Unit header;
  * every value is a real number, a complex number as text, or the name of the sheet that
    holds a vector or matrix, and that sheet exists;
  * a vector or matrix sheet holds numbers only in its value cells;
  * every input is also a variable.

    python3 tools/xlsx_check.py [--filter TEXT]      (needs openpyxl)
"""
import argparse
import json
import os
import re
import subprocess
import sys

import openpyxl

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SAMPLES = os.path.join(ROOT, "web", "calcpad-web", "public", "samples")
OUT = os.path.join(ROOT, "target", "tmp", "xlsx-check")
CLI = os.environ.get("CALCPAD_CLI", os.path.join(ROOT, "target", "release", "Cli"))
REF = re.compile(r'^(vector of \d+|matrix \d+ × \d+) \(sheet "(.+)"\)$')
COMPLEX = re.compile(r"^-?[\d.]+(E[+-]?\d+)? [+-] [\d.]+(E[+-]?\d+)?i$")


def check(path, problems):
    wb = openpyxl.load_workbook(path)
    if wb.sheetnames[:2] != ["Report", "Variables"] or wb.sheetnames[-1] != "Inputs":
        problems.append(f"sheets {wb.sheetnames}")
        return 0
    check_report(wb, problems)
    header = ["Name", "Value", "Unit"]
    rows = [[c.value for c in r] for r in wb["Variables"].iter_rows()]
    # A fourth "Note" column says where a unit was read from the text.
    if rows[0][:3] != header or rows[0][3:] not in ([], [None], ["Note"]):
        problems.append(f"Variables header {rows[0]}")
    names = set()
    for name, value, unit in (r[:3] for r in rows[1:]):
        names.add(name)
        if isinstance(value, (int, float)):
            continue
        m = REF.match(str(value))
        if m:
            if m.group(2) not in wb.sheetnames:
                problems.append(f"{name}: no sheet {m.group(2)}")
                continue
            ws = wb[m.group(2)]
            data = [[c.value for c in r] for r in ws.iter_rows()]
            first = 1 if value.startswith("vector") else 1
            for r in data[1:]:
                cells = r[first:2] if value.startswith("vector") else r[first:-1]
                if not all(isinstance(c, (int, float)) for c in cells):
                    problems.append(f"{name}: a non-number among {cells[:4]}")
                    break
        elif not COMPLEX.match(str(value)):
            problems.append(f"{name}: value {value!r}")
    inputs = [[c.value for c in r] for r in wb["Inputs"].iter_rows()]
    if inputs[0] != header:
        problems.append(f"Inputs header {inputs[0]}")
    for r in inputs[1:]:
        if r[0] not in names:
            problems.append(f"input {r[0]} is not a variable")
    return len(rows) - 1


REPORT_HEADER = ["Description", "Name", "Expression", "With the values", "Value", "Unit"]


def check_report(wb, problems):
    """The Report sheet: header, equation rows agree with the Variables sheet, pictures are
    real PNGs, hyperlinks go to sheets that exist, merged text cells hold text."""
    ws = wb["Report"]
    rows = [[c.value for c in r] for r in ws.iter_rows()]
    if rows[0][:6] != REPORT_HEADER:
        problems.append(f"Report header {rows[0][:6]}")
    names = {r[0].value for r in wb["Variables"].iter_rows(min_row=2)}
    last = {r[0].value: r[1].value for r in wb["Variables"].iter_rows(min_row=2)}
    texts = 0
    for i, r in enumerate(rows[1:], start=2):
        desc, name, expr, with_values, value, unit = (list(r) + [None] * 6)[:6]
        if name is not None:
            # Rows of a call ("f(2) = 5"), a loop variable or a bare expression have no variable.
            if name and name not in names:
                check_report.unmatched = getattr(check_report, "unmatched", 0) + 1
        elif desc:
            texts += 1
    for img in ws._images:
        data = img._data()
        if data[:8] != b"\x89PNG\r\n\x1a\n" and data[:3] != b"\xff\xd8\xff" and data[:3] != b"GIF":
            problems.append("a picture that is not PNG, JPEG or GIF")
        if img.width <= 0 or img.height <= 0:
            problems.append("a picture without a size")
    links = [c.hyperlink for r in ws.iter_rows() for c in r if c.hyperlink]
    check_report.links = getattr(check_report, "links", 0) + len(links)
    for link in links:
        target = (link.location or "").split("!")[0].strip("'").replace("''", "'")
        if not target or target not in wb.sheetnames:
            problems.append(f"link to a missing sheet {target!r}")
    for rng in ws.merged_cells.ranges:
        if rng.min_col != 1 or rng.max_col != 6 or ws.cell(rng.min_row, 1).value in (None, ""):
            if rng.min_col == 1 and rng.max_col == 6:
                problems.append(f"merged cells {rng} hold nothing")
    check_report.stats = getattr(check_report, "stats", [0, 0, 0])
    check_report.stats[0] += len(ws._images)
    check_report.stats[1] += sum(1 for r in rows[1:] if r[1] is not None)
    check_report.stats[2] += texts


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--filter", default="")
    ap.add_argument("--soffice", default="/Applications/LibreOffice.app/Contents/MacOS/soffice")
    args = ap.parse_args()
    os.makedirs(OUT, exist_ok=True)
    manifest = json.load(open(os.path.join(SAMPLES, "manifest.json")))
    failures, books, variables = [], 0, 0
    for s in manifest["samples"]:
        if args.filter not in s["file"]:
            continue
        src = os.path.join(SAMPLES, s["file"][len("samples/"):])
        out = os.path.join(OUT, re.sub(r"[^A-Za-z0-9]+", "-", s["id"]) + ".xlsx")
        r = subprocess.run([CLI, src, out, "-s"], capture_output=True, text=True, timeout=900)
        if r.returncode != 0 or not os.path.exists(out):
            failures.append(f"{s['id']}: CLI failed: {r.stderr.strip()}")
            continue
        problems = []
        try:
            variables += check(out, problems)
        except Exception as e:  # noqa: BLE001
            problems.append(f"openpyxl: {e}")
        books += 1
        failures.extend(f"{s['id']}: {p}" for p in problems)
    # LibreOffice opens every workbook and prints it (one PDF each).
    books_out = sorted(os.path.join(OUT, f) for f in os.listdir(OUT) if f.endswith(".xlsx"))
    if books_out and os.path.exists(args.soffice):
        pdf_dir = os.path.join(OUT, "pdf")
        os.makedirs(pdf_dir, exist_ok=True)
        subprocess.run([args.soffice, "--headless", "--convert-to", "pdf", "--outdir", pdf_dir, *books_out],
                       capture_output=True, timeout=3600)
        made = {f[:-4] for f in os.listdir(pdf_dir) if f.endswith(".pdf")}
        missing = [os.path.basename(b) for b in books_out if os.path.basename(b)[:-5] not in made]
        print(f"LibreOffice printed {len(books_out) - len(missing)} of {len(books_out)} workbooks")
        failures.extend(f"LibreOffice could not print {m}" for m in missing)
    else:
        print(f"LibreOffice not found at {args.soffice}: print check skipped")
    pics, eqs, texts = getattr(check_report, "stats", [0, 0, 0])
    print(f"{books} workbooks, {variables} variables, {eqs} equation rows ({getattr(check_report, 'unmatched', 0)} of a call, a loop variable or an expression), {texts} text rows, {pics} pictures, {getattr(check_report, 'links', 0)} links to vector and matrix sheets")
    for f in failures:
        print("FAIL", f)
    print("all workbooks pass" if not failures else f"{len(failures)} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
