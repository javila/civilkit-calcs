// calcpad-rs: files in the browser build. Upstream's file handling (open, save, recent files,
// drag and drop) is wired to Tauri's native dialogs only; this module gives the web build its
// own, without a server:
//
//   File menu (menu bar, top)      New, Open, Samples, Save, Save As, Recent, Print, Export, the library
//   Ctrl+O / Ctrl+S / Ctrl+Shift+S open and save
//   drag and drop                   .cpd opens in the editor, any other file joins the library
//
// Saving uses the File System Access API where the browser has it (the file is written in
// place) and falls back to a download. Recent files are listed in localStorage; where the
// API exists their handles are kept in IndexedDB so they reopen without a picker. Every
// opened worksheet and every added file goes into the library (library.ts), which is what
// #include and #read resolve against. Files a worksheet writes are added to the library and
// downloaded when their content changes.

import { replaceCStyleOperators } from './c-style';
import type { FileLibrary } from './library';
import { dbRequest, libraryPath, STORE_HANDLES } from './library';
import type { EngineFile } from './transport';

/** The parts of calcpad-web's TabManager this module uses. */
export interface TabsLike {
    readonly activeTab: { id: string; title: string; filePath: string | null; dirty: boolean } | null;
    readonly activeModel: { getValue(): string } | null;
    findByPath(filePath: string): { id: string } | null;
    openFile(filePath: string, content: string): string;
    activate(id: string): void;
    newUntitled(content?: string, title?: string): string;
    markActiveSaved(opts?: { filePath?: string }): void;
}

export interface WebFilesOptions {
    library: FileLibrary;
    /** The active editor group's tabs. */
    tabs: () => TabsLike;
    log: (level: 'info' | 'warn' | 'error', message: string) => void;
    /** Prints the active worksheet's report (the Export tab's Save PDF). */
    print: () => void;
    /** Opens the Samples browser. */
    openSamples?: () => void;
    /** Shows the Export tab (HTML, PDF, Word, #write). */
    openExport?: () => void;
    /** Saves the variables as an Excel workbook (calcpad-rs's own export). */
    exportXlsx?: () => void;
}

const RECENT_KEY = 'calcpad-rs-recent-files';
const MAX_RECENT = 10;
const WORKSHEET_EXT = /\.cpd$/i;
const ACCEPT_ALL = '.cpd,.txt,.csv,.xlsx,.xlsm';

type Handle = FileSystemFileHandle;
type RecentEntry = { path: string; name: string; time: number };

interface PickerWindow {
    showOpenFilePicker?: (o?: unknown) => Promise<Handle[]>;
    showSaveFilePicker?: (o?: unknown) => Promise<Handle>;
}

const picker = window as unknown as PickerWindow;

function readRecent(): RecentEntry[] {
    try {
        const v = JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]');
        return Array.isArray(v) ? v.filter(e => e && typeof e.path === 'string') : [];
    } catch {
        return [];
    }
}

function writeRecent(list: RecentEntry[]): void {
    try {
        localStorage.setItem(RECENT_KEY, JSON.stringify(list.slice(0, MAX_RECENT)));
    } catch {
        // Storage may be unavailable; the list is a convenience.
    }
}

function download(name: string, data: BlobPart, type = 'application/octet-stream'): void {
    const url = URL.createObjectURL(new Blob([data], { type }));
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function baseName(path: string): string {
    return path.slice(path.lastIndexOf('/') + 1);
}

/** `<input type=file>` for browsers without showOpenFilePicker. */
function pickWithInput(accept: string, multiple: boolean): Promise<File[]> {
    return new Promise(resolve => {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = accept;
        input.multiple = multiple;
        input.style.display = 'none';
        input.onchange = () => {
            resolve(Array.from(input.files ?? []));
            input.remove();
        };
        input.addEventListener('cancel', () => {
            resolve([]);
            input.remove();
        });
        document.body.appendChild(input);
        input.click();
    });
}

export class WebFiles {
    private handles = new Map<string, Handle>();
    private menu: HTMLDivElement | null = null;

    constructor(private readonly o: WebFilesOptions) {}

    install(): void {
        this.installButton();
        window.addEventListener('keydown', e => this.onKey(e), true);
        document.addEventListener('dragover', e => {
            if (e.dataTransfer?.types.includes('Files')) {
                e.preventDefault();
                e.dataTransfer.dropEffect = 'copy';
            }
        }, true);
        document.addEventListener('drop', e => {
            if (!e.dataTransfer?.files.length) return;
            e.preventDefault();
            e.stopPropagation();
            void this.openFiles(Array.from(e.dataTransfer.files).map(f => ({ file: f })));
        }, true);
    }

    // ----- written files -------------------------------------------------------------------

    /** Files a render's #write/#append produced: kept in the library, downloaded when new. */
    async onFilesWritten(files: EngineFile[]): Promise<void> {
        for (const f of files) {
            const changed = await this.o.library.put(f.path, f.data);
            if (changed) {
                download(baseName(f.path), f.data as BlobPart);
                this.o.log('info', `#write: ${baseName(f.path)} downloaded and kept in the library.`);
            }
        }
    }

    // ----- commands ------------------------------------------------------------------------

    newWorksheet(): void {
        this.o.tabs().newUntitled('');
    }

    async open(): Promise<void> {
        if (picker.showOpenFilePicker) {
            let handles: Handle[];
            try {
                handles = await picker.showOpenFilePicker({
                    multiple: true,
                    types: [{
                        description: 'Calcpad worksheets and data',
                        accept: { 'text/plain': ['.cpd', '.txt', '.csv'], 'application/vnd.ms-excel': ['.xlsx', '.xlsm'] },
                    }],
                });
            } catch {
                return; // cancelled
            }
            await this.openFiles(await Promise.all(handles.map(async h => ({ file: await h.getFile(), handle: h }))));
        } else {
            await this.openFiles((await pickWithInput(ACCEPT_ALL, true)).map(file => ({ file })));
        }
    }

    async addToLibrary(): Promise<void> {
        const files = await pickWithInput('', true);
        for (const f of files) await this.o.library.put(f.name, new Uint8Array(await f.arrayBuffer()));
        if (files.length) this.o.log('info', `Added ${files.map(f => f.name).join(', ')} to the library.`);
    }

    /** Worksheets open in the editor; every file goes into the library. */
    async openFiles(items: { file: File; handle?: Handle }[]): Promise<void> {
        for (const { file, handle } of items) {
            const data = new Uint8Array(await file.arrayBuffer());
            const path = libraryPath(file.name);
            await this.o.library.put(path, data);
            if (WORKSHEET_EXT.test(file.name)) {
                if (handle) await this.rememberHandle(path, handle);
                this.openTab(path, new TextDecoder().decode(data));
                this.addRecent(path);
            } else {
                this.o.log('info', `${file.name} is in the library, for #read or #include.`);
            }
        }
    }

    private openTab(path: string, content: string): void {
        const tabs = this.o.tabs();
        const existing = tabs.findByPath(path);
        if (existing) {
            tabs.activate(existing.id);
            return;
        }
        // As the desktop app opens a file: C-style operator shortcuts ("%%", "==") become Calcpad's.
        tabs.openFile(path, replaceCStyleOperators(content));
    }

    async openRecent(entry: RecentEntry): Promise<void> {
        const handle = this.handles.get(entry.path) ?? await dbRequest<Handle>(STORE_HANDLES, 'readonly', s => s.get(entry.path));
        if (handle) {
            try {
                const h = handle as Handle & { requestPermission?: (o: unknown) => Promise<string> };
                if (h.requestPermission && await h.requestPermission({ mode: 'readwrite' }) !== 'granted') throw new Error('denied');
                await this.openFiles([{ file: await handle.getFile(), handle }]);
                return;
            } catch {
                // Fall back to the library copy below.
            }
        }
        const text = this.o.library.text(entry.path);
        if (text !== undefined) {
            this.openTab(entry.path, text);
            this.addRecent(entry.path);
        } else {
            this.o.log('warn', `${entry.name} is no longer in the library; open it again.`);
            await this.open();
        }
    }

    async save(saveAs = false): Promise<boolean> {
        const tabs = this.o.tabs();
        const tab = tabs.activeTab;
        if (!tab) return false;
        const content = tabs.activeModel?.getValue() ?? '';
        let path = tab.filePath;
        let handle = path ? this.handles.get(path) : undefined;
        if (saveAs || !path) {
            const suggested = path ? baseName(path) : `${tab.title.replace(/\.cpd$/i, '')}.cpd`;
            if (picker.showSaveFilePicker) {
                try {
                    handle = await picker.showSaveFilePicker({
                        suggestedName: suggested,
                        types: [{ description: 'Calcpad worksheet', accept: { 'text/plain': ['.cpd'] } }],
                    });
                } catch {
                    return false; // cancelled
                }
                path = libraryPath(handle.name);
            } else {
                const name = window.prompt('Save as', suggested);
                if (!name) return false;
                handle = undefined;
                path = libraryPath(/\.\w+$/.test(name) ? name : `${name}.cpd`);
            }
        }
        if (handle) {
            try {
                const writable = await handle.createWritable();
                await writable.write(content);
                await writable.close();
                await this.rememberHandle(path!, handle);
            } catch (e) {
                this.o.log('error', `Could not save ${baseName(path!)}: ${e instanceof Error ? e.message : String(e)}`);
                return false;
            }
        } else {
            download(baseName(path!), content, 'text/plain;charset=utf-8');
        }
        await this.o.library.putText(path!, content);
        tabs.markActiveSaved({ filePath: path! });
        this.addRecent(path!);
        return true;
    }

    private async rememberHandle(path: string, handle: Handle): Promise<void> {
        this.handles.set(path, handle);
        await dbRequest(STORE_HANDLES, 'readwrite', s => s.put(handle, path));
    }

    private addRecent(path: string): void {
        const list = readRecent().filter(e => e.path !== path);
        list.unshift({ path, name: baseName(path), time: Date.now() });
        writeRecent(list);
    }

    private onKey(e: KeyboardEvent): void {
        if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
        const key = e.key.toLowerCase();
        if (key === 'o' && !e.shiftKey) {
            e.preventDefault();
            e.stopPropagation();
            void this.open();
        } else if (key === 's') {
            e.preventDefault();
            e.stopPropagation();
            void this.save(e.shiftKey);
        }
    }

    // ----- the menu --------------------------------------------------------------------------

    private installButton(): void {
        const style = document.createElement('style');
        style.textContent = `
body{display:flex;flex-direction:column}
#app{height:auto!important;flex:1;min-height:0}
.cprs-menubar{flex:none;display:flex;align-items:center;height:28px;padding:0 4px;gap:2px;font-size:13px;
  background:var(--vscode-titleBar-activeBackground,var(--vscode-sideBar-background,#f3f3f3));
  color:var(--vscode-titleBar-activeForeground,var(--vscode-foreground,#333));
  border-bottom:1px solid var(--vscode-titleBar-border,var(--vscode-panel-border,#ddd))}
.cprs-file-btn{height:22px;border:0;padding:0 10px;border-radius:3px;background:none;color:inherit;font:inherit;cursor:pointer}
.cprs-file-btn:hover,.cprs-file-btn.open{background:var(--vscode-toolbar-hoverBackground,rgba(0,0,0,.08))}
.cprs-menubar .cprs-brand{margin-left:auto;padding:0 10px;opacity:.6;font-size:12px}
.cprs-menu{position:fixed;left:4px;top:28px;z-index:1001;min-width:260px;max-height:80vh;overflow:auto;
  background:var(--vscode-menu-background,#fff);color:var(--vscode-menu-foreground,#222);
  border:1px solid var(--vscode-menu-border,#c8c8c8);box-shadow:0 4px 16px rgba(0,0,0,.2);font-size:13px;padding:4px 0}
.cprs-menu button{display:flex;justify-content:space-between;gap:16px;width:100%;border:0;background:none;
  color:inherit;font:inherit;text-align:left;padding:5px 14px;cursor:pointer}
.cprs-menu button:hover{background:var(--vscode-menu-selectionBackground,#0060c0);color:var(--vscode-menu-selectionForeground,#fff)}
.cprs-menu .cprs-key{opacity:.7}
.cprs-menu hr{border:0;border-top:1px solid var(--vscode-menu-separatorBackground,#ddd);margin:4px 0}
.cprs-menu .cprs-head{padding:4px 14px;font-size:11px;text-transform:uppercase;opacity:.7}
.cprs-menu .cprs-row{display:flex;align-items:center}
.cprs-menu .cprs-row button:first-child{flex:1}
.cprs-menu .cprs-row button.cprs-x{width:auto;padding:5px 10px}
.cprs-menu .cprs-empty{padding:4px 14px;opacity:.6}`;
        document.head.appendChild(style);
        // A menu bar above the app (the app is a full-height flex column below it).
        const bar = document.createElement('div');
        bar.className = 'cprs-menubar';
        bar.setAttribute('role', 'menubar');
        const btn = document.createElement('button');
        btn.className = 'cprs-file-btn';
        btn.textContent = 'File';
        btn.setAttribute('role', 'menuitem');
        btn.title = 'New, open, samples, save, recent files, print and export';
        btn.addEventListener('click', e => {
            e.stopPropagation();
            this.toggleMenu();
        });
        this.fileButton = btn;
        bar.appendChild(btn);
        document.body.insertBefore(bar, document.body.firstChild);
        document.addEventListener('mousedown', e => {
            if (this.menu && !this.menu.contains(e.target as Node) && e.target !== btn) this.closeMenu();
        });
        document.addEventListener('keydown', e => {
            if (e.key === 'Escape') this.closeMenu();
        });
    }

    private fileButton: HTMLButtonElement | null = null;

    private closeMenu(): void {
        this.menu?.remove();
        this.menu = null;
        this.fileButton?.classList.remove('open');
    }

    private toggleMenu(): void {
        if (this.menu) return this.closeMenu();
        const menu = document.createElement('div');
        menu.className = 'cprs-menu';
        menu.setAttribute('role', 'menu');
        const item = (label: string, action: () => void, key = '') => {
            const b = document.createElement('button');
            b.setAttribute('role', 'menuitem');
            b.innerHTML = '<span></span><span class="cprs-key"></span>';
            (b.firstChild as HTMLElement).textContent = label;
            (b.lastChild as HTMLElement).textContent = key;
            b.addEventListener('click', () => {
                this.closeMenu();
                action();
            });
            menu.appendChild(b);
            return b;
        };
        const head = (text: string) => {
            const d = document.createElement('div');
            d.className = 'cprs-head';
            d.textContent = text;
            menu.appendChild(d);
        };
        const empty = (text: string) => {
            const d = document.createElement('div');
            d.className = 'cprs-empty';
            d.textContent = text;
            menu.appendChild(d);
        };
        const mod = /Mac|iPhone|iPad/.test(navigator.platform) ? 'Cmd' : 'Ctrl';
        item('New', () => this.newWorksheet());
        item('Open...', () => void this.open(), `${mod}+O`);
        if (this.o.openSamples) item('Samples...', () => this.o.openSamples!());
        item('Save', () => void this.save(), `${mod}+S`);
        item('Save As...', () => void this.save(true), `${mod}+Shift+S`);
        menu.appendChild(document.createElement('hr'));
        head('Recent');
        const recent = readRecent();
        if (!recent.length) empty('No recent worksheets');
        for (const r of recent) item(r.name, () => void this.openRecent(r), new Date(r.time).toLocaleDateString());
        menu.appendChild(document.createElement('hr'));
        item('Print PDF...', () => this.o.print());
        if (this.o.openExport) item('Export...', () => this.o.openExport!());
        if (this.o.exportXlsx) item('Export Excel...', () => this.o.exportXlsx!());
        menu.appendChild(document.createElement('hr'));
        head('Library (#include and #read)');
        const files = this.o.library.list();
        if (!files.length) empty('Empty. Open or add files.');
        for (const f of files) {
            const row = document.createElement('div');
            row.className = 'cprs-row';
            menu.appendChild(row);
            const open = item(f.path.slice(1), () => {
                if (WORKSHEET_EXT.test(f.name)) this.openTab(f.path, new TextDecoder().decode(f.data));
                else download(f.name, f.data as BlobPart);
            }, WORKSHEET_EXT.test(f.name) ? 'open' : 'download');
            const x = document.createElement('button');
            x.className = 'cprs-x';
            x.title = `Remove ${f.name} from the library`;
            x.textContent = '×';
            x.addEventListener('click', () => {
                this.closeMenu();
                void this.o.library.remove(f.path);
            });
            row.append(open, x);
        }
        item('Add files to the library...', () => void this.addToLibrary());
        document.body.appendChild(menu);
        this.menu = menu;
        this.fileButton?.classList.add('open');
    }
}
