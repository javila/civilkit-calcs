"""Generate calcpad-core/src/messages.rs from Calcpad.Core/Messages.resx (English)."""
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ref = Path(sys.argv[1])
out = Path(sys.argv[2])
root = ET.parse(ref / "Calcpad.Core" / "Messages.resx").getroot()
lines = [
    "//! English message strings, generated from Calcpad.Core/Messages.resx by tools/gen_messages.py.",
    "#![allow(non_upper_case_globals, dead_code)]",
    "",
]
for d in root.findall("data"):
    name = d.get("name")
    v = d.find("value").text or ""
    lit = v.replace("\\", "\\\\").replace('"', '\\"').replace("\r\n", "\\n").replace("\n", "\\n")
    lines.append(f'pub const {name}: &str = "{lit}";')
out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(len(lines) - 3, "messages")
