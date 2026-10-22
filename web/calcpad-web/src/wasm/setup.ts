// calcpad-rs: wires the browser build's file handling (web-files.ts, library.ts) into
// calcpad-web. Called once from main.ts in the web (non-Tauri) build.

import { FileLibrary } from './library';
import type { WasmTransport } from './transport';
import { type TabsLike, WebFiles } from './web-files';
import LibrariesSettings from './LibrariesSettings.vue';
import LibrariesInsert from './LibrariesInsert.vue';
import { installEmptyEditorLink, SamplesBrowser } from './samples';

interface Host {
    /** The active editor group's tabs (main.ts keeps them on window.calcpadTabs). */
    tabs: () => TabsLike;
    appendOutput: (level: 'info' | 'warn' | 'error', message: string) => void;
    /** The message bridge: the File menu prints through the Export tab's Save PDF path. */
    bridge: { handleMessage(message: unknown): void };
    /** Shows a sidebar tab ('export'). */
    showSidebarTab?: (id: string) => void;
}

export function installWebFiles(transport: WasmTransport, host: Host): WebFiles {
    const library = new FileLibrary(transport);
    const samples = new SamplesBrowser({ library, tabs: host.tabs, log: host.appendOutput });
    const files = new WebFiles({
        library,
        tabs: host.tabs,
        log: host.appendOutput,
        print: () => host.bridge.handleMessage({ type: 'generatePdf', variant: 'report' }),
        openSamples: () => void samples.open(),
        openExport: host.showSidebarTab ? () => host.showSidebarTab!('export') : undefined,
        exportXlsx: () => host.bridge.handleMessage({ type: 'saveXlsx' }),
        exportDocx: () => host.bridge.handleMessage({ type: 'saveDocx', variant: 'report' }),
    });
    // Images a worksheet names (Images/..., ./x.png) are shown from the library.
    transport.resolveFile = (key) => library.get(key)?.data;
    installEmptyEditorLink(host.tabs, () => void samples.open());
    transport.onFilesWritten = written => void files.onFilesWritten(written);
    files.install();
    (window as unknown as { calcpadFiles: unknown }).calcpadFiles = { files, library, samples };
    return files;
}

/** The sections the browser build adds to the sidebar (see CalcpadSettingsTab/CalcpadInsertTab). */
export function webExtensions() {
    return { settingsSections: [LibrariesSettings], insertSections: [LibrariesInsert] };
}
