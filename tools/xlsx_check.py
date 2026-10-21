#!/usr/bin/env python3
"""Excel export check (calcpad-rs's own feature) over the Samples, read with openpyxl.

For every sample, the CLI writes its variables as .xlsx; openpyxl must open the workbook, and:
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
CLI = os.path.join(ROOT, "target", "release", "Cli")
REF = re.compile(r'^(vector of \d+|matrix \d+ × \d+) \(sheet "(.+)"\)$')
COMPLEX = re.compile(r"^-?[\d.]+(E[+-]?\d+)? [+-] [\d.]+(E[+-]?\d+)?i$")


def check(path, problems):
    wb = openpyxl.load_workbook(path)
    if wb.sheetnames[0] != "Variables" or wb.sheetnames[-1] != "Inputs":
        problems.append(f"sheets {wb.sheetnames}")
        return 0
    header = ["Name", "Value", "Unit"]
    rows = [[c.value for c in r] for r in wb["Variables"].iter_rows()]
    if rows[0] != header:
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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--filter", default="")
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
    print(f"{books} workbooks, {variables} variables")
    for f in failures:
        print("FAIL", f)
    print("all workbooks pass" if not failures else f"{len(failures)} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
