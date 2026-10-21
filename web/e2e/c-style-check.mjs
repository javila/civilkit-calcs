// Holds the browser's copy of the C-style operator conversion (src/wasm/c-style.ts) to the
// engine's (calcpad-wasm replace_c_style_operators, the CLI's CalcpadReader port) over every line
// of every upstream worksheet and a set of edge cases.
//   node e2e/c-style-check.mjs [path to calcpad-ref]
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { replaceCStyleLine } from '../calcpad-web/src/wasm/c-style.ts';

const web = join(dirname(fileURLToPath(import.meta.url)), '..');
const ref = process.argv[2] ?? join(web, '..', '..', 'calcpad-ref');
const wasmDir = join(web, 'calcpad-web', 'public', 'wasm');
const engine = await import(join(wasmDir, 'calcpad_wasm.js'));
engine.initSync({ module: readFileSync(join(wasmDir, 'calcpad_wasm_bg.wasm')) });

const lines = [
    'a == b', 'a != b', 'x >= 1', 'x <= 1', 'n%%2', 'a && b', 'a || b', 'a << 30', 'x <* 3',
    "'text == not converted' a == b", '"head == text"', "a == b 'note <= x", '"unterminated <= quote',
    'p = 3%', "' %%", '==', '<<=', '<==', 'a ===b', '', 'Δ == 2 \u{1F600} <= 3',
];
const walk = (d) => readdirSync(d).flatMap((f) => {
    const p = join(d, f);
    return statSync(p).isDirectory() ? walk(p) : /\.(cpd|txt)$/.test(f) ? [p] : [];
});
for (const f of walk(ref)) {
    for (const l of readFileSync(f, 'utf8').replace(/^﻿/, '').split(/\r?\n/)) {
        lines.push(l.split('\u000B')[0].replace(/^\t+/, ''));
    }
}
let bad = 0;
for (const l of lines) {
    const want = engine.replace_c_style_operators(l);
    const got = replaceCStyleLine(l);
    if (got !== want) {
        if (bad++ < 10) console.log(`FAIL ${JSON.stringify(l)}: ${JSON.stringify(got)}, engine ${JSON.stringify(want)}`);
    }
}
const changed = lines.filter((l) => engine.replace_c_style_operators(l) !== l).length;
console.log(`${lines.length} lines (${changed} converted): ${bad ? bad + ' differ' : 'the browser and the engine agree'}`);
process.exit(bad ? 1 : 0);
