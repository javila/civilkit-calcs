// A screenshot of the built app (dist), for looking at a change:
//   node e2e/shot.mjs <out.png> <light|dark> [worksheet text] [menu]
// "menu" opens the File menu before the shot.
import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFileSync, existsSync, statSync } from 'node:fs';
import { join, extname, normalize } from 'node:path';
const dist = new URL('../calcpad-web/dist/', import.meta.url).pathname;
const T = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.woff2': 'font/woff2', '.otf': 'font/otf' };
const srv = createServer((q, r) => { let p = decodeURIComponent(new URL(q.url, 'http://x').pathname); if (p.endsWith('/')) p += 'index.html'; const f = normalize(join(dist, p)); if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { r.writeHead(404); return r.end(); } r.writeHead(200, { 'Content-Type': T[extname(f)] ?? 'application/octet-stream' }); r.end(readFileSync(f)); });
await new Promise(ok => srv.listen(4195, '127.0.0.1', ok));
const b = await chromium.launch(); const page = await b.newPage({ viewport: { width: 1400, height: 820 }, colorScheme: process.argv[3] === 'dark' ? 'dark' : 'light' });
await page.goto('http://127.0.0.1:4195/');
await page.waitForFunction(() => window.calcpadActiveEditor != null);
if (process.argv[4]) { await page.evaluate(t => window.calcpadActiveEditor.setValue(t), process.argv[4]); await page.waitForTimeout(3000); }
if (process.argv[5] === 'menu') { await page.locator('.cprs-file-btn').click(); await page.waitForTimeout(300); }
await page.screenshot({ path: process.argv[2] });
await b.close(); srv.close();
