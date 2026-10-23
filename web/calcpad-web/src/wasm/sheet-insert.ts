// calcpad-rs: what a ribbon button or snippet inserts into a SHEET (docs/sheet-syntax.md). The
// buttons are written in Calcpad; in a sheet each one inserts its friendly form:
//
//   an entry with `sheet` text      that text (headings, lists, tables, plots, if/for/while blocks)
//   an expression glyph or function the same text with the sheet's argument and matrix punctuation
//                                   (`root(x; n)` -> `root(x, n)`; the symbols, units, constants
//                                   and functions are otherwise the same in both syntaxes)
//   a definition `c = 299792458m/s` the quantity line `c: 299792458 m/s`
//   anything else (#UI forms, $Map, directives, macros, ...)
//                                   the Calcpad text as `>>` pass-through lines, so every engine
//                                   feature stays reachable; the button is marked as raw Calcpad
//
// Plain functions with no imports, so the Node verifier (e2e/verify-ribbon-sheet.mjs) loads them.

export interface SheetInsert {
    text: string;
    /** Goes on a line of its own. */
    line: boolean;
    /** Calcpad text passed through with `>>`. */
    raw: boolean;
}

/** Calcpad punctuation to the sheet's: `;` between arguments is `,`; in `[...]` a row ends with `|`, which is `;` here. */
export function toSheetExpr(s: string): string {
    const stack: string[] = [];
    let out = '';
    let quote = false;
    for (const c of s) {
        if (c === '"' || c === "'") { if (c === '"') quote = !quote; out += c; continue; }
        if (quote) { out += c; continue; }
        if (c === '(' || c === '[') stack.push(c);
        else if (c === ')' || c === ']') stack.pop();
        const inner = stack[stack.length - 1];
        if (c === ';' && inner === '(') out += ',';
        else if (c === ';' && inner === '[') out += ',';
        else if (c === '|' && inner === '[') out += ';';
        else out += c;
    }
    return out;
}

/** `name = expression` as `name: expression`; null if the line is not a plain definition. */
export function defLine(line: string, convertExpr = true): string | null {
    const m = /^\s*([\p{L}][\p{L}\p{N}_′″]*(?:\([^)=]*\))?)\s*=\s*([^=].*?)\s*$/u.exec(line);
    if (!m) return null;
    const [, name, rhs] = m;
    if (/['#${}"@]|<[A-Za-z/!]/.test(rhs) || /\{sel\}/.test(rhs)) return null;
    let e = convertExpr ? toSheetExpr(rhs) : rhs;
    // A unit follows its number after a space: 299792458m/s -> 299792458 m/s.
    if (convertExpr) e = e.replace(/(^|[^\p{L}\p{N}_.])(\d+(?:\.\d+)?)([A-Za-zμΩ°])/gu, '$1$2 $3');
    return `${convertExpr ? name.replace(/;/g, ',') : name}: ${e}`;
}

function raw(text: string): string {
    const nl = text.endsWith('\n');
    const lines = (nl ? text.slice(0, -1) : text).split('\n');
    return lines.map(l => `>> ${l}`).join('\n') + (nl ? '\n' : '');
}

/** Whether a snippet with no flag of its own is a line: a command or text line (`#...`, `$...`, `'...`, `\"...`) or several lines. */
export function looksLikeLine(text: string): boolean {
    return text.includes('\n') || /^[#$'"]/.test(text);
}

/** The insert of an entry in a sheet. `sheet` is the entry's own friendly form, if it has one. */
export function sheetInsert(text: string, line: boolean, sheet?: string): SheetInsert {
    if (sheet !== undefined) return { text: sheet, line, raw: false };
    if (!line) return { text: toSheetExpr(text), line, raw: false };
    const body = text.endsWith('\n') ? text.slice(0, -1) : text;
    if (!body.includes('\n')) {
        const d = defLine(body);
        if (d !== null) return { text: `${d}\n`, line, raw: false };
    }
    return { text: raw(text), line, raw: true };
}
