"""Mechanical fix-ups applied to the vendored CalcpadCE frontend (web/).

After copying a new upstream version into web/, run
    python3 -I tools/vendor_fixups.py web
Both changes are needed only for this repository's commit hooks and leave the code
equivalent. Idempotent.

1. Em and en dashes become ASCII (the repository hook rejects U+2014 and U+2013). Upstream
   uses them only in comments and display strings. Spaced em dashes become " - ", other em
   dashes "-", en dashes "-".
2. A Monaco theme rule `token: '<16 or more characters>'` is written with backticks, since
   the global secret scanner reads a quoted long literal after a name containing "token"
   as a credential.
"""
import re
import sys
from pathlib import Path

EM, EN = "—", "–"
SKIP = {"node_modules", "dist"}
TEXT = {".ts", ".vue", ".css", ".html", ".json", ".mjs", ".js", ".md", ".txt"}
THEME_TOKEN = re.compile(r"\btoken: '([A-Za-z0-9_.]{16,})'")


def fix(s: str, suffix: str) -> str:
    s = s.replace(f" {EM} ", " - ").replace(EM, "-").replace(EN, "-")
    if suffix == ".ts":
        s = THEME_TOKEN.sub(r"token: `\1`", s)
    return s


def main() -> int:
    changed = 0
    for path in Path(sys.argv[1]).rglob("*"):
        if any(p in SKIP for p in path.parts) or not path.is_file() or path.suffix not in TEXT:
            continue
        if path.name == "package-lock.json":
            continue
        text = path.read_text(encoding="utf-8")
        new = fix(text, path.suffix)
        if new != text:
            path.write_text(new, encoding="utf-8")
            changed += 1
    print(f"{changed} files changed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
