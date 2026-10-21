/// <reference lib="webworker" />
// calcpad-rs: runs the calcpad-highlighter-wasm module (the Rust port of Calcpad.Highlighter)
// off the main thread. It is a separate wasm, loaded the first time the editor asks for
// highlighting, linting, definitions, symbols, prettify or snippets.
//
//   { id, kind: 'files', files }                       replaces the files #include can see
//   { id, kind: 'call', method, body }                 method(body) -> JSON string
//   { id, kind: 'snippets', category }                 snippets(category) -> JSON string

export {};

type FileEntry = { path: string; data: Uint8Array };

type Req =
    | { id: number; base: string; kind: 'files'; files: FileEntry[] }
    | { id: number; base: string; kind: 'call'; method: string; body: string }
    | { id: number; base: string; kind: 'snippets'; category: string };

interface Highlighter {
    default: (input?: unknown) => Promise<unknown>;
    set_file: (path: string, content: string) => void;
    clear_files: () => void;
    snippets: (category: string) => string;
    [method: string]: unknown;
}

let module_: Promise<Highlighter> | null = null;

function load(base: string): Promise<Highlighter> {
    if (!module_) {
        module_ = (async () => {
            const mod = (await import(/* @vite-ignore */ `${base}wasm/calcpad_highlighter_wasm.js`)) as Highlighter;
            await mod.default({ module_or_path: `${base}wasm/calcpad_highlighter_wasm_bg.wasm` });
            return mod;
        })();
    }
    return module_;
}

const decoder = new TextDecoder();

self.onmessage = async (e: MessageEvent<Req>) => {
    const req = e.data;
    try {
        const mod = await load(req.base);
        switch (req.kind) {
            case 'files':
                mod.clear_files();
                for (const f of req.files) mod.set_file(f.path, decoder.decode(f.data));
                self.postMessage({ id: req.id, result: '{}' });
                break;
            case 'snippets':
                self.postMessage({ id: req.id, result: mod.snippets(req.category) });
                break;
            default: {
                const fn = mod[req.method];
                if (typeof fn !== 'function') throw new Error(`unknown highlighter method ${req.method}`);
                self.postMessage({ id: req.id, result: (fn as (b: string) => string)(req.body) });
            }
        }
    } catch (err) {
        self.postMessage({ id: req.id, error: err instanceof Error ? err.message : String(err) });
    }
};
