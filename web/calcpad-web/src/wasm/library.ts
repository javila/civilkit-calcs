// calcpad-rs: the in-page file library. Worksheets the user opened and files they added
// (other worksheets to #include, CSV and Excel files to #read) live here, at "/<name>", and
// are what the engine's #include and #read see. Files a worksheet writes land here too.
//
// The library is kept in IndexedDB so it survives a reload; where storage is unavailable
// (a private window, blocked site data) it works for the session only.

import type { EngineFile, WasmTransport } from './transport';

export interface LibraryFile {
    path: string;
    name: string;
    data: Uint8Array;
    modified: number;
}

const DB_NAME = 'calcpad-rs';
const DB_VERSION = 1;
export const STORE_LIBRARY = 'library';
export const STORE_HANDLES = 'handles';

let dbPromise: Promise<IDBDatabase | null> | null = null;

/** The app's IndexedDB, or null where storage is not available. */
export function openDb(): Promise<IDBDatabase | null> {
    dbPromise ??= new Promise<IDBDatabase | null>((resolve) => {
        try {
            const req = indexedDB.open(DB_NAME, DB_VERSION);
            req.onupgradeneeded = () => {
                const db = req.result;
                if (!db.objectStoreNames.contains(STORE_LIBRARY)) db.createObjectStore(STORE_LIBRARY, { keyPath: 'path' });
                if (!db.objectStoreNames.contains(STORE_HANDLES)) db.createObjectStore(STORE_HANDLES);
            };
            req.onsuccess = () => resolve(req.result);
            req.onerror = () => resolve(null);
            req.onblocked = () => resolve(null);
        } catch {
            resolve(null);
        }
    });
    return dbPromise;
}

/** Runs one request against a store; undefined when storage is unavailable or it fails. */
export async function dbRequest<T>(
    store: string,
    mode: IDBTransactionMode,
    make: (s: IDBObjectStore) => IDBRequest<T>,
): Promise<T | undefined> {
    const db = await openDb();
    if (!db) return undefined;
    try {
        return await new Promise<T | undefined>((resolve) => {
            const tx = db.transaction(store, mode);
            const req = make(tx.objectStore(store));
            req.onsuccess = () => resolve(req.result);
            req.onerror = () => resolve(undefined);
        });
    } catch {
        return undefined;
    }
}

/** "/<name>" for a plain file name; paths keep their folders, always rooted at "/". */
export function libraryPath(name: string): string {
    const parts = name.replace(/\\/g, '/').split('/').filter(p => p && p !== '.');
    return '/' + parts.join('/');
}

export class FileLibrary {
    private files = new Map<string, LibraryFile>();
    private listeners: (() => void)[] = [];
    readonly ready: Promise<void>;

    constructor(private readonly transport: WasmTransport) {
        this.ready = this.restore();
    }

    private async restore(): Promise<void> {
        const stored = await dbRequest<LibraryFile[]>(STORE_LIBRARY, 'readonly', s => s.getAll() as IDBRequest<LibraryFile[]>);
        for (const f of stored ?? []) {
            if (f && typeof f.path === 'string' && f.data instanceof Uint8Array) this.files.set(f.path, f);
        }
        await this.sync();
    }

    onChange(listener: () => void): void {
        this.listeners.push(listener);
    }

    list(): LibraryFile[] {
        return [...this.files.values()].sort((a, b) => a.path.localeCompare(b.path));
    }

    get(path: string): LibraryFile | undefined {
        return this.files.get(libraryPath(path));
    }

    text(path: string): string | undefined {
        const f = this.get(path);
        return f ? new TextDecoder().decode(f.data) : undefined;
    }

    /** Adds or replaces a file; true when its content changed. */
    async put(name: string, data: Uint8Array): Promise<boolean> {
        const path = libraryPath(name);
        const previous = this.files.get(path);
        const changed = !previous || !sameBytes(previous.data, data);
        const file: LibraryFile = { path, name: path.slice(path.lastIndexOf('/') + 1), data, modified: Date.now() };
        this.files.set(path, file);
        await dbRequest(STORE_LIBRARY, 'readwrite', s => s.put(file));
        await this.sync();
        return changed;
    }

    /**
     * Adds or replaces many files and removes others with one write and one sync (an import brings
     * hundreds of pictures; syncing after each would resend the whole library each time).
     */
    async putMany(files: { path: string; data: Uint8Array }[], removePaths: string[] = []): Promise<void> {
        const added: LibraryFile[] = [];
        for (const f of files) {
            const path = libraryPath(f.path);
            const file: LibraryFile = { path, name: path.slice(path.lastIndexOf('/') + 1), data: f.data, modified: Date.now() };
            this.files.set(path, file);
            added.push(file);
        }
        const removed = removePaths.map(libraryPath).filter(p => this.files.delete(p));
        const db = await openDb();
        if (db) {
            await new Promise<void>((resolve) => {
                try {
                    const tx = db.transaction(STORE_LIBRARY, 'readwrite');
                    const store = tx.objectStore(STORE_LIBRARY);
                    for (const f of added) store.put(f);
                    for (const p of removed) store.delete(p);
                    tx.oncomplete = () => resolve();
                    tx.onerror = () => resolve();
                    tx.onabort = () => resolve();
                } catch {
                    resolve();
                }
            });
        }
        await this.sync();
    }

    async putText(name: string, text: string): Promise<boolean> {
        return this.put(name, new TextEncoder().encode(text));
    }

    async remove(path: string): Promise<void> {
        const p = libraryPath(path);
        this.files.delete(p);
        await dbRequest(STORE_LIBRARY, 'readwrite', s => s.delete(p));
        await this.sync();
    }

    private async sync(): Promise<void> {
        const files: EngineFile[] = [...this.files.values()].map(f => ({ path: f.path, data: f.data }));
        try {
            await this.transport.setFiles(files);
        } catch {
            // A worker restart re-sends the current files, so a failed sync heals itself.
        }
        for (const l of this.listeners) l();
    }
}

function sameBytes(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
    return true;
}
