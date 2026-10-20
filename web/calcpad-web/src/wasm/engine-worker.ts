/// <reference lib="webworker" />
// calcpad-rs: runs the calcpad-wasm engine off the main thread.
//
// The worker loads public/wasm/calcpad_wasm.js (built by tools/build_wasm.sh) and answers
// { id, kind: 'convert', body, unwrap } with the JSON string that calcpad-wasm's convert()
// returns, and keeps the in-memory #include files in step with { kind: 'files' } messages.

type Req =
    | { id: number; base: string; kind: 'convert'; body: string; unwrap: boolean }
    | { id: number; base: string; kind: 'files'; files: Record<string, string> };

interface Engine {
    default: (input?: unknown) => Promise<unknown>;
    convert: (body: string, unwrap: boolean) => string;
    set_file: (path: string, content: string) => void;
    clear_files: () => void;
    version: () => string;
}

let engine: Promise<Engine> | null = null;

function load(base: string): Promise<Engine> {
    if (!engine) {
        engine = (async () => {
            const mod = (await import(/* @vite-ignore */ `${base}wasm/calcpad_wasm.js`)) as Engine;
            await mod.default({ module_or_path: `${base}wasm/calcpad_wasm_bg.wasm` });
            return mod;
        })();
    }
    return engine;
}

self.onmessage = async (e: MessageEvent<Req>) => {
    const req = e.data;
    try {
        const mod = await load(new URL(req.base, self.location.origin).href);
        if (req.kind === 'files') {
            mod.clear_files();
            for (const [path, content] of Object.entries(req.files)) mod.set_file(path, content);
            self.postMessage({ id: req.id, result: '{}' });
        } else {
            self.postMessage({ id: req.id, result: mod.convert(req.body, req.unwrap) });
        }
    } catch (err) {
        self.postMessage({ id: req.id, error: err instanceof Error ? err.message : String(err) });
    }
};
