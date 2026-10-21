#!/usr/bin/env python3
"""Word export check over the Samples (there is no upstream Word oracle: see DEVIATIONS D6).

For every sample in web/calcpad-web/public/samples/manifest.json:
  * the CLI writes it as .docx (plots as SVG, so they go through resvg);
  * word/document.xml must be well-formed XML;
  * the document must hold as many Office Math equations as the HTML render has equations
    that the writer can see (not in comments, hidden elements or matrix tables);
  * no image may be missing ("Error adding image").
Then LibreOffice (headless, one process, files in turn) converts every document to PDF, which
it can only do for a document it can open.

    python3 -I tools/docx_check.py [--soffice PATH] [--filter TEXT]
"""
import argparse
import json
import os
import re
import subprocess
import sys
import zipfile
import xml.etree.ElementTree as ET
from html.parser import HTMLParser

VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}


class EquationCounter(HTMLParser):
    """The equations the writer can see: span.eq outside comments, hidden elements
    (display:none, visibility:hidden, .lineLink, .errorHeader) and matrix tables."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack, self.count = [], 0

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        cls = (a.get("class") or "").split()
        style = re.sub(r"\s+", "", (a.get("style") or "").lower())
        hidden = ("lineLink" in cls or "errorHeader" in cls or "display:none" in style
                  or "visibility:hidden" in style or (tag == "table" and "matrix" in cls))
        if tag == "span" and "eq" in cls and not any(self.stack):
            self.count += 1
        if tag not in VOID:
            self.stack.append(hidden)

    def handle_endtag(self, tag):
        if tag not in VOID and self.stack:
            self.stack.pop()


def equations_in(html):
    c = EquationCounter()
    c.feed(html)
    return c.count

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SAMPLES = os.path.join(ROOT, "web", "calcpad-web", "public", "samples")
OUT = os.path.join(ROOT, "target", "tmp", "docx-check")
CLI = os.path.join(ROOT, "target", "release", "Cli")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--soffice", default="/Applications/LibreOffice.app/Contents/MacOS/soffice")
    ap.add_argument("--filter", default="")
    args = ap.parse_args()
    os.makedirs(OUT, exist_ok=True)
    settings = os.path.join(OUT, "settings.xml")
    with open(settings, "w") as f:
        f.write("<Settings><Math><Decimals>4</Decimals><FormatEquations>true</FormatEquations></Math>"
                "<Plot><VectorGraphics>true</VectorGraphics></Plot></Settings>")
    env = dict(os.environ, CALCPAD_SETTINGS=settings)
    manifest = json.load(open(os.path.join(SAMPLES, "manifest.json")))
    failures, docs = [], []
    for s in manifest["samples"]:
        if args.filter not in s["file"]:
            continue
        src = os.path.join(SAMPLES, s["file"][len("samples/"):])
        name = re.sub(r"[^A-Za-z0-9]+", "-", s["id"]) + ".docx"
        docx = os.path.join(OUT, name)
        html = docx[:-5] + ".html"
        for out in (docx, html):
            r = subprocess.run([CLI, src, out, "-s", "-b"], env=env, capture_output=True, text=True, timeout=900)
            if r.returncode != 0:
                failures.append(f"{s['id']}: CLI failed for {os.path.basename(out)}: {r.stderr.strip()}")
        if not os.path.exists(docx):
            continue
        try:
            z = zipfile.ZipFile(docx)
            doc = z.read("word/document.xml").decode("utf-8")
            ET.fromstring(doc)
        except Exception as e:  # noqa: BLE001
            failures.append(f"{s['id']}: invalid package or XML: {e}")
            continue
        maths = doc.count("<m:oMath>")
        eqs = equations_in(open(html, encoding="utf-8").read())
        if maths != eqs:
            failures.append(f"{s['id']}: {maths} Office Math equations, {eqs} in the HTML")
        # A file upstream does not ship (manifest "missingAssets") cannot be in the document.
        if "Error adding image" in doc and not s.get("missingAssets"):
            m = re.search(r"Error adding image[^<]*", doc)
            failures.append(f"{s['id']}: {m.group(0)}")
        docs.append(docx)
    pdf_dir = os.path.join(OUT, "pdf")
    os.makedirs(pdf_dir, exist_ok=True)
    for d in docs:
        p = os.path.join(pdf_dir, os.path.basename(d)[:-5] + ".pdf")
        if os.path.exists(p):
            os.remove(p)
    if docs and os.path.exists(args.soffice):
        subprocess.run([args.soffice, "--headless", "--convert-to", "pdf", "--outdir", pdf_dir, *docs],
                       capture_output=True, timeout=3600)
        for d in docs:
            p = os.path.join(pdf_dir, os.path.basename(d)[:-5] + ".pdf")
            if not os.path.exists(p) or os.path.getsize(p) < 1000:
                failures.append(f"{os.path.basename(d)}: LibreOffice could not convert it")
    elif docs:
        print(f"LibreOffice not found at {args.soffice}: load check skipped")
    print(f"{len(docs)} documents written")
    for f in failures:
        print("FAIL", f)
    print("all documents pass" if not failures else f"{len(failures)} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
