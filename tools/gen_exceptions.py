"""Generate calcpad-core/src/exceptions.rs from Calcpad.Core/Exceptions.cs.

Each `internal static MathParserException Name(args) => new(expr);` becomes a Rust
function with the same name returning MathParserException. string.Format(...) maps to
the `fmt` helper; ItemToString / Messages.X map to their Rust counterparts.
"""
import re
import sys
from pathlib import Path

ref = Path(sys.argv[1])
out = Path(sys.argv[2])
src = (ref / "Calcpad.Core" / "Exceptions.cs").read_text(encoding="utf-8-sig")
pat = re.compile(r"internal static MathParserException (\w+)\(([^)]*)\)\s*=>\s*new\s*\((.*?)\);", re.S)
type_map = {"string": "&str", "char": "char", "int": "i32", "Items": "Items"}
fns = []
seen = {}
for m in pat.finditer(src):
    name, params, body = m.group(1), m.group(2).strip(), " ".join(m.group(3).split())
    rparams = []
    names = []
    ptypes = []
    for p in [p.strip() for p in params.split(",") if p.strip()]:
        t, n = p.split()
        rn = n + "_" if n in ("type",) else n
        rparams.append(f"{rn}: {type_map[t]}")
        names.append(n)
        ptypes.append(t)
    seen[name] = seen.get(name, 0) + 1
    rname = name if seen[name] == 1 else f"{name}{seen[name]}"

    def conv(e):
        e = e.strip()
        fm = re.fullmatch(r"string\.Format\((Messages\.\w+),\s*(.*)\)", e)
        if fm:
            args = [a.strip() for a in fm.group(2).split(",")]
            rargs = []
            for a in args:
                if a in names:
                    ra = a + "_" if a in ("type",) else a
                    pt = ptypes[names.index(a)]
                    rargs.append(ra if pt == "string" else f"&{ra}.to_string()")
                elif a == "Vector.MaxLength":
                    rargs.append('"10000000"')
                elif a == "Matrix.MaxSize":
                    rargs.append('"1000000"')
                elif a == "UiDto.MaxSize":
                    rargs.append('"100000"')
                elif a.startswith("ExpressionParser.MaxEmbeddedDataSize"):
                    rargs.append('"10"')
                else:
                    raise SystemExit(f"unhandled arg {a} in {name}")
            return f"fmt(messages::{fm.group(1)[9:]}, &[{', '.join(rargs)}])"
        im = re.fullmatch(r"ItemToString\((\w+)\)\s*\+\s*Messages\.(\w+)", e)
        if im:
            return f"format!(\"{{}}{{}}\", item_to_string({im.group(1)}), messages::{im.group(2)})"
        mm = re.fullmatch(r"Messages\.(\w+)", e)
        if mm:
            return f"messages::{mm.group(1)}.to_string()"
        raise SystemExit(f"unhandled body {e} in {name}")

    fns.append(f"pub fn {rname}({', '.join(rparams)}) -> MathParserException {{\n    MathParserException::new({conv(body)})\n}}\n")

header = '''//! Error constructors, generated from Calcpad.Core/Exceptions.cs by tools/gen_exceptions.py.
//! Function names keep the C# names so call sites read the same as upstream.
#![allow(non_snake_case, dead_code)]

use crate::math_parser_exception::MathParserException;
use crate::messages;
use crate::netfmt::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Items {
    Index,
    Count,
    NumRows,
    NumCols,
    Argument,
    Result,
    Variable,
    IndexTarget,
    Limit,
    Value,
}

fn item_to_string(item: Items) -> &'static str {
    match item {
        Items::Index => messages::Index,
        Items::Count => messages::Count,
        Items::NumRows => messages::Number_of_rows,
        Items::NumCols => messages::Number_of_columns,
        Items::Argument => messages::Argument,
        Items::Result => messages::Result,
        Items::Variable => messages::Variable,
        Items::IndexTarget => messages::Index_target,
        Items::Limit => messages::Limit,
        Items::Value => messages::Value,
    }
}

'''
fns.append('pub fn InvalidNumber(s: &str) -> MathParserException {\n    InvalidLiteral(s, "number")\n}\n')
out.write_text(header + "\n".join(fns), encoding="utf-8")
print(len(fns), "constructors")
