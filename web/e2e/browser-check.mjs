// End-to-end check of the browser build (calcpad-web on the calcpad-wasm engine), headless
// Chromium via Playwright.
//
//   1. load the page, type a worksheet into the editor, read the result in the preview;
//   2. run a sample of CalcpadCE's Examples/Tests through the UI (editor content, then
//      Export > Preview > Save HTML, the path that renders without line anchors as the CLI
//      stubs do) and save the bodies for tools/compare_browser.py;
//   3. record every network request and fail if any calculation request leaves the page;
//   4. save a screenshot to tools/out/browser.png.
//
// Usage (from web/, after `npm run build:wasm && npm run build`):
//   node e2e/browser-check.mjs [--ref ../../calcpad-ref] [--port 4180]
import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const repo = resolve(web, '..');
const arg = (name, def) => {
    const i = process.argv.indexOf(name);
    return i > 0 ? process.argv[i + 1] : def;
};
const ref = resolve(arg('--ref', resolve(repo, '..', 'calcpad-ref')));
const port = Number(arg('--port', '4180'));
const origin = `http://localhost:${port}`;
const out = join(repo, 'tools', 'out');
const renderedDir = join(out, 'browser');
rmSync(renderedDir, { recursive: true, force: true });
mkdirSync(renderedDir, { recursive: true });

const failures = [];
const check = (ok, what) => {
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
    if (!ok) failures.push(what);
};

// ---- serve web/calcpad-web/dist on our own port and make sure it is this tree ----------
const server = spawn('npx', ['vite', 'preview', '--port', String(port), '--strictPort'], {
    cwd: join(web, 'calcpad-web'),
    stdio: ['ignore', 'pipe', 'pipe'],
});
let serverLog = '';
server.stdout.on('data', (d) => (serverLog += d));
server.stderr.on('data', (d) => (serverLog += d));
const stop = () => server.kill();
process.on('exit', stop);

async function waitForServer() {
    for (let i = 0; i < 100; i++) {
        try {
            const r = await fetch(`${origin}/wasm/calcpad_wasm_bg.wasm`);
            if (r.ok) return new Uint8Array(await r.arrayBuffer());
        } catch { /* not up yet */ }
        await new Promise((r) => setTimeout(r, 200));
    }
    throw new Error(`server did not start:\n${serverLog}`);
}
const served = await waitForServer();
const local = readFileSync(join(web, 'calcpad-web', 'dist', 'wasm', 'calcpad_wasm_bg.wasm'));
check(Buffer.compare(Buffer.from(served), local) === 0, `server on :${port} serves this tree's dist/wasm`);

// ---- browser --------------------------------------------------------------------------------
const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1500, height: 900 }, acceptDownloads: true });
const requests = [];
context.on('request', (r) => requests.push({ method: r.method(), url: r.url() }));
// The settings the stubs were rendered with (.github/scripts/CliSettingsComparison.xml):
// the web defaults with vector graphics on.
await context.addInitScript(() => {
    if (window !== window.top) return; // the sandboxed result frames have no storage
    if (sessionStorage.getItem('e2e-seeded')) return;
    sessionStorage.setItem('e2e-seeded', '1');
    localStorage.clear();
    localStorage.setItem('calcpad-settings', JSON.stringify({
        math: { formatString: '', decimals: 2, degrees: 0, isComplex: false, substitute: true, formatEquations: true,
            zeroSmallMatrixElements: true, showHiddenOutput: false, maxOutputCount: 20, precision: 1e-14, tol: 1e-6 },
        plot: { imagePath: '', imageUri: '', screenScaleFactor: 2, lightDirection: 'NorthWest', vectorGraphics: true,
            colorScale: 'Rainbow', smoothScale: false, shadows: true, isAdaptive: true, width: 500, height: 300, step: 0 },
        server: { url: '', mode: 'auto' },
        isUs: true,
    }));
});
const page = await context.newPage();
const pageErrors = [];
page.on('pageerror', (e) => pageErrors.push(e.message));
const consoleLines = [];
page.on('console', (m) => !m.text().includes('[Snippets]') && consoleLines.push(`${new Date().toISOString().slice(17, 23)} ${m.type()} ${m.text().slice(0, 200)}`));
await page.goto(`${origin}/`);
await page.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });

/** Text of the results frame that contains `marker`. */
async function previewText(marker) {
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
        if (t.includes(marker)) return t;
    }
    return '';
}

// ---- 1. type a worksheet ------------------------------------------------------------------
const editorBox = page.locator('.monaco-editor').first();
await editorBox.click();
await page.keyboard.press('ControlOrMeta+A');
await page.keyboard.press('Delete');
for (const line of ["'Typed in the browser", 'L = 6m', 'q = 10kN/m', 'M = q*L^2/8']) {
    await page.keyboard.type(line, { delay: 5 });
    await page.keyboard.press('Escape');
    await page.keyboard.press('Enter');
}
const expected = /M\s*=[\s\S]*=\s*45\s*kNm/;
let typed = '';
for (let i = 0; i < 100 && !expected.test(typed); i++) {
    await page.waitForTimeout(200);
    typed = await previewText('Typed in the browser');
}
check(expected.test(typed), `typed worksheet renders M = q·L²/8 = ... = 45 kNm (preview: ${JSON.stringify(typed.slice(0, 160))})`);
await page.screenshot({ path: join(out, 'browser.png') });

// ---- 2. sample of Examples through the UI ------------------------------------------------
const only = arg('--only', '');
const sample = JSON.parse(readFileSync(join(here, 'sample.json'), 'utf8')).filter((f) => f.includes(only));
// Chromium blocks a tab's automatic downloads after about ten (the export download comes
// after an async render, outside the click's user gesture), so reload every few files.
const DOWNLOADS_PER_PAGE = 8;
async function openExportTab() {
    await page.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });
    await page.getByText('Export', { exact: true }).first().click();
}
await openExportTab();
const previewGroup = page.locator('.export-group').filter({ has: page.locator('.export-group-title', { hasText: /^Preview$/ }) });
const saveHtml = previewGroup.getByRole('button', { name: /Save HTML/ });
const index = [];
for (const [n, rel] of sample.entries()) {
    if (n > 0 && n % DOWNLOADS_PER_PAGE === 0) {
        await page.reload();
        await openExportTab();
    }
    const text = readFileSync(join(ref, rel), 'utf8').replace(/^﻿/, '');
    await page.evaluate((t) => (window).calcpadActiveEditor.setValue(t), text);
    const started = Date.now();
    let download;
    try {
        [download] = await Promise.all([page.waitForEvent('download', { timeout: 120000 }), saveHtml.click()]);
    } catch (err) {
        console.log(`no download for ${rel}; last console lines:\n${consoleLines.slice(-40).join('\n')}`);
        throw err;
    }
    const html = readFileSync(await download.path(), 'utf8');
    // The template mentions <body> in a CSS comment, so look after </head>.
    const open = html.indexOf('<body', html.indexOf('</head>'));
    const bodyStart = html.indexOf('>', open) + 1;
    const bodyEnd = html.lastIndexOf('</body>');
    // template.html: "<body>\n    {{CONTENT}}\n</body>"
    let body = html.slice(bodyStart, bodyEnd);
    if (body.startsWith('\n    ')) body = body.slice(5);
    if (body.endsWith('\n')) body = body.slice(0, -1);
    const name = rel.replace(/\//g, '__') + '.html';
    writeFileSync(join(renderedDir, name), `<div class="calcpad-output">${body}</div>`);
    index.push({ file: rel, rendered: name, ms: Date.now() - started });
    console.log(`     rendered ${rel} (${Date.now() - started} ms)`);
}
writeFileSync(join(renderedDir, 'index.json'), JSON.stringify(index, null, 1));

// ---- server-only exports say so ----------------------------------------------------------
const reportGroup = page.locator('.export-group').filter({ has: page.locator('.export-group-title', { hasText: /^Report$/ }) });
// A persistent handler: with only waitForEvent('dialog') the alert stayed open and the click hung.
const dialogs = [];
page.on('dialog', (d) => {
    dialogs.push(d.message());
    void d.dismiss();
});
for (const [button, what] of [[/Save PDF/, 'PDF export'], [/Save Word/, 'Word export']]) {
    dialogs.length = 0;
    await reportGroup.getByRole('button', { name: button }).click({ timeout: 10000 });
    for (let i = 0; i < 50 && dialogs.length === 0; i++) await page.waitForTimeout(100);
    const message = dialogs[0] ?? '';
    check(message.startsWith(`${what} is not available in the browser build`), `${what} explains it needs the server (${JSON.stringify(message.slice(0, 80))})`);
}

// ---- 3. no calculation request left the page ---------------------------------------------
const STATIC = /^\/(|index\.html|assets\/.*|wasm\/.*|fonts\/.*|report-fonts\/.*|icon\.png|favicon\.ico)$/;
const remote = requests.filter((r) => !r.url.startsWith('data:') && !r.url.startsWith('blob:'));
const api = remote.filter((r) => new URL(r.url).pathname.includes('/api/'));
const foreign = remote.filter((r) => new URL(r.url).origin !== origin);
const nonStatic = remote.filter((r) => new URL(r.url).origin === origin && (r.method !== 'GET' || !STATIC.test(new URL(r.url).pathname)));
check(api.length === 0, `no /api/ request on the network (${api.length}: ${api.map((r) => r.url).join(', ')})`);
check(foreign.length === 0, `no request to another origin (${foreign.map((r) => r.url).join(', ')})`);
check(nonStatic.length === 0, `only GETs of static files (${nonStatic.map((r) => `${r.method} ${r.url}`).join(', ')})`);
check(remote.some((r) => r.url.endsWith('/wasm/calcpad_wasm_bg.wasm')), 'the engine was loaded from /wasm/calcpad_wasm_bg.wasm');
const methods = [...new Set(remote.map((r) => r.method))];
console.log(`     ${remote.length} network requests in total, methods: ${methods.join(', ')}`);
writeFileSync(join(out, 'browser-requests.json'), JSON.stringify(requests, null, 1));
check(pageErrors.length === 0, `no uncaught page errors (${pageErrors.join(' | ')})`);

await browser.close();
stop();
console.log(failures.length ? `${failures.length} check(s) failed` : 'all browser checks passed');
console.log(`sample rendered to ${renderedDir}; diff with: python3 -I tools/compare_browser.py`);
process.exit(failures.length ? 1 : 0);
