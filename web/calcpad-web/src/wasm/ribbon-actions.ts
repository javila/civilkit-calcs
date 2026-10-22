// calcpad-rs: what the ribbon's `action` buttons do. Each key is an `action` id from ribbon-data.ts;
// ids with a colon (`mode:report`, `tab:insert`, `theme:Dark`) share one handler and take the part
// after the colon as their argument.
import type { WebFiles } from './web-files';
import { getResolvedAppTheme } from '../editor/app-theme';

export interface RibbonCtx {
    files: WebFiles;
    /** The mounted App.vue instance (its exposed API). */
    app: any;
    /** The message bridge (settings, theme, exports). */
    bridge: any;
    /** The active Monaco editor, if any. */
    editor: () => any;
    /** Opens a tab of the sidebar. */
    showSidebarTab: (id: string) => void;
    /** The element a dropdown should hang from. */
    anchor: HTMLElement;
}

const DOCS_URL = 'https://github.com/imartincei/CalcpadCE';
const mod = /Mac|iPhone|iPad/.test(navigator.platform) ? 'Cmd' : 'Ctrl';

const edit = (c: RibbonCtx, cmd: string) => {
    const e = c.editor();
    e?.focus();
    e?.trigger('ribbon', cmd, null);
};
const zoom = (c: RibbonCtx, delta: number) => {
    const e = c.editor();
    if (!e) return;
    const cur = e.getOption((window as any).monaco.editor.EditorOption.fontSize) as number;
    e.updateOptions({ fontSize: Math.max(8, Math.min(40, cur + delta)) });
};
const say = (c: RibbonCtx, lines: string[]) => {
    for (const l of lines) c.app.appendOutput?.('info', l);
    c.app.showOutput?.();
};

type Handler = (c: RibbonCtx, arg: string) => void;
export const ACTIONS: Record<string, Handler> = {
    new: c => c.files.newWorksheet(),
    open: c => void c.files.open(),
    save: c => void c.files.save(),
    saveas: c => void c.files.save(true),
    filemenu: c => c.files.toggleMenu(c.anchor),
    samples: c => c.files.o.openSamples?.(),
    pdf: c => c.files.o.print(),
    word: c => c.files.o.exportDocx?.(),
    excel: c => c.files.o.exportXlsx?.(),
    exportTab: c => c.files.o.openExport?.(),
    undo: c => edit(c, 'undo'),
    redo: c => edit(c, 'redo'),
    find: c => edit(c, 'actions.find'),
    cut: c => edit(c, 'editor.action.clipboardCutAction'),
    copy: c => edit(c, 'editor.action.clipboardCopyAction'),
    paste: c => {
        const e = c.editor();
        e?.focus();
        void navigator.clipboard?.readText().then(t => {
            if (t) e?.executeEdits('ribbon', [{ range: e.getSelection(), text: t, forceMoveMarkers: true }]);
        }).catch(() => e?.trigger('ribbon', 'editor.action.clipboardPasteAction', null));
    },
    calc: c => c.app.onRunRequest?.(),
    mode: (c, m) => void c.app.setResultMode(m),
    togglePreview: c => c.app.togglePreview?.(),
    sidebar: c => c.app.toggleSidebar?.(),
    tab: (c, id) => c.showSidebarTab(id),
    settings: c => c.showSidebarTab('settings'),
    theme: (c, t) => c.bridge?.handleMessage({ type: 'updateColorTheme', theme: t }),
    zoomIn: c => zoom(c, 1),
    zoomOut: c => zoom(c, -1),
    autocalc: c => c.bridge?.setExtraSetting('autoRun', c.bridge.getExtraSetting('autoRun') === 'false' ? 'true' : 'false'),
    docs: () => void window.open(DOCS_URL, '_blank', 'noopener'),
    shortcuts: c => say(c, [
        'Keyboard shortcuts',
        `${mod}+O open, ${mod}+S save, ${mod}+Shift+S save as, ${mod}+N new`,
        `F5 or ${mod}+Alt+X calculate`,
        `${mod}+Shift+B show or hide the sidebar`,
        `${mod}+F find, ${mod}+Z undo, ${mod}+Shift+Z redo`,
        'Alt+Z wrap long lines',
    ]),
    about: c => say(c, [
        `CalcpadCE web editor ${import.meta.env.VITE_APP_VERSION ?? ''}`.trim(),
        'The calcpad-rs engine runs in your browser. MIT licence.',
    ]),
};

export const ACTIVE: Record<string, (c: RibbonCtx, arg: string) => boolean> = {
    mode: (c, m) => c.app.getResultMode?.() === m,
    togglePreview: c => !!c.app.isPreviewVisible?.(),
    theme: (_c, t) => getResolvedAppTheme() === t.toLowerCase(),
    autocalc: c => c.bridge?.getExtraSetting('autoRun') !== 'false',
};

export function runAction(id: string, c: RibbonCtx): void {
    const i = id.indexOf(':');
    const key = i < 0 ? id : id.slice(0, i);
    ACTIONS[key]?.(c, i < 0 ? '' : id.slice(i + 1));
}
export function isActive(id: string, c: RibbonCtx): boolean {
    const i = id.indexOf(':');
    const key = i < 0 ? id : id.slice(0, i);
    try {
        return !!ACTIVE[key]?.(c, i < 0 ? '' : id.slice(i + 1));
    } catch {
        return false;
    }
}

/**
 * Inserts text at the editor cursor. A `line` snippet goes on its own line (after the current one when
 * that has content); `{sel}` in the text stands for the selection, or the word "text" when nothing is selected.
 */
export function insertSnippet(editor: any, text: string, line: boolean): boolean {
    const model = editor?.getModel();
    if (!model) return false;
    const sel = editor.getSelection();
    const selected = model.getValueInRange(sel);
    const body = text.replace(/\{sel\}/g, selected || 'text');
    if (line) {
        const ln = sel.endLineNumber;
        const blank = model.getLineContent(ln).trim() === '';
        const col = model.getLineMaxColumn(ln);
        const at = blank ? { startLineNumber: ln, startColumn: 1, endLineNumber: ln, endColumn: col } : { startLineNumber: ln, startColumn: col, endLineNumber: ln, endColumn: col };
        editor.executeEdits('ribbon', [{ range: at, text: (blank ? '' : '\n') + body, forceMoveMarkers: true }]);
    } else {
        editor.executeEdits('ribbon', [{ range: sel, text: body, forceMoveMarkers: true }]);
    }
    editor.focus();
    return true;
}
