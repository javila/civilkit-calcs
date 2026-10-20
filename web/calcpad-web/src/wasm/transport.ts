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
//   pdf, docx, cpdz, portable, log-level    501: these need the server (Chromium for PDF,
//                                           OpenXml for Word, the file system)

import { reportHeadMarkup } from './report-fonts';

const API_PREFIX = '/api/calcpad/';

const NOT_PORTED: Record<string, string> = {
    lint: 'Linting',
    highlight: 'Semantic highlighting',
    definitions: 'Definitions',
    'symbol-at-position': 'Go to definition, references and rename',
    prettify: 'Prettify',
};

const NEEDS_SERVER: Record<string, string> = {
    pdf: 'PDF export',
    'pdf/health': 'PDF export',
    'pdf/browser': 'PDF export',
    'pdf/browser/install': 'PDF export',
    docx: 'Word export',
    'cpdz/decode': 'Compiled .cpdz worksheets',
    'cpdz/encode': 'Compiled .cpdz worksheets',
    'portable/bundle': 'Portable worksheets',
    'portable/package': 'Portable worksheets',
    'log-level': 'Server logging',
};

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
    private files: Record<string, string> = {};

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
        // A fresh worker has no #include files yet.
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

    setFiles(files: Record<string, string>): Promise<string> {
        this.files = { ...files };
        return this.post({ kind: 'files', files: this.files });
    }
}

function notAvailable(what: string, reason: string): Response {
    return new Response(`${what} is not available in the browser build of CalcpadCE (calcpad-rs): ${reason}.`, {
        status: 501,
        headers: { 'Content-Type': 'text/plain; charset=utf-8' },
    });
}

/** Installs the adapter. Call once, before the editor makes its first request. */
export function installWasmTransport(): { setIncludeFiles(files: Record<string, string>): Promise<void> } {
    const base = import.meta.env.BASE_URL || '/';
    const engine = new EngineWorker(base);
    const nativeFetch = window.fetch.bind(window);

    const handle = async (url: URL, method: string, body: string | null, signal?: AbortSignal | null): Promise<Response> => {
        const endpoint = url.pathname.slice(API_PREFIX.length).replace(/\/+$/, '');
        if (endpoint === 'health') return Response.json({ status: 'ok' });
        // The snippet library lives in Calcpad.Highlighter (not ported). An empty list, not an
        // error, so the Insert panel does not poll for it every few seconds.
        if (endpoint === 'snippets') return Response.json({ count: 0, snippets: [] });
        if (endpoint === 'convert' && method === 'POST') {
            let request: Record<string, unknown>;
            try {
                request = body ? JSON.parse(body) : {};
            } catch {
                return new Response('Invalid JSON', { status: 400 });
            }
            request.headMarkup = await reportHeadMarkup(base);
            const unwrap = url.searchParams.get('unwrap') === 'true';
            const out = JSON.parse(await engine.convert(JSON.stringify(request), unwrap, signal)) as {
                html?: string; errors?: unknown[]; status?: number; error?: string;
            };
            if (out.status) return new Response(out.error ?? '', { status: out.status });
            return new Response(out.html ?? '', {
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

    return {
        async setIncludeFiles(files) {
            await engine.setFiles(files);
        },
    };
}
