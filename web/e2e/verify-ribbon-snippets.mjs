// Runs every snippet the ribbon can insert through the engine's command line build and fails on any error.
//   node e2e/verify-ribbon-snippets.mjs [--cli /path/to/Cli] [--library api-snippets.json]
// With --library (the JSON of /api/calcpad/snippets, saved by `ribbon-check.mjs --dump-library`) every snippet of the
// library is run as well, with a context chosen by its category; the ones that error are listed with the engine's message.
// Line snippets run after a small prelude; inline snippets are substituted into their test template.
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { allSnippets, PRELUDE, PRELUDE_NOX, VERIFY_FILES } from '../calcpad-web/src/wasm/ribbon-data.ts';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const cli = arg('--cli', resolve(here, '..', '..', 'target', 'release', 'Cli'));
if (!existsSync(cli)) { console.error(`no CLI at ${cli}`); process.exit(2); }
const dir = mkdtempSync(join(tmpdir(), 'ribbon-snip-'));
// The engine reads its settings from $HOME/Documents/.config/calcpad/Settings.xml: use the browser build's
// (vector plots, complex numbers on) from a throwaway home.
const settings = (complex) => `<?xml version="1.0" encoding="utf-8"?>
<Settings><Math><Decimals>2</Decimals><Degrees>0</Degrees><IsComplex>${complex}</IsComplex><Substitute>true</Substitute><FormatEquations>true</FormatEquations></Math>
<Plot><IsAdaptive>true</IsAdaptive><ScreenScaleFactor>2</ScreenScaleFactor><ImagePath /><ImageUri /><VectorGraphics>true</VectorGraphics><ColorScale>Rainbow</ColorScale><SmoothScale>false</SmoothScale><Shadows>true</Shadows><LightDirection>NorthWest</LightDirection></Plot><Units>m</Units></Settings>`;
const homes = {};
for (const cx of [false, true]) {
    homes[cx] = join(dir, `home-${cx}`);
    mkdirSync(join(homes[cx], 'Documents', '.config', 'calcpad'), { recursive: true });
    writeFileSync(join(homes[cx], 'Documents', '.config', 'calcpad', 'Settings.xml'), settings(cx));
}
for (const [n, t] of Object.entries(VERIFY_FILES)) writeFileSync(join(dir, n), t);

function run(name, src, cx = false) {
    const f = join(dir, `${name}.cpd`);
    writeFileSync(f, src);
    try { execFileSync(cli, [f, join(dir, `${name}.html`)], { cwd: dir, stdio: 'pipe', timeout: 15000, killSignal: 'SIGKILL', env: { ...process.env, HOME: homes[cx] } }); } catch (e) { return [e.code === 'ETIMEDOUT' ? 'timed out after 15 s (an endless loop)' : `CLI failed: ${e.message.split('\n')[0]}`]; }
    const html = readFileSync(join(dir, `${name}.html`), 'utf8');
    const errs = [...html.matchAll(/<p class="err">([\s\S]*?)<\/p>/g)].map(m => m[1].replace(/<[^>]*>/g, '').trim());
    for (const m of html.matchAll(/<span class="err">([^<]*)<\/span>/g)) if (m[1].trim() !== 'text') errs.push(m[1].trim());
    return errs.filter(m => !/Paused/.test(m));
}
// A workbook for the Excel read snippet: written by the engine itself.
run('mkxlsx', 'A = [1; 2| 3; 4]\n#write A to data.xlsx@Sheet1!A1:B2 TYPE=N\n');

let n = 0, skipped = 0;
const bad = [];
for (const s of allSnippets()) {
    const text = s.text.replace(/\{sel\}/g, 'text').replace(/\n$/, '');
    const pre = s.raw ? '' : s.nox ? PRELUDE_NOX : PRELUDE;
    let body;
    if (s.line) body = s.test ? s.test.replace('{}', text) : text;
    else body = (s.test ?? 'r = {}').replace('{}', text);
    const needsExcel = /xlsx/.test(s.text);
    if (s.skip && !needsExcel) { skipped++; console.log(`skip ${s.id}: ${s.skip}`); continue; }
    n++;
    const errs = run(`s${n}`, pre + body + '\n', !!s.cx);
    if (errs.length) bad.push(`${s.id}: ${JSON.stringify(s.text)} -> ${errs[0].slice(0, 120)}`);
}
console.log(`${n} snippets run, ${skipped} skipped, ${bad.length} failed`);
for (const b of bad) console.log('FAIL', b);

// ---- the whole snippet library --------------------------------------------------------------
const libPath = arg('--library', '');
let libFailed = 0;
if (libPath) {
    const lib = JSON.parse(readFileSync(libPath, 'utf8')).snippets;
    // Values for the markers, by parameter name, else by type.
    const BYNAME = { 'f(x)': 'x^2 - 2', 'f(k)': 'k', 'f(x;y)': 'x*y', 'f(x; y)': 'x*y', 'x(t)': 'cos(t)', 'y(t)': 'sin(t)', f1: 'sin(x)', f2: 'cos(x)', expr: 'k', expressions: 'k = k + 1', condition: 'k < 3', x: 'x', k: 'k', t: 't', y: 'y', a: '0', b: '3', c: '1', d: '2', i: '1', j: '1', n: '1', m: '2', i1: '1', i2: '2', j1: '1', j2: '2', ne: '2', p: '2', r: '1' };
    const byType = (t) => (t === 'Integer' ? '1' : t === 'Vector' ? 'u' : t === 'Matrix' ? 'S' : 'x');
    const fill = (sn) => { let i = 0; return sn.insert.replace(/§/g, () => { const p = sn.parameters?.[i++]; const iscmd = /^\$/.test(sn.insert); return (iscmd && BYNAME[p?.name]) || (p?.type === 'Vector' ? 'u' : p?.type === 'Matrix' ? 'S' : p?.type === 'Integer' ? '1' : (BYNAME[p?.name] && /^\$/.test(sn.insert)) || 'x'); }); };
    void byType;
    // Bare keywords are names the editor completes, not statements: give them a body.
    const BARE = { '#if': '#if x > 0\n#end if', '#repeat': '#repeat 2\n#loop', '#for': '#for k = 1 : 2\n#loop', '#while': '#while x < 1\n    x = x + 1\n#loop', '#def': '#def m$ = 1', '#const': '#const c_9 = 1', '#html': '#html\n#end html', '#markdown': '#markdown\n#end markdown', '#cpd': '#cpd\n#end cpd' };
    const WORDS = [[/\bcondition\b/g, 'x > 0'], [/\bexpression\b/g, 'x'], [/\bcount\b/g, '2'], [/\bdigits\b/g, '3'], [/\bspec\b/g, '0.00'], [/\bpath\b/g, '.'], [/\bvalue\b/g, '1'], [/\bname\$/g, 'm$'], [/\bparam\$/g, 'p$'], [/^#include filename$/, '#include module.cpd'], [/\bfilename(?=\.)/g, 'data'], [/\bfilename\b/g, 'data.csv']];
    function build(sn) {
        let t = BARE[sn.insert] ?? fill(sn);
        if (/^#while condition/.test(sn.insert)) t = '#while x < 5\n\tx = x + 1\n#loop';
        for (const [re, to] of WORDS) t = t.replace(re, to);
        if (/^#(read|write|append) M /.test(t)) { t = t.replace(/ M /, ' A '); if (!/^#read/.test(t)) t = t.replace(/ to data/, ' to out'); }
        const cat = sn.category;
        let pre = PRELUDE;
        let body;
        if (/^#(else|end |loop|break|continue)/.test(t)) {
            if (/^#else/.test(t)) body = `#if x > 0\n    x\n${t}\n    x\n#end if`;
            else if (/^#end if/.test(t)) body = `#if x > 0\n    x\n${t}`;
            else if (/^#end def/.test(t)) body = `#def m$\n    x\n${t}`;
            else if (/^#end (hide|show|pre|post|val|equ|noc|varsub|nosub|novar)/.test(t)) body = `#${t.slice(5)}\n${t}`;
            else if (/^#end (html|markdown|cpd)/.test(t)) body = `#${t.slice(5)}\n${t}`;
            else if (/^#loop/.test(t)) body = `#repeat 2\n    x\n${t}`;
            else body = `#repeat 3\n    #if x > 1\n        ${t}\n    #end if\n#loop`;
        } else if (cat === 'Constants') body = /=/.test(t) ? t : `r = ${t}`;
        else if (cat.startsWith('Units/')) body = `r = 2${t}`;
        else if (cat.startsWith('Symbols/')) body = `${t} = 1`;
        else if (cat === 'Operators') body = t === '!' ? 'r = n!' : t === '=' ? 'r = 1' : t === '←' ? `#if x > 0\n    r ← 1\n#end if` : t === '∠' ? 'r = 5∠30' : `r = x ${t} y`;
        else if (cat.startsWith('Functions/')) body = `r = ${t}`;
        else if (cat === 'Format Strings') body = `r = x${t}`;
        else if (cat === 'SVG') body = `'<svg width="400" height="300">\n'${t}\n'</svg>`;
        else if (cat === 'Markdown') body = `#md on\n${t}`;
        else body = t;
        if (sn.insert.startsWith('$Plot') || sn.insert.startsWith('$Map') || /\$(Root|Find|Sup|Inf|Area|Integral|Slope)/.test(sn.insert)) pre = PRELUDE_NOX;
        if (/^#(deg|rad|gra|round|format|md|phasor|complex|settings)/.test(t) || cat === 'Constants' && /=/.test(t)) pre = PRELUDE;
        // Domain-limited functions need an argument inside their domain: |x| > 1, or x inside the data of line and spline.
        if (/^(acsc|asec|acosh|acoth)\(/.test(sn.insert)) pre = PRELUDE.replace('x = 0.5', 'x = 3');
        if (/^(line|spline)\(/.test(sn.insert)) pre = PRELUDE.replace('x = 0.5', 'x = 1.5');
        return { pre, body, cx: /\b(phase|conj|re|im)\(|∠|#phasor|#complex/.test(t) };
    }
    let ok = 0;
    const errs = [];
    let k = 0;
    for (const sn of lib) {
        k++;
        const { pre, body, cx } = build(sn);
        const e = run(`lib${k}`, pre + body + '\n', cx);
        if (e.length) errs.push({ cat: sn.category, insert: sn.insert, msg: e[0].replace(/^Error in "[\s\S]*?" on line \[\d*\]:?\s*/, '').slice(0, 110) }); else ok++;
    }
    libFailed = errs.length;
    console.log(`library: ${lib.length} snippets run, ${ok} clean, ${errs.length} errored`);
    const by = new Map();
    for (const e of errs) { const key = e.msg || '(no message)'; if (!by.has(key)) by.set(key, []); by.get(key).push(e); }
    for (const [msg, list] of [...by].sort((a, b) => b[1].length - a[1].length)) {
        console.log(`  ${list.length} x ${msg}`);
        for (const e of list.slice(0, 40)) console.log(`      ${e.cat} :: ${JSON.stringify(e.insert).slice(0, 80)}`);
    }
}
process.exit(bad.length ? 1 : 0);
