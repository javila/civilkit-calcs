// Runs every bundled sample through the browser build and compares the result with the command line engine and linter
// working on the same file with its included sheets, images and data on disk.
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/samples-sweep.mjs [--url U] [--cli PATH] [--lint-list FILE] [--only TEXT] [--skip-convert]
// For each sample it records
//   preview   errors in the page the Samples dialog shows (the convert call with the sample's files as extraFiles)
//   unwrap    errors the unwrapped listing reports in modules and macros
//   opened    errors in the page for the sample opened as a copy (its files in the library, no extraFiles)
//   lint      the editor's diagnostics of the opened copy, by code, against lint_dir's on the same file
// and prints only where the browser differs from the command line. lint_dir is
//   cargo run --release -p calcpad-highlighter --example lint_dir -- web/calcpad-web/public/samples --list > FILE
import { chromium } from 'playwright';
import { replaceCStyleOperators } from '../calcpad-web/src/wasm/c-style.ts';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4173/');
const cli = arg('--cli', resolve(here, '..', '..', 'target', 'release', 'Cli'));
const lintList = arg('--lint-list', '');
const only = arg('--only', '');
const skipConvert = process.argv.includes('--skip-convert');
const samplesDir = resolve(here, '..', 'calcpad-web', 'public', 'samples');
const manifest = JSON.parse(readFileSync(join(samplesDir, 'manifest.json'), 'utf8'));
const samples = manifest.samples.filter(s => s.file.includes(only));

// ---- command line baseline -------------------------------------------------------------------
const home = mkdtempSync(join(tmpdir(), 'sweep-home-'));
mkdirSync(join(home, 'Documents', '.config', 'calcpad'), { recursive: true });
writeFileSync(join(home, 'Documents', '.config', 'calcpad', 'Settings.xml'), `<?xml version="1.0" encoding="utf-8"?>
<Settings><Math><Decimals>2</Decimals><Degrees>0</Degrees><IsComplex>false</IsComplex><Substitute>true</Substitute><FormatEquations>true</FormatEquations></Math>
<Plot><IsAdaptive>true</IsAdaptive><ScreenScaleFactor>2</ScreenScaleFactor><ImagePath /><ImageUri /><VectorGraphics>true</VectorGraphics><ColorScale>Rainbow</ColorScale><SmoothScale>false</SmoothScale><Shadows>true</Shadows><LightDirection>NorthWest</LightDirection></Plot><Units>m</Units></Settings>`);
const countErr = (html) => (html.match(/<p class="err">/g) ?? []).length;
const nativeLint = new Map();
if (lintList) for (const line of readFileSync(lintList, 'utf8').split('\n')) {
    const m = /^(.*\.cpd):\d+:\d+ \[(CPD-\d+)\]/.exec(line);
    if (!m) continue;
    const f = nativeLint.get(m[1]) ?? new Map();
    f.set(m[2], (f.get(m[2]) ?? 0) + 1);
    nativeLint.set(m[1], f);
}
const base = {};
if (!skipConvert) for (const s of samples) {
    const f = join(samplesDir, s.file.replace(/^samples\//, ''));
    const out = join(home, 'out.html');
    try { execFileSync(cli, [f, out], { cwd: dirname(f), stdio: 'pipe', timeout: 240000, env: { ...process.env, HOME: home } }); base[s.id] = countErr(readFileSync(out, 'utf8')); } catch (e) { base[s.id] = `cli failed: ${e.code ?? e.message.split('\n')[0]}`; }
}

// ---- browser -----------------------------------------------------------------------------------
const browser = await chromium.launch();
let page;
async function fresh() {
    await page?.context().close();
    const ctx = await browser.newContext({ viewport: { width: 1300, height: 800 } });
    page = await ctx.newPage();
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
}
await fresh();
const rows = [];
let n = 0;
for (const s of samples) {
    if (n++ % 8 === 7) await fresh();
    const r = { id: s.id, file: s.file.replace(/^samples\//, ''), cli: base[s.id] };
    try {
        // The dialog's preview.
        if (!skipConvert) r.preview = await page.evaluate(async ([s, text]) => {
            const b = (u) => fetch(u).then(x => x.arrayBuffer()).then(a => new Uint8Array(a));
            const extra = {};
            for (const a of s.assets) if (/\.(cpd|txt|csv|xlsx|xlsm)$/i.test(a.key)) { const u = await b(a.url); let t = ''; for (let i = 0; i < u.length; i += 0x8000) t += String.fromCharCode(...u.subarray(i, i + 0x8000)); extra[a.key] = btoa(t); }
            const res = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: text, extraFiles: extra, includeLineAnchors: false }) });
            const html = await res.text();
            return (html.match(/<p class="err">/g) ?? []).length;
        }, [s, replaceCStyleOperators(readFileSync(join(samplesDir, s.file.replace(/^samples\//, '')), 'utf8').replace(/^\uFEFF/, ''))]);
        // Open as a copy.
        await page.evaluate(async (s) => { const m = await (await fetch('samples/manifest.json')).json(); await window.calcpadFiles.samples.openSample(m.samples.find(x => x.id === s.id)); }, s);
        // The lint of the copy: wait for the markers to hold still.
        let last = -1, same = 0;
        for (let i = 0; i < 40 && same < 3; i++) {
            await page.waitForTimeout(500);
            const c = await page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).length);
            same = c === last ? same + 1 : 0;
            last = c;
        }
        const diag = await page.evaluate(() => { const m = {}; for (const k of window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri })) { const c = typeof k.code === 'string' ? k.code : k.code?.value ?? '?'; m[c] = (m[c] ?? 0) + 1; } return m; });
        r.lint = diag;
        const nat = nativeLint.get(s.file.replace(/^samples\//, '')) ?? new Map();
        r.lintDiff = [];
        for (const c of new Set([...Object.keys(diag), ...nat.keys()])) if ((diag[c] ?? 0) !== (nat.get(c) ?? 0)) r.lintDiff.push(`${c}: browser ${diag[c] ?? 0}, native ${nat.get(c) ?? 0}`);
        // The opened copy through the engine, and the unwrapped listing.
        if (!skipConvert) {
            const o = await page.evaluate(async () => {
                const content = window.calcpadActiveEditor.getValue();
                const sp = '/' + (window.calcpadTabs.activeTab.title || 'Untitled').replace(/[\\/:*?"<>|]/g, '_') + '.cpd';
                const run = async (q) => (await (await fetch('/api/calcpad/convert' + q, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, sourceFilePath: sp, includeLineAnchors: false }) })).text());
                const html = await run('');
                const un = await run('?unwrap=true');
                return { opened: (html.match(/<p class="err">/g) ?? []).length, unwrap: (un.match(/class="errorHeader">Found <b>(\d+)/) ?? [0, 0])[1] * 1 };
            });
            r.opened = o.opened;
            r.unwrap = o.unwrap;
        }
    } catch (e) { r.error = String(e).split('\n')[0]; }
    rows.push(r);
    process.stderr.write(`${rows.length}/${samples.length} ${r.file}\n`);
}
await browser.close();

// ---- report ------------------------------------------------------------------------------------
const diffs = [];
for (const r of rows) {
    const d = [];
    if (r.error) d.push(`error: ${r.error}`);
    if (!skipConvert) {
        const cli = typeof r.cli === 'number' ? r.cli : null;
        if (cli !== null && r.preview !== cli) d.push(`preview errors ${r.preview} (cli ${cli})`);
        if (cli !== null && r.opened !== cli) d.push(`opened errors ${r.opened} (cli ${cli})`);
        if (r.unwrap) d.push(`unwrapped listing: ${r.unwrap} errors in modules and macros`);
        if (cli === null) d.push(`cli: ${r.cli}`);
    }
    if (lintList && r.lintDiff?.length) d.push(`lint differs: ${r.lintDiff.join('; ')}`);
    if (d.length) diffs.push({ file: r.file, d });
}
const warnings = rows.reduce((t, r) => t + Object.values(r.lint ?? {}).reduce((a, b) => a + b, 0), 0);
console.log(`${rows.length} samples swept; ${diffs.length} differ from the command line; ${warnings} editor diagnostics in total`);
for (const x of diffs) console.log(`- ${x.file}\n    ${x.d.join('\n    ')}`);
writeFileSync(join(tmpdir(), 'samples-sweep.json'), JSON.stringify(rows, null, 1));
process.exit(diffs.length ? 1 : 0);
