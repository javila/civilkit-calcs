/// <reference lib="webworker" />
// calcpad-rs: runs the calcpad-wasm engine off the main thread.
//
// The worker loads public/wasm/calcpad_wasm.js (built by tools/build_wasm.sh) and answers
//   { id, kind: 'convert', body, unwrap }    with the JSON string calcpad-wasm's convert() returns
//   { id, kind: 'files', files }             replacing the in-memory files #include and #read see
//   { id, kind: 'head', markup }             setting the report head (the bundled fonts)
//   { id, kind: 'docx', html, expressions }  the .docx bytes of a rendered page (calcpad-docx-wasm,
//                                            loaded on first use with DejaVu Sans for drawing text)

type FileEntry = { path: string; data: Uint8Array };

type Req =
    | { id: number; base: string; kind: 'convert'; body: string; unwrap: boolean }
    | { id: number; base: string; kind: 'files'; files: FileEntry[] }
    | { id: number; base: string; kind: 'head'; markup: string }
    | { id: number; base: string; kind: 'docx'; html: string; expressions: string };

interface DocxModule {
    default: (input?: unknown) => Promise<unknown>;
    docx: (html: string, expressions: string) => Uint8Array;
    add_font: (data: Uint8Array) => void;
}

let docxModule: Promise<DocxModule> | null = null;

function loadDocx(base: string): Promise<DocxModule> {
    if (!docxModule) {
        docxModule = (async () => {
            const mod = (await import(/* @vite-ignore */ `${base}wasm/calcpad_docx_wasm.js`)) as DocxModule;
            await mod.default({ module_or_path: `${base}wasm/calcpad_docx_wasm_bg.wasm` });
            const font = await fetch(`${base}report-fonts/DejaVuSans.ttf`);
            if (font.ok) mod.add_font(new Uint8Array(await font.arrayBuffer()));
            return mod;
        })();
        docxModule.catch(() => { docxModule = null; });
    }
    return docxModule;
}

interface Engine {
    default: (input?: unknown) => Promise<unknown>;
    convert: (body: string, unwrap: boolean) => string;
    set_file_bytes: (path: string, content: Uint8Array) => void;
    clear_files: () => void;
    set_report_head: (markup: string) => void;
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
        if (req.kind === 'docx') {
            const docx = await loadDocx(req.base);
            const bytes = docx.docx(req.html, req.expressions);
            self.postMessage({ id: req.id, result: bytes }, [bytes.buffer]);
            return;
        }
        const mod = await load(req.base);
        switch (req.kind) {
            case 'files':
                mod.clear_files();
                for (const f of req.files) mod.set_file_bytes(f.path, f.data);
                self.postMessage({ id: req.id, result: '{}' });
                break;
            case 'head':
                mod.set_report_head(req.markup);
                self.postMessage({ id: req.id, result: '{}' });
                break;
            default:
                self.postMessage({ id: req.id, result: mod.convert(req.body, req.unwrap) });
        }
    } catch (err) {
        self.postMessage({ id: req.id, error: err instanceof Error ? err.message : String(err) });
    }
};
