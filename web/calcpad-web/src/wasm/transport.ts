// calcpad-rs: answers the editor's Calcpad.Server requests with the calcpad-wasm engine.
//
// Upstream calcpad-web talks to Calcpad.Server over HTTP (calcpad-frontend's
// CalcpadApiClient and a few direct fetch calls). This adapter replaces that transport:
// requests to `<origin>/api/calcpad/*` are answered in the page, everything else goes to
// the network as before. No calculation request leaves the page.
//
//   convert, convert?unwrap=true, health   the calcpad-wasm engine (in a worker)
//   lint, highlight, definitions,           501: Calcpad.Highlighter is not ported, so the
//   symbol-at-position, prettify            editor runs without diagnostics, semantic
//                                           highlighting, go-to-definition and prettify
//   snippets                                an empty list (the library is in the Highlighter)
//   docx, cpdz, portable, log-level, pdf    501: these need Calcpad.Server. PDF is printed by
//                                           the browser instead (message-bridge.ts)

import { reportHeadMarkup } from './report-fonts';
import { uiAssetsMarkup } from './ui-assets';

const API_PREFIX = '/api/calcpad/';
/** calcpad-wasm's UI_ASSETS_MARKER: where the datagrid libraries go. */
const UI_ASSETS_MARKER = '<!--calcpad-ui-assets-->';

const NOT_PORTED: Record<string, string> = {
    lint: 'Linting',
    highlight: 'Semantic highlighting',
    definitions: 'Definitions',
    'symbol-at-position': 'Go to definition, references and rename',
    prettify: 'Prettify',
};

const NEEDS_SERVER: Record<string, string> = {
    pdf: 'Server PDF export',
    'pdf/health': 'Server PDF export',
    'pdf/browser': 'Server PDF export',
    'pdf/browser/install': 'Server PDF export',
    docx: 'Word export',
    'cpdz/decode': 'Compiled .cpdz worksheets',
    'cpdz/encode': 'Compiled .cpdz worksheets',
    'portable/bundle': 'Portable worksheets',
    'portable/package': 'Portable worksheets',
    'log-level': 'Server logging',
};

/** A file the engine can read (#include, #read): an absolute path and its bytes. */
export type EngineFile = { path: string; data: Uint8Array };

type Job = {
    id: number;
    msg: Record<string, unknown>;
    resolve: (v: string) => void;
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
        w.onmessage = (e: MessageEvent<{ id: number; result?: string; error?: string }>) => {
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

    private post(msg: Record<string, unknown>, signal?: AbortSignal | null): Promise<string> {
        return new Promise<string>((resolve, reject) => {
            const job: Job = { id: ++this.seq, msg, resolve, reject };
            if (signal?.aborted) {
                reject(signal.reason ?? new DOMException('Aborted', 'AbortError'));
                return;
            }
            signal?.addEventListener('abort', () => {
                const reason = signal.reason ?? new DOMException('Aborted', 'AbortError');
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
}

/** Installs the adapter. Call once, before the editor makes its first request. */
export function installWasmTransport(): WasmTransport {
    // Absolute, so the worker (whose own URL is under assets/) resolves the engine correctly
    // when the app is served from a sub folder.
    const base = new URL(import.meta.env.BASE_URL || './', document.baseURI).href;
    const engine = new EngineWorker(base);
    const nativeFetch = window.fetch.bind(window);
    const head = reportHeadMarkup(base).then(m => engine.setHead(m));
    const transport: WasmTransport = {
        async setFiles(files) {
            await engine.setFiles(files);
        },
        onFilesWritten: null,
    };

    const handle = async (url: URL, method: string, body: string | null, signal?: AbortSignal | null): Promise<Response> => {
        const endpoint = url.pathname.slice(API_PREFIX.length).replace(/\/+$/, '');
        if (endpoint === 'health') return Response.json({ status: 'ok' });
        // The snippet library lives in Calcpad.Highlighter (not ported). An empty list, not an
        // error, so the Insert panel does not poll for it every few seconds.
        if (endpoint === 'snippets') return Response.json({ count: 0, snippets: [] });
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
        if (endpoint in NOT_PORTED) return notAvailable(NOT_PORTED[endpoint], 'Calcpad.Highlighter is not ported yet');
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
