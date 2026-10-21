// Records the external requests each upstream example in a list makes in the app, to find the
// files a library loads beyond its main script (for tools/gen_library_defaults.mjs). Run it on
// a build without the script policy to see everything an example asks for.
//   node e2e/record-urls.mjs <list of example paths, one per line> <output.json>
import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFileSync, existsSync, statSync, writeFileSync } from 'node:fs';
import { join, extname, normalize } from 'node:path';
const dist = new URL('../calcpad-web/dist/', import.meta.url).pathname;
const ref = new URL('../../../calcpad-ref/Examples/', import.meta.url).pathname;
const T = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.woff2': 'font/woff2', '.otf': 'font/otf' };
const srv = createServer((q, r) => { let p = decodeURIComponent(new URL(q.url, 'http://x').pathname); if (p.endsWith('/')) p += 'index.html'; const f = normalize(join(dist, p)); if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { r.writeHead(404); return r.end(); } r.writeHead(200, { 'Content-Type': T[extname(f)] ?? 'application/octet-stream' }); r.end(readFileSync(f)); });
await new Promise(ok => srv.listen(4193, '127.0.0.1', ok));
const files = readFileSync(process.argv[2], 'utf8').trim().split('\n').map(l => l.replace(/^\.\//, ''));
const b = await chromium.launch(); const ctx = await b.newContext();
const page = await ctx.newPage();
await page.goto('http://127.0.0.1:4193/');
await page.waitForFunction(() => window.calcpadActiveEditor != null);
const result = {};
for (const f of files) {
  const seen = new Map();
  const on = r => { const u = r.url(); if (!u.startsWith('http://127.0.0.1') && !u.startsWith('data:') && !u.startsWith('blob:')) seen.set(u, r.resourceType()); };
  ctx.on('request', on);
  await page.evaluate(t => window.calcpadActiveEditor.setValue(t), readFileSync(ref + f, 'utf8'));
  await page.waitForTimeout(9000);
  ctx.off('request', on);
  result[f] = Object.fromEntries(seen);
  console.log(f, seen.size);
}
writeFileSync(process.argv[3], JSON.stringify(result, null, 1));
await b.close(); srv.close();
