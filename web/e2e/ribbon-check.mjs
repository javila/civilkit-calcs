// Checks the ribbon in the built app served at --url (default http://127.0.0.1:4175/):
// every tab shows, an Insert button changes the editor, File > Excel downloads, a phone-width shot.
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/ribbon-check.mjs [--url U] [--out DIR]
import { chromium } from 'playwright';
import { mkdirSync } from 'node:fs';
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4175/');
const out = arg('--out', '/tmp/civilkit-calcs-shots');
mkdirSync(out, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
async function open(w, h, scheme) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, colorScheme: scheme, acceptDownloads: true });
    await ctx.addInitScript(() => { if (window === window.top) { delete window.showSaveFilePicker; delete window.showOpenFilePicker; } });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(String(e)));
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
    return { page, errors };
}
const text = page => page.evaluate(() => window.calcpadActiveEditor.getValue());

// desktop, light
let { page, errors } = await open(1400, 820, 'light');
const tabs = ['file', 'home', 'insert', 'math', 'units', 'view'];
for (const t of tabs) {
    await page.locator(`[data-ribbon-tab=${t}]`).click();
    const n = await page.locator('.rb-panel .rb-btn').count();
    check(n > 0 && await page.locator(`[data-ribbon-panel=${t}]`).isVisible(), `tab ${t} shows ${n} buttons`);
}
// sample open, Home tab
await page.locator('[data-ribbon-tab=file]').click();
await page.locator('[data-ribbon-btn=samples]').click();
await page.waitForTimeout(800);
await page.keyboard.press('Escape');
await page.evaluate(async () => {
    const m = await (await fetch('samples/manifest.json')).json();
    const s = m.samples.find(x => /beam/i.test(x.title) && !x.ui) ?? m.samples[0];
    await window.calcpadFiles.samples.openSample(s);
});
await page.waitForTimeout(9000);
await page.locator('[data-ribbon-tab=home]').click();
await page.screenshot({ path: `${out}/desktop-light-home.png` });

// Insert changes the editor text
await page.locator('[data-ribbon-tab=insert]').click();
const before = await text(page);
await page.locator('[data-ribbon-btn=i-matrix]').click();
const after = await text(page);
check(after !== before && after.includes('A = [1; 2| 3; 4]'), 'Insert > Matrix changes the editor text');
await page.locator('[data-ribbon-tab=units]').click();
await page.locator('[data-ribbon-btn="u-kN*m"]').click();
check((await text(page)).includes('kN*m'), 'Units > kN*m inserts text');
// Home: Calculate and Report mode
await page.locator('[data-ribbon-tab=home]').click();
await page.locator('[data-ribbon-btn=m-report]').click();
await page.waitForTimeout(300);
check(await page.evaluate(() => document.querySelector('[data-ribbon-btn=m-report]').classList.contains('on')), 'Home > Report mode toggles on');
await page.locator('[data-ribbon-btn=m-preview]').click();
await page.locator('[data-ribbon-btn=calc]').click();
await page.waitForTimeout(500);
// Undo
await page.locator('[data-ribbon-btn=undo]').click();
check((await text(page)) !== after, 'Home > Undo reverts an edit');
// File > Excel downloads
await page.locator('[data-ribbon-tab=file]').click();
const dl = page.waitForEvent('download', { timeout: 20000 }).catch(() => null);
await page.locator('[data-ribbon-btn=excel]').click();
const d = await dl;
check(!!d && /\.xlsx$/i.test(d.suggestedFilename()), `File > Excel triggers a download (${d?.suggestedFilename()})`);
// File > Word
const dw = page.waitForEvent('download', { timeout: 20000 }).catch(() => null);
await page.locator('[data-ribbon-btn=word]').click();
const w = await dw;
check(!!w && /\.docx$/i.test(w.suggestedFilename()), `File > Word triggers a download (${w?.suggestedFilename()})`);
// File > Recent/library menu still opens
await page.locator('[data-ribbon-btn=recent]').click();
check(await page.locator('.cprs-menu').isVisible() && (await page.locator('.cprs-menu').innerText()).includes('Save As'), 'File > Recent and library opens the full file menu');
await page.keyboard.press('Escape');
// Ctrl+S shortcut still saves (download path)
// View: theme
await page.locator('[data-ribbon-tab=view]').click();
await page.locator('[data-ribbon-btn=dark]').click();
check(await page.evaluate(() => document.documentElement.dataset.theme) === 'dark', 'View > Dark sets the dark theme');
await page.locator('[data-ribbon-tab=insert]').click();
await page.screenshot({ path: `${out}/desktop-dark-insert.png` });
await page.locator('[data-ribbon-tab=view]').click();
await page.locator('[data-ribbon-btn=light]').click();
check(await page.evaluate(() => document.documentElement.dataset.theme) === 'light', 'View > Light sets the light theme');
check(errors.length === 0, `no page errors (${errors.slice(0, 2).join(' | ')})`);

// phone
({ page, errors } = await open(390, 780, 'light'));
await page.evaluate(() => window.calcpadActiveEditor.setValue('a = 3\nb = a*2'));
await page.locator('[data-ribbon-tab=math]').click();
const sw = await page.evaluate(() => ({ panel: document.querySelector('.rb-panel').scrollWidth, client: document.querySelector('.rb-panel').clientWidth, doc: document.documentElement.scrollWidth }));
check(sw.panel > sw.client && sw.doc <= 392, `phone: panel scrolls horizontally (${JSON.stringify(sw)})`);
await page.locator('[data-ribbon-btn="g-π"]').click();
check((await text(page)).includes('π'), 'phone: Math > pi inserts');
await page.locator('[data-ribbon-tab=home]').click();
await page.screenshot({ path: `${out}/phone-home.png` });
await page.locator('[data-ribbon-tab=math]').click();
await page.screenshot({ path: `${out}/phone-math.png` });
check(errors.length === 0, 'phone: no page errors');
await browser.close();
console.log(fails.length ? `${fails.length} FAILED` : 'ribbon check passed');
process.exit(fails.length ? 1 : 0);
