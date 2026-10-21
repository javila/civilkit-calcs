// calcpad-rs: answers the editor's Calcpad.Server requests with the calcpad-wasm engine.
//
// Upstream calcpad-web talks to Calcpad.Server over HTTP (calcpad-frontend's
// CalcpadApiClient and a few direct fetch calls). This adapter replaces that transport:
// requests to `<origin>/api/calcpad/*` are answered in the page, everything else goes to
// the network as before. No calculation request leaves the page.
//
//   convert, convert?unwrap=true, health   the calcpad-wasm engine (in a worker)
//   lint, highlight, definitions,           the calcpad-highlighter-wasm module (a second wasm,
//   symbol-at-position, prettify,           loaded in its own worker on first use)
//   snippets
//   docx                                    calcpad-docx-wasm (Word export, loaded on demand)
//   xlsx                                    calcpad-rs's own: the variables as an Excel workbook
//   cpdz, portable, log-level, pdf          501: these need Calcpad.Server. PDF is printed by
//                                           the browser instead (message-bridge.ts)

import { reportHeadMarkup } from './report-fonts';
import { uiAssetsMarkup } from './ui-assets';
import { libraries } from './library-list';
import { applyScriptPolicy } from './script-policy';

const API_PREFIX = '/api/calcpad/';
/** calcpad-wasm's UI_ASSETS_MARKER: where the datagrid libraries go. */
const UI_ASSETS_MARKER = '<!--calcpad-ui-assets-->';

/** Endpoints answered by calcpad-highlighter-wasm, with the method each one calls. */
const HIGHLIGHTER_METHODS: Record<string, string> = {
    lint: 'lint',
    highlight: 'highlight',
    definitions: 'definitions',
    'symbol-at-position': 'symbol_at_position',
    prettify: 'prettify',
};

const NEEDS_SERVER: Record<string, string> = {
    pdf: 'Server PDF export',
    'pdf/health': 'Server PDF export',
    'pdf/browser': 'Server PDF export',
    'pdf/browser/install': 'Server PDF export',
    'cpdz/decode': 'Compiled .cpdz worksheets',
    'cpdz/encode': 'Compiled .cpdz worksheets',
    'portable/bundle': 'Portable worksheets',
    'portable/package': 'Portable worksheets',
    'log-level': 'Server logging',
};

/**
 * In the dark theme, $Plot/$Map SVGs (transparent, with dark grey text and grid) sit on a white
 * "paper" card, as they print and as upstream's PNG plots look. Screen only: print is unchanged,
 * and the light theme has no rule.
 */
const PLOT_PAPER = '<style>@media screen{.dark-theme svg.plot,.dark-theme img.plot{background:#fff;padding:6pt;border-radius:6px;box-sizing:content-box}}</style>';

function addPlotPaper(html: string): string {
    const i = html.toLowerCase().lastIndexOf('</head>');
    return i < 0 ? html : html.slice(0, i) + PLOT_PAPER + html.slice(i);
}

/** A file the engine can read (#include, #read): an absolute path and its bytes. */
export type EngineFile = { path: string; data: Uint8Array };

type Job = {
    id: number;
    msg: Record<string, unknown>;
    resolve: (v: any) => void;
    reject: (e: unknown) => void;
};

/**
 * One worker, one job at a time, so a superseded request can be dropped without touching the
 * others: a queued job is just removed, and the running one (which cannot be interrupted
 * inside the engine) costs a fresh worker, after which the queue carries on.
 */
class EngineWorker {
    private worker: Worker | null = null;
    private seq = 0;
    private queue: Job[] = [];
    private running: Job | null = null;
    private files: EngineFile[] = [];
    private head = '';

    constructor(private readonly base: string) {}

    private start(): Worker {
        if (this.worker) return this.worker;
        const w = new Worker(new URL('./engine-worker.ts', import.meta.url), { type: 'module' });
        w.onmessage = (e: MessageEvent<{ id: number; result?: string | Uint8Array; error?: string }>) => {
            const job = this.running;
            if (!job || job.id !== e.data.id) return;
            this.running = null;
            if (e.data.error !== undefined) job.reject(new Error(e.data.error));
            else job.resolve(e.data.result ?? '');
            this.pump();
        };
        w.onerror = (e: ErrorEvent) => {
            const job = this.running;
            this.running = null;
            this.replaceWorker();
            job?.reject(new Error(`calcpad-wasm worker failed: ${e.message || 'could not load'}`));
            this.pump();
        };
        this.worker = w;
        // A fresh worker has neither the report head nor the library yet (id 0: no reply wanted).
        w.postMessage({ id: 0, base: this.base, kind: 'head', markup: this.head });
        w.postMessage({ id: 0, base: this.base, kind: 'files', files: this.files });
        return w;
    }

    private replaceWorker(): void {
        this.worker?.terminate();
        this.worker = null;
    }

    private pump(): void {
        if (this.running || this.queue.length === 0) return;
        const job = this.queue.shift()!;
        this.running = job;
        this.start().postMessage({ id: job.id, base: this.base, ...job.msg });
    }

    private post<T = string>(msg: Record<string, unknown>, signal?: AbortSignal | null): Promise<T> {
        return new Promise<T>((resolve, reject) => {
            const job: Job = { id: ++this.seq, msg, resolve, reject };
            if (signal?.aborted) {
                reject(signal.reason ?? new DOMException('Aborted', 'AbortError'));
                return;
            }
            signal?.addEventListener('abort', () => {
                const reason = signal.reason ?? new DOMException('Aborted', 'AbortError');
                // The API client gives each request a 60 s timeout, meant for a server. A render in
                // the page has no network to wait on, and a heavy sheet (Soil Stress and
                // Deformations takes about a minute) must be allowed to finish; a newer request or
                // an edit still supersedes it.
                if ((reason as { name?: string })?.name === 'TimeoutError') return;
                if (this.running === job) {
                    this.running = null;
                    this.replaceWorker();
                    reject(reason);
                    this.pump();
                } else {
                    const i = this.queue.indexOf(job);
                    if (i >= 0) {
                        this.queue.splice(i, 1);
                        reject(reason);
                    }
                }
            }, { once: true });
            this.queue.push(job);
            this.pump();
        });
    }

    convert(body: string, unwrap: boolean, signal?: AbortSignal | null): Promise<string> {
        return this.post({ kind: 'convert', body, unwrap }, signal);
    }

    /** The Word document of a rendered page (calcpad-docx-wasm, loaded on first use). */
    docx(html: string, expressions: string[], signal?: AbortSignal | null): Promise<Uint8Array> {
        return this.post<Uint8Array>({ kind: 'docx', html, expressions: JSON.stringify(expressions) }, signal);
    }

    setHead(markup: string): void {
        if (markup === this.head) return;
        this.head = markup;
        if (this.worker) void this.post({ kind: 'head', markup }).catch(() => {});
    }

    setFiles(files: EngineFile[]): Promise<string> {
        this.files = files;
        return this.post({ kind: 'files', files });
    }
}

/**
 * The worker of the highlighter wasm. Requests run one at a time in arrival order; a request whose
 * caller gave up is not interrupted (the answers are quick), its result is just dropped.
 */
class HighlighterWorker {
    private worker: Worker | null = null;
    private seq = 0;
    private pending = new Map<number, { resolve: (v: string) => void; reject: (e: unknown) => void }>();
    private files: EngineFile[] = [];

    constructor(private readonly base: string) {}

    private start(): Worker {
        if (this.worker) return this.worker;
        const w = new Worker(new URL('./highlighter-worker.ts', import.meta.url), { type: 'module' });
        w.onmessage = (e: MessageEvent<{ id: number; result?: string; error?: string }>) => {
            const p = this.pending.get(e.data.id);
            if (!p) return;
            this.pending.delete(e.data.id);
            if (e.data.error !== undefined) p.reject(new Error(e.data.error));
            else p.resolve(e.data.result ?? '');
        };
        w.onerror = (e: ErrorEvent) => {
            const err = new Error(`calcpad-highlighter worker failed: ${e.message || 'could not load'}`);
            for (const p of this.pending.values()) p.reject(err);
            this.pending.clear();
            this.worker?.terminate();
            this.worker = null;
        };
        this.worker = w;
        w.postMessage({ id: 0, base: this.base, kind: 'files', files: this.files });
        return w;
    }

    private post(msg: Record<string, unknown>, signal?: AbortSignal | null): Promise<string> {
        return new Promise<string>((resolve, reject) => {
            if (signal?.aborted) {
                reject(signal.reason ?? new DOMException('Aborted', 'AbortError'));
                return;
            }
            const id = ++this.seq;
            this.pending.set(id, { resolve, reject });
            signal?.addEventListener('abort', () => {
                if (this.pending.delete(id)) reject(signal.reason ?? new DOMException('Aborted', 'AbortError'));
            }, { once: true });
            this.start().postMessage({ id, base: this.base, ...msg });
        });
    }

    call(method: string, body: string, signal?: AbortSignal | null): Promise<string> {
        return this.post({ kind: 'call', method, body }, signal);
    }

    snippets(category: string, signal?: AbortSignal | null): Promise<string> {
        return this.post({ kind: 'snippets', category }, signal);
    }

    setFiles(files: EngineFile[]): void {
        this.files = files;
        // Only tell a running worker; a worker started later receives the files when it starts.
        if (this.worker) void this.post({ kind: 'files', files }).catch(() => {});
    }
}

function notAvailable(what: string, reason: string): Response {
    return new Response(`${what} is not available in the browser build of CalcpadCE (calcpad-rs): ${reason}.`, {
        status: 501,
        headers: { 'Content-Type': 'text/plain; charset=utf-8' },
    });
}

function fromBase64(s: string): Uint8Array {
    const bin = atob(s);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
}

export interface WasmTransport {
    /** Replaces the files #include and #read can see. */
    setFiles(files: EngineFile[]): Promise<void>;
    /** Called with the files a render's #write/#append produced. */
    onFilesWritten: ((files: EngineFile[]) => void) | null;
    /** A library file by its path ("/Images/a.png"), for the images a worksheet shows. */
    resolveFile: ((path: string) => Uint8Array | undefined) | null;
}

const IMAGE_TYPES: Record<string, string> = {
    png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', gif: 'image/gif', svg: 'image/svg+xml', webp: 'image/webp', bmp: 'image/bmp',
};

/** The library path of a relative reference, rooted at "/" as the engine resolves it. */
function libraryKey(ref: string): string {
    const parts: string[] = [];
    for (const p of ref.replace(/\\/g, '/').split('/')) {
        if (p === '' || p === '.') continue;
        if (p === '..') parts.pop(); else parts.push(p);
    }
    return '/' + parts.join('/');
}

/** Relative <img src> that names a library file becomes a data: URL of that file. */
function libraryImages(html: string, resolve: (path: string) => Uint8Array | undefined): string {
    return html.replace(/(<img\b[^>]*?\bsrc\s*=\s*)(["'])([^"']+)\2/gi, (m, pre: string, q: string, src: string) => {
        if (/^[a-z][a-z0-9+.-]*:|^\/\//i.test(src)) return m;
        const type = IMAGE_TYPES[src.split('?')[0].split('.').pop()?.toLowerCase() ?? ''];
        let name = src;
        try { name = decodeURI(src); } catch { /* a malformed escape: use the text as written */ }
        const data = type ? resolve(libraryKey(name)) : undefined;
        if (!data) return m;
        let bin = '';
        for (let i = 0; i < data.length; i += 0x8000) bin += String.fromCharCode(...data.subarray(i, i + 0x8000));
        return `${pre}${q}data:${type};base64,${btoa(bin)}${q}`;
    });
}

/** Installs the adapter. Call once, before the editor makes its first request. */
export function installWasmTransport(): WasmTransport {
    // Absolute, so the worker (whose own URL is under assets/) resolves the engine correctly
    // when the app is served from a sub folder.
    const base = new URL(import.meta.env.BASE_URL || './', document.baseURI).href;
    const engine = new EngineWorker(base);
    const highlighter = new HighlighterWorker(base);
    const nativeFetch = window.fetch.bind(window);
    const head = reportHeadMarkup(base).then(m => engine.setHead(m));
    const transport: WasmTransport = {
        async setFiles(files) {
            highlighter.setFiles(files);
            await engine.setFiles(files);
        },
        onFilesWritten: null,
        resolveFile: null,
    };

    const handle = async (url: URL, method: string, body: string | null, signal?: AbortSignal | null): Promise<Response> => {
        const endpoint = url.pathname.slice(API_PREFIX.length).replace(/\/+$/, '');
        if (endpoint === 'health') return Response.json({ status: 'ok' });
        if (endpoint === 'snippets' && method === 'GET') {
            const out = await highlighter.snippets(url.searchParams.get('category') ?? '', signal);
            return new Response(out, { status: 200, headers: { 'Content-Type': 'application/json; charset=utf-8' } });
        }
        if (endpoint in HIGHLIGHTER_METHODS && method === 'POST') {
            if (body == null || body === '') return new Response('Invalid JSON', { status: 400 });
            const out = await highlighter.call(HIGHLIGHTER_METHODS[endpoint], body, signal);
            // Where the server answers a status, the module answers {"status": n, "error": "..."}.
            if (out.startsWith('{"status":')) {
                const err = JSON.parse(out) as { status?: number; error?: string };
                if (typeof err.status === 'number' && typeof err.error === 'string') {
                    return new Response(err.error, { status: err.status });
                }
            }
            return new Response(out, { status: 200, headers: { 'Content-Type': 'application/json; charset=utf-8' } });
        }
        if (endpoint === 'convert' && method === 'POST') {
            if (body == null || body === '') return new Response('Invalid JSON', { status: 400 });
            await head;
            const unwrap = url.searchParams.get('unwrap') === 'true';
            const out = JSON.parse(await engine.convert(body, unwrap, signal)) as {
                html?: string; errors?: unknown[]; status?: number; error?: string;
                files?: { path: string; data: string }[];
            };
            if (out.status) return new Response(out.error ?? '', { status: out.status });
            let html = out.html ?? '';
            if (html.includes(UI_ASSETS_MARKER)) html = html.replace(UI_ASSETS_MARKER, await uiAssetsMarkup(base));
            if (transport.resolveFile) html = libraryImages(html, transport.resolveFile);
            html = addPlotPaper(html);
            // Worksheet scripts: only listed library files, checked by hash; no network access.
            html = applyScriptPolicy(html, libraries(), window.location.origin);
            if (out.files?.length && transport.onFilesWritten) {
                transport.onFilesWritten(out.files.map(f => ({ path: f.path, data: fromBase64(f.data) })));
            }
            return new Response(html, {
                status: 200,
                headers: {
                    'Content-Type': 'text/html; charset=utf-8',
                    'X-Calcpad-Errors': encodeURIComponent(JSON.stringify(out.errors ?? [])),
                },
            });
        }
        if (endpoint === 'docx' && method === 'POST') {
            // Calcpad.Server's /docx: the page rendered with its equations as Office Math, then
            // the Word writer (calcpad-docx). Images from the library go in as data: URLs.
            if (body == null || body === '') return new Response('Invalid JSON', { status: 400 });
            await head;
            const request = JSON.parse(body) as Record<string, unknown>;
            request.captureOpenXml = true;
            request.enableUi = false;
            request.includeLineAnchors = false;
            const out = JSON.parse(await engine.convert(JSON.stringify(request), false, signal)) as {
                html?: string; status?: number; error?: string; openXmlExpressions?: string[];
                files?: { path: string; data: string }[];
            };
            if (out.status) return new Response(out.error ?? '', { status: out.status });
            let html = out.html ?? '';
            if (transport.resolveFile) html = libraryImages(html, transport.resolveFile);
            if (out.files?.length && transport.onFilesWritten) {
                transport.onFilesWritten(out.files.map(f => ({ path: f.path, data: fromBase64(f.data) })));
            }
            const bytes = await engine.docx(html, out.openXmlExpressions ?? [], signal);
            return new Response(new Uint8Array(bytes), {
                status: 200,
                headers: { 'Content-Type': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document' },
            });
        }
        if (endpoint === 'xlsx' && method === 'POST') {
            // calcpad-rs's own export: every variable with its value, vectors and matrices on
            // their own sheets, and the input fields (openxml::variables_export).
            if (body == null || body === '') return new Response('Invalid JSON', { status: 400 });
            const request = JSON.parse(body) as Record<string, unknown>;
            request.exportXlsx = true;
            request.enableUi = false;
            request.includeLineAnchors = false;
            const out = JSON.parse(await engine.convert(JSON.stringify(request), false, signal)) as {
                status?: number; error?: string; xlsx?: string; xlsxError?: string;
            };
            if (out.status) return new Response(out.error ?? '', { status: out.status });
            if (!out.xlsx) {
                return new Response(out.xlsxError ?? 'The worksheet could not be calculated (check it for macro errors).', { status: 422 });
            }
            return new Response(fromBase64(out.xlsx), {
                status: 200,
                headers: { 'Content-Type': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' },
            });
        }
        if (endpoint in NEEDS_SERVER) return notAvailable(NEEDS_SERVER[endpoint], 'it needs Calcpad.Server');
        return notAvailable(endpoint, 'unknown endpoint');
    };

    window.fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
        const req = input instanceof Request ? input : null;
        const url = new URL(req ? req.url : String(input), window.location.href);
        if (url.origin !== window.location.origin || !url.pathname.startsWith(API_PREFIX)) {
            return nativeFetch(input, init);
        }
        const method = (init?.method ?? req?.method ?? 'GET').toUpperCase();
        let body: string | null = null;
        if (init?.body != null) body = typeof init.body === 'string' ? init.body : await new Response(init.body).text();
        else if (req) body = await req.text();
        return handle(url, method, body, init?.signal ?? req?.signal);
    };

    return transport;
}
