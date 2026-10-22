// Runs the sheet form of every ribbon button and palette entry through the sheet compiler and the engine,
// the sheet counterpart of verify-ribbon-snippets.mjs (which runs the Calcpad forms).
//   node e2e/verify-ribbon-sheet.mjs [--list]
// Each entry is inserted as sheet-insert.ts does in a sheet, placed in a small sheet that defines the
// same variables as the Calcpad prelude, compiled, and rendered by the browser build's own engine (run
// here in Node). It fails on a compile error, on an engine error, and on an entry that should be a
// quantity line but was read as prose. Prints how many entries have a native sheet form and how many
// insert raw Calcpad (a `>>` line).
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { allSnippets } from '../calcpad-web/src/wasm/ribbon-data.ts';
import { defLine, sheetInsert } from '../calcpad-web/src/wasm/sheet-insert.ts';

const here = dirname(fileURLToPath(import.meta.url));
const wasm = resolve(here, '..', 'calcpad-web', 'public', 'wasm');
const engine = await import(join(wasm, 'calcpad_wasm.js'));
engine.initSync({ module: readFileSync(join(wasm, 'calcpad_wasm_bg.wasm')) });
const sheet = await import(join(wasm, 'calcpad_sheet_wasm.js'));
sheet.initSync({ module: readFileSync(join(wasm, 'calcpad_sheet_wasm_bg.wasm')) });
const files = { '/data.csv': '1,2\n3,4\n', '/module.cpd': 'm_1 = 5\n' };
for (const [p, t] of Object.entries(files)) engine.set_file_bytes(p, new TextEncoder().encode(t));
// A workbook for the Excel read snippet, written by the engine itself.
for (const f of JSON.parse(engine.convert(JSON.stringify({ content: 'A = [1; 2| 3; 4]\n#write A to data.xlsx@Sheet1!A1:B2 TYPE=N\n', write: true }), false)).files ?? []) {
    engine.set_file_bytes(`/${f.path.replace(/^\//, '')}`, Uint8Array.from(Buffer.from(f.data, 'base64')));
}

// The same variables as the Calcpad prelude (ribbon-data.ts), written as a sheet.
const PRELUDE = ['x: 0.5', 'y: 3', 'n: 3', 'v: [1; 2; 3]', 'w: [4; 5; 6]', 'u: [1; 2]', 'A: [1, 2; 3, 4]', 'B: [5, 6; 7, 8]',
    '>> S = symmetric(2)', '>> S.(1; 1) = 4', '>> S.(1; 2) = 1', '>> S.(2; 2) = 3', 'f(t): t^2'].join('\n') + '\n';
const PRELUDE_NOX = PRELUDE.replace('x: 0.5\n', '').replace('f(t): t^2', 'f(x): x^2');
const REPEAT_TEMPLATE = '#repeat 3\n    #if x > 1\n        {}\n    #end if\n#loop';
const SHEET_REPEAT = 'repeat 3:\n    if x > 1:\n        {}\n';

const failures = [];
let n = 0, skipped = 0, native = 0, raw = 0, shared = 0;
const list = process.argv.includes('--list');
for (const s of allSnippets()) {
    if (s.skip && !/xlsx/.test(s.text)) { skipped++; continue; }
    n++;
    const text = s.text.replace(/\{sel\}/g, 'text');
    const ins = sheetInsert(text, s.line, s.sheet);
    if (s.sheet !== undefined) native++; else if (ins.raw) raw++; else shared++;
    let body;
    let expectQuantity = false;
    if (s.line) {
        if (s.test && s.test !== REPEAT_TEMPLATE) { failures.push(`${s.id}: no sheet template for ${JSON.stringify(s.test)}`); continue; }
        if (s.test) {
            const block = ins.text.replace(/\n$/, '').split('\n').map(l => `        ${l}`).join('\n');
            body = SHEET_REPEAT.replace('        {}', block);
        } else body = ins.text;
    } else {
        // An inline entry goes into its Calcpad test template; each line of the template becomes a sheet line.
        const tpl = s.test ?? 'r = {}';
        const filled = tpl.replace('{}', ins.text);
        body = filled.split('\n').map((l) => {
            if (l.startsWith("'")) return l.slice(1);
            const d = defLine(l, false);
            if (d === null) { failures.push(`${s.id}: template line ${JSON.stringify(l)} is not a definition`); return ''; }
            expectQuantity = true;
            return d;
        }).join('\n') + '\n';
    }
    const pre = s.raw ? '' : s.nox ? PRELUDE_NOX : PRELUDE;
    const src = `<!--sheet-->\n${pre}${body}`;
    const c = JSON.parse(sheet.compile(src, false));
    const errs = c.diagnostics.filter(d => d.severity === 'error').map(d => `compile: ${d.message}`);
    if (expectQuantity) {
        const own = JSON.parse(sheet.compile(`<!--sheet-->\n${body}`, false));
        if (own.text.includes('#markdown')) errs.push(`read as prose: ${JSON.stringify(body)}`);
    }
    const req = { content: c.text, settings: { math: { isComplex: !!s.cx, decimals: 2 } } };
    const out = JSON.parse(engine.convert(JSON.stringify(req), false));
    for (const e of out.errors ?? []) errs.push(`engine (sheet line ${c.map[e.sourceLine - 1] + 1}): ${String(e.message).slice(0, 110)}`);
    if (out.status) errs.push(`engine: ${out.error}`);
    if (errs.length) failures.push(`${s.id}: ${JSON.stringify(s.text)} -> ${JSON.stringify(ins.text)}: ${errs[0]}`);
    if (list) console.log(`${s.id}\t${s.sheet !== undefined ? 'native' : ins.raw ? 'raw' : 'shared'}\t${JSON.stringify(ins.text)}`);
}
console.log(`${n} sheet inserts run (${skipped} skipped): ${native} native sheet forms, ${shared} shared or converted, ${raw} raw Calcpad (>>); ${failures.length} failed`);
for (const f of failures) console.log('FAIL', f);
process.exit(failures.length ? 1 : 0);
