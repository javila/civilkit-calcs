// calcpad-rs: wires the browser build's file handling (web-files.ts, library.ts) into
// calcpad-web. Called once from main.ts in the web (non-Tauri) build.

import { FileLibrary } from './library';
import type { WasmTransport } from './transport';
import { type TabsLike, WebFiles } from './web-files';

interface Host {
    /** The active editor group's tabs (main.ts keeps them on window.calcpadTabs). */
    tabs: () => TabsLike;
    appendOutput: (level: 'info' | 'warn' | 'error', message: string) => void;
    /** The message bridge: the File menu prints through the Export tab's Save PDF path. */
    bridge: { handleMessage(message: unknown): void };
}

export function installWebFiles(transport: WasmTransport, host: Host): WebFiles {
    const library = new FileLibrary(transport);
    const files = new WebFiles({
        library,
        tabs: host.tabs,
        log: host.appendOutput,
        print: () => host.bridge.handleMessage({ type: 'generatePdf', variant: 'report' }),
    });
    transport.onFilesWritten = written => void files.onFilesWritten(written);
    files.install();
    (window as unknown as { calcpadFiles: unknown }).calcpadFiles = { files, library };
    return files;
}
