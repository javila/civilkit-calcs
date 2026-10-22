// calcpad-rs: the ribbon, as data. Tabs -> groups -> buttons. To add a command, add one entry here;
// nothing else needs to change. A button either inserts text at the editor cursor (`insertText`)
// or calls an existing action (`run`). Buttons without an `icon` render their label as a glyph
// and sit in a compact grid (math symbols, units).
import type { WebFiles } from './web-files';
import { getResolvedAppTheme, setAppTheme } from '../editor/app-theme';

export interface RibbonCtx {
    files: WebFiles;
    /** The mounted App.vue instance (its exposed API). */
    app: any;
    /** The active Monaco editor, if any. */
    editor: () => any;
    /** The element a dropdown should hang from. */
    anchor: HTMLElement;
}

export interface RibbonButton {
    id: string;
    label: string;
    icon?: string;
    title?: string;
    insertText?: string;
    run?: (c: RibbonCtx) => void;
    active?: (c: RibbonCtx) => boolean;
}
export interface RibbonGroup { name: string; buttons: RibbonButton[] }
export interface RibbonTab { id: string; label: string; groups: RibbonGroup[] }

/** Inner SVG markup for 24x24 stroke icons (currentColor). */
export const ICONS: Record<string, string> = {
    file: '<path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4"/>',
    open: '<path d="M3 7h6l2 2h10v10H3z"/>',
    save: '<path d="M5 3h12l2 2v16H5z"/><path d="M8 3v6h8V3M8 21v-7h8v7"/>',
    saveas: '<path d="M5 3h12l2 2v7M5 3v18h7"/><path d="M15 21l1-4 5-5 2 2-5 5z"/>',
    samples: '<rect x="4" y="4" width="7" height="7"/><rect x="13" y="4" width="7" height="7"/><rect x="4" y="13" width="7" height="7"/><rect x="13" y="13" width="7" height="7"/>',
    recent: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    pdf: '<path d="M6 3h8l4 4v14H6z"/><path d="M9 15h6M9 18h4M9 12h2"/>',
    word: '<path d="M6 3h8l4 4v14H6z"/><path d="M8.5 11l1.5 6 2-5 2 5 1.5-6"/>',
    excel: '<path d="M6 3h8l4 4v14H6z"/><path d="M9 11l5 7M14 11l-5 7"/>',
    export: '<path d="M12 3v12M7 8l5-5 5 5M5 14v7h14v-7"/>',
    run: '<path d="M7 4l13 8-13 8z"/>',
    report: '<rect x="5" y="3" width="14" height="18"/><path d="M8 8h8M8 12h8M8 16h5"/>',
    input: '<rect x="3" y="7" width="18" height="10" rx="1"/><path d="M7 12h4"/>',
    preview: '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
    source: '<path d="M8 8l-5 4 5 4M16 8l5 4-5 4M14 5l-4 14"/>',
    undo: '<path d="M9 7L4 12l5 5"/><path d="M4 12h10a6 6 0 010 8"/>',
    redo: '<path d="M15 7l5 5-5 5"/><path d="M20 12H10a6 6 0 000 8"/>',
    find: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6"/>',
    heading: '<path d="M6 4v16M18 4v16M6 12h12"/>',
    comment: '<path d="M4 5h16v11H9l-5 4z"/>',
    ui: '<rect x="3" y="5" width="18" height="14" rx="1"/><path d="M6 10h6M6 14h9"/>',
    plot: '<path d="M4 4v16h16"/><path d="M7 15l4-5 3 3 5-7"/>',
    table: '<rect x="3" y="5" width="18" height="14"/><path d="M3 10h18M3 15h18M9 5v14M15 5v14"/>',
    vector: '<path d="M8 4H5v16h3M16 4h3v16h-3"/><path d="M12 8v8"/>',
    matrix: '<path d="M8 4H5v16h3M16 4h3v16h-3"/><path d="M10 9h1M13 9h1M10 15h1M13 15h1"/>',
    cond: '<path d="M12 3l8 9-8 9-8-9z"/>',
    loop: '<path d="M4 12a8 8 0 0114-5M20 12a8 8 0 01-14 5"/><path d="M18 3v4h-4M6 21v-4h4"/>',
    sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M5 19l2-2M17 7l2-2"/>',
    moon: '<path d="M20 14A8 8 0 0110 4a8 8 0 1010 10z"/>',
    sidebar: '<rect x="3" y="4" width="18" height="16"/><path d="M9 4v16"/>',
    zoomin: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6M10 7v6M7 10h6"/>',
    zoomout: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6M7 10h6"/>',
    menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
};

const ins = (id: string, label: string, text: string, title?: string): RibbonButton => ({ id, label, insertText: text, title: title ?? `Insert ${text.replace(/\n/g, ' ')}` });
const insI = (id: string, label: string, icon: string, text: string, title: string): RibbonButton => ({ id, label, icon, insertText: text, title });
const run = (id: string, label: string, icon: string, title: string, fn: (c: RibbonCtx) => void, active?: (c: RibbonCtx) => boolean): RibbonButton => ({ id, label, icon, title, run: fn, active });
const mode = (id: string, label: string, icon: string, m: string, title: string): RibbonButton =>
    run(id, label, icon, title, c => void c.app.setResultMode(m), c => c.app.getResultMode?.() === m);
const edit = (c: RibbonCtx, cmd: string) => {
    const e = c.editor();
    e?.focus();
    e?.trigger('ribbon', cmd, null);
};
const setFont = (c: RibbonCtx, delta: number) => {
    const e = c.editor();
    if (!e) return;
    const cur = e.getOption((window as any).monaco.editor.EditorOption.fontSize) as number;
    e.updateOptions({ fontSize: Math.max(8, Math.min(40, cur + delta)) });
};

const greek = 'αβγδεζηθικλμνξπρσφχψω'.split('').concat('ΔΣΩΦΨΘΛΠ'.split(''));
const operators: [string, string][] = [['+', '+'], ['−', '-'], ['×', '*'], ['÷', '/'], ['^', '^'], ['=', '='], ['≠', '≠'], ['≤', '≤'], ['≥', '≥'], ['<', '<'], ['>', '>'], ['(', '('], [')', ')'], ['|', '|'], ['\\', '\\'], ['%', '%'], ['∧', '&'], ['∨', '|'], ['≡', '≡'], ['÷', '÷'], ['°', '°'], ['∠', '∠']];
const functions = ['sqrt', 'abs', 'sin', 'cos', 'tan', 'asin', 'acos', 'atan', 'ln', 'log', 'exp', 'round', 'floor', 'ceiling', 'min', 'max', 'sum', 'sumsq', 'product', 'mean', 'if', 'switch', 'root', 'atan2'];
const unitGroups: [string, string[]][] = [
    ['Length', ['mm', 'cm', 'm', 'km', 'in', 'ft']],
    ['Force', ['N', 'kN', 'MN', 'lbf', 'kip']],
    ['Stress and moment', ['Pa', 'kPa', 'MPa', 'GPa', 'psi', 'ksi', 'N*m', 'kN*m', 'kN*m²']],
    ['Mass, time, other', ['kg', 't', 'g', 's', 'min', 'h', '°C', 'deg', 'rad', 'm²', 'm³', 'mm²', 'mm⁴']],
];

export const RIBBON: RibbonTab[] = [
    {
        id: 'file', label: 'File', groups: [
            { name: 'Worksheet', buttons: [
                run('new', 'New', 'file', 'New worksheet', c => c.files.newWorksheet()),
                run('open', 'Open', 'open', 'Open a worksheet (Ctrl+O)', c => void c.files.open()),
                run('save', 'Save', 'save', 'Save (Ctrl+S)', c => void c.files.save()),
                run('saveas', 'Save As', 'saveas', 'Save As (Ctrl+Shift+S)', c => void c.files.save(true)),
            ] },
            { name: 'Browse', buttons: [
                run('samples', 'Samples', 'samples', 'Browse the sample worksheets', c => c.files.o.openSamples?.()),
                run('recent', 'Recent and library', 'recent', 'Recent worksheets, the #include and #read library, add files', c => c.files.toggleMenu(c.anchor)),
            ] },
            { name: 'Export', buttons: [
                run('pdf', 'Print PDF', 'pdf', 'Print the report as a PDF', c => c.files.o.print()),
                run('word', 'Word', 'word', 'Save the report as a Word document', c => c.files.o.exportDocx?.()),
                run('excel', 'Excel', 'excel', 'Save the variables as an Excel workbook', c => c.files.o.exportXlsx?.()),
                run('export', 'Export tab', 'export', 'HTML, PDF, Word, #write: the Export tab', c => c.files.o.openExport?.()),
            ] },
        ],
    },
    {
        id: 'home', label: 'Home', groups: [
            { name: 'Calculate', buttons: [
                run('calc', 'Calculate', 'run', 'Run the worksheet', c => c.app.onRunRequest?.()),
            ] },
            { name: 'Results', buttons: [
                mode('m-preview', 'Preview', 'preview', 'preview', 'The document as written'),
                mode('m-ui', 'Input', 'input', 'ui', 'The #UI input form'),
                mode('m-report', 'Report', 'report', 'report', 'The print layout'),
                run('toggle-preview', 'Show results', 'source', 'Show or hide the results pane', c => c.app.togglePreview?.(), c => !!c.app.isPreviewVisible?.()),
            ] },
            { name: 'Edit', buttons: [
                run('undo', 'Undo', 'undo', 'Undo (Ctrl+Z)', c => edit(c, 'undo')),
                run('redo', 'Redo', 'redo', 'Redo (Ctrl+Shift+Z)', c => edit(c, 'redo')),
                run('find', 'Find', 'find', 'Find in the editor (Ctrl+F)', c => edit(c, 'actions.find')),
            ] },
            { name: 'Files', buttons: [
                run('h-samples', 'Samples', 'samples', 'Browse the sample worksheets', c => c.files.o.openSamples?.()),
                run('h-pdf', 'Print PDF', 'pdf', 'Print the report as a PDF', c => c.files.o.print()),
                run('h-word', 'Word', 'word', 'Save the report as a Word document', c => c.files.o.exportDocx?.()),
                run('h-excel', 'Excel', 'excel', 'Save the variables as an Excel workbook', c => c.files.o.exportXlsx?.()),
            ] },
        ],
    },
    {
        id: 'insert', label: 'Insert', groups: [
            { name: 'Text', buttons: [
                insI('i-heading', 'Heading', 'heading', '"<h2>Heading</h2>\n', 'Insert a heading line'),
                insI('i-comment', 'Comment', 'comment', "'Comment\n", 'Insert a comment line'),
            ] },
            { name: 'Input and output', buttons: [
                insI('i-ui', 'Input', 'ui', 'a = ? {10}\n', 'Insert an input field with a default value'),
                insI('i-plot', 'Plot', 'plot', '$Plot{sin(x) @ x = 0 : 2*pi}\n', 'Insert a plot'),
                insI('i-table', 'Table', 'table', "'<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>\n", 'Insert an HTML table'),
            ] },
            { name: 'Arrays', buttons: [
                insI('i-vector', 'Vector', 'vector', 'v = [1; 2; 3]\n', 'Insert a vector'),
                insI('i-matrix', 'Matrix', 'matrix', 'A = [1; 2| 3; 4]\n', 'Insert a matrix'),
            ] },
            { name: 'Control', buttons: [
                insI('i-if', '#if', 'cond', '#if x > 0\n    \n#else\n    \n#end if\n', 'Insert a conditional block'),
                insI('i-for', '#for', 'loop', '#for i = 1 : 5\n    \n#loop\n', 'Insert a counting loop'),
            ] },
        ],
    },
    {
        id: 'math', label: 'Math', groups: [
            { name: 'Greek', buttons: greek.map(g => ins(`g-${g}`, g, g, `Insert ${g}`)) },
            { name: 'Operators', buttons: operators.map(([l, t], i) => ins(`o-${i}`, l, t, `Insert ${t}`)) },
            { name: 'Functions', buttons: functions.map(f => ins(`f-${f}`, f, `${f}()`, `Insert ${f}()`)) },
        ],
    },
    {
        id: 'units', label: 'Units', groups: unitGroups.map(([name, us]) => ({
            name, buttons: us.map(u => ins(`u-${u}`, u, u, `Insert the unit ${u}`)),
        })),
    },
    {
        id: 'view', label: 'View', groups: [
            { name: 'Theme', buttons: [
                run('light', 'Light', 'sun', 'Light theme', () => setAppTheme('Light'), () => getResolvedAppTheme() === 'light'),
                run('dark', 'Dark', 'moon', 'Dark theme', () => setAppTheme('Dark'), () => getResolvedAppTheme() === 'dark'),
            ] },
            { name: 'Results', buttons: [
                mode('v-preview', 'Preview', 'preview', 'preview', 'The document as written'),
                mode('v-unwrapped', 'Unwrapped', 'source', 'unwrapped', 'The source listing with macros and includes resolved'),
                mode('v-ui', 'Input', 'input', 'ui', 'The #UI input form'),
                mode('v-report', 'Report', 'report', 'report', 'The print layout'),
                run('v-toggle', 'Show results', 'preview', 'Show or hide the results pane', c => c.app.togglePreview?.(), c => !!c.app.isPreviewVisible?.()),
            ] },
            { name: 'Panels', buttons: [
                run('sidebar', 'Sidebar', 'sidebar', 'Show or hide the sidebar (Ctrl+Shift+B)', c => c.app.toggleSidebar?.()),
            ] },
            { name: 'Zoom', buttons: [
                run('zoom-in', 'Zoom in', 'zoomin', 'Larger editor text', c => setFont(c, 1)),
                run('zoom-out', 'Zoom out', 'zoomout', 'Smaller editor text', c => setFont(c, -1)),
            ] },
        ],
    },
];

/** Inserts text at the editor cursor, replacing the selection. */
export function insertAtCursor(editor: any, text: string): boolean {
    if (!editor || !editor.getModel()) return false;
    const range = editor.getSelection();
    editor.executeEdits('ribbon', [{ range, text, forceMoveMarkers: true }]);
    editor.focus();
    return true;
}
