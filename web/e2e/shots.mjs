// Screenshots of samples in the browser build, for looking at themes and modes.
//
//   node e2e/shots.mjs --url http://127.0.0.1:4173 --theme dark --mode preview --out DIR "Section Properties Excel" ...
//
// Each title (a substring) is opened from the Samples browser; the preview area is saved as
// DIR/<title>-<theme>-<mode>.png. Needs PLAYWRIGHT_BROWSERS_PATH when the default is missing.
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from 'playwright';

const arg = (name, def) => { const i = process.argv.indexOf(name); return i > 0 ? process.argv[i + 1] : def; };
const url = arg('--url', 'http://127.0.0.1:4173');
const theme = arg('--theme', 'dark');
const mode = arg('--mode', 'preview');
const out = arg('--out', 'tools/out/shots');
const previewTheme = arg('--preview-theme', '');
const scroll = Number(arg('--scroll', '0'));
const titles = process.argv.slice(2).filter((a, i, all) => !a.startsWith('--') && !(i > 0 && all[i - 1].startsWith('--')));
mkdirSync(out, { recursive: true });

const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1500, height: 1000 }, colorScheme: theme });
await context.addInitScript((__PT__) => {
    if (window !== window.top) return;
    if (sessionStorage.getItem('shots-seeded')) return;
    sessionStorage.setItem('shots-seeded', '1');
    localStorage.clear();
    if (__PT__) localStorage.setItem('calcpad-preview-theme', __PT__);
}, previewTheme);
const page = await context.newPage();
await page.goto(url + '/');
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 30000 });
for (const title of titles) {
    await page.locator('[data-ribbon-tab=file]').click();
    await page.locator('[data-ribbon-btn=recent]').click();
    await page.locator('.cprs-menu button', { hasText: 'Samples...' }).click();
    await page.locator('.cprs-samples').waitFor({ timeout: 10000 });
    await page.locator('.cprs-s-search').fill(title);
    await page.locator('.cprs-s-item').first().click();
    await page.locator('.cprs-s-open').click();
    await page.waitForTimeout(500);
    const btn = page.locator('.preview-mode-group .toolbar-btn', { hasText: new RegExp(`^${mode[0].toUpperCase()}${mode.slice(1)}$`) });
    if (await btn.count()) await btn.click({ timeout: 10000 }).catch(() => {});
    const dontSave = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
    if (await dontSave.waitFor({ timeout: 1000 }).then(() => true, () => false)) await dontSave.click();
    await page.waitForTimeout(6000);
    if (scroll) for (const f of page.frames()) if (f !== page.mainFrame()) await f.evaluate((y) => window.scrollTo(0, y), scroll).catch(() => {});
    await page.waitForTimeout(500);
    const file = join(out, `${title.replace(/[^A-Za-z0-9]+/g, '-')}-${theme}${previewTheme ? '-preview-' + previewTheme : ''}-${mode}.png`);
    await page.screenshot({ path: file });
    console.log(file);
    await page.evaluate(() => window.calcpadTabs.close(window.calcpadTabs.activeId));
}
await browser.close();
