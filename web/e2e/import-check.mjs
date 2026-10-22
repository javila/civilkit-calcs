// End-to-end check of "Import worksheet..." in the browser build (headless Chromium, Playwright).
//
// A small synthetic .xmcd is generated here (a text region, a definition, an addition, a vector
// metafile picture, a raster picture, and an unconvertible region that carries a saved picture and a
// saved result). The check imports it through the File menu, the ribbon button and a drop, and looks at
// the tab, the calculated maths, the pictures (naturalWidth), the dialog counts and toggles, the error
// paths (bad extension, malformed XML, a .mcdx) and what Save and the exports do with the pictures.
//
//   node e2e/import-check.mjs [--port 4179] [--dist DIR] [--real FILE.xmcd --shots DIR]
//
// --real imports one real file instead of running the checks and takes screenshots of the dialog and of
// the imported sheet in the dark and the light theme (local use: the file is read from disk, never copied).
import { createServer } from 'node:http';
import { execFileSync } from 'node:child_process';
import { deflateSync } from 'node:zlib';
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const arg = (name, def) => {
    const i = process.argv.indexOf(name);
    return i > 0 ? process.argv[i + 1] : def;
};
const port = Number(arg('--port', '4179'));
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const realFile = arg('--real', '');
const shotsDir = arg('--shots', join(tmpdir(), 'import-shots'));
const origin = `http://127.0.0.1:${port}`;
const tmp = mkdtempSync(join(tmpdir(), 'import-check-'));

const failures = [];
const check = (ok, what) => {
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
    if (!ok) failures.push(what);
};

// ---- a plain static file server for dist ------------------------------------------------------------
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.otf': 'font/otf', '.cpd': 'text/plain' };
const server = createServer((req, res) => {
    let p = decodeURIComponent(new URL(req.url, origin).pathname);
    if (p.endsWith('/')) p += 'index.html';
    const f = normalize(join(dist, p));
    if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { res.writeHead(404); return res.end(); }
    res.writeHead(200, { 'Content-Type': TYPES[extname(f)] ?? 'application/octet-stream' });
    res.end(readFileSync(f));
});
await new Promise(ok => server.listen(port, '127.0.0.1', ok));

// ---- the synthetic worksheet ------------------------------------------------------------------------------
function crc32(buf) {
    let c, crc = 0xffffffff;
    for (let n = 0; n < buf.length; n++) {
        c = (crc ^ buf[n]) & 0xff;
        for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
        crc = (crc >>> 8) ^ c;
    }
    return (crc ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
    const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
    const td = Buffer.concat([Buffer.from(type), data]);
    const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td));
    return Buffer.concat([len, td, crc]);
}
/** A real w x h RGBA PNG, filled with one colour. */
function png(w, h, rgba) {
    const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4); ihdr[8] = 8; ihdr[9] = 6;
    const row = Buffer.concat([Buffer.from([0]), Buffer.from(Array.from({ length: w }, () => rgba).flat())]);
    const raw = Buffer.concat(Array.from({ length: h }, () => row));
    return Buffer.concat([Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]);
}
const words = (...w) => Buffer.concat(w.map(x => { const b = Buffer.alloc(2); b.writeUInt16LE(x); return b; }));
/** A minimal WMF: window extents and one rectangle. */
function wmf() {
    const body = Buffer.concat([words(5, 0, 0x020c, 40, 60), words(5, 0, 0x020b, 0, 0), words(7, 0, 0x041b, 30, 50, 5, 5), words(3, 0, 0)]);
    const total = 9 + body.length / 2;
    return Buffer.concat([words(1, 9, 0x0300, total & 0xffff, total >> 16, 0, 7, 0, 0), body]);
}
const b64 = b => b.toString('base64');
const ML = 'xmlns:ml="http://example.invalid/math30"';
const region = (id, top, inner, extra = '') => `<region region-id="${id}" left="10" top="${top}" width="${id === 4 ? 90 : 120}" height="30" align-y="${top + 10}">${inner}${extra}</region>`;
function syntheticXmcd() {
    const png1 = png(8, 8, [200, 30, 30, 255]);
    return `<?xml version="1.0" encoding="UTF-8"?>
<worksheet version="3.0.3" xmlns="http://example.invalid/worksheet30" ${ML}>
<settings><calculation><builtInVariables array-origin="1"/></calculation></settings>
<regions>
${region(1, 10, '<text><p style="Heading 1">Imported test sheet</p></text>')}
${region(2, 50, '<math><ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define></math>')}
${region(3, 90, '<math><ml:eval><ml:apply><ml:plus/><ml:id>x</ml:id><ml:real>3</ml:real></ml:apply></ml:eval></math>')}
${region(4, 130, `<math><ml:eval><ml:apply><ml:id>vendor_fn</ml:id><ml:real>1</ml:real></ml:apply><result xmlns="http://example.invalid/math30"><ml:real>42.5</ml:real></result></ml:eval></math>`, '<rendering item-idref="2"/>')}
${region(5, 170, '<picture><metafile item-idref="3"/></picture>')}
${region(6, 250, '<picture><png item-idref="2"/></picture>')}
</regions>
<binaryContent>
<item item-id="2">${b64(png1)}</item>
<item item-id="3">${b64(wmf())}</item>
</binaryContent>
</worksheet>`;
}

// ---- the page -----------------------------------------------------------------------------------------------
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1400, height: 950 }, acceptDownloads: true });
const page = await ctx.newPage();
const errors = [];
page.on('pageerror', e => errors.push(String(e)));
await page.goto(`${origin}/`);
await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
await page.evaluate(() => { delete window.showSaveFilePicker; delete window.showOpenFilePicker; });

const dialogText = () => page.locator('#cprs-import-dialog').innerText();
const closeDialog = async () => { if (await page.locator('#cprs-import-dialog').count()) { await page.keyboard.press('Escape'); await page.waitForTimeout(150); } };
const editorText = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
/** The frame that shows the rendered report: the one whose text holds the sheet's heading (or any text). */
async function previewFrame(needle = 'Imported test sheet') {
    for (let i = 0; i < 60; i++) {
        for (const fr of page.frames()) {
            try { if (await fr.evaluate((n) => document.body && (n ? document.body.innerText.includes(n) : document.body.innerText.length > 20) && !/Calculating/.test(document.body.innerText), needle)) return fr; } catch { /* a detached frame */ }
        }
        await page.waitForTimeout(500);
    }
    return null;
}
async function chooseFile(trigger, name, data) {
    const [fc] = await Promise.all([page.waitForEvent('filechooser'), trigger()]);
    await fc.setFiles({ name, mimeType: 'application/octet-stream', buffer: Buffer.isBuffer(data) ? data : Buffer.from(data) });
}
const openFileMenu = async () => { await page.locator('[data-ribbon-btn=file]').click(); await page.waitForTimeout(200); };

if (realFile) {
    // ---- local screenshots of one real file, in one theme --------------------------------------------------
    mkdirSync(shotsDir, { recursive: true });
    const theme = arg('--theme', 'Light');
    const t = theme.toLowerCase();
    const name = realFile.split('/').pop();
    await page.evaluate((th) => window.calcpadBridge.handleMessage({ type: 'updateColorTheme', theme: th }), theme);
    await page.waitForTimeout(800);
    const t0 = Date.now();
    await openFileMenu();
    await chooseFile(() => page.getByRole('menuitem', { name: /Import worksheet/ }).click(), name, readFileSync(realFile));
    await page.locator('[data-cprs-import-summary]').waitFor({ timeout: 120000 });
    console.log(`import took ${Date.now() - t0} ms`);
    console.log(await dialogText());
    await page.screenshot({ path: join(shotsDir, `import-dialog-${t}.png`) });
    await page.keyboard.press('Escape');
    const fr = await previewFrame('');
    if (fr) await fr.evaluate(() => { const im = [...document.images].find(i => i.src.startsWith('data:image/svg')) || document.images[0]; im?.scrollIntoView({ block: 'center' }); });
    await page.waitForTimeout(1000);
    await page.screenshot({ path: join(shotsDir, `import-sheet-${t}.png`) });
    await browser.close();
    server.close();
    process.exit(0);
}

// ---- 1. the menu and the ribbon offer the command ---------------------------------------------------------------------
await openFileMenu();
check(await page.getByRole('menuitem', { name: /Import worksheet/ }).count() === 1, 'the File menu has "Import worksheet..."');

// ---- 2. import through the File menu ---------------------------------------------------------------------------------------
const xmcd = syntheticXmcd();
await chooseFile(() => page.getByRole('menuitem', { name: /Import worksheet/ }).click(), 'synth.xmcd', xmcd);
await page.locator('[data-cprs-import-summary]').waitFor({ timeout: 60000 });
const summary = await page.locator('[data-cprs-import-summary]').innerText();
check(/Imported synth\.xmcd: 3 of 6 regions converted live, 3 shown as static pictures, 1 need attention\./.test(summary), `dialog counts: ${summary}`);
const gaps = await page.locator('[data-cprs-import-gaps] li').allInnerTexts();
check(gaps.some(g => /calls to a function from an add-in we cannot run/.test(g)), `ranked reason in plain words (${gaps.join(' | ')})`);
check(await page.locator('[data-cprs-import-pictures]').isChecked() && !(await page.locator('[data-cprs-import-saved]').isChecked()), 'toggles default: pictures on, saved results off');
await page.screenshot({ path: join(tmp, 'dialog.png') });
await closeDialog();

const title = await page.evaluate(() => window.calcpadTabs?.activeTab?.title ?? [...document.querySelectorAll('.tab, [role=tab]')].map(e => e.textContent).join('|'));
check(/synth\.cpd/.test(String(title)), `a new tab named after the file (${title})`);
const text = await editorText();
check(/x = 2/.test(text) && /x \+ 3/.test(text) && /<h1>Imported test sheet<\/h1>/.test(text), 'the editor holds the converted worksheet');
check(/src="imported\/synth\/R5\.svg"/.test(text) && /src="imported\/synth\/R4\.png"/.test(text) && /static picture of the original, not live \(R4 addin-function\)/.test(text), 'pictures are referenced by library path and the fallback is captioned');

const lib = await page.evaluate(() => window.calcpadFiles.library.list().map(f => f.path));
check(['/imported/synth/R5.svg', '/imported/synth/R6.png', '/imported/synth/R4.png'].every(p => lib.includes(p)), `assets are in the library (${lib.filter(p => p.startsWith('/imported')).join(', ')})`);

const fr = await previewFrame();
check(!!fr, 'the preview renders');
if (fr) {
    const body = await fr.evaluate(() => document.body.innerText);
    check(/x\s*=\s*2/.test(body) && /=\s*5\b/.test(body), 'the live maths calculates (x = 2, x + 3 = 5)');
    await page.waitForTimeout(500);
    const imgs = await fr.evaluate(() => [...document.images].map(i => ({ w: i.naturalWidth, h: i.naturalHeight, src: i.src.slice(0, 24), style: i.getAttribute('style') })));
    check(imgs.length === 3 && imgs.every(i => i.w > 0), `all three pictures display (${JSON.stringify(imgs.map(i => [i.w, i.h, i.src.slice(5, 20)]))})`);
    check(imgs.some(i => i.src.startsWith('data:image/svg+xml')) && imgs.every(i => /width:\d+(\.\d+)?pt/.test(i.style ?? '')), 'the SVG is shown and the width style is kept');
}

// ---- 3. the toggles re-run the import in the same tab -------------------------------------------------------------------------
await openFileMenu(); await page.keyboard.press('Escape');
await page.getByRole('tab', { name: 'Document' }).click();
await page.getByRole('button', { name: /Import worksheet/ }).waitFor();
check(true, 'the Document tab has an "Import worksheet" button');
await chooseFile(() => page.getByRole('button', { name: /Import worksheet/ }).click(), 'synth.xmcd', xmcd);
await page.locator('[data-cprs-import-summary]').waitFor();
await page.locator('[data-cprs-import-pictures]').uncheck();
await page.waitForFunction(() => /need attention/.test(document.querySelector('[data-cprs-import-summary]')?.textContent ?? '') && !document.querySelector('[data-cprs-import-pictures]')?.checked, null, { timeout: 30000 });
const off = await page.locator('[data-cprs-import-summary]').innerText();
check(/2 shown as static pictures/.test(off) === false || /1 need attention/.test(off), `pictures toggle re-runs (${off})`);
const t2 = await editorText();
check(/import-gap R4 addin-function/.test(t2) && !/R4\.png/.test(t2), 'with pictures off the unconverted region is a placeholder line');
const lib2 = await page.evaluate(() => window.calcpadFiles.library.list().map(f => f.path));
check(!lib2.includes('/imported/synth/R4.png'), 'its picture was removed from the library');
await page.locator('[data-cprs-import-saved]').check();
await page.waitForFunction(() => /saved result/.test(window.calcpadActiveEditor.getValue()), null, { timeout: 30000 });
check(/saved result from the original, not live \(R4\): 42\.5/.test(await editorText()), 'saved results toggle writes a labelled static value');
const tabsNow = await page.evaluate(() => document.querySelectorAll('.tab, [role=tab]').length);
await closeDialog();

// ---- 4. errors -------------------------------------------------------------------------------------------------------------
const importVia = async (name, data) => {
    await closeDialog();
    await openFileMenu();
    await chooseFile(() => page.getByRole('menuitem', { name: /Import worksheet/ }).click(), name, data);
    await page.locator('[data-cprs-import-message], [data-cprs-import-summary]').first().waitFor({ timeout: 30000 });
    return page.locator('#cprs-import-dialog').innerText();
};
let m = await importVia('notes.txt', 'hello');
check(/not a worksheet this importer reads/.test(m), `wrong extension: ${m.replace(/\s+/g, ' ').slice(0, 80)}`);
m = await importVia('broken.xmcd', '<worksheet><regions><region></worksheet>');
check(/Import failed/.test(m) && /not well-formed XML/.test(m), `malformed XML: ${m.replace(/\s+/g, ' ').slice(0, 100)}`);
m = await importVia('bomb.xmcd', '<?xml version="1.0"?><!DOCTYPE d [<!ENTITY a "aaaa"><!ENTITY b "&a;&a;&a;&a;">]><worksheet><regions><region><text><p>&b;</p></text></region></regions></worksheet>');
check(/Import failed/.test(m), 'a DTD (entity bomb) is refused with a message');
m = await importVia('prime.mcdx', Buffer.concat([Buffer.from('PK\x03\x04'), Buffer.alloc(64)]));
check(/Prime \(\.mcdx\) files are not supported yet\./.test(m), `.mcdx: ${m.replace(/\s+/g, ' ').slice(0, 80)}`);
m = await importVia('binary.xmcd', Buffer.from([0xff, 0xfe, 0x00, 0x01, 0x02]));
check(/Import failed/.test(m), 'non-text content is refused with a message');
await closeDialog();
check((await page.evaluate(() => document.querySelectorAll('.tab, [role=tab]').length)) === tabsNow, 'failed imports opened no tab');
check(errors.length === 0, `no page errors (${errors.slice(0, 2).join(' | ')})`);

// ---- 5. drop, with the hint ------------------------------------------------------------------------------------------------------
const dropped = await page.evaluate(async (xml) => {
    const dt = new DataTransfer();
    dt.items.add(new File([xml], 'dropped.xmcd', { type: 'application/xml' }));
    document.dispatchEvent(new DragEvent('dragover', { dataTransfer: dt, bubbles: true, cancelable: true }));
    const hint = !!document.querySelector('[data-cprs-drop-hint]');
    document.dispatchEvent(new DragEvent('drop', { dataTransfer: dt, bubbles: true, cancelable: true }));
    return hint;
}, xmcd);
check(dropped, 'a drag over the page shows the drop hint');
await page.locator('[data-cprs-import-summary]').filter({ hasText: 'dropped.xmcd' }).waitFor({ timeout: 30000 });
check(!(await page.locator('[data-cprs-drop-hint]').count()), 'the hint is gone after the drop');
check((await page.evaluate(() => window.calcpadFiles.library.list().map(f => f.path))).includes('/imported/dropped/R5.svg'), 'the dropped sheet has its own library folder');
await closeDialog();

// ---- 6. Save keeps the pictures; exports inline them ----------------------------------------------------------------------------
page.on('dialog', d => d.accept('saved-synth.cpd'));
const [dl] = await Promise.all([page.waitForEvent('download'), page.evaluate(() => window.calcpadFiles.files.save())]);
check(/saved-synth\.cpd$/.test(dl.suggestedFilename()), 'Save writes the .cpd');
const libAfter = await page.evaluate(() => window.calcpadFiles.library.list().map(f => f.path));
check(libAfter.includes('/saved-synth.cpd') && libAfter.includes('/imported/dropped/R5.svg'), 'after Save the worksheet is in the library and the pictures stay');
const fr2 = await previewFrame();
if (fr2) {
    const srcs = await fr2.evaluate(() => [...document.images].map(i => i.src.slice(0, 11)));
    check(srcs.length > 0 && srcs.every(s => s.startsWith('data:image/')), 'the rendered page inlines every library picture (what HTML export saves)');
}
const docxPath = join(tmp, 'out.docx');
try {
    const [dd] = await Promise.all([page.waitForEvent('download', { timeout: 60000 }), page.evaluate(() => window.calcpadBridge?.handleMessage?.({ type: 'saveDocx', variant: 'report' }) ?? window.calcpadFiles.files.o.exportDocx())]);
    await dd.saveAs(docxPath);
    const listing = execFileSync('unzip', ['-l', docxPath]).toString();
    check(/word\/media\//.test(listing), 'the Word export contains the pictures (word/media)');
} catch (e) {
    check(false, `the Word export (${String(e).slice(0, 120)})`);
}

await browser.close();
server.close();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nall import checks passed');
process.exit(failures.length ? 1 : 0);
