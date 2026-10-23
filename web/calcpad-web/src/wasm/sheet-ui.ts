// calcpad-rs: the sheet syntax commands (docs/sheet-syntax.md): New sheet, Show compiled Calcpad
// and Convert to Calcpad. A sheet is Markdown-like text that compiles to the Calcpad worksheet
// language (sheet.ts); these commands let a user start one, see the Calcpad it becomes, and leave
// the sheet syntax for plain Calcpad when they want to.
import * as monaco from 'monaco-editor';
import { compileSheet, isSheetText, SHEET_MARKER } from './sheet';
import type { TabsLike } from './web-files';

export const NEW_SHEET_TEXT = `${SHEET_MARKER}
# Untitled calculation

Say what is being checked. Prose is Markdown: **bold**, lists, tables and $x^2$ formulas.

## Input
w: 12 kN/m | design load

## Calculation
M: w * (6 m)^2 / 8 -> kN*m | bending moment of a simply supported 6 m span
`;

/** The first tab of a fresh start: a sheet that says what a sheet is and where the other syntax lives. */
export const WELCOME_SHEET_TEXT = `${SHEET_MARKER}
# Welcome to CivilKit Calcs

This is a sheet: Markdown prose and \`name: value unit | note\` lines, run by the Calcpad engine.
For the advanced Calcpad syntax use **File > New Calcpad worksheet**; a sheet can also hold
Calcpad lines, written after \`>>\`.

## Input
a: 3 m | first side
b: 4 m | second side

## Calculation
c: sqrt(a^2 + b^2) | hypotenuse
`;

/** Opens a new untitled sheet. */
export function newSheet(tabs: TabsLike): void {
    tabs.newUntitled(NEW_SHEET_TEXT);
}

function modal(title: string, sub: string): { box: HTMLDivElement; bar: HTMLDivElement; close: () => void } {
    document.getElementById('cprs-compiled')?.remove();
    const back = document.createElement('div');
    back.id = 'cprs-compiled';
    back.style.cssText = 'position:fixed;inset:0;z-index:2000;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.35)';
    const box = document.createElement('div');
    box.setAttribute('role', 'dialog');
    box.setAttribute('aria-label', title);
    box.style.cssText = 'width:min(980px,94vw);height:min(720px,88vh);display:flex;flex-direction:column;border-radius:6px;font:13px system-ui,sans-serif;' +
        'background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 28px rgba(0,0,0,.35)';
    const head = document.createElement('div');
    head.style.cssText = 'display:flex;align-items:center;gap:12px;padding:8px 12px;border-bottom:1px solid var(--vscode-panel-border,#ddd)';
    const h = document.createElement('b');
    h.textContent = title;
    const s = document.createElement('span');
    s.style.cssText = 'opacity:.7;font-size:12px;flex:1';
    s.textContent = sub;
    const bar = document.createElement('div');
    bar.style.cssText = 'display:flex;gap:8px';
    const x = document.createElement('button');
    x.textContent = 'Close';
    x.style.cssText = 'padding:3px 12px;cursor:pointer;font:inherit';
    head.append(h, s, bar, x);
    box.append(head);
    back.append(box);
    document.body.append(back);
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') { e.stopPropagation(); close(); } };
    const close = () => { back.remove(); document.removeEventListener('keydown', onKey, true); };
    x.onclick = close;
    back.onmousedown = (e) => { if (e.target === back) close(); };
    document.addEventListener('keydown', onKey, true);
    return { box, bar, close };
}

/** A read-only view of the Calcpad the active sheet compiles to; line numbers are the sheet's lines. */
export async function showCompiled(editor: monaco.editor.IStandaloneCodeEditor | null | undefined): Promise<void> {
    const model = editor?.getModel();
    if (!model || !isSheetText(model.getValue())) {
        const { box, close } = modal('Show compiled Calcpad', 'The active file is not a sheet.');
        const p = document.createElement('p');
        p.style.padding = '12px';
        p.textContent = `A sheet starts with the line ${SHEET_MARKER}. Use File > New sheet to start one.`;
        box.append(p);
        void close;
        return;
    }
    const c = await compileSheet(model.getValue(), true);
    const { box, bar } = modal('Compiled Calcpad', 'Read only. The line numbers are the lines of your sheet.');
    const holder = document.createElement('div');
    holder.style.cssText = 'flex:1;min-height:0';
    holder.setAttribute('data-testid', 'compiled-view');
    box.append(holder);
    const copy = document.createElement('button');
    copy.textContent = 'Copy';
    copy.style.cssText = 'padding:3px 12px;cursor:pointer;font:inherit';
    copy.onclick = () => void navigator.clipboard?.writeText(c.text);
    bar.append(copy);
    const view = monaco.editor.create(holder, {
        value: c.text,
        language: 'calcpad',
        readOnly: true,
        minimap: { enabled: false },
        automaticLayout: true,
        scrollBeyondLastLine: false,
        lineNumbers: (n: number) => {
            const src = c.map[n - 1];
            const prev = c.map[n - 2];
            return src === undefined ? '' : src === prev ? '' : String(src + 1);
        },
        lineNumbersMinChars: 4,
        wordWrap: 'on',
    });
    // The dialog owns the editor: dispose it with the dialog.
    const obs = new MutationObserver(() => {
        if (!document.body.contains(holder)) { view.dispose(); obs.disconnect(); }
    });
    obs.observe(document.body, { childList: true });
}

/** Replaces the active sheet with the Calcpad it compiles to (undoable). */
export async function convertToCalcpad(editor: monaco.editor.IStandaloneCodeEditor | null | undefined): Promise<boolean> {
    const model = editor?.getModel();
    if (!editor || !model || !isSheetText(model.getValue())) return false;
    const c = await compileSheet(model.getValue(), true);
    editor.executeEdits('convert-sheet', [{ range: model.getFullModelRange(), text: c.text, forceMoveMarkers: true }]);
    editor.pushUndoStop();
    return true;
}
