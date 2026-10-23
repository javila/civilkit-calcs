/// <reference lib="webworker" />
// calcpad-rs: runs the Python of a worksheet's `#python` blocks on MicroPython, off the main thread.
//
//   in   { type: 'start', token, base }                       a fresh interpreter (one per calculation)
//        { type: 'run', token, id, source, first }            one block, in the interpreter's session
//   out  { type: 'ready', token } after 'start'
//        { type: 'result', token, id, result: BlockResult }
//        { type: 'boot-error', token, error }
//
// The runtime and the stand-ins load on the first 'start' (nothing before that): micropython.mjs and .wasm and the
// files listed in py/files.json, all from the app's own origin. This worker never talks to the network again after
// that load: its fetch is replaced by one that serves the wasm bytes it already holds, the other ways out (XMLHttpRequest,
// WebSocket, importScripts, IndexedDB and the rest) are removed, and the interpreter's own `js` and `jsffi` modules are
// refused by an import guard (public/py/lib/calcpad_py.py). A run that does not finish is killed by the page.
// Sandbox limits are in docs/python-blocks.md.

import { renderFigure } from './python-chart';
import { readTraceback, type BlockResult } from './python-blocks';

type StartMsg = { type: 'start'; token: string; base: string };
type RunMsg = { type: 'run'; token: string; id: number; source: string; first: number };

const OUTPUT_MAX = 100_000;      // characters of print output kept per worksheet run
const FIGURES_MAX = 20;          // figures per worksheet run
const SVG_MAX = 3_000_000;       // characters of SVG per worksheet run
const EXPORTS_MAX = 500;         // exported names per worksheet run
const HEAP = 32 * 1024 * 1024;   // the interpreter's heap

interface Loaded {
    loadMicroPython: (o: Record<string, unknown>) => Promise<any>;
    wasm: Uint8Array;
    wasmUrl: string;
    files: [string, string][];
}

let loaded: Promise<Loaded> | null = null;
let session: { mp: any; state: SessionState } | null = null;

interface SessionState {
    parts: BlockResult['parts'];
    exports: BlockResult['exports'];
    truncated: boolean;
    chars: number;
    svgChars: number;
    figures: number;
    hostError: string | null;
}

const send = self.postMessage.bind(self);

function load(base: string): Promise<Loaded> {
    if (!loaded) {
        loaded = (async () => {
            const realFetch = self.fetch.bind(self);
            const wasmUrl = new URL(`${base}py/micropython/micropython.wasm`, self.location.href).href;
            const [mod, wasmBuf, list] = await Promise.all([
                import(/* @vite-ignore */ `${base}py/micropython/micropython.mjs`),
                realFetch(wasmUrl).then((r) => { if (!r.ok) throw new Error(`micropython.wasm: ${r.status}`); return r.arrayBuffer(); }),
                realFetch(`${base}py/files.json`).then((r) => { if (!r.ok) throw new Error(`files.json: ${r.status}`); return r.json() as Promise<string[]>; }),
            ]);
            const files = await Promise.all(list.map(async (f): Promise<[string, string]> => {
                const r = await realFetch(`${base}py/lib/${f}`);
                if (!r.ok) throw new Error(`${f}: ${r.status}`);
                return [f, await r.text()];
            }));
            const wasm = new Uint8Array(wasmBuf);
            // From here on the only thing this worker can fetch is the interpreter's own wasm, from memory.
            Object.defineProperty(self, 'fetch', {
                configurable: true, writable: true,
                value: (input: unknown) => {
                    const url = String((input as { url?: string })?.url ?? input);
                    if (url === wasmUrl || url.endsWith('micropython.wasm')) {
                        return Promise.resolve(new Response(wasm.slice(), { headers: { 'Content-Type': 'application/wasm' } }));
                    }
                    return Promise.reject(new TypeError('network access is not available to a worksheet script'));
                },
            });
            return { loadMicroPython: mod.loadMicroPython, wasm, wasmUrl, files };
        })();
        loaded.catch(() => { loaded = null; });
    }
    return loaded;
}

/** Removes the ways a script could reach the network or storage if it ever got hold of the global object. */
function lockDown(): void {
    const g = self as unknown as Record<string, unknown>;
    for (const k of ['XMLHttpRequest', 'WebSocket', 'EventSource', 'indexedDB', 'caches', 'BroadcastChannel',
        'Worker', 'SharedWorker', 'WebTransport', 'RTCPeerConnection']) {
        try { Object.defineProperty(g, k, { configurable: true, writable: true, value: undefined }); } catch { /* not removable */ }
    }
    // Emscripten's loader tells a worker from a shell by `typeof importScripts == "function"`, and a new interpreter starts
    // for every calculation, so the name stays and the function refuses.
    try {
        Object.defineProperty(g, 'importScripts', { configurable: true, writable: true, value: () => { throw new Error('not available to a worksheet script'); } });
    } catch { /* not replaceable */ }
}

async function start(msg: StartMsg): Promise<void> {
    const l = await load(msg.base);
    const state: SessionState = { parts: [], exports: [], truncated: false, chars: 0, svgChars: 0, figures: 0, hostError: null };
    let pend: number[] = [];
    const dec = new TextDecoder();
    const text = (s: string) => {
        if (!s || state.truncated) return;
        if (state.chars + s.length > OUTPUT_MAX) { s = s.slice(0, OUTPUT_MAX - state.chars); state.truncated = true; }
        state.chars += s.length;
        const last = state.parts[state.parts.length - 1];
        if (last && 'text' in last) last.text += s; else state.parts.push({ text: s });
    };
    const flush = () => { if (pend.length) { text(dec.decode(Uint8Array.from(pend))); pend = []; } };
    const write = (bytes: Uint8Array) => {
        if (state.truncated) return;
        for (const c of bytes) { pend.push(c); if (c === 10) flush(); }
        if (pend.length > 65536) flush();
    };
    const host = {
        figure(specText: string) {
            flush();
            if (state.figures >= FIGURES_MAX) { state.truncated = true; return; }
            try {
                const svg = renderFigure(String(specText).replace(/-?\bInfinity\b/g, 'null').replace(/\bNaN\b/g, 'null'));
                state.svgChars += svg.length;
                if (state.svgChars > SVG_MAX) { state.truncated = true; return; }
                state.figures++;
                state.parts.push({ svg });
            } catch (e) {
                state.hostError ??= `figure could not be drawn: ${e instanceof Error ? e.message : String(e)}`;
            }
        },
        export(t: string) {
            if (state.exports.length >= EXPORTS_MAX) { state.hostError ??= `a worksheet may export at most ${EXPORTS_MAX} names`; return; }
            try { state.exports.push(JSON.parse(String(t))); } catch { state.hostError ??= 'export: unreadable value'; }
        },
    };
    const mp = await l.loadMicroPython({ heapsize: HEAP, stdout: write, stderr: write, linebuffer: false, url: l.wasmUrl });
    mp.registerJsModule('_cp_raw', host);
    const made = new Set<string>();
    const mkdirs = (dir: string) => {
        let p = '';
        for (const part of dir.split('/').filter(Boolean)) {
            p += `/${part}`;
            if (made.has(p)) continue;
            made.add(p);
            try { mp.FS.mkdir(p); } catch { /* exists */ }
        }
    };
    for (const [path, src] of l.files) {
        const full = `/lib/${path}`;
        mkdirs(full.slice(0, full.lastIndexOf('/')));
        mp.FS.writeFile(full, src);
    }
    mp.runPython('import _cp_raw\nimport calcpad_py\ncalcpad_py.install(_cp_raw)\n_cp_run = calcpad_py.run\ndel _cp_raw\ndel calcpad_py\n');
    (state as SessionState & { flush?: () => void }).flush = flush;
    session = { mp, state };
    lockDown();
}

function run(msg: RunMsg): BlockResult {
    if (!session) throw new Error('no interpreter');
    const { mp, state } = session;
    state.parts = [];
    state.exports = [];
    state.hostError = null;
    const before = state.truncated;
    let error: BlockResult['error'] = null;
    mp.globals.set('_cp_src', '\n'.repeat(msg.first) + msg.source);
    try {
        mp.runPython('_cp_run(_cp_src)');
    } catch (e) {
        const t = readTraceback(String((e as Error)?.message ?? e));
        error = { type: t.type, message: t.message, line: t.line };
    }
    (state as SessionState & { flush?: () => void }).flush?.();
    if (!error && state.hostError) error = { type: 'Error', message: state.hostError, line: null };
    return { parts: state.parts, exports: state.exports, error, truncated: state.truncated && !before ? true : state.truncated };
}

self.onmessage = async (e: MessageEvent<StartMsg | RunMsg>) => {
    const m = e.data;
    try {
        if (m.type === 'start') {
            await start(m);
            send({ type: 'ready', token: m.token });
        } else if (m.type === 'run') {
            send({ type: 'result', token: m.token, id: m.id, result: run(m) });
        }
    } catch (err) {
        send({ type: 'boot-error', token: m.token, id: (m as RunMsg).id, error: err instanceof Error ? err.message : String(err) });
    }
};
