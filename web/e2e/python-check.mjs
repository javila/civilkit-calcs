// Checks `#python` blocks in the built app served at --url (default http://127.0.0.1:4180/):
// lazy loading, the once-per-tab question, print text, an SVG figure, an exported variable used by later Calcpad lines,
// error line mapping, the timeout, the sandbox, and the exports (Report, Word, Excel).
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/python-check.mjs [--url U] [--out DIR]
import { chromium } from 'playwright';
import { mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const out = arg('--out', join(tmpdir(), 'civilkit-calcs-python'));
mkdirSync(out, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

async function open(scheme = 'light') {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 }, colorScheme: scheme });
    const page = await ctx.newPage();
    const requests = [];
    const errors = [];
    page.on('request', r => requests.push(r.url()));
    page.on('pageerror', e => errors.push(String(e)));
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
    return { ctx, page, requests, errors };
}
const setText = (page, t) => page.evaluate(v => window.calcpadActiveEditor.setValue(v), t);
/** The text of the results frame once `needle` shows in it (or '' after the timeout). */
async function frameText(page, needle, ms = 40000) {
    const t0 = Date.now();
    while (Date.now() - t0 < ms) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
            if (t.includes(needle)) return t;
        }
        await page.waitForTimeout(300);
    }
    return '';
}
const frameHas = async (page, sel) => {
    for (const f of page.frames()) if (f !== page.mainFrame() && await f.locator(sel).count().catch(() => 0)) return true;
    return false;
};
const yes = page => page.locator('#cprs-python-yes');

const SHEET = [
    '"Python check',                         // 1
    'a = 5',                                  // 2
    '#python',                                // 3
    'import numpy as np',                     // 4
    'import matplotlib.pyplot as plt',        // 5
    'x = np.linspace(0, 1, 11)',              // 6
    'print("sum is", round(float(x.sum()), 2), "<b>&")',// 7
    'plt.plot(x, x * x, label="x2")',         // 8
    'plt.xlabel("x")',                        // 9
    'plt.legend()',                           // 10
    'plt.show()',                             // 11
    'export(k=float(x.sum()), v=[1, 2, 3], m=[[1, 2], [3, 4]])', // 12
    '#end python',                            // 13
    'r = k*2',                                // 14
    'z = a + 1',                              // 15
].join('\n');

// ---- 1. a worksheet without #python never loads the runtime ------------------------------
{
    const { ctx, page, requests } = await open();
    await setText(page, 'a = 3\nb = a^2');
    check((await frameText(page, 'b =')).length > 0, 'a plain worksheet calculates');
    await page.waitForTimeout(1500);
    check(!requests.some(u => /\/py\//.test(u)), 'no request to /py/ for a worksheet without #python');
    check(await page.evaluate(() => window.calcpadPython.started === false), 'the Python worker was never started');
    check(await page.locator('#cprs-python-yes').count() === 0, 'no question for a plain worksheet');
    await ctx.close();
}

// ---- 2. the question, text, figure, export, later lines, exports ---------------------------
{
    const { ctx, page, requests, errors } = await open();
    await setText(page, SHEET);
    await yes(page).waitFor({ timeout: 20000 });
    check(true, 'the confirmation prompt appears before the first run');
    check(!requests.some(u => /micropython/.test(u)), 'nothing of the runtime is fetched before the answer');
    await page.screenshot({ path: join(out, 'prompt.png') });
    await yes(page).click();
    const t = await frameText(page, 'r =');
    check(/sum is 5\.5 <b>&/.test(t), 'print() text is shown, HTML-escaped' + (/sum is 5\.5 <b>&/.test(t) ? '' : ' ' + JSON.stringify(t.slice(0, 300))));
    check(await frameHas(page, 'svg.cprs-py-figure'), 'the figure is an inline SVG');
    check(/k = 5\.5/.test(t) && /r = .*= 11/.test(t.replace(/\s+/g, ' ')), 'an exported number is used by a later Calcpad line (r = k*2 = 11)');
    check(/v⃗?\s*=\s*\[1; 2; 3\]|v = \[1; 2; 3\]/.test(t) || t.includes('[1; 2; 3]'), 'an exported list is a Calcpad vector');
    check(/\[1; 2 ?\| 3; 4\]/.test(t), 'an exported 2-D list is a Calcpad matrix');
    check(/z = .*6/.test(t.replace(/\s+/g, ' ')), 'Calcpad lines after the block still calculate');
    check(requests.some(u => /micropython\.wasm/.test(u)), 'the runtime loaded only now');
    await page.screenshot({ path: join(out, 'figure-light.png') });
    // the question is not repeated in this tab, even after an edit
    await setText(page, SHEET.replace('a = 5', 'a = 7'));
    const t2 = await frameText(page, '7 + 1');
    check(/z = .*8/.test(t2.replace(/\s+/g, ' ')) && await yes(page).count() === 0, 'no second question in the same tab' + (/z = .*8/.test(t2.replace(/\s+/g, ' ')) ? '' : ' ' + JSON.stringify(t2.slice(-200))));

    // exports through the same transport
    const exportStatus = await page.evaluate(async (content) => {
        const o = {};
        for (const ep of ['convert', 'docx', 'xlsx']) {
            const r = await fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: ep === 'convert', includeLineAnchors: false }) });
            const b = await r.arrayBuffer();
            o[ep] = { status: r.status, bytes: b.byteLength, text: ep === 'convert' ? new TextDecoder().decode(b) : '' };
        }
        return o;
    }, SHEET);
    check(exportStatus.convert.status === 200 && /<svg[^>]*cprs-py-figure/.test(exportStatus.convert.text) && /sum is/.test(exportStatus.convert.text), 'Report / Print HTML carries the figure and the text');
    check(exportStatus.docx.status === 200 && exportStatus.docx.bytes > 5000, `Word export works (${exportStatus.docx.bytes} bytes)`);
    check(exportStatus.xlsx.status === 200 && exportStatus.xlsx.bytes > 3000, `Excel export works (${exportStatus.xlsx.bytes} bytes)`);
    check(errors.length === 0, `no page errors${errors.length ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

// ---- 3. dark theme figure, error line mapping, sandbox, timeout ---------------------------
{
    const { ctx, page } = await open('dark');
    await setText(page, SHEET);
    await yes(page).click({ timeout: 20000 });
    await frameText(page, 'r =');
    await page.screenshot({ path: join(out, 'figure-dark.png') });

    const BAD = ['"Error mapping', 'q = 1', '#python', 'x = 1', 'y = 1 / 0', '#end python', 'w = nothing_here + 1'].join('\n');
    await setText(page, BAD);
    const t = await frameText(page, 'Python error');
    check(/Python error on line 5: ZeroDivisionError/.test(t), 'a Python exception names the worksheet line (5) and its type');
    check(/nothing_here/.test(t), 'Calcpad lines after a failed block still calculate (and report their own errors)');
    const sourceLines = await page.evaluate(async (content) => {
        const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) });
        return JSON.parse(decodeURIComponent(r.headers.get('X-Calcpad-Errors') ?? '[]')).map(e => [e.source, e.sourceLine]);
    }, BAD);
    check(sourceLines.some(([s, l]) => s === 'Python' && l === 5), 'the error list has the Python error on line 5');
    check(sourceLines.some(([s, l]) => s !== 'Python' && l === 7), 'an engine error after the block keeps its worksheet line (7)');
    await page.screenshot({ path: join(out, 'error-line.png') });

    await setText(page, ['"Sandbox1', '#python', 'import js', '#end python'].join('\n'));
    const s1 = await frameText(page, 'Sandbox1');
    check(/ImportError/.test(s1) && /not available/.test(s1), '`import js` is refused ' + JSON.stringify(s1.slice(0, 200)));
    await setText(page, ['"Sandbox2', '#python', 'from jsffi import create_proxy', '#end python'].join('\n'));
    check(/ImportError/.test(await frameText(page, 'Sandbox2')), '`import jsffi` is refused');
    await setText(page, ['"Sandbox3', '#python', 'import _cp_host', 'print(hasattr(_cp_host, "constructor"))', 'import builtins', 'print(builtins.__import__.__class__)', '#end python'].join('\n'));
    const s3 = await frameText(page, 'Sandbox3');
    check(!/constructor.*True/.test(s3) && /False/.test(s3), 'the host object is not reachable through a JS proxy ' + JSON.stringify(s3.slice(0, 200)));

    await setText(page, ['"Timeout', 'ok = 1', '#python', 'while True:', '    pass', '#end python', 'after = ok + 1'].join('\n'));
    const t0 = Date.now();
    const tt = await frameText(page, 'was stopped', 30000);
    check(/ran longer than 10 s and was stopped/.test(tt), `an endless loop is killed (${Math.round((Date.now() - t0) / 1000)} s)`);
    check(/after =/.test(tt), 'the worksheet after the timed-out block still calculates');
    // the worker comes back for the next run
    await setText(page, ['"After timeout', '#python', 'print("alive")', '#end python'].join('\n'));
    check((await frameText(page, 'alive', 30000)).length > 0, 'a new run works after a timeout');

    await setText(page, ['"Open', '#python', 'x = 1'].join('\n'));
    const openText = await frameText(page, "Open", 15000);
    check(/no "#end python"/.test(openText), "an unclosed block is an error line " + JSON.stringify(openText.slice(0, 200)));
    await ctx.close();
}

// ---- 4. Not now ----------------------------------------------------------------------------
{
    const { ctx, page, requests } = await open();
    await setText(page, SHEET);
    await page.locator('#cprs-python-no').click({ timeout: 20000 });
    const t = await frameText(page, 'Python was not run');
    check(t.length > 0, 'Not now: the block is not run and says so');
    check(!requests.some(u => /micropython/.test(u)), 'Not now: the runtime is never loaded');
    check(/z = /.test(t), 'Not now: the rest of the worksheet calculates');
    await ctx.close();
}

// ---- 5. the Python samples ---------------------------------------------------------------
{
    const { ctx, page } = await open();
    const manifest = await page.evaluate(async () => (await fetch('samples/manifest.json')).json());
    const py = manifest.samples.filter(s => s.group === 'Python');
    check(py.length === 4 && manifest.groups.includes('Python'), `four samples in the Python group (${py.length})`);
    check(py[0].title === 'Beam Explorer: Python Solve, Plotly Charts', 'the beam explorer sample comes first');
    for (const s of py) {
        await page.evaluate(async (id) => { const m = (await (await fetch('samples/manifest.json')).json()).samples.find(x => x.id === id); await window.calcpadFiles.samples.openSample(m); }, s.id);
        if (await yes(page).count().catch(() => 0) || await yes(page).waitFor({ timeout: 4000 }).then(() => true, () => false)) await yes(page).click();
        const t = await frameText(page, 'Python', 40000) || await frameText(page, '=', 5000);
        const bad = /Python error|Error in "/.test(t);
        check(t.length > 0 && !bad && await frameHas(page, 'svg.cprs-py-figure'), `sample "${s.title}" runs with a figure and no errors`);
        await page.waitForTimeout(2500);
        const markers = await page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).filter(m => m.severity >= 8).map(m => `${m.startLineNumber}: ${m.message}`));
        check(markers.length === 0, `sample "${s.title}" has no error markers in the editor${markers.length ? ' ' + JSON.stringify(markers.slice(0, 3)) : ''}`);
    }
    await ctx.close();
}

// ---- 6. the ribbon inserts a block that runs ----------------------------------------------
{
    const { ctx, page } = await open();
    await setText(page, 'a = 1');
    await page.locator('[data-ribbon-tab=math]').click();
    await page.locator('[data-ribbon-btn="p-programming"]').click();
    await page.locator('.rb-pop [data-ribbon-glyph="#python plot"]').click();
    const text = await page.evaluate(() => window.calcpadActiveEditor.getValue());
    check(/#python\n[\s\S]*plt\.show\(\)\n#end python/.test(text), 'Math > Programming > #python plot inserts a block');
    await yes(page).click({ timeout: 20000 });
    await frameText(page, 'a = 1');
    await page.waitForTimeout(3000);
    check(await frameHas(page, 'svg.cprs-py-figure'), 'the inserted block runs and draws a figure');
    await ctx.close();
}

// ---- 7. python fences in a sheet -------------------------------------------------------------
{
    const { ctx, page } = await open();
    const SHEET_SRC = [
        '<!--sheet-->', '# Sheet python', '', 'a: 5', '', '```python', 'import numpy as np', 'x = np.linspace(0, 1, 11)',
        'print("sum is", round(float(x.sum()), 2))', 'export(k=float(x.sum()))', '```', '', 'r: k * 2', 'z: a + nothing_here',
    ].join('\n');
    await setText(page, SHEET_SRC);
    await yes(page).click({ timeout: 20000 });
    const t = await frameText(page, 'Sheet python');
    await frameText(page, 'sum is');
    const t2 = await frameText(page, 'r =');
    check(/sum is 5\.5/.test(t2) && /r = .*= 11/.test(t2.replace(/\s+/g, ' ')), 'a python fence in a sheet runs, and a quantity line after it uses the exported value');
    check(/nothing_here/.test(t2), 'an engine error after the fence is reported');
    const lines = await page.evaluate(async (content) => {
        const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) });
        return JSON.parse(decodeURIComponent(r.headers.get('X-Calcpad-Errors') ?? '[]')).map(e => e.sourceLine);
    }, SHEET_SRC);
    check(lines.includes(14), `the error maps to its sheet line (14): ${JSON.stringify(lines)}`);
    const BAD = ['<!--sheet-->', '# Bad', '```python', 'x = 1', 'y = 1 / 0', '```', 'q: 1'].join('\n');
    await setText(page, BAD);
    const eb = await frameText(page, 'Bad');
    await frameText(page, 'Python error');
    const eb2 = await frameText(page, 'ZeroDivisionError');
    check(/Python error on line 5: ZeroDivisionError/.test(eb2), 'a Python error in a sheet fence names the sheet line (5)');
    // the editor leaves the fence alone: no error markers on its lines
    const markers = await page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).filter(m => m.severity >= 8 && m.startLineNumber >= 3 && m.startLineNumber <= 6).map(m => m.message));
    check(markers.length === 0, `no Calcpad error markers inside the python fence ${JSON.stringify(markers)}`);
    // the ribbon follows the sheet mode
    await setText(page, '<!--sheet-->\nx: 1\n');
    await page.locator('[data-ribbon-tab=math]').click();
    await page.locator('[data-ribbon-btn="p-programming"]').click();
    await page.locator('.rb-pop [data-ribbon-glyph="#python plot"]').click();
    const ins = await page.evaluate(() => window.calcpadActiveEditor.getValue());
    check(/```python\n[\s\S]*plt\.show\(\)\n```/.test(ins) && !/#python/.test(ins), 'in a sheet the ribbon inserts a python fence');
    await ctx.close();
}

// ---- 8. the beam explorer: Python results drive a Plotly chart ----------------------------------
{
    const { ctx, page } = await open();
    // Opened the way a user opens it (the Samples browser converts C-style operators, which must not touch the Python).
    await page.evaluate(async () => { const m = (await (await fetch('samples/manifest.json')).json()).samples.find(s => s.title.startsWith('Beam Explorer')); await window.calcpadFiles.samples.openSample(m); });
    await yes(page).click({ timeout: 20000 });
    const t = await frameText(page, 'Design check', 60000);
    check(/Equilibrium: the reactions sum to the load/.test(t) && /UDL on all spans 300\.0 of 300\.0 kN/.test(t), 'the Python solve ran and its equilibrium check holds');
    check(/η = /.test(t) && /M_Ed|MEd/.test(t) && /within the capacity/.test(t), 'the Calcpad design check uses the exported envelope');
    const frame = async () => { for (const f of page.frames()) if (f !== page.mainFrame() && await f.locator('#cx-read').count().catch(() => 0)) return f; return null; };
    let f = null;
    for (let i = 0; i < 100 && !f; i++) { f = await frame(); if (!f) await page.waitForTimeout(300); }
    check(!!f, 'the explorer is on the page');
    const plotly = f ? await f.waitForFunction(() => typeof Plotly !== 'undefined' && document.querySelectorAll('#cx-plot .main-svg').length > 0, null, { timeout: 45000 }).then(() => true, () => false) : false;
    if (!plotly) console.log('skip the chart checks: Plotly did not load (offline?)');
    else {
        const r0 = await f.evaluate(() => document.getElementById('cx-read').innerText);
        // three-moment equations for the three UDL spans (5, 6, 4 m, 20 kN/m): M_B = -63.61, M_C = -50.91 kN.m
        check(/hogging max -63\.6 kN\.m at x = 5\.00 m/.test(r0), `the chart's hogging moment over support B matches the three-moment solution (-63.6) ${JSON.stringify(r0.slice(0, 160))}`);
        const traces = await f.evaluate(() => document.getElementById('cx-plot').data.length);
        check(traces === 5, `five traces drawn (${traces})`);
        await f.evaluate(() => { const s = document.getElementById('cx-fac'); s.value = '2'; s.dispatchEvent(new Event('input')); });
        const r2 = await f.evaluate(() => document.getElementById('cx-read').innerText);
        check(/hogging max -127\.2 kN\.m/.test(r2), 'the load factor slider doubles the moments (-127.2)');
        await f.evaluate(() => { const c = document.getElementById('cx-case'); c.value = '3'; c.dispatchEvent(new Event('input')); const s = document.getElementById('cx-pos'); s.value = '2'; s.dispatchEvent(new Event('input')); });
        const r3 = await f.evaluate(() => document.getElementById('cx-read').innerText);
        check(/P = 120\.0 kN at x = 2\.5 m/.test(r3), `the point load case and its position slider respond ${JSON.stringify(r3.slice(0, 60))}`);
        await page.screenshot({ path: join(out, 'explorer.png') });
    }
    await ctx.close();
}

await browser.close();
console.log(fails.length ? `\n${fails.length} FAILED` : '\nall passed');
process.exit(fails.length ? 1 : 0);
