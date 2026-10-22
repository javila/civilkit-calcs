// Runs every snippet the ribbon can insert through the engine's command line build and fails on any error.
//   node e2e/verify-ribbon-snippets.mjs [--cli /path/to/Cli]
// Line snippets run after a small prelude; inline snippets are substituted into their test template.
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { allSnippets, PRELUDE, PRELUDE_NOX, VERIFY_FILES } from '../calcpad-web/src/wasm/ribbon-data.ts';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const cli = arg('--cli', resolve(here, '..', '..', 'target', 'release', 'Cli'));
if (!existsSync(cli)) { console.error(`no CLI at ${cli}`); process.exit(2); }
const dir = mkdtempSync(join(tmpdir(), 'ribbon-snip-'));
// The engine reads its settings from $HOME/Documents/.config/calcpad/Settings.xml: use the browser build's
// (vector plots, complex numbers on) from a throwaway home.
const settings = (complex) => `<?xml version="1.0" encoding="utf-8"?>
<Settings><Math><Decimals>2</Decimals><Degrees>0</Degrees><IsComplex>${complex}</IsComplex><Substitute>true</Substitute><FormatEquations>true</FormatEquations></Math>
<Plot><IsAdaptive>true</IsAdaptive><ScreenScaleFactor>2</ScreenScaleFactor><ImagePath /><ImageUri /><VectorGraphics>true</VectorGraphics><ColorScale>Rainbow</ColorScale><SmoothScale>false</SmoothScale><Shadows>true</Shadows><LightDirection>NorthWest</LightDirection></Plot><Units>m</Units></Settings>`;
const homes = {};
for (const cx of [false, true]) {
    homes[cx] = join(dir, `home-${cx}`);
    mkdirSync(join(homes[cx], 'Documents', '.config', 'calcpad'), { recursive: true });
    writeFileSync(join(homes[cx], 'Documents', '.config', 'calcpad', 'Settings.xml'), settings(cx));
}
for (const [n, t] of Object.entries(VERIFY_FILES)) writeFileSync(join(dir, n), t);

function run(name, src, cx = false) {
    const f = join(dir, `${name}.cpd`);
    writeFileSync(f, src);
    try { execFileSync(cli, [f, join(dir, `${name}.html`)], { cwd: dir, stdio: 'pipe', env: { ...process.env, HOME: homes[cx] } }); } catch (e) { return [`CLI failed: ${e.message.split('\n')[0]}`]; }
    const html = readFileSync(join(dir, `${name}.html`), 'utf8');
    return [...html.matchAll(/class="err"[^>]*>([^<]*)/g)].map(m => m[1]).filter(m => m !== 'text' && !/Paused/.test(m));
}
// A workbook for the Excel read snippet: written by the engine itself.
run('mkxlsx', 'A = [1; 2| 3; 4]\n#write A to data.xlsx@Sheet1!A1:B2 TYPE=N\n');

let n = 0, skipped = 0;
const bad = [];
for (const s of allSnippets()) {
    const text = s.text.replace(/\{sel\}/g, 'text').replace(/\n$/, '');
    const pre = s.raw ? '' : s.nox ? PRELUDE_NOX : PRELUDE;
    let body;
    if (s.line) body = s.test ? s.test.replace('{}', text) : text;
    else body = (s.test ?? 'r = {}').replace('{}', text);
    const needsExcel = /xlsx/.test(s.text);
    if (s.skip && !needsExcel) { skipped++; console.log(`skip ${s.id}: ${s.skip}`); continue; }
    n++;
    const errs = run(`s${n}`, pre + body + '\n', !!s.cx);
    if (errs.length) bad.push(`${s.id}: ${JSON.stringify(s.text)} -> ${errs[0].slice(0, 120)}`);
}
console.log(`${n} snippets run, ${skipped} skipped, ${bad.length} failed`);
for (const b of bad) console.log('FAIL', b);
process.exit(bad.length ? 1 : 0);
