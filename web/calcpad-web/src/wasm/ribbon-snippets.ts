// calcpad-rs: the full snippet library in the ribbon. The sidebar's Insert tab gets its snippets from the
// highlighter module (the /api/calcpad/snippets call); the ribbon asks the same call, so the two cannot
// drift and snippets added to the library appear in both without a change here.
import { buildInsertSnippet, replaceParameterPlaceholders } from 'calcpad-frontend';
import type { InsertItem } from 'calcpad-frontend';
import type { Entry, MoreSpec, Section } from './ribbon-data';

export interface Snip {
    insert: string;
    description: string;
    documentation?: string | null;
    example?: string | null;
    label?: string | null;
    category: string;
    parameters?: { name: string }[] | null;
}

let cache: Promise<Snip[]> | null = null;
export function loadSnippets(): Promise<Snip[]> {
    cache ??= fetch(`${window.location.origin}/api/calcpad/snippets`)
        .then(r => (r.ok ? r.json() : Promise.reject(new Error(`snippets: HTTP ${r.status}`))))
        .then((j: { snippets: Snip[] }) => j.snippets)
        .catch(e => {
            cache = null;
            throw e;
        });
    return cache;
}

const toItem = (s: Snip): InsertItem => ({ tag: s.insert, description: s.description, parameters: s.parameters ?? undefined } as InsertItem);

/** The text a snippet button shows: its label, else the insert text with parameter names in place of the markers. */
export function snipLabel(s: Snip): string {
    const raw = (s.label ?? '').trim() || replaceParameterPlaceholders(s.insert, { parameters: s.parameters ?? undefined });
    const first = raw.split('\n')[0].replace(/\t/g, ' ').trim();
    return first.length > 44 ? `${first.slice(0, 43)}...` : first;
}

/** True when the snippet is a whole line (a directive, a comment, a setting) rather than something inside an expression. */
export function isLineSnippet(s: Snip): boolean {
    return /^[#'"]/.test(s.insert) || s.insert.includes('\n') || /^(Settings|Metadata|SVG|CSS)/.test(s.category);
}

export function snipKey(s: Snip): string {
    return `${s.category}|${s.insert}`;
}

export function snipEntry(s: Snip): Entry {
    const tip = [s.description, s.insert.includes('\n') || s.insert.length > 60 ? '' : replaceParameterPlaceholders(s.insert, { parameters: s.parameters ?? undefined })].filter(Boolean).join('\n');
    return { label: snipLabel(s), text: s.insert, title: tip, line: isLineSnippet(s), snipKey: snipKey(s), search: `${s.description} ${s.insert} ${s.label ?? ''}`.toLowerCase() };
}

function heading(cat: string, spec: MoreSpec): string {
    if (spec.headings?.[cat]) return spec.headings[cat];
    const prefix = spec.categories.find(c => cat === c || cat.startsWith(`${c}/`));
    let rest = prefix && cat !== prefix ? cat.slice(prefix.length + 1) : cat;
    if (prefix && cat === prefix) rest = cat.slice(cat.lastIndexOf('/') + 1);
    // "Units/Metric/Mass" under the prefix "Units" reads "Metric: Mass".
    return rest.split('/').join(': ');
}

/** Sections for one "full list" popover: the snippets of its categories, grouped under headings, in library order. */
export function sectionsFor(all: Snip[], spec: MoreSpec): Section[] {
    const out = new Map<string, Entry[]>();
    for (const s of all) {
        if (!spec.categories.some(c => s.category === c || s.category.startsWith(`${c}/`))) continue;
        const h = heading(s.category, spec);
        if (!out.has(h)) out.set(h, []);
        out.get(h)!.push(snipEntry(s));
    }
    return [...out].map(([h, items]) => ({ heading: h, items }));
}

/**
 * Inserts a library snippet at the cursor with Monaco's snippet engine, so the first marker is selected and
 * Tab moves to the next (the same insertion the sidebar uses). Line snippets start on their own line.
 */
export function insertLibrarySnippet(editor: any, text: string, line: boolean, snip?: Snip): boolean {
    const model = editor?.getModel();
    if (!model) return false;
    const sel = editor.getSelection();
    if (line) {
        const ln = sel.endLineNumber;
        if (model.getLineContent(ln).trim() !== '') {
            const col = model.getLineMaxColumn(ln);
            editor.executeEdits('ribbon', [{ range: { startLineNumber: ln, startColumn: col, endLineNumber: ln, endColumn: col }, text: '\n', forceMoveMarkers: true }]);
            editor.setPosition({ lineNumber: ln + 1, column: 1 });
        }
    }
    editor.focus();
    const str = buildInsertSnippet(toItem({ ...(snip ?? { description: '', category: '' }), insert: text } as Snip));
    const ctl = editor.getContribution?.('snippetController2');
    if (ctl?.insert) ctl.insert(str);
    else editor.executeEdits('ribbon', [{ range: editor.getSelection(), text: text.replace(/§/g, ''), forceMoveMarkers: true }]);
    return true;
}
