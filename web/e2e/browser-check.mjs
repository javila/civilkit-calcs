// End-to-end check of the browser build (calcpad-web on the calcpad-wasm engine), headless
// Chromium via Playwright.
//
//   1. load the page, type a worksheet into the editor, read the result in the preview;
//   2. run a sample of CalcpadCE's Examples/Tests through the UI (editor content, then
//      Export > Preview > Save HTML, the path that renders without line anchors as the CLI
//      stubs do) and save the bodies for tools/compare_browser.py;
//   3. record every network request and fail if any calculation request leaves the page;
//   4. save a screenshot to tools/out/browser.png;
//   5. input forms (#UI), the file library (#include, #read), #write downloads, open by
//      drop, save, recent files and printing;
//   6. the same build served from a sub folder.
//
// The build is served by a plain static file server (no Calcpad.Server, no vite), at "/" and
// under "/calcs/", which is what any static host does.
//
// Usage (from web/, after `npm run build:wasm && npm run build`):
//   node e2e/browser-check.mjs [--ref ../../calcpad-ref] [--port 4180]
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
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
const origin = `http://127.0.0.1:${port}`;
const out = join(repo, 'tools', 'out');
const renderedDir = join(out, 'browser');
rmSync(renderedDir, { recursive: true, force: true });
mkdirSync(renderedDir, { recursive: true });

const failures = [];
const check = (ok, what) => {
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
    if (!ok) failures.push(what);
};

// ---- serve web/calcpad-web/dist as static files, at / and under /calcs/ -------------------
const dist = join(web, 'calcpad-web', 'dist');
const TYPES = {
    '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css',
    '.wasm': 'application/wasm', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml',
    '.woff2': 'font/woff2', '.otf': 'font/otf', '.ttf': 'font/ttf', '.ico': 'image/x-icon', '.md': 'text/plain',
};
const server = createServer((req, res) => {
    let path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    if (path.startsWith('/calcs/')) path = path.slice('/calcs'.length);
    if (path.endsWith('/')) path += 'index.html';
    const file = normalize(join(dist, path));
    if (!file.startsWith(dist) || !existsSync(file) || !statSync(file).isFile()) {
        res.writeHead(404);
        res.end('not found');
        return;
    }
    res.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
    res.end(readFileSync(file));
});
await new Promise((ok) => server.listen(port, '127.0.0.1', ok));
const serverLog = '';
const stop = () => server.close();

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
// Printing: count print() calls from the print frame instead of opening a dialog.
await context.addInitScript(() => {
    if (window === window.top) return;
    window.print = () => {
        try {
            window.top.__printed = (window.top.__printed ?? 0) + 1;
            window.top.__printedHtml = document.documentElement.outerHTML;
        } catch { /* a sandboxed result frame cannot reach the top window */ }
    };
});
// Without the File System Access API, so saving takes the download path, which the test can see.
await context.addInitScript(() => {
    if (window !== window.top) return;
    delete window.showSaveFilePicker;
    delete window.showOpenFilePicker;
});
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

/** Text of a frame that contains `marker` and matches `re`, else of any frame with `marker`. */
async function framesMatching(marker, re) {
    let fallback = '';
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
        if (!t.includes(marker)) continue;
        if (re.test(t)) return t;
        fallback = t;
    }
    return fallback;
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

// ---- 5. phase 4: input forms, files, #read/#write, print ---------------------------------
await page.reload();
await page.waitForFunction(() => (window).calcpadActiveEditor != null && (window).calcpadFiles != null, null, { timeout: 30000 });
const setEditor = (text) => page.evaluate((t) => (window).calcpadActiveEditor.setValue(t), text);
async function resultMode(name) {
    const modal = page.locator('.modal-backdrop');
    if (await modal.count()) {
        console.log(`     modal before ${name}: ${JSON.stringify((await modal.first().innerText()).slice(0, 300))}`);
        await page.screenshot({ path: join(out, 'browser-modal.png') });
    }
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: new RegExp(`^${name}$`) }).click({ timeout: 10000 });
    // Leaving the input form after entering values asks whether to keep them (upstream's prompt).
    const dontSave = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
    if (await dontSave.waitFor({ timeout: 1500 }).then(() => true, () => false)) await dontSave.click();
}
async function frameWith(selector, timeout = 15000) {
    for (let t = 0; t < timeout; t += 200) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            if (await f.$(selector).catch(() => null)) return f;
        }
        await page.waitForTimeout(200);
    }
    return null;
}
async function frameText(marker, timeout = 15000) {
    for (let t = 0; t < timeout; t += 200) {
        const text = await previewText(marker);
        if (text) return text;
        await page.waitForTimeout(200);
    }
    return '';
}
async function drop(files) {
    await page.evaluate((files) => {
        const dt = new DataTransfer();
        for (const f of files) dt.items.add(new File([f.text], f.name));
        document.body.dispatchEvent(new DragEvent('drop', { dataTransfer: dt, bubbles: true, cancelable: true }));
    }, files);
}

// #UI input form: an entry the user changes, and a datagrid hydrated by jspreadsheet.
await setEditor("'UI form check\n#UI L = 10m\nA = L*2m\n#UI v = [1; 2; 3]\ns = sum(v)");
await resultMode('Input');
const form = await frameWith('input.calcpad-ui-input');
check(form != null, 'input mode renders the #UI entry as an input box');
if (form) {
    check(await form.$eval('input.calcpad-ui-input', (e) => e.value) === '10', 'the entry holds the value without its unit (10)');
    const grid = await frameWith('.calcpad-ui-datagrid .jss_worksheet, .calcpad-ui-datagrid table');
    check(grid != null, 'the datagrid is hydrated (the libraries were injected)');
    // The form can render more than once while it settles, and typing into a frame that was
    // replaced posts nothing, so the live frame is looked up again for each attempt.
    let text = '';
    for (let attempt = 0; attempt < 3 && !/=\s*24\s*m/.test(text); attempt++) {
        await page.waitForTimeout(1500);
        const live = await frameWith('input.calcpad-ui-input');
        if (!live) continue;
        await live.fill('input.calcpad-ui-input', '12');
        await live.press('input.calcpad-ui-input', 'Enter');
        for (let i = 0; i < 75 && !/=\s*24\s*m/.test(text); i++) {
            await page.waitForTimeout(200);
            // The preview is double buffered, so the stale frame can still be there: look at all.
            text = await framesMatching('UI form check', /=\s*24\s*m/);
        }
    }
    check(/=\s*24\s*m/.test(text), `changing L to 12 re-renders A = 24 m² (form: ${JSON.stringify(text.slice(0, 120))})`);
}
await page.screenshot({ path: join(out, 'browser-input-form.png') });
await resultMode('Preview');

const downloads = [];
page.on('download', (d) => downloads.push(d));

// The library: a dropped worksheet serves #include, a dropped CSV serves #read.
await drop([{ name: 'lib-check.cpd', text: 'k = 5\n' }, { name: 'data-check.csv', text: '1,2\n3,4\n' }]);
await page.waitForFunction(() => (window).calcpadFiles.library.list().length >= 2, null, { timeout: 10000 });
// The dropped worksheet opened in a tab of its own; the check runs in a new one.
await page.locator('.cprs-file-btn').click();
await page.locator('.cprs-menu button', { hasText: 'New worksheet' }).click();
await setEditor("'Library check\n#include lib-check.cpd\nk2 = k*2\n#read M from data-check.csv\nt = sum(M)\n#write M to out-check.csv");
const libText = await frameText('Library check');
check(/k2\s*=[\s\S]*=\s*10/.test(libText), `#include resolves against the library (k2 = 10) (${JSON.stringify(libText.slice(0, 160))})`);
check(/t\s*=[\s\S]*=\s*10/.test(libText) && libText.includes('was successfully read from'), '#read reads a CSV from the library (t = 10)');
check(libText.includes('will be written to'), 'a preview render only reports the #write (the default write mode is Report Only)');

// #write in the report downloads the file, once: a render that writes the same content again
// (the report re-renders often) does not download it again.
await resultMode('Report');
await page.waitForTimeout(3000);
await resultMode('Preview');
await resultMode('Report');
for (let i = 0; i < 50 && !downloads.some((d) => d.suggestedFilename() === 'out-check.csv'); i++) await page.waitForTimeout(200);
await page.waitForTimeout(2000);
const written = downloads.filter((d) => d.suggestedFilename() === 'out-check.csv');
check(written.length === 1, `#write downloads out-check.csv once over two report renders (${written.length})`);
if (written.length) check(readFileSync(await written[0].path(), 'utf8') === '1,2\n3,4\n', '#write writes the matrix as CSV');

// Print: the report goes to the browser's print().
await page.evaluate(() => { (window).__printed = 0; });
await page.locator('.preview-toolbar .toolbar-btn', { hasText: /^Print PDF$/ }).click();
await page.waitForFunction(() => (window).__printed > 0, null, { timeout: 15000 }).catch(() => {});
const printed = await page.evaluate(() => ({ n: (window).__printed, html: (window).__printedHtml ?? '' }));
check(printed.n === 1 && printed.html.includes('@page') && printed.html.includes('Library check'),
    `Print PDF prints the report through the browser (${printed.n} print call)`);
await resultMode('Preview');

// Open by drop, edit, save (download), and the recent list.
await drop([{ name: 'opened-check.cpd', text: "'Opened by drop\nz = 3" }]);
await page.waitForFunction(() => (window).calcpadTabs?.activeTab?.filePath === '/opened-check.cpd', null, { timeout: 10000 });
check(true, 'a dropped .cpd opens in its own tab');
await setEditor("'Opened by drop\nz = 4");
await page.locator('.monaco-editor').first().click();
const [saved] = await Promise.all([page.waitForEvent('download', { timeout: 10000 }), page.keyboard.press('ControlOrMeta+S')]);
check(saved.suggestedFilename() === 'opened-check.cpd' && readFileSync(await saved.path(), 'utf8') === "'Opened by drop\nz = 4",
    `Ctrl+S saves the worksheet (${saved.suggestedFilename()})`);
check(await page.evaluate(() => !(window).calcpadTabs.activeTab.dirty), 'the saved tab is no longer dirty');
const savedText = await (async () => {
    let t = '';
    for (let i = 0; i < 75 && !/z\s*=\s*4/.test(t); i++) {
        await page.waitForTimeout(200);
        t = await framesMatching('Opened by drop', /z\s*=\s*4/);
    }
    return t;
})();
check(/z\s*=\s*4/.test(savedText), `the preview follows the edit (z = 4) (${JSON.stringify(savedText.slice(0, 60))})`);
await page.locator('.cprs-file-btn').click();
const menuText = await page.locator('.cprs-menu').innerText();
check(menuText.includes('opened-check.cpd') && menuText.includes('lib-check.cpd') && menuText.includes('data-check.csv'),
    'the File menu lists the recent worksheet and the library');
await page.screenshot({ path: join(out, 'browser-file-menu.png') });
await page.keyboard.press('Escape');

// ---- server-only exports say so ----------------------------------------------------------
const reportGroup = page.locator('.export-group').filter({ has: page.locator('.export-group-title', { hasText: /^Report$/ }) });
// A persistent handler: with only waitForEvent('dialog') the alert stayed open and the click hung.
const dialogs = [];
page.on('dialog', (d) => {
    dialogs.push(d.message());
    void d.dismiss();
});
await openExportTab();
for (const [button, what] of [[/Save Word/, 'Word export']]) {
    dialogs.length = 0;
    await reportGroup.getByRole('button', { name: button }).click({ timeout: 10000 });
    for (let i = 0; i < 50 && dialogs.length === 0; i++) await page.waitForTimeout(100);
    const message = dialogs[0] ?? '';
    check(message.startsWith(`${what} is not available in the browser build`), `${what} explains it needs the server (${JSON.stringify(message.slice(0, 80))})`);
}

// ---- 6. the same build from a sub folder ---------------------------------------------------
const sub = await context.newPage();
sub.on('pageerror', (e) => pageErrors.push(`/calcs/: ${e.message}`));
await sub.goto(`${origin}/calcs/`);
await sub.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });
await sub.evaluate(() => (window).calcpadActiveEditor.setValue("'Sub folder check\nx = 6*7"));
let subText = '';
for (let i = 0; i < 75 && !/=\s*42/.test(subText); i++) {
    await sub.waitForTimeout(200);
    for (const f of sub.frames()) {
        if (f === sub.mainFrame()) continue;
        const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
        if (t.includes('Sub folder check')) subText = t;
    }
}
check(/=\s*42/.test(subText), 'served from /calcs/, the app loads the engine and calculates (x = 42)');
const subFailed = requests.filter((r) => r.url.includes('/calcs/') && r.url.endsWith('.wasm')).length;
check(subFailed > 0, 'under /calcs/ the engine is fetched from /calcs/wasm/');

// ---- 3. no calculation request left the page ---------------------------------------------
const STATIC = /^\/(calcs\/)?(|index\.html|assets\/.*|wasm\/.*|fonts\/.*|report-fonts\/.*|ui-assets\/.*|icon\.png|favicon\.ico)$/;
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
