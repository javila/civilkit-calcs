// calcpad-rs: the script library list. Calcpad passes comment HTML through, so a worksheet can
// carry <script> tags. In the browser build only the files on this list may load, each from its
// exact URL and checked against its SRI hash (script-policy.ts). The built-in entries are the
// libraries the upstream examples use (library-defaults.json, made by
// tools/gen_library_defaults.mjs); the user can remove and restore them and add their own.
// The list is stored with the other app settings (localStorage, calcpad-* keys).

import DEFAULTS from './library-defaults.json';

export interface LibraryFileRef {
    url: string;
    /** "sha384-..." */
    integrity: string;
    size?: number;
    /** An ES module (loaded with import or <script type="module">). */
    module?: boolean;
}

export interface ScriptLibrary {
    id: string;
    name: string;
    version: string;
    files: LibraryFileRef[];
    /** One of the pre-loaded entries. */
    builtin?: boolean;
}

interface Stored {
    removed: string[];
    added: ScriptLibrary[];
}

const KEY = 'calcpad-libraries';
const BUILTIN: ScriptLibrary[] = DEFAULTS as ScriptLibrary[];
const listeners: (() => void)[] = [];

function read(): Stored {
    try {
        const v = JSON.parse(localStorage.getItem(KEY) ?? 'null');
        if (v && Array.isArray(v.removed) && Array.isArray(v.added)) return v;
    } catch {
        // Storage unavailable or corrupt: fall back to the built-in list.
    }
    return { removed: [], added: [] };
}

function write(s: Stored): void {
    try {
        localStorage.setItem(KEY, JSON.stringify(s));
    } catch {
        // The change holds for this page only.
        memory = s;
    }
    for (const l of listeners) l();
}

let memory: Stored | null = null;
const current = (): Stored => memory ?? read();

/** The entries in effect: the built-in ones not removed, then the user's own. */
export function libraries(): ScriptLibrary[] {
    const s = current();
    return [...BUILTIN.filter(b => !s.removed.includes(b.id)), ...s.added];
}

/** Built-in entries the user removed. */
export function removedBuiltins(): ScriptLibrary[] {
    const s = current();
    return BUILTIN.filter(b => s.removed.includes(b.id));
}

export function onLibrariesChanged(listener: () => void): () => void {
    listeners.push(listener);
    return () => {
        const i = listeners.indexOf(listener);
        if (i >= 0) listeners.splice(i, 1);
    };
}

export function removeLibrary(id: string): void {
    const s = current();
    if (BUILTIN.some(b => b.id === id)) {
        if (!s.removed.includes(id)) s.removed.push(id);
    } else {
        s.added = s.added.filter(a => a.id !== id);
    }
    write(s);
}

export function restoreLibrary(id: string): void {
    const s = current();
    s.removed = s.removed.filter(r => r !== id);
    write(s);
}

export function addLibrary(lib: ScriptLibrary): void {
    const s = current();
    s.added = [...s.added.filter(a => a.id !== lib.id), { ...lib, builtin: false }];
    write(s);
}

/** sha384 of `bytes` in SRI form. */
export async function sriHash(bytes: ArrayBuffer): Promise<string> {
    const digest = new Uint8Array(await crypto.subtle.digest('SHA-384', bytes));
    let bin = '';
    for (const b of digest) bin += String.fromCharCode(b);
    return 'sha384-' + btoa(bin);
}

/** A name and version guessed from a CDN URL, for the add form to start from. */
export function guessNameVersion(url: string): { name: string; version: string } {
    const m = /\/npm\/(@?[^@/]+(?:\/[^@/]+)?)@([^/]+)/.exec(url)
        ?? /\/libs\/([^/]+)\/([^/]+)\//.exec(url)
        ?? /\/([a-z][\w.-]*?)-(\d[\w.]*?)(?:\.min)?\.js$/i.exec(url);
    if (m) return { name: m[1], version: m[2] };
    const file = url.split('?')[0].split('/').pop() ?? url;
    return { name: file.replace(/(\.min)?\.m?js$/, ''), version: '' };
}

/** The lines Insert > Libraries adds for an entry, as Calcpad comment lines. */
export function insertText(lib: ScriptLibrary): string {
    const ident = lib.name.replace(/\(.*\)/, '').replace(/[^A-Za-z0-9]+/g, ' ').trim().split(' ')[0] || 'lib';
    const main = lib.files[0];
    if (main?.module && main.url.endsWith('+esm')) {
        return `'<script type="module">import * as ${ident.replace(/^\d/, '_$&')} from "${main.url}";</script>\n`;
    }
    return lib.files
        .filter(f => !f.module || !f.url.endsWith('+esm'))
        .map(f => `'<script${f.module ? ' type="module"' : ''} src="${f.url}"></script>\n`)
        .join('');
}
