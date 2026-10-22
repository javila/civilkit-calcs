// Screenshot: the ribbon's Math tab with a calculated worksheet, light theme.
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/shot-ribbon-math.mjs [--url U] [--out FILE]
import { chromium } from 'playwright';
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4173/');
const out = arg('--out', 'ribbon-math-light.png');
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1400, height: 820 }, colorScheme: 'light' });
const page = await ctx.newPage();
await page.goto(url);
await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
await page.evaluate(() => window.calcpadActiveEditor.setValue("'Cantilever check\nL = 4\nF = 10\nM = F*L\nV = F\n$Plot{F*(L - x) @ x = 0 : L}"));
await page.locator('[data-ribbon-tab=math]').click();
await page.waitForTimeout(6000);
await page.screenshot({ path: out });
await browser.close();
