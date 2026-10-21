// select / radio / checkbox elements mapped to the text boxes of an element (docs/programming.md,
// "Advanced UI with Html and CSS"), through real clicks in the Input tab, on the sheets that use
// them; with before/after screenshots.
//
//   node e2e/select-map-check.mjs [--url http://127.0.0.1:4173] [--out DIR]
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from 'playwright';

const arg = (name, def) => { const i = process.argv.indexOf(name); return i > 0 ? process.argv[i + 1] : def; };
const url = arg('--url', 'http://127.0.0.1:4173');
const out = arg('--out', 'tools/out/select-map');
mkdirSync(out, { recursive: true });
let failed = 0;
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failed++; };

const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1500, height: 1000 }, colorScheme: 'dark' });
await context.addInitScript(() => {
    if (window !== window.top) return;
    if (sessionStorage.getItem('seeded')) return;
    sessionStorage.setItem('seeded', '1');
    localStorage.clear();
});
const page = await context.newPage();
await page.goto(url + '/');
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 30000 });

async function openInput(title) {
    await page.locator('.cprs-file-btn').click();
    await page.locator('.cprs-menu button', { hasText: 'Samples...' }).click();
    await page.locator('.cprs-samples').waitFor({ timeout: 10000 });
    await page.locator('.cprs-s-search').fill(title);
    await page.locator('.cprs-s-item').first().click();
    await page.locator('.cprs-s-open').click();
    await page.waitForTimeout(500);
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click({ timeout: 10000 });
    const dontSave = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
    if (await dontSave.waitFor({ timeout: 1000 }).then(() => true, () => false)) await dontSave.click();
}
async function formFrame(selector, timeout = 30000) {
    for (let t = 0; t < timeout; t += 250) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            if (await f.$(selector).catch(() => null)) return f;
        }
        await page.waitForTimeout(250);
    }
    return null;
}
const boxes = (f, container) => f.$$eval(`#${container} input.calcpad-ui-input`, (els) => els.map((e) => e.value));
// A render replaces the frame, so a call can land on the old one: such a call just tries again.
async function until(fn, ms = 20000) {
    for (let t = 0; t < ms; t += 250) {
        const v = await fn().catch(() => null);
        if (v) return v;
        await page.waitForTimeout(250);
    }
    return null;
}
// The text of every frame that has the element: a render swaps two buffered frames, so the old one can linger.
const allText = async (selector) => {
    const texts = [];
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        if (await f.$(selector).catch(() => null)) texts.push(await f.evaluate(() => document.body.innerText).catch(() => ''));
    }
    return texts;
};
const textMatches = (selector, re) => until(async () => (await allText(selector)).find((t) => re.test(t)) ?? null);

// ---- I-Section Properties: a select with eight values per option -------------------------------
await openInput('I-Section Properties');
let f = await formFrame('select[data-target="section"]');
check(f != null, 'Input tab: the section select is shown');
if (f) {
    await until(async () => (await boxes(f, 'section')).length === 8);
    check(JSON.stringify(await boxes(f, 'section')) === JSON.stringify(['550', '19', '200', '30', '200', '30', '19', '11.9']), 'before: the boxes hold the sample defaults');
    await page.screenshot({ path: join(out, 'i-section-input-before.png') });
    await f.selectOption('select[data-target="section"]', { label: 'IPN 100' });
    const want = ['100', '4.5', '50', '6.8', '50', '6.8', '4.5', '2.7'];
    const filled = await until(async () => {
        const live = await formFrame('select[data-target="section"]', 2000);
        return live && JSON.stringify(await boxes(live, 'section')) === JSON.stringify(want) ? live : null;
    });
    check(filled != null, 'choosing IPN 100 fills the eight boxes of #section with 100; 4.5; 50; 6.8; 50; 6.8; 4.5; 2.7 and the page re-renders');
    if (filled) {
        f = filled;
        check(!!(await until(async () => (await (await formFrame('select[data-target="section"]', 1000)).$eval('select[data-target="section"]', (s) => s.options[s.selectedIndex].text)) === 'IPN 100')), 'the select still shows IPN 100 after the re-render');
        const hw = await textMatches('select[data-target="section"]', /100\s*[−-]\s*6\.8\s*[−-]\s*6\.8\s*=\s*86\.4/);
        check(!!hw, 'dependent results recalculate (h_w = 100 - 6.8 - 6.8 = 86.4)');
        await page.screenshot({ path: join(out, 'i-section-input-after.png') });
        await (await formFrame('select[data-target="rolled"]', 5000)).selectOption('select[data-target="rolled"]', { label: 'Welded' });
        const rolled = await until(async () => {
            const live = await formFrame('select[data-target="rolled"]', 2000);
            return live && (await live.$eval('select[data-target="rolled"]', (s) => s.options[s.selectedIndex].text)) === 'Welded' && (await boxes(live, 'rolled'))[0] === '0';
        });
        check(!!rolled, 'a second select (Rolled/Welded, one box) maps the same way');
        check(!!(await until(async () => (await (await formFrame('select[data-target="section"]', 1000)).$eval('select[data-target="section"]', (s) => s.options[s.selectedIndex].text)) === 'IPN 100')), 'both selections survive the next render');
    }
}
await page.evaluate(() => window.calcpadTabs.close(window.calcpadTabs.activeId));

// ---- the Excel variant: one box holding the row number the sheet reads from the workbook ------------
await openInput('I-Section Properties from an Excel');
f = await formFrame('select[data-target="section"]');
check(f != null, 'Excel catalogue sheet, Input tab: the select is shown');
if (f) {
    await f.selectOption('select[data-target="section"]', { label: 'IPE 120' });
    const ok = await until(async () => {
        const live = await formFrame('select[data-target="section"]', 2000);
        return live && (await boxes(live, 'section'))[0] === '3';
    });
    check(!!ok, 'choosing IPE 120 puts its row number (3) in the box of #section');
    await page.screenshot({ path: join(out, 'i-section-excel-input-after.png') });
    // The results are not part of the form: the Report applies the entered values.
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Report$/ }).click({ timeout: 10000 });
    // Leaving the form asks whether to keep the entered values.
    const keep = page.locator('.modal-backdrop button', { hasText: /^Save$/ });
    if (await keep.waitFor({ timeout: 2000 }).then(() => true, () => false)) await keep.click();
    check(!!(await textMatches('body', /h\s*=\s*120\s*mm/)), 'the Report shows the dimensions read from the workbook for IPE 120 (h = 120 mm)');
    await page.screenshot({ path: join(out, 'i-section-excel-report-after.png') });
}
await page.evaluate(() => window.calcpadTabs.close(window.calcpadTabs.activeId));

// ---- Wind Load: a select that fills a hidden box ------------------------------------------------
await openInput('Wind Load on a Duopitch');
f = await formFrame('select[data-target="C"]', 120000);
check(f != null, 'Wind Load on a Duopitch Building, Input tab: the select is shown');
if (f) {
    const before = await boxes(f, 'C');
    const options = await f.$$eval('select[data-target="C"] option', (os) => os.map((o) => [o.value, o.text]));
    const other = options.find(([v]) => v !== before[0]);
    await f.selectOption('select[data-target="C"]', other[0]);
    const ok = await until(async () => {
        const live = await formFrame('select[data-target="C"]', 2000);
        return live && (await boxes(live, 'C'))[0] === other[0];
    });
    check(!!ok, `choosing "${other[1]}" sets the hidden box of #C to ${other[0]} (was ${before[0]})`);
}
await page.evaluate(() => window.calcpadTabs.close(window.calcpadTabs.activeId));

// ---- radio buttons and a checkbox, from a worksheet written as the documentation shows ----------
await page.evaluate(() => window.calcpadTabs.newUntitled(''));
await page.evaluate(() => window.calcpadActiveEditor.setValue([
    "'<p>Select: <input name=\"t2\" type=\"radio\" id=\"o1\" value=\"1\"/><label for=\"o1\">one</label>",
    "'<input name=\"t2\" type=\"radio\" id=\"o2\" value=\"2\"/><label for=\"o2\">two</label></p>",
    "#UI '<p id=\"t2\">'opt = 1'</p>",
    "'<p><input name=\"t3\" type=\"checkbox\" id=\"c1\" value=\"3\"/><label for=\"c1\">box</label></p>",
    "#UI '<p id=\"t3\">'chk = 0'</p>",
    "r = opt*10 + chk",
].join('\n')));
await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click({ timeout: 10000 });
f = await formFrame('input[name="t2"]');
check(f != null, 'a radio group and a checkbox render in the Input tab');
if (f) {
    await f.evaluate(() => document.querySelector('#o2').click());
    const radio = await until(async () => {
        const live = await formFrame('input[name="t2"]', 2000);
        return live && (await boxes(live, 't2'))[0] === '2' && await live.$eval('#o2', (e) => e.checked) ? live : null;
    });
    check(!!radio, 'radio: choosing "two" puts 2 in the box of #t2 and keeps the choice');
    const live = radio ?? f;
    await live.evaluate(() => document.querySelector('#c1').click());
    const cb = await until(async () => {
        const l = await formFrame('input[name="t3"]', 2000);
        return l && (await boxes(l, 't3'))[0] === '3' && await l.$eval('#c1', (e) => e.checked) ? l : null;
    });
    check(!!cb, 'checkbox: ticking it puts its value (3) in the box of #t3 and keeps it ticked');
    if (cb) {
        check(!!(await textMatches('input[name="t3"]', /2\s*·\s*10\s*\+\s*3\s*=\s*23/)), 'the result follows: r = 2 * 10 + 3 = 23');
        await (await formFrame('input[name="t3"]', 5000)).evaluate(() => document.querySelector('#c1').click());
        const off = await until(async () => {
            const l = await formFrame('input[name="t3"]', 2000);
            return l && (await boxes(l, 't3'))[0] === '0' ? l : null;
        });
        check(!!off, 'unticking it sets the box back to 0');
    }
}
await browser.close();
process.exit(failed ? 1 : 0);
