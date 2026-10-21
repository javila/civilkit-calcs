// Builds web/calcpad-web/src/wasm/library-defaults.json: the script libraries the upstream
// CalcpadCE examples load, each file with its exact CDN URL and an SRI hash (sha384) computed
// from the file as served now. Re-run after changing the list; it needs the network.
//
//   node tools/gen_library_defaults.mjs
//
// The URLs are the ones the examples use (calcpad-ref/Examples). The extra dxf-viewer files
// are the modules its jsDelivr "+esm" bundle imports, recorded by loading the examples.
import { createHash } from 'node:crypto';
import { writeFileSync } from 'node:fs';

const JSD = 'https://cdn.jsdelivr.net/npm';
const LIBRARIES = [
    { id: 'jquery', name: 'jQuery', version: '3.6.3', files: [{ url: 'https://code.jquery.com/jquery-3.6.3.min.js' }] },
    { id: 'd3', name: 'D3', version: '3.5.17', files: [{ url: 'https://cdnjs.cloudflare.com/ajax/libs/d3/3.5.17/d3.min.js' }] },
    { id: 'plotly', name: 'Plotly', version: '2.26.0', files: [{ url: 'https://cdn.plot.ly/plotly-2.26.0.min.js' }] },
    {
        id: 'three-classic', name: 'three.js with OrbitControls', version: '0.145',
        files: [{ url: `${JSD}/three@0.145/build/three.min.js` }, { url: `${JSD}/three@0.145/examples/js/controls/OrbitControls.js` }],
    },
    { id: 'three-esm', name: 'three.js (ES module)', version: '0.160.0', files: [{ url: `${JSD}/three@0.160.0/+esm`, module: true }] },
    {
        id: 'dxf-viewer', name: 'dxf-viewer (ES module)', version: '1.0.46',
        files: [
            { url: `${JSD}/dxf-viewer@1.0.46/+esm`, module: true },
            { url: `${JSD}/earcut@3.0.2/+esm`, module: true },
            { url: `${JSD}/loglevel@1.9.2/+esm`, module: true },
            { url: `${JSD}/opentype.js@1.3.4/+esm`, module: true },
            { url: `${JSD}/three@0.161.0/+esm`, module: true },
            { url: `${JSD}/three@0.161.0/src/extras/ShapeUtils.js/+esm`, module: true },
            { url: `${JSD}/three@0.161.0/src/extras/core/ShapePath.js/+esm`, module: true },
        ],
    },
    { id: 'dxf-writer', name: 'dxf-writer (ES module)', version: '1.18.4', files: [{ url: `${JSD}/dxf-writer@1.18.4/+esm`, module: true }] },
    {
        id: 'model-viewer', name: 'model-viewer', version: '3.0.1',
        files: [{ url: 'https://ajax.googleapis.com/ajax/libs/model-viewer/3.0.1/model-viewer.min.js', module: true }],
    },
];

const out = [];
for (const lib of LIBRARIES) {
    const files = [];
    for (const f of lib.files) {
        const res = await fetch(f.url);
        if (!res.ok) throw new Error(`${f.url}: HTTP ${res.status}`);
        const body = Buffer.from(await res.arrayBuffer());
        const integrity = 'sha384-' + createHash('sha384').update(body).digest('base64');
        files.push({ url: f.url, integrity, size: body.length, ...(f.module ? { module: true } : {}) });
        console.log(`${integrity}  ${body.length}  ${f.url}`);
    }
    out.push({ ...lib, files, builtin: true });
}
const target = new URL('../web/calcpad-web/src/wasm/library-defaults.json', import.meta.url);
writeFileSync(target, JSON.stringify(out, null, 2) + '\n');
console.log(`wrote ${target.pathname}`);
