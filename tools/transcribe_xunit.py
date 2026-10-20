"""Transcribe CalcpadCE xunit test classes (C#) into Rust integration tests.

The scalar tests use a small, regular subset of C#: TestCalc calls, Assert.* calls,
System.Math, numeric and string literals, local variables and class level constants.
This tool translates that subset token by token so each [Fact] becomes a #[test] with
the same name, inputs, expected values and tolerances. Methods it cannot translate are
taken from tools/xunit_overrides/<Class>.<Method>.rs (hand transcriptions) or emitted
as #[ignore] with the reason, so nothing is silently dropped.

Usage: python -I tools/transcribe_xunit.py <ref repo> <out dir> <relative .cs path>...
"""
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
OVERRIDES = HERE / "xunit_overrides"

TOKEN_RE = re.compile(
    r"""
    (?P<ws>\s+)
  | (?P<comment>//[^\n]*)
  | (?P<istr>\$"(?:[^"\\]|\\.)*")
  | (?P<str>"(?:[^"\\]|\\.)*")
  | (?P<chr>'(?:[^'\\]|\\.)+')
  | (?P<num>(?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?[dDfFmM]?)
  | (?P<id>[^\W\d][\w]*)
  | (?P<op>=>|==|!=|<=|>=|&&|\|\||\+\+|--|[-+*/%<>=!&|^~?:;,.(){}\[\]])
    """,
    re.VERBOSE,
)


def tokenize(src):
    out = []
    for m in TOKEN_RE.finditer(src):
        kind = m.lastgroup
        if kind in ("ws", "comment"):
            continue
        out.append((kind, m.group()))
    return out


class Untranslatable(Exception):
    pass


def rust_string(cs):
    body = cs[1:-1]
    out = []
    i = 0
    while i < len(body):
        c = body[i]
        if c == "\\":
            n = body[i + 1]
            if n == "u":
                out.append("\\u{" + body[i + 2 : i + 6] + "}")
                i += 6
                continue
            out.append(c + n)
            i += 2
            continue
        out.append(c)
        i += 1
    return '"' + "".join(out) + '"'


def rust_number(tok, as_float=True):
    t = tok
    suffix = ""
    if t[-1] in "dDfFmM":
        t = t[:-1]
        as_float = True
    if not as_float:
        return t
    if "." in t or "e" in t or "E" in t:
        if t.startswith("."):
            t = "0" + t
        if t.endswith("."):
            t += "0"
        return t + suffix
    return t + ".0"


SETTING_PROPS = {"Degrees", "IsComplex", "Decimals", "Substitute", "FormatEquations"}


class Translator:
    def __init__(self, fields_fn, string_fields, helpers):
        self.fields_fn = fields_fn  # names of readonly fields emitted as functions
        self.string_fields = string_fields
        self.helpers = helpers  # helper method name -> list of (param types)

    def expr(self, toks, i, stop):
        """Translate tokens from i until a token in stop at depth 0. Returns (text, i)."""
        out = []
        depth = 0
        while i < len(toks):
            kind, v = toks[i]
            if depth == 0 and v in stop:
                break
            if v in "([{":
                depth += 1
            elif v in ")]}":
                if depth == 0:
                    break
                depth -= 1
            text, i = self.atom(toks, i)
            out.append(text)
        return " ".join(out), i

    def args(self, toks, i):
        """toks[i] == '('; returns (list of arg texts, index after ')')."""
        assert toks[i][1] == "(", toks[i]
        i += 1
        res = []
        if toks[i][1] == ")":
            return res, i + 1
        while True:
            a, i = self.expr(toks, i, {",", ")"})
            res.append(a)
            if toks[i][1] == ",":
                i += 1
                continue
            assert toks[i][1] == ")"
            return res, i + 1

    def settings(self, toks, i):
        """toks[i] == 'new', toks[i+1] == '(' : `new()` or `new() { P = v, ... }`."""
        assert toks[i][1] == "new" and toks[i + 1][1] == "(" and toks[i + 2][1] == ")"
        i += 3
        s = "ms()"
        if i < len(toks) and toks[i][1] == "{":
            i += 1
            while toks[i][1] != "}":
                prop = toks[i][1]
                if prop not in SETTING_PROPS or toks[i + 1][1] != "=":
                    raise Untranslatable(f"settings property {prop}")
                val, i = self.expr(toks, i + 2, {",", "}"})
                s += f".{prop}({val})"
                if toks[i][1] == ",":
                    i += 1
            i += 1
        return s, i

    def atom(self, toks, i):
        kind, v = toks[i]
        nxt = toks[i + 1][1] if i + 1 < len(toks) else None
        if kind == "num":
            prev = toks[i - 1][1] if i > 0 else None
            nxt_v = nxt
            is_index = prev == "[" and nxt_v == "]" and i >= 2 and toks[i - 2][0] == "id"
            return rust_number(v, not is_index), i + 1
        if kind == "str":
            return rust_string(v), i + 1
        if kind == "istr":
            raise Untranslatable("interpolated string")
        if kind == "chr":
            return v, i + 1
        if kind == "id":
            if v == "new" and nxt == "TestCalc":
                args, j = self.args(toks, i + 2) if False else (None, None)
                # new TestCalc(<settings>)
                assert toks[i + 2][1] == "("
                s, j = self.settings(toks, i + 3)
                assert toks[j][1] == ")", toks[j]
                return f"TestCalc::new({s})", j + 1
            if v == "new":
                raise Untranslatable("new " + str(nxt))
            if v == "Assert" and nxt == ".":
                return self.assert_call(toks, i)
            if v == "Math" and nxt == ".":
                return f"Math::{toks[i + 2][1]}", i + 3
            if v == "double" and nxt == ".":
                name = toks[i + 2][1]
                m = {"NaN": "f64::NAN", "PositiveInfinity": "f64::INFINITY", "NegativeInfinity": "f64::NEG_INFINITY", "Epsilon": "4.9406564584124654e-324", "MaxValue": "f64::MAX"}
                if name not in m:
                    raise Untranslatable("double." + name)
                return m[name], i + 3
            if v == "TestCalc" and nxt == ".":
                name = toks[i + 2][1]
                args, j = self.args(toks, i + 3)
                return f"TestCalc::{name}({', '.join(args)})", j
            if v in self.helpers and nxt == "(":
                args, j = self.args(toks, i + 1)
                return self.helper_call(v, args), j
            if v in self.fields_fn:
                return f"{v}()", i + 1
            if v == "var":
                return "let mut", i + 1
            if v == "true" or v == "false":
                return v, i + 1
            return v, i + 1
        if v == ".":
            name = nxt
            after = toks[i + 2][1] if i + 2 < len(toks) else None
            if name == "Run" and after == "(":
                if toks[i + 3][1] == "[":
                    # Run([ ... ]) -> RunAll(&[ ... ])
                    inner, j = self.expr(toks, i + 4, {"]"})
                    assert toks[j][1] == "]" and toks[j + 1][1] == ")"
                    return f".RunAll(&[{inner}])", j + 2
                return ".Run", i + 2
            if name in ("Real", "Imaginary", "Complex") and after != "(":
                return f".{name}()", i + 2
            if name == "Message":
                return ".message", i + 2
            return "." + name, i + 2
        if v == "=>":
            raise Untranslatable("lambda")
        return v, i + 1

    def helper_call(self, name, args):
        overloads = self.helpers[name]
        if len(overloads) == 1:
            return f"{name}({', '.join(args)})"
        is_str = args[1].startswith('"')
        return f"{name}_{'str' if is_str else 'f64'}({', '.join(args)})"

    def assert_call(self, toks, i):
        name = toks[i + 2][1]
        j = i + 3
        if name == "Throws":
            # Assert.Throws<MathParserException>( () => { body } )
            if [t[1] for t in toks[j : j + 3]] != ["<", "MathParserException", ">"]:
                raise Untranslatable("Throws type")
            j += 3
            assert toks[j][1] == "("
            assert [t[1] for t in toks[j + 1 : j + 5]] == ["(", ")", "=>", "{"], toks[j : j + 5]
            body, k = self.block(toks, j + 4)
            assert toks[k][1] == ")"
            return f"assert_throws(|| {body})", k + 1
        args, k = self.args(toks, j)
        if name == "Equal":
            if len(args) == 3:
                return f"assert_equal_tol({', '.join(args)})", k
            if args[0].startswith('"') or args[1].startswith('"'):
                return f"assert_equal_str({args[0]}, &{args[1]})", k
            return f"assert_equal({', '.join(args)})", k
        if name == "True":
            return f"assert!({args[0]})", k
        if name == "False":
            return f"assert!(!({args[0]}))", k
        if name == "InRange":
            return f"assert_in_range({', '.join(args)})", k
        if name == "Contains":
            return f"assert_contains({args[0]}, &{args[1]})", k
        if name == "DoesNotContain":
            return f"assert_does_not_contain({args[0]}, &{args[1]})", k
        raise Untranslatable("Assert." + name)

    def block(self, toks, i):
        """toks[i] == '{'; translate statements until matching '}'. Returns (text, index after)."""
        assert toks[i][1] == "{"
        i += 1
        stmts = []
        while toks[i][1] != "}":
            s, i = self.expr(toks, i, {";"})
            assert toks[i][1] == ";", toks[i]
            i += 1
            stmts.append(s + ";")
        return "{ " + " ".join(stmts) + " }", i + 1


def strip_comments(src):
    """Remove // comments outside string literals."""
    out = []
    i = 0
    in_str = False
    while i < len(src):
        c = src[i]
        if in_str:
            out.append(c)
            if c == "\\":
                out.append(src[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
            out.append(c)
        elif src.startswith("//", i):
            j = src.find("\n", i)
            i = len(src) if j < 0 else j
            continue
        else:
            out.append(c)
        i += 1
    return "".join(out)


def split_members(src):
    """Yield (attributes, header, body or expr) for class members."""
    src = strip_comments(src)
    # strip namespace and class wrappers
    m = re.search(r"public\s+class\s+(\w+)", src)
    cls = m.group(1)
    start = src.index("{", m.end())
    end = src.rindex("}")
    end = src.rindex("}", 0, end)
    body = src[start + 1 : end]
    return cls, body


def find_matching(s, i, open_c="{", close_c="}"):
    depth = 0
    k = i
    in_str = None
    while k < len(s):
        c = s[k]
        if in_str:
            if c == "\\":
                k += 2
                continue
            if c == in_str:
                in_str = None
        elif c == '"':
            in_str = '"'
        elif c == open_c:
            depth += 1
        elif c == close_c:
            depth -= 1
            if depth == 0:
                return k
        k += 1
    raise ValueError("unbalanced")


def parse_members(body):
    members = []
    i = 0
    n = len(body)
    attrs = []
    while i < n:
        # skip whitespace, comments, regions
        m = re.match(r"\s+|//[^\n]*|#(?:region|endregion)[^\n]*", body[i:])
        if m and m.end() > 0:
            i += m.end()
            continue
        if body[i] == "[":
            j = find_matching(body, i, "[", "]")
            attrs.append(body[i : j + 1])
            i = j + 1
            continue
        # member declaration up to '{', '=>' or ';' (field)
        rest = body[i:]
        hm = re.match(r"([^{;=]*?)(\(\s*[^)]*\))?\s*(=>|\{|;|=)", rest, re.S)
        if not hm:
            raise ValueError(rest[:200])
        header = (hm.group(1) + (hm.group(2) or "")).strip()
        sep = hm.group(3)
        k = i + hm.end()
        if sep == "{":
            j = find_matching(body, k - 1)
            members.append((attrs, header, "block", body[k - 1 : j + 1]))
            i = j + 1
        elif sep == "=>":
            j = body.index(";\n", k)
            members.append((attrs, header, "expr", body[k:j]))
            i = j + 1
        elif sep == "=":
            # field initializer, may contain [ ... ] spanning lines
            depth = 0
            j = k
            while j < n:
                c = body[j]
                if c in "[({":
                    depth += 1
                elif c in "])}":
                    depth -= 1
                elif c == '"':
                    j = body.index('"', j + 1)
                elif c == ";" and depth == 0:
                    break
                j += 1
            members.append((attrs, header, "field", body[k:j]))
            i = j + 1
        else:
            members.append((attrs, header, "decl", ""))
            i = i + hm.end()
        attrs = []
    return members


def rust_ident(name):
    out = []
    for ch in name:
        if ch.isalnum() or ch == "_":
            out.append(ch)
        else:
            out.append(f"_u{ord(ch):04X}_")
    return "".join(out)


def transcribe(ref, rel, out_dir):
    src = (ref / rel).read_text(encoding="utf-8-sig")
    cls, body = split_members(src)
    members = parse_members(body)
    consts = []
    fields_fn = {}
    string_fields = set()
    helpers = {}
    tests = []
    for attrs, header, kind, text in members:
        words = header.split()
        if kind == "field":
            typ = words[-2]
            name = words[-1]
            fields = (typ, name, text.strip())
            if "const" in words:
                consts.append(fields)
            elif typ in ("string[]", "double[]"):
                consts.append(fields)
            else:
                fields_fn[name] = fields
        elif any(a.startswith("[Fact") or a.startswith("[Theory") for a in attrs):
            tests.append((attrs, header, kind, text))
        elif kind in ("block", "expr"):
            m = re.search(r"(\w+)\s*\(([^)]*)\)", header)
            helpers.setdefault(m.group(1), []).append((header, kind, text))

    tr = Translator(set(fields_fn), string_fields, {k: [h[0] for h in v] for k, v in helpers.items()})
    lines = [
        f"//! Transcribed from Calcpad.Tests/{rel} by tools/transcribe_xunit.py.",
        "#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_variables, clippy::approx_constant, clippy::excessive_precision, clippy::unnecessary_cast, clippy::eq_op, clippy::identity_op)]",
        "mod common;",
        "use common::*;",
        "",
    ]
    for typ, name, text in consts:
        toks = tokenize(text)
        if typ == "string[]":
            items = [rust_string(v) for k, v in toks if k == "str"]
            lines.append(f"const {name}: [&str; {len(items)}] = [{', '.join(items)}];")
        elif typ == "double[]":
            items = []
            sign = ""
            for k, v in toks:
                if v == "-":
                    sign = "-"
                elif k == "num":
                    items.append(sign + rust_number(v))
                    sign = ""
            lines.append(f"const {name}: [f64; {len(items)}] = [{', '.join(items)}];")
        elif typ == "string":
            lines.append(f"const {name}: &str = {rust_string(toks[0][1])};")
        elif typ == "double":
            e, _ = tr.expr(toks + [("op", ";")], 0, {";"})
            e = e.replace("Math::E", "std::f64::consts::E").replace("Math::PI", "std::f64::consts::PI")
            lines.append(f"const {name}: f64 = {e};")
    for name, (typ, _, text) in fields_fn.items():
        toks = tokenize(text)
        e, _ = tr.expr(toks + [("op", ";")], 0, {";"})
        rt = {"double": "f64", "string": "&'static str"}[typ]
        lines.append(f"fn {name}() -> {rt} {{ {e} }}")
    lines.append("")
    # helpers
    for hname, overloads in helpers.items():
        for header, kind, text in overloads:
            m = re.search(r"(\w+)\s*\(([^)]*)\)", header)
            params = [p.strip().split() for p in m.group(2).split(",") if p.strip()]
            rparams = []
            for t, n in params:
                rparams.append(f"{n}: {'&str' if t == 'string' else 'f64'}")
            suffix = ""
            if len(overloads) > 1:
                suffix = "_str" if params[1][0] == "string" else "_f64"
            toks = tokenize(text)
            try:
                if kind == "block":
                    b, _ = tr.block(toks, 0)
                else:
                    e, _ = tr.expr(toks + [("op", ";")], 0, {";"})
                    b = "{ " + e + "; }"
            except (Untranslatable, AssertionError) as ex:
                raise SystemExit(f"helper {hname} untranslatable: {ex}")
            lines.append("#[track_caller]")
            lines.append(f"fn {hname}{suffix}({', '.join(rparams)}) {b}")
    lines.append("")
    stats = {"translated": 0, "override": 0, "ignored": 0}
    for attrs, header, kind, text in tests:
        name = header.split()[-1].split("(")[0]
        rname = rust_ident(name)
        override = OVERRIDES / f"{cls}.{name}.rs"
        lines.append("#[test]")
        if override.exists():
            lines.append(f"fn {rname}() {{")
            lines.append(override.read_text(encoding="utf-8").rstrip())
            lines.append("}")
            stats["override"] += 1
            continue
        try:
            toks = tokenize(text)
            if kind == "block":
                b, _ = tr.block(toks, 0)
            else:
                e, _ = tr.expr(toks + [("op", ";")], 0, {";"})
                b = "{ " + e + "; }"
            lines.append(f"fn {rname}() {b}")
            stats["translated"] += 1
        except (Untranslatable, AssertionError, IndexError) as ex:
            lines.append(f'#[ignore = "not transcribed: {str(ex).replace(chr(34), "")}"]')
            lines.append(f"fn {rname}() {{}}")
            stats["ignored"] += 1
    out = out_dir / f"xunit_{cls}.rs"
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return cls, stats


def main():
    ref = Path(sys.argv[1])
    out_dir = Path(sys.argv[2])
    for rel in sys.argv[3:]:
        cls, stats = transcribe(ref / "Calcpad.Tests", rel, out_dir)
        print(cls, stats)


if __name__ == "__main__":
    main()
