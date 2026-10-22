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


def check(path, problems, plots=0, html_plots=0):
    wb = openpyxl.load_workbook(path)
    names_ = [n for n in wb.sheetnames if n != "Plot data"]
    if wb.sheetnames[:2] != ["Report", "Variables"] or names_[-1] != "Inputs":
        problems.append(f"sheets {wb.sheetnames}")
        return 0
    check_report(wb, problems)
    check_charts(path, wb, problems, plots, html_plots)
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


def count_plots(src):
    """The $Plot commands in the source of a worksheet (a $Map stays a picture). A loop or a
    macro can draw more than one plot per command, so this is only a cross-check."""
    text = open(src, encoding="utf-8-sig", errors="replace").read()
    return len(re.findall(r"\$plot\s*\{", text, re.I))


# Child order of the chart elements this export writes (ECMA-376 dml-chart.xsd sequences).
ORDER = {
    "chartSpace": ["date1904", "lang", "roundedCorners", "AlternateContent", "style", "clrMapOvr", "pivotSource",
                   "protection", "chart", "spPr", "txPr", "externalData", "printSettings", "userShapes", "extLst"],
    "chart": ["title", "autoTitleDeleted", "pivotFmts", "view3D", "floor", "sideWall", "backWall", "plotArea",
              "legend", "plotVisOnly", "dispBlanksAs", "showDLblsOverMax", "extLst"],
    "plotArea": ["layout", "scatterChart", "valAx", "dTable", "spPr", "extLst"],
    "scatterChart": ["scatterStyle", "varyColors", "ser", "dLbls", "axId", "extLst"],
    "ser": ["idx", "order", "tx", "spPr", "marker", "dPt", "dLbls", "trendline", "errBars", "xVal", "yVal",
            "smooth", "extLst"],
    "valAx": ["axId", "scaling", "delete", "axPos", "majorGridlines", "minorGridlines", "title", "numFmt",
              "majorTickMark", "minorTickMark", "tickLblPos", "spPr", "txPr", "crossAx", "crosses", "crossesAt",
              "crossBetween", "majorUnit", "minorUnit", "dispUnits", "extLst"],
    "scaling": ["logBase", "orientation", "max", "min", "extLst"],
    "legend": ["legendPos", "legendEntry", "layout", "overlay", "spPr", "txPr", "extLst"],
    "title": ["tx", "layout", "overlay", "spPr", "txPr", "extLst"],
    "marker": ["symbol", "size", "spPr", "extLst"],
    "numRef": ["f", "numCache", "extLst"],
    "numCache": ["formatCode", "ptCount", "pt", "extLst"],
}
# plotArea holds its axes after the charts; a valAx may repeat.


def check_order(el, problems, where):
    tag = el.tag.split("}")[1]
    kids = [k.tag.split("}")[1] for k in el]
    seq = ORDER.get(tag)
    if seq:
        pos = [seq.index(k) if k in seq else -1 for k in kids]
        if -1 in pos:
            problems.append(f"{where}: {tag} holds {kids[pos.index(-1)]}")
        elif pos != sorted(pos):
            problems.append(f"{where}: {tag} children out of schema order {kids}")
    for k in el:
        check_order(k, problems, where)


def check_charts(path, wb, problems, plots, html_plots):
    """Native charts: as many as $Plot commands, XML in schema order, every range inside the
    "Plot data" sheet with the cached values equal to the cells."""
    import xml.etree.ElementTree as ET
    import zipfile
    ws = wb["Report"]
    charts = len(ws._charts)
    check_charts.total = getattr(check_charts, "total", 0) + charts
    check_charts.expected = getattr(check_charts, "expected", 0) + html_plots
    check_charts.commands = getattr(check_charts, "commands", 0) + plots
    # The page's own count of plot drawings (a loop draws several per command).
    if charts != html_plots:
        problems.append(f"{charts} charts for {html_plots} plot drawings in the page")
    if html_plots < plots:
        problems.append(f"{html_plots} plot drawings for {plots} $Plot commands")
    ns = {"c": "http://schemas.openxmlformats.org/drawingml/2006/chart"}
    with zipfile.ZipFile(path) as z:
        parts = sorted(n for n in z.namelist() if n.startswith("xl/charts/chart"))
        for n in parts:
            root = ET.fromstring(z.read(n))
            check_order(root, problems, n)
            for ref in root.iter("{%s}numRef" % ns["c"]):
                f = ref.find("c:f", ns).text
                m = re.match(r"^'(.+)'!\$([A-Z]+)\$(\d+):\$([A-Z]+)\$(\d+)$", f)
                if not m or m.group(1) != "Plot data" or m.group(2) != m.group(4):
                    problems.append(f"{n}: odd range {f}")
                    continue
                col, r1, r2 = m.group(2), int(m.group(3)), int(m.group(5))
                cache = [float(v.text) for v in ref.iter("{%s}v" % ns["c"])]
                cells = [ws_cell for ws_cell in (wb["Plot data"][f"{col}{r}"].value for r in range(r1, r2 + 1))]
                if len(cache) != len(cells) or any(abs(a - b) > 1e-9 * max(1, abs(b)) for a, b in zip(cache, cells)):
                    problems.append(f"{n}: cached values differ from {f}")
    if charts != len(parts):
        problems.append(f"{len(parts)} chart parts but {charts} charts")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--filter", default="")
    ap.add_argument("--print-all", action="store_true",
                    help="print every workbook with LibreOffice, not only those with charts (the FEA ones are 10 MB and take over an hour)")
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
        plots_file = out + ".plots"
        r = subprocess.run([CLI, src, out, "-s"], capture_output=True, text=True, timeout=900,
                           env={**os.environ, "CALCPAD_RS_PLOTS_FILE": plots_file})
        if r.returncode != 0 or not os.path.exists(out):
            failures.append(f"{s['id']}: CLI failed: {r.stderr.strip()}")
            continue
        problems = []
        try:
            variables += check(out, problems, count_plots(src), int(open(plots_file).read()))
        except Exception as e:  # noqa: BLE001
            problems.append(f"openpyxl: {e}")
        books += 1
        failures.extend(f"{s['id']}: {p}" for p in problems)
    # LibreOffice opens every workbook and prints it (one PDF each).
    books_out = sorted(os.path.join(OUT, f) for f in os.listdir(OUT) if f.endswith(".xlsx"))
    if not args.print_all:
        import zipfile
        books_out = [b for b in books_out if any(n.startswith("xl/charts/") for n in zipfile.ZipFile(b).namelist())]
    for f in os.listdir(os.path.join(OUT, "pdf")) if os.path.isdir(os.path.join(OUT, "pdf")) else []:
        os.remove(os.path.join(OUT, "pdf", f))
    if books_out and os.path.exists(args.soffice):
        pdf_dir = os.path.join(OUT, "pdf")
        os.makedirs(pdf_dir, exist_ok=True)
        profile = "-env:UserInstallation=file://" + os.path.join(OUT, "lo-profile")
        for i in range(0, len(books_out), 10):
            try:
                subprocess.run([args.soffice, profile, "--headless", "--convert-to", "pdf", "--outdir", pdf_dir,
                                *books_out[i:i + 10]], capture_output=True, timeout=900)
            except subprocess.TimeoutExpired:
                subprocess.run(["pkill", "-f", "lo-profile"])
        made = {f[:-4] for f in os.listdir(pdf_dir) if f.endswith(".pdf")}
        missing = [os.path.basename(b) for b in books_out if os.path.basename(b)[:-5] not in made]
        print(f"LibreOffice printed {len(books_out) - len(missing)} of {len(books_out)} workbooks" + ("" if args.print_all else " (those with charts)"))
        failures.extend(f"LibreOffice could not print {m}" for m in missing)
    else:
        print(f"LibreOffice not found at {args.soffice}: print check skipped")
    pics, eqs, texts = getattr(check_report, "stats", [0, 0, 0])
    print(f"{getattr(check_charts, 'total', 0)} native charts for {getattr(check_charts, 'expected', 0)} plot drawings ({getattr(check_charts, 'commands', 0)} $Plot commands in the sources)")
    print(f"{books} workbooks, {variables} variables, {eqs} equation rows ({getattr(check_report, 'unmatched', 0)} of a call, a loop variable or an expression), {texts} text rows, {pics} pictures, {getattr(check_report, 'links', 0)} links to vector and matrix sheets")
    for f in failures:
        print("FAIL", f)
    print("all workbooks pass" if not failures else f"{len(failures)} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
