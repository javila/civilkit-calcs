// calcpad-rs: the ribbon, as data. Tabs -> groups -> buttons. To add a command, add one entry here;
// nothing else needs to change. This file is plain data (no imports of app code) so that
// e2e/verify-ribbon-snippets.mjs can load it under node and run every snippet through the engine.
//
// A button is one of:
//   text      inserts text at the editor cursor (`line: true` puts it on its own line)
//   action    names an entry in ribbon-actions.ts
//   palette   opens a popover of glyph buttons grouped under headings
// `size: 'small'` buttons stack three to a column; the rest are big.
//
// Verification (verify-ribbon-snippets.mjs): a line snippet is run after PRELUDE as written; an inline
// snippet is substituted for `{}` in `test` (default `r = {}`). `nox` runs without a variable named x
// (plots and numerical methods use x as their own variable); `raw` runs with no prelude at all;
// `skip` lists a snippet the engine cannot check on its own, with the reason.

export interface Entry {
    label: string;
    text: string;
    title?: string;
    line?: boolean;
    test?: string;
    nox?: boolean;
    raw?: boolean;
    /** Verify with complex numbers switched on (the real-number default rejects them). */
    cx?: boolean;
    skip?: string;
}
export interface Section { heading: string; items: Entry[] }
export interface RibbonButton {
    id: string;
    label: string;
    icon?: string;
    title?: string;
    size?: 'big' | 'small';
    text?: string;
    line?: boolean;
    test?: string;
    nox?: boolean;
    raw?: boolean;
    skip?: string;
    action?: string;
    palette?: Section[];
}
export interface RibbonGroup { name: string; buttons: RibbonButton[] }
export interface RibbonTab { id: string; label: string; groups: RibbonGroup[] }

/** Defined before every verified snippet, so that snippets can name ordinary variables. */
export const PRELUDE = 'x = 0.5\ny = 3\nn = 3\nv = [1; 2; 3]\nw = [4; 5; 6]\nu = [1; 2]\nA = [1; 2| 3; 4]\nB = [5; 6| 7; 8]\nS = symmetric(2)\nS.(1; 1) = 4\nS.(1; 2) = 1\nS.(2; 2) = 3\nf(t) = t^2\n';
export const PRELUDE_NOX = 'y = 3\nn = 3\nv = [1; 2; 3]\nw = [4; 5; 6]\nu = [1; 2]\nA = [1; 2| 3; 4]\nB = [5; 6| 7; 8]\nS = symmetric(2)\nS.(1; 1) = 4\nS.(1; 2) = 1\nS.(2; 2) = 3\nf(x) = x^2\n';
/** Files the verifier writes next to the snippet, for #read and #include. */
export const VERIFY_FILES: Record<string, string> = {
    'data.csv': '1,2\n3,4\n',
    'module.cpd': 'm_1 = 5\n',
};

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
    text: '<path d="M5 5h14M12 5v15M9 20h6"/>',
    mathline: '<path d="M4 8h9M4 16h9"/><path d="M16 9l4 6M20 9l-4 6"/>',
    image: '<rect x="3" y="5" width="18" height="14"/><circle cx="9" cy="10" r="1.5"/><path d="M4 18l5-5 4 4 3-3 4 4"/>',
    ui: '<rect x="3" y="5" width="18" height="14" rx="1"/><path d="M6 10h6M6 14h9"/>',
    plot: '<path d="M4 4v16h16"/><path d="M7 15l4-5 3 3 5-7"/>',
    map: '<rect x="4" y="4" width="16" height="16"/><path d="M4 10h16M4 15h16M10 4v16M15 4v16"/><path d="M4 20L20 4" opacity=".5"/>',
    table: '<rect x="3" y="5" width="18" height="14"/><path d="M3 10h18M3 15h18M9 5v14M15 5v14"/>',
    vector: '<path d="M8 4H5v16h3M16 4h3v16h-3"/><path d="M12 8v8"/>',
    matrix: '<path d="M8 4H5v16h3M16 4h3v16h-3"/><path d="M10 9h1M13 9h1M10 15h1M13 15h1"/>',
    cond: '<path d="M12 3l8 9-8 9-8-9z"/>',
    loop: '<path d="M4 12a8 8 0 0114-5M20 12a8 8 0 01-14 5"/><path d="M18 3v4h-4M6 21v-4h4"/>',
    program: '<path d="M9 4C6 4 7 9 5 12c2 3 1 8 4 8M15 4c3 0 2 5 4 8-2 3-1 8-4 8"/>',
    operators: '<path d="M12 5v10M7 10h10M7 19h10"/>',
    symbols: '<path d="M6 18c-3-4-2-10 3-10s5 6 9 6M13 6c2 4 0 12 5 12"/>',
    constants: '<path d="M5 7h14M9 7v11M15 7c0 6 0 9 3 11"/>',
    units: '<path d="M3 15h18v4H3z"/><path d="M6 15v-3M10 15v-2M14 15v-3M18 15v-2"/>',
    cube: '<path d="M12 3l8 4v10l-8 4-8-4V7z"/><path d="M12 12l8-5M12 12v9M12 12L4 7"/>',
    fx: '<path d="M10 4C7 4 8 9 8 12s-1 8-4 8M5 11h6"/><path d="M13 12l6 6M19 12l-6 6"/>',
    sub: '<path d="M4 6l8 10M12 6L4 16"/><path d="M15 17h5v4h-5z"/>',
    split: '<path d="M4 8h16M4 16h8"/><path d="M16 14l3 2-3 2"/>',
    wrap: '<path d="M4 8h16v5H8"/><path d="M11 10l-3 3 3 3"/>',
    cut: '<circle cx="7" cy="17" r="3"/><circle cx="17" cy="17" r="3"/><path d="M8 14L18 4M16 14L6 4"/>',
    copy: '<rect x="8" y="8" width="12" height="12"/><path d="M16 8V4H4v12h4"/>',
    paste: '<rect x="5" y="5" width="14" height="16"/><path d="M9 5V3h6v2M9 11h6M9 15h6"/>',
    entry: '<rect x="3" y="8" width="18" height="8" rx="1"/><path d="M7 12h5"/>',
    dropdown: '<rect x="3" y="6" width="18" height="6" rx="1"/><path d="M15 8l2 2 2-2M3 16h18M3 20h18"/>',
    radio: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3.5" fill="currentColor"/>',
    checkbox: '<rect x="4" y="4" width="16" height="16" rx="2"/><path d="M8 12l3 3 5-6"/>',
    grid: '<rect x="3" y="4" width="18" height="16"/><path d="M3 9h18M3 14h18M9 4v16M15 4v16"/>',
    read: '<path d="M12 3v12M7 10l5 5 5-5M5 19h14"/>',
    write: '<path d="M12 15V3M7 8l5-5 5 5M5 19h14"/>',
    append: '<path d="M12 4v10M8 10l4 4 4-4M4 18h16M12 18v3"/>',
    pause: '<rect x="6" y="5" width="4" height="14"/><rect x="14" y="5" width="4" height="14"/>',
    round: '<path d="M4 17h4M10 17h2M14 17h6"/><path d="M6 12V6l-2 2M12 12c0-4 4-4 4 0s-4 4-4 0"/>',
    angle: '<path d="M4 19h16M4 19L16 7"/><path d="M11 19a7 7 0 00-2-5"/>',
    hide: '<path d="M3 3l18 18M2 12s4-7 10-7c2 0 3.700.6 5 1.500M22 12s-4 7-10 7c-2 0-3.700-.6-5-1.500"/>',
    show: '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
    bold: '<path d="M7 4h6a4 4 0 010 8H7zM7 12h7a4 4 0 010 8H7z"/>',
    italic: '<path d="M10 4h8M6 20h8M14 4l-4 16"/>',
    underline: '<path d="M7 4v7a5 5 0 0010 0V4M5 21h14"/>',
    strike: '<path d="M4 12h16M16 6c-1-2-7-3-8 1s6 4 8 7-4 5-8 3"/>',
    list: '<path d="M9 6h11M9 12h11M9 18h11"/><circle cx="4.500" cy="6" r="1"/><circle cx="4.500" cy="12" r="1"/><circle cx="4.500" cy="18" r="1"/>',
    olist: '<path d="M10 6h10M10 12h10M10 18h10M4 5h2v4M4 12h3l-3 3h3M4 18h3v3H4"/>',
    br: '<path d="M4 6h16M4 12h12a3 3 0 010 6H12"/><path d="M14 16l-2 2 2 2"/>',
    hr: '<path d="M3 12h18M7 7h10M7 17h10" opacity=".5"/><path d="M3 12h18"/>',
    colour: '<path d="M12 3C8 9 6 12 6 15a6 6 0 0012 0c0-3-2-6-6-12z"/>',
    page: '<path d="M6 3h12v7H6zM6 14h12v7H6z"/><path d="M3 12h18" stroke-dasharray="2 2"/>',
    sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M5 19l2-2M17 7l2-2"/>',
    moon: '<path d="M20 14A8 8 0 0110 4a8 8 0 1010 10z"/>',
    sidebar: '<rect x="3" y="4" width="18" height="16"/><path d="M9 4v16"/>',
    zoomin: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6M10 7v6M7 10h6"/>',
    zoomout: '<circle cx="10" cy="10" r="6"/><path d="M15 15l6 6M7 10h6"/>',
    settings: '<circle cx="12" cy="12" r="3"/><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.500 5.500l2 2M16.500 16.500l2 2M5.500 18.500l2-2M16.500 7.500l2-2"/>',
    auto: '<path d="M7 4l10 8-10 8z"/><path d="M19 5v6M16 8h6" />',
    help: '<circle cx="12" cy="12" r="9"/><path d="M9.500 9.500a2.500 2.500 0 115 0c0 2-2.500 2-2.500 4M12 17h.01"/>',
    keyboard: '<rect x="2" y="6" width="20" height="12" rx="1"/><path d="M6 10h1M10 10h1M14 10h1M18 10h1M7 14h10"/>',
    info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7h.01"/>',
    link: '<path d="M10 14a4 4 0 005 0l3-3a4 4 0 00-5-5l-1 1M14 10a4 4 0 00-5 0l-3 3a4 4 0 005 5l1-1"/>',
    fold: '<path d="M4 6h16M4 12h8M4 18h16M16 10l3 3-3 3"/>',
    complex: '<path d="M4 20V8l8 12V8M16 20l4-12"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
    func: '<path d="M14 4c-4 0-3 5-3.500 8S9 20 5 20M6 11h8"/>',
    stat: '<path d="M4 20V10M10 20V4M16 20v-8M22 20H2"/>',
    sigma: '<path d="M18 5H6l7 7-7 7h12"/>',
    trig: '<path d="M2 12c3-9 5-9 8 0s5 9 8 0 3-6 4-6"/>',
};

type Item = [label: string, text: string, title?: string];
const sec = (heading: string, items: Item[], o: Partial<Entry> = {}): Section => ({
    heading,
    items: items.map(([label, text, title]) => ({ label, text, title, ...o })),
});
/** Inline glyphs, tested as `r = <template>` with `{}` for the glyph. */
const op = (heading: string, items: Item[], test = 'r = x {} y'): Section => sec(heading, items, { test });
const lines = (heading: string, items: Item[], o: Partial<Entry> = {}): Section => sec(heading, items, { line: true, ...o });
const fn = (heading: string, names: string[], o: Partial<Entry> = {}): Section =>
    sec(heading, names.map((s): Item => [s.slice(0, s.indexOf('(')), s]), o);
const big = (id: string, label: string, icon: string, title: string, b: Partial<RibbonButton>): RibbonButton => ({ id, label, icon, title, ...b });
const small = (id: string, label: string, icon: string, title: string, b: Partial<RibbonButton>): RibbonButton => ({ id, label, icon, title, size: 'small', ...b });
const pal = (id: string, label: string, icon: string, title: string, palette: Section[], size?: 'small'): RibbonButton => ({ id, label, icon, title, palette, size });
const action = (id: string, label: string, icon: string, title: string, a = id, size?: 'small'): RibbonButton => ({ id, label, icon, title, action: a, size });

const greekLower = 'αβγδεζηθικλμνξοπρστυφχψω'.split('');
const greekUpper = 'ΓΔΘΛΞΠΣΦΨΩ'.split('');

export const QUICK: RibbonButton[] = [
    action('q-new', 'New', 'file', 'New worksheet (Ctrl+N)', 'new'),
    action('q-open', 'Open', 'open', 'Open a worksheet (Ctrl+O)', 'open'),
    action('q-save', 'Save', 'save', 'Save (Ctrl+S)', 'save'),
    action('q-undo', 'Undo', 'undo', 'Undo (Ctrl+Z)', 'undo'),
    action('q-redo', 'Redo', 'redo', 'Redo (Ctrl+Shift+Z)', 'redo'),
];

export const RIBBON: RibbonTab[] = [
    {
        id: 'math', label: 'Math', groups: [
            { name: 'Regions', buttons: [
                big('r-math', 'Math line', 'mathline', 'Insert a line of math', { text: 'a = 1\n', line: true }),
                big('r-text', 'Text', 'comment', 'Insert a comment line (text in single quotes)', { text: "'Text\n", line: true }),
                big('r-heading', 'Heading', 'heading', 'Insert a heading line (text in double quotes)', { text: '"Heading\n', line: true }),
                big('r-image', 'Image', 'image', 'Insert an image reference', { text: '\'<img src="./image.png" alt="Image">\n', line: true }),
                big('r-plot', 'Plot', 'plot', 'Insert a function plot', { text: '$Plot{sin(x) @ x = 0 : 2*π}\n', line: true, nox: true }),
                big('r-table', 'Table', 'table', 'Insert an HTML table', { text: "'<table class=\"bordered\"><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>\n", line: true }),
            ] },
            { name: 'Palettes', buttons: [
                pal('p-operators', 'Operators', 'operators', 'Operators', [
                    op('Arithmetic', [['+', '+'], ['−', '-', 'Subtract'], ['×', '*', 'Multiply'], ['/', '/', 'Divide'], ['÷', '÷', 'Division bar'], ['\\', '\\', 'Integer division'], ['mod', '⦼', 'Modulo'], ['xʸ', '^', 'Power']]),
                    sec('', [['n!', '!', 'Factorial']], { test: 'r = n{}' }),
                    op('Comparison', [['≡', '≡', 'Equal to'], ['≠', '≠'], ['<', '<'], ['>', '>'], ['≤', '≤'], ['≥', '≥']]),
                    op('Logic', [['∧', '∧', 'AND'], ['∨', '∨', 'OR'], ['⊕', '⊕', 'XOR']]),
                    sec('Power and root', [['√', 'sqrt(x)', 'Square root'], ['∛', 'cbrt(x)', 'Cube root'], ['ⁿ√', 'root(x; n)', 'n-th root'], ['|x|', 'abs(x)', 'Absolute value'], ['x²', 'x^2', 'Square'], ['eˣ', 'exp(x)', 'Exponential']]),
                    lines('Sums and calculus', [
                        ['Σ', '$Sum{k @ k = 1 : n}\n', 'Sum'], ['Π', '$Product{k @ k = 1 : n}\n', 'Product'],
                        ['∫', '$Integral{f(x) @ x = 0 : 1}\n', 'Integral'], ['Area', '$Area{f(x) @ x = 0 : 1}\n', 'Area under a curve'],
                        ['d/dx', '$Derivative{f(x) @ x = 1}\n', 'Derivative'], ['Slope', '$Slope{f(x) @ x = 1}\n', 'Numerical slope'],
                        ['Root', '$Root{f(x) = 4 @ x = 0 : 5}\n', 'Solve f(x) = value'], ['Min', '$Inf{f(x) @ x = 0 : 5}\n', 'Minimum'], ['Max', '$Sup{f(x) @ x = 0 : 5}\n', 'Maximum'],
                    ], { nox: true }),
                ]),
                pal('p-symbols', 'Symbols', 'symbols', 'Greek letters, subscripts and other symbols', [
                    { heading: 'Greek lower case', items: greekLower.map((g): Entry => ({ label: g, text: g, test: g === 'π' ? 'r = {}' : '{} = 1' })) },
                    sec('Greek upper case', greekUpper.map((g): Item => [g, g]), { test: '{} = 1' }),
                    sec('Subscripts', ['_0', '_1', '_2', '_i', '_x', '_y', '_z', '_max', '_min', '_d', '_k', '_cr'].map((s): Item => [s, s]), { test: 'a{} = 1' }),
                    sec('Other', [['°', '°', 'Degree'], ['%', '%', 'Percent']], { test: 'r = 5{}' }),
                    sec('', [['∠', '∠', 'Phasor angle (complex numbers)']], { test: 'r = 5{}30', cx: true, raw: true }),
                ]),
                pal('p-programming', 'Programming', 'program', 'Conditions, loops, macros and blocks', [
                    lines('Conditions', [
                        ['#if', '#if x > 0\n    \n#end if\n', 'If'],
                        ['#if / #else', '#if x > 0\n    \n#else\n    \n#end if\n', 'If and else'],
                        ['#else if', '#if x > 5\n    \n#else if x > 0\n    \n#else\n    \n#end if\n', 'If, else if and else'],
                    ]),
                    lines('Loops', [
                        ['#for', '#for k = 1 : 5\n    \n#loop\n', 'Counting loop'],
                        ['#while', '#while x < 5\n    x = x + 1\n#loop\n', 'Loop while a condition holds'],
                        ['#repeat', '#repeat 3\n    \n#loop\n', 'Repeat a number of times'],
                        ['#break', '#break\n', 'Leave the loop', ], ['#continue', '#continue\n', 'Next iteration'],
                    ], { test: '#repeat 3\n    #if x > 1\n        {}\n    #end if\n#loop' }),
                    lines('Macros and definitions', [
                        ['#def', '#def name$ = text\n', 'Inline string macro'],
                        ['#def block', '#def block$\n    \n#end def\n', 'Multi-line macro'],
                        ['#const', '#const c_1 = 1\n', 'Read-only variable'],
                        ['$Block', '$Block{k = 1; k + 1}\n', 'Block of expressions'],
                    ]),
                ]),
                pal('p-constants', 'Constants', 'constants', 'Constants', [
                    sec('Math', [['π', 'π', 'Pi'], ['e', 'e', 'Euler number'], ['2π', '2*π'], ['π/2', 'π/2']]),
                    lines('Physics', [
                        ['c', 'c = 299792458m/s\n', 'Speed of light'],
                        ['g', 'g = 9.80665m/s^2\n', 'Standard gravity'],
                        ['G', 'G = 6.6743*10^-11m^3/(kg*s^2)\n', 'Gravitational constant'],
                        ['h', 'h_P = 6.62607015*10^-34J*s\n', 'Planck constant'],
                        ['k_B', 'k_B = 1.380649*10^-23J/K\n', 'Boltzmann constant'],
                        ['N_A', 'N_A = 6.02214076*10^23/mol\n', 'Avogadro constant'],
                        ['R', 'R = 8.314462618J/(mol*K)\n', 'Gas constant'],
                        ['ε₀', 'ε_0 = 8.8541878128*10^-12F/m\n', 'Vacuum permittivity'],
                        ['μ₀', 'μ_0 = 1.25663706212*10^-6N/A^2\n', 'Vacuum permeability'],
                        ['q_e', 'q_e = 1.602176634*10^-19C\n', 'Elementary charge'],
                        ['σ', 'σ_SB = 5.670374419*10^-8W/(m^2*K^4)\n', 'Stefan-Boltzmann constant'],
                    ]),
                    lines('Engineering', [
                        ['E steel', 'E_s = 200GPa\n', 'Elastic modulus of steel'],
                        ['ρ steel', 'ρ_s = 7850kg/m^3\n', 'Density of steel'],
                        ['ρ water', 'ρ_w = 1000kg/m^3\n', 'Density of water'],
                        ['p atm', 'p_atm = 101325Pa\n', 'Standard atmosphere'],
                    ]),
                ]),
            ] },
            { name: 'Units', buttons: [
                pal('p-units', 'Units', 'units', 'Units', [
                    sec('Length', ['mm', 'cm', 'm', 'km', 'in', 'ft', 'yd', 'mi'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Area and volume', [['m²', 'm^2'], ['mm²', 'mm^2'], ['m³', 'm^3'], ['L', 'L'], ['ha', 'ha'], ['mm⁴', 'mm^4']], { test: 'r = 2{}' }),
                    sec('Mass', ['g', 'kg', 't', 'lb'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Time', ['s', 'min', 'h', 'd'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Force', ['N', 'kN', 'MN', 'lbf', 'kip'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Stress and pressure', ['Pa', 'kPa', 'MPa', 'GPa', 'psi', 'ksi'].map((u): Item => [u, u]), { test: 'r = 2{}' }),
                    sec('Moment and energy', [['N·m', 'N*m'], ['kN·m', 'kN*m'], ['J', 'J'], ['kJ', 'kJ'], ['W', 'W'], ['kW', 'kW']], { test: 'r = 2{}' }),
                    sec('Angle', [['°', '°'], ['deg', 'deg'], ['rad', 'rad']], { test: 'r = 2{}' }),
                    sec('Temperature', [['°C', '°C'], ['K', 'K'], ['°F', '°F']], { test: 'r = 2{}' }),
                    sec('Conversion', [['|', ' | ', 'Convert to the unit that follows']], { test: 'r = 2m{}cm' }),
                ]),
                pal('p-baseunits', 'Base units', 'cube', 'The base units', [
                    sec('Base units', [['kg', 'kg', 'Mass'], ['m', 'm', 'Length'], ['s', 's', 'Time'], ['A', 'A', 'Electric current'], ['°C', '°C', 'Temperature'], ['mol', 'mol', 'Amount of substance'], ['cd', 'cd', 'Luminous intensity']], { test: 'r = 2{}' }),
                ]),
            ] },
            { name: 'Style', buttons: [
                small('s-sub', 'Subscript', 'sub', 'Subscript: _ after a name', { text: '_', test: 'a{}1 = 1' }),
                small('s-index', 'Index', 'vector', 'Element index: a dot and the index, as in v.2', { text: '.1', test: 'r = v{}' }),
                small('s-mindex', 'Matrix index', 'matrix', 'Matrix element: a dot and (row; column)', { text: '.(1; 2)', test: 'r = A{}' }),
                small('s-split', 'Split equation', 'split', 'Split equations after the = sign', { text: '#split\n', line: true }),
                small('s-wrap', 'Wrap equation', 'wrap', 'Wrap equations at the end of the line', { text: '#wrap\n', line: true }),
            ] },
            { name: 'Clipboard', buttons: [
                action('c-cut', 'Cut', 'cut', 'Cut the selection', 'cut', 'small'),
                action('c-copy', 'Copy', 'copy', 'Copy the selection', 'copy', 'small'),
                action('c-paste', 'Paste', 'paste', 'Paste', 'paste', 'small'),
            ] },
        ],
    },
    {
        id: 'io', label: 'Input/Output', groups: [
            { name: 'Input fields', buttons: [
                big('u-entry', 'Entry', 'entry', 'An input box for a number', { text: '#UI L = 10m\n', line: true }),
                big('u-dropdown', 'Dropdown', 'dropdown', 'A drop-down list of choices', { text: '#UI {"type": "dropdown", "keys": ["Low", "Medium", "High"], "values": ["1", "2", "3"]} grade = 1\n', line: true }),
                big('u-radio', 'Radio', 'radio', 'A group of radio buttons', { text: '#UI {"type": "radio", "keys": ["Steel", "Concrete"], "values": ["200GPa", "25GPa"]} E = 200GPa\n', line: true }),
                big('u-check', 'Checkbox', 'checkbox', 'A checkbox (1 or 0)', { text: '#UI {"type": "checkbox"} useSteel = 1\n', line: true }),
                big('u-vector', 'Vector', 'vector', 'An editable vector', { text: '#UI v_1 = [1; 2; 3]\n', line: true }),
                big('u-matrix', 'Matrix', 'matrix', 'An editable matrix', { text: '#UI M_1 = [1; 2| 3; 4]\n', line: true }),
                big('u-grid', 'Data grid', 'grid', 'An empty editable grid', { text: '#UI G_1 = matrix(3; 3)\n', line: true }),
                small('u-label', 'Labelled', 'comment', 'An entry with a label', { text: "#UI 'Span, 'L_2 = 6m\n", line: true }),
                small('u-expr', 'Expression', 'fx', 'An entry that accepts a formula', { text: '#UI {"allowExpression": true} A_1 = 2*3\n', line: true }),
            ] },
            { name: 'Files', buttons: [
                pal('io-read', 'Read', 'read', 'Read a matrix from a file', [
                    lines('Text and CSV', [
                        ['CSV', "#read M_r from data.csv@R1C1:R2C2 TYPE=R SEP=','\n", 'Read a matrix from a CSV file'],
                    ]),
                    lines('Excel', [
                        ['Range', '#read M_x from data.xlsx@Sheet1!A1:B2 TYPE=R\n', 'Read a matrix from an Excel range'],
                    ], { skip: 'needs a workbook; the verifier writes one first' }),
                    lines('Other worksheets', [['#include', '#include module.cpd\n', 'Include another worksheet']]),
                ]),
                pal('io-write', 'Write', 'write', 'Write a matrix to a file', [
                    lines('Text and CSV', [
                        ['Write CSV', "#write A to out.csv@R1C1:R2C2 TYPE=N SEP=','\n", 'Write a matrix to a CSV file'],
                        ['Append CSV', "#append A to out.csv@R1C1:R2C2 TYPE=N SEP=','\n", 'Append a matrix to a CSV file'],
                    ]),
                    lines('Excel', [
                        ['Write range', '#write A to out.xlsx@Sheet1!A1:B2 TYPE=N\n', 'Write a matrix to an Excel range'],
                        ['Append range', '#append A to out.xlsx@Sheet1!A1:B2 TYPE=N\n', 'Append a matrix to an Excel range'],
                    ]),
                ]),
            ] },
            { name: 'Flow', buttons: [
                small('f-input', 'Input form', 'input', 'Show an input form here and wait', { text: '#input\n', line: true }),
                small('f-pause', 'Pause', 'pause', 'Calculate to this line and wait', { text: '#pause\n', line: true }),
            ] },
            { name: 'Results', buttons: [
                action('o-toggle', 'Show results', 'preview', 'Show or hide the results pane', 'togglePreview'),
                action('o-input', 'Input mode', 'input', 'The #UI input form', 'mode:ui'),
                action('o-report', 'Report mode', 'report', 'The print layout', 'mode:report'),
            ] },
        ],
    },
    {
        id: 'functions', label: 'Functions', groups: [
            { name: 'Elementary', buttons: [
                pal('fn-trig', 'Trigonometric', 'trig', 'Trigonometric functions', [
                    fn('Trigonometric', ['sin(x)', 'cos(x)', 'tan(x)', 'csc(x)', 'sec(x)', 'cot(x)']),
                    fn('Inverse', ['asin(x)', 'acos(x)', 'atan(x)', 'atan2(x; y)', 'acot(x)']),
                    fn('Inverse, argument beyond 1', ['acsc(x)', 'asec(x)'], { test: 'x = 3\nr = {}' }),
                ]),
                pal('fn-hyp', 'Hyperbolic', 'trig', 'Hyperbolic functions', [
                    fn('Hyperbolic', ['sinh(x)', 'cosh(x)', 'tanh(x)', 'csch(x)', 'sech(x)', 'coth(x)']),
                    fn('Inverse', ['asinh(x)', 'atanh(x)', 'acsch(x)', 'asech(x)']),
                    fn('Inverse, argument beyond 1', ['acosh(x)', 'acoth(x)'], { test: 'x = 3\nr = {}' }),
                ]),
                pal('fn-exp', 'Log and roots', 'func', 'Logarithms, exponentials and roots', [
                    fn('Logarithm and exponential', ['log(x)', 'ln(x)', 'log_2(x)', 'exp(x)']),
                    fn('Roots', ['sqrt(x)', 'cbrt(x)', 'root(x; n)']),
                ]),
                pal('fn-round', 'Rounding', 'round', 'Rounding and integer functions', [
                    fn('Rounding', ['round(x)', 'floor(x)', 'ceiling(x)', 'trunc(x)']),
                    fn('Integer', ['mod(x; y)', 'fact(n)']),
                    fn('Integer arguments', ['gcd(n; y)', 'lcm(n; y)']),
                ]),
            ] },
            { name: 'Data', buttons: [
                pal('fn-stat', 'Statistical', 'stat', 'Aggregate and interpolation functions', [
                    fn('Aggregate', ['min(v)', 'max(v)', 'sum(v)', 'sumsq(v)', 'srss(v)', 'average(v)', 'product(v)', 'mean(v)']),
                    fn('Interpolation', ['take(n; v)']),
                    fn('Interpolation, x inside the data', ['line(x; v)', 'spline(x; v)'], { test: 'x = 2\nr = {}' }),
                ]),
                pal('fn-vector', 'Vector', 'vector', 'Vector functions', [
                    fn('Structure', ['len(v)', 'size(v)', 'resize(v; n)', 'join(v; w)', 'slice(v; 1; 2)', 'first(v; 2)', 'last(v; 2)']),
                    fn('Data', ['sort(v)', 'rsort(v)', 'order(v)', 'revorder(v)', 'reverse(v)', 'count(v; 1; 1)', 'search(v; 1; 1)', 'find(v; 1; 1)', 'lookup(v; w; 1)']),
                    fn('Math', ['norm(v)', 'norm_1(v)', 'norm_i(v)', 'unit(v)', 'dot(v; w)']),
                    sec('3D', [['cross', 'cross(v; w)']]),
                ]),
                pal('fn-matrix', 'Matrix', 'matrix', 'Matrix functions', [
                    fn('Properties', ['det(A)', 'rank(A)', 'trace(A)', 'transp(A)', 'adj(A)', 'cofactor(A)', 'inverse(A)']),
                    fn('Norms', ['mnorm(A)', 'mnorm_1(A)', 'mnorm_i(A)', 'mnorm_e(A)', 'cond(A)', 'cond_1(A)']),
                    fn('Solvers', ['lsolve(A; u)', 'msolve(A; B)']),
                    fn('Decompositions', ['eigenvals(S; 2)', 'eigenvecs(S; 2)', 'cholesky(S)', 'lu(A)', 'qr(A)', 'svd(A)']),
                ]),
            ] },
            { name: 'Other', buttons: [
                pal('fn-logic', 'Logical', 'cond', 'Conditional and logical functions', [
                    sec('Conditional', [['if', 'if(x > 1; 1; 0)', 'if(condition; then; else)'], ['switch', 'switch(x > 5; 1; x > 1; 2; 3)', 'switch(cond1; value1; cond2; value2; ...; default)']]),
                    fn('Logical', ['not(x)', 'and(x; y)', 'or(x; y)', 'xor(x; y)']),
                ]),
                pal('fn-complex', 'Complex', 'complex', 'Complex number functions', [
                    sec('Complex (switch complex numbers on in Settings)', [['re', 're(2 + 3i)'], ['im', 'im(2 + 3i)'], ['abs', 'abs(2 + 3i)'], ['phase', 'phase(2 + 3i)'], ['conj', 'conj(2 + 3i)']], { cx: true, raw: true }),
                ]),
                pal('fn-units', 'Units', 'units', 'Functions on units', [
                    fn('Units', ['getunits(x)', 'setunits(x; m)', 'clrunits(x)']),
                    fn('Other', ['abs(x)', 'sign(x)', 'random(x)']),
                ]),
            ] },
            { name: 'Define', buttons: [
                big('d-func', 'Function', 'fx', 'Define a function', { text: 'g_1(t) = t^2 + 1\n', line: true }),
                big('d-const', 'Constant', 'constants', 'Define a read-only constant', { text: '#const c_2 = 1\n', line: true }),
                big('d-macro', 'Macro', 'program', 'Define an inline macro', { text: '#def m$ = text\n', line: true }),
            ] },
        ],
    },
    {
        id: 'matrices', label: 'Matrices/Tables', groups: [
            { name: 'Create', buttons: [
                big('m-vector', 'Vector', 'vector', 'A vector', { text: 'v_1 = [1; 2; 3]\n', line: true }),
                big('m-matrix', 'Matrix', 'matrix', 'A matrix', { text: 'A_1 = [1; 2| 3; 4]\n', line: true }),
                pal('m-create', 'Special', 'grid', 'Special matrices and vectors', [
                    lines('Matrices', [
                        ['identity', 'I_1 = identity(3)\n'], ['diagonal', 'D_1 = diagonal(3; 1)\n'], ['matrix', 'Z_1 = matrix(2; 3)\n'],
                        ['column', 'C_1 = column(3; 1)\n'], ['symmetric', 'S_1 = symmetric(3)\n'], ['upper tri', 'U_1 = utriang(3)\n'], ['lower tri', 'L_1 = ltriang(3)\n'],
                    ]),
                    lines('Vectors', [['vector', 'z_1 = vector(3)\n'], ['to diagonal', 'D_2 = vec2diag(v)\n'], ['to row', 'R_1 = vec2row(v)\n'], ['to column', 'C_2 = vec2col(v)\n']]),
                    lines('Combine', [['stack', 'S_2 = stack(A; B)\n'], ['augment', 'S_3 = augment(A; B)\n'], ['join rows', 'S_4 = join_rows(v; w)\n'], ['join cols', 'S_5 = join_cols(v; w)\n']]),
                ]),
            ] },
            { name: 'Structure', buttons: [
                pal('m-structure', 'Structure', 'matrix', 'Size, rows, columns, sub-matrices', [
                    fn('Size', ['n_rows(A)', 'n_cols(A)', 'mresize(A; 3; 3)', 'mfill(A; 0)']),
                    fn('Parts', ['row(A; 1)', 'col(A; 1)', 'submatrix(A; 1; 1; 1; 2)', 'diag2vec(A)', 'extract_rows(A; [1])', 'extract_cols(A; [1])']),
                    fn('Fill', ['fill_row(A; 1; 0)', 'fill_col(A; 1; 0)']),
                ]),
                pal('m-sort', 'Sort', 'list', 'Sort and order', [
                    fn('Vectors', ['sort(v)', 'rsort(v)', 'order(v)', 'revorder(v)']),
                    fn('Matrices', ['sort_cols(A; 1)', 'rsort_cols(A; 1)', 'sort_rows(A; 1)', 'rsort_rows(A; 1)', 'order_rows(A; 1)']),
                ]),
            ] },
            { name: 'Lookup', buttons: [
                pal('m-lookup', 'Lookup', 'find', 'Search and lookup functions', [
                    fn('Vectors', ['lookup(v; w; 2)', 'search(v; 2; 1)', 'find(v; 2; 1)', 'count(v; 2; 1)']),
                    fn('Matrices', ['hlookup(A; 1; 1; 2)', 'vlookup(A; 1; 1; 2)', 'mfind(A; 1)', 'mcount(A; 1)', 'msearch(A; 1; 1; 1)']),
                    fn('Interpolation', ['take(1; 1; A)', 'line(1; 1; A)', 'spline(1; 1; A)']),
                ]),
            ] },
            { name: 'Linear algebra', buttons: [
                big('m-det', 'Determinant', 'matrix', 'Determinant', { text: 'det(A)', test: 'r = {}' }),
                big('m-inv', 'Inverse', 'matrix', 'Inverse', { text: 'inverse(A)', test: 'r = {}' }),
                big('m-transp', 'Transpose', 'matrix', 'Transpose', { text: 'transp(A)', test: 'r = {}' }),
                pal('m-solve', 'Solve', 'fx', 'Solve and decompose', [
                    fn('Solve', ['lsolve(A; u)', 'msolve(A; B)', 'clsolve(S; u)']),
                    fn('Decompose', ['eigenvals(S; 2)', 'eigenvecs(S; 2)', 'cholesky(S)', 'lu(A)', 'qr(A)', 'svd(A)']),
                    fn('Products', ['hprod(A; B)', 'fprod(A; B)', 'kprod(A; B)']),
                ]),
            ] },
            { name: 'Tables', buttons: [
                big('t-table', 'Table', 'table', 'An HTML table', { text: "'<table class=\"bordered\"><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>\n", line: true }),
                big('t-data', 'Data table', 'table', 'A table with a left-aligned first column', { text: "'<table class=\"data\"><tr><th>Item</th><th>Value</th></tr><tr><td>Load</td><td>1</td></tr></table>\n", line: true }),
                big('t-grid', 'Data grid', 'grid', 'An editable grid in the input form', { text: '#UI G_2 = matrix(3; 3)\n', line: true }),
            ] },
        ],
    },
    {
        id: 'plots', label: 'Plots', groups: [
            { name: 'Function plots', buttons: [
                big('pl-plot', 'Function', 'plot', 'y = f(x)', { text: '$Plot{x^2 - 5*x + 3 @ x = -1 : 6}\n', line: true, nox: true }),
                big('pl-param', 'Parametric', 'plot', 'x(t) and y(t)', { text: '$Plot{cos(t) | sin(t) @ t = 0 : 2*π}\n', line: true, nox: true }),
                big('pl-multi', 'Multiple', 'plot', 'Several functions on one plot', { text: '$Plot{sin(x) & cos(x) @ x = 0 : 2*π}\n', line: true, nox: true }),
                big('pl-multip', 'Multi parametric', 'plot', 'Several parametric curves', { text: '$Plot{cos(t) | sin(t) & 2*cos(t) | 2*sin(t) @ t = 0 : 2*π}\n', line: true, nox: true }),
                big('pl-polar', 'Polar', 'plot', 'A polar curve r(θ)', { text: '$Plot{(1 + cos(θ))*cos(θ) | (1 + cos(θ))*sin(θ) @ θ = 0 : 2*π}\n', line: true, nox: true }),
            ] },
            { name: 'Maps', buttons: [
                big('pl-map', 'Colour map', 'map', 'A colour map of f(x; y)', { text: '$Map{x*y @ x = -1 : 1 & y = -1 : 1}\n', line: true, nox: true }),
            ] },
            { name: 'Options', buttons: [
                action('pl-settings', 'Plot settings', 'settings', 'Size, colours, lighting and mesh step (Settings tab)', 'settings'),
                small('pl-image', 'Image', 'image', 'Insert an image reference', { text: '\'<img src="./image.png" alt="Image">\n', line: true }),
            ] },
        ],
    },
    {
        id: 'mathfmt', label: 'Math Formatting', groups: [
            { name: 'Rounding', buttons: [
                big('mf-round', 'Round', 'round', 'Round results to n decimals', { text: '#round 3\n', line: true }),
                small('mf-rounddef', 'Round default', 'round', 'Back to the default rounding', { text: '#round default\n', line: true }),
                small('mf-format', 'Format', 'round', 'A custom number format', { text: '#format 0.00\n', line: true }),
                small('mf-formatdef', 'Format default', 'round', 'Back to the default format', { text: '#format default\n', line: true }),
            ] },
            { name: 'Angles', buttons: [
                big('mf-deg', 'Degrees', 'angle', 'Trigonometric functions use degrees', { text: '#deg\n', line: true }),
                big('mf-rad', 'Radians', 'angle', 'Trigonometric functions use radians', { text: '#rad\n', line: true }),
                big('mf-gra', 'Gradians', 'angle', 'Trigonometric functions use gradians', { text: '#gra\n', line: true }),
            ] },
            { name: 'Visibility', buttons: [
                big('mf-hide', 'Hide', 'hide', 'Hide the contents of a block from the report', { text: '#hide\n    \n#end hide\n', line: true }),
                big('mf-show', 'Show', 'show', 'Always show the contents', { text: '#show\n', line: true }),
                big('mf-pre', 'Pre', 'input', 'Show only before calculations', { text: '#pre\n    \n#end pre\n', line: true }),
                big('mf-post', 'Post', 'report', 'Show only after calculations', { text: '#post\n    \n#end post\n', line: true }),
            ] },
            { name: 'Output', buttons: [
                small('mf-val', 'Values only', 'show', 'Show only the result, without the equation', { text: '#val\n', line: true }),
                small('mf-equ', 'Equations', 'show', 'Show equations and results', { text: '#equ\n', line: true }),
                small('mf-noc', 'No results', 'show', 'Show equations without results', { text: '#noc\n', line: true }),
                small('mf-nosub', 'No substitution', 'show', 'Do not substitute variables', { text: '#nosub\n', line: true }),
                small('mf-novar', 'Values only in equations', 'show', 'Show equations with substituted values only', { text: '#novar\n', line: true }),
                small('mf-varsub', 'Variables and values', 'show', 'Show variables and substituted values', { text: '#varsub\n', line: true }),
            ] },
            { name: 'Layout', buttons: [
                small('mf-split', 'Split', 'split', 'Split equations after the = sign', { text: '#split\n', line: true }),
                small('mf-wrap', 'Wrap', 'wrap', 'Wrap equations at the end of the line', { text: '#wrap\n', line: true }),
                small('mf-phasor', 'Phasor', 'complex', 'Show complex numbers as A∠φ', { text: '#phasor\n', line: true }),
                small('mf-complex', 'Algebraic', 'complex', 'Show complex numbers as a + bi', { text: '#complex\n', line: true }),
            ] },
        ],
    },
    {
        id: 'textfmt', label: 'Text Formatting', groups: [
            { name: 'Headings', buttons: [1, 2, 3, 4, 5].map(n => big(`tf-h${n}`, `H${n}`, 'heading', `Heading ${n}`, { text: `<h${n}>{sel}</h${n}>`, test: "'{}" })),
            },
            { name: 'Font', buttons: [
                big('tf-b', 'Bold', 'bold', 'Bold (select text first)', { text: '<b>{sel}</b>', test: "'a {} b" }),
                big('tf-i', 'Italic', 'italic', 'Italic', { text: '<i>{sel}</i>', test: "'a {} b" }),
                big('tf-u', 'Underline', 'underline', 'Underline', { text: '<u>{sel}</u>', test: "'a {} b" }),
                big('tf-s', 'Strike', 'strike', 'Strike through', { text: '<s>{sel}</s>', test: "'a {} b" }),
                small('tf-sub', 'Subscript', 'sub', 'Subscript', { text: '<sub>{sel}</sub>', test: "'a {} b" }),
                small('tf-sup', 'Superscript', 'sub', 'Superscript', { text: '<sup>{sel}</sup>', test: "'a {} b" }),
            ] },
            { name: 'Colour', buttons: [
                big('tf-red', 'Red', 'colour', 'Red text', { text: '<span class="err">{sel}</span>', test: "'a {} b" }),
                big('tf-green', 'Green', 'colour', 'Green text', { text: '<span class="ok">{sel}</span>', test: "'a {} b" }),
                small('tf-color', 'Any colour', 'colour', 'Text in a colour of your choice', { text: '<span style="color:blue;">{sel}</span>', test: "'a {} b" }),
                small('tf-ref', 'Reference', 'link', 'A right-aligned reference to a design code', { text: '<span class="ref">{sel}</span>', test: "'a {} b" }),
            ] },
            { name: 'Paragraph', buttons: [
                big('tf-p', 'Paragraph', 'text', 'A paragraph', { text: '<p>{sel}</p>', test: "'{}" }),
                big('tf-br', 'Line break', 'br', 'A line break', { text: '<br/>', test: "'a {} b" }),
                big('tf-hr', 'Rule', 'hr', 'A horizontal rule', { text: "'<hr/>\n", line: true }),
                big('tf-page', 'Page break', 'page', 'Start a new page when printing', { text: "'<div style=\"page-break-after:always\"></div>\n", line: true }),
            ] },
            { name: 'Lists', buttons: [
                big('tf-ul', 'Bullets', 'list', 'A bulleted list', { text: "'<ul><li>First</li><li>Second</li></ul>\n", line: true }),
                big('tf-ol', 'Numbered', 'olist', 'A numbered list', { text: "'<ol><li>First</li><li>Second</li></ol>\n", line: true }),
                big('tf-fold', 'Fold', 'fold', 'A section that folds in the report', { text: "'<div class=\"fold\">\n'<b>Heading</b> (click to unfold)\n'<p>Content</p>\n'</div>\n", line: true }),
            ] },
            { name: 'Markdown', buttons: [
                small('tf-mdon', 'Markdown on', 'text', 'Comments use Markdown', { text: '#md on\n', line: true }),
                small('tf-mdoff', 'Markdown off', 'text', 'Comments use HTML', { text: '#md off\n', line: true }),
            ] },
        ],
    },
    {
        id: 'calc', label: 'Calculation', groups: [
            { name: 'Run', buttons: [
                action('k-calc', 'Calculate', 'run', 'Run the worksheet (F5)', 'calc'),
                action('k-auto', 'Auto-calculate', 'auto', 'Recalculate while typing', 'autocalc'),
            ] },
            { name: 'Results', buttons: [
                action('k-preview', 'Preview', 'preview', 'The document as written', 'mode:preview'),
                action('k-unwrapped', 'Unwrapped', 'source', 'The source listing with macros and includes resolved', 'mode:unwrapped'),
                action('k-input', 'Input', 'input', 'The #UI input form', 'mode:ui'),
                action('k-report', 'Report', 'report', 'The print layout', 'mode:report'),
                action('k-toggle', 'Show results', 'show', 'Show or hide the results pane', 'togglePreview'),
            ] },
            { name: 'Angles', buttons: [
                small('k-deg', 'Degrees', 'angle', 'Trigonometric functions use degrees', { text: '#deg\n', line: true }),
                small('k-rad', 'Radians', 'angle', 'Trigonometric functions use radians', { text: '#rad\n', line: true }),
                small('k-gra', 'Gradians', 'angle', 'Trigonometric functions use gradians', { text: '#gra\n', line: true }),
            ] },
            { name: 'Numbers', buttons: [
                small('k-round', 'Round', 'round', 'Round results to n decimals', { text: '#round 3\n', line: true }),
                small('k-phasor', 'Phasor', 'complex', 'Complex numbers as A∠φ', { text: '#phasor\n', line: true }),
                small('k-complex', 'Algebraic', 'complex', 'Complex numbers as a + bi', { text: '#complex\n', line: true }),
            ] },
            { name: 'Settings', buttons: [
                action('k-settings', 'Calculation settings', 'settings', 'Decimals, precision, complex mode, plot options (Settings tab)', 'settings'),
                action('k-variables', 'Variables', 'list', 'The variables of the worksheet', 'tab:variables'),
                action('k-errors', 'Errors', 'info', 'Errors and warnings', 'tab:errors'),
            ] },
        ],
    },
    {
        id: 'document', label: 'Document', groups: [
            { name: 'Export', buttons: [
                action('d-pdf', 'Print PDF', 'pdf', 'Print the report as a PDF', 'pdf'),
                action('d-word', 'Word', 'word', 'Save the report as a Word document', 'word'),
                action('d-excel', 'Excel', 'excel', 'Save the variables as an Excel workbook', 'excel'),
                action('d-export', 'Export tab', 'export', 'HTML, PDF, Word and #write: the Export tab', 'exportTab'),
            ] },
            { name: 'Worksheet', buttons: [
                action('d-new', 'New', 'file', 'New worksheet', 'new'),
                action('d-open', 'Open', 'open', 'Open a worksheet', 'open'),
                action('d-save', 'Save', 'save', 'Save', 'save'),
                action('d-saveas', 'Save As', 'saveas', 'Save As', 'saveas'),
            ] },
            { name: 'Page', buttons: [
                action('d-pdfsettings', 'Header and footer', 'settings', 'Page numbers, date and title for PDF export (Settings tab)', 'settings'),
                big('d-pagebreak', 'Page break', 'page', 'Start a new page when printing', { text: "'<div style=\"page-break-after:always\"></div>\n", line: true }),
            ] },
            { name: 'Report', buttons: [
                action('d-report', 'Report', 'report', 'The print layout', 'mode:report'),
                action('d-samples', 'Samples', 'samples', 'Browse the sample worksheets', 'samples'),
                action('d-toc', 'Contents', 'list', 'The table of contents of the worksheet', 'tab:toc'),
            ] },
        ],
    },
    {
        id: 'resources', label: 'Resources', groups: [
            { name: 'Examples', buttons: [
                action('x-samples', 'Samples', 'samples', 'Browse the sample worksheets', 'samples'),
                action('x-library', 'Library', 'recent', 'Recent worksheets and the #include and #read library', 'filemenu'),
            ] },
            { name: 'Insert panel', buttons: [
                action('x-insert', 'Insert tab', 'plus', 'Symbols, functions and snippets in the sidebar', 'tab:insert'),
                action('x-settings', 'Settings', 'settings', 'Settings', 'settings'),
            ] },
            { name: 'Help', buttons: [
                action('x-docs', 'Documentation', 'help', 'Open the CalcpadCE documentation', 'docs'),
                action('x-keys', 'Shortcuts', 'keyboard', 'List the keyboard shortcuts in the output panel', 'shortcuts'),
                action('x-about', 'About', 'info', 'About this editor', 'about'),
            ] },
            { name: 'View', buttons: [
                action('x-light', 'Light', 'sun', 'Light theme', 'theme:Light'),
                action('x-dark', 'Dark', 'moon', 'Dark theme', 'theme:Dark'),
                action('x-zoomin', 'Zoom in', 'zoomin', 'Larger editor text', 'zoomIn'),
                action('x-zoomout', 'Zoom out', 'zoomout', 'Smaller editor text', 'zoomOut'),
                action('x-sidebar', 'Sidebar', 'sidebar', 'Show or hide the sidebar (Ctrl+Shift+B)', 'sidebar'),
            ] },
        ],
    },
];

/** Every insertable snippet, flattened, for the verifier. */
export interface Snippet { id: string; text: string; line: boolean; test?: string; nox?: boolean; raw?: boolean; cx?: boolean; skip?: string }
export function allSnippets(): Snippet[] {
    const out: Snippet[] = [];
    for (const tab of RIBBON) for (const g of tab.groups) for (const b of g.buttons) {
        if (b.text !== undefined) out.push({ id: `${tab.id}/${b.id}`, text: b.text, line: !!b.line, test: b.test, nox: b.nox, raw: b.raw, skip: b.skip });
        for (const s of b.palette ?? []) for (const e of s.items) {
            out.push({ id: `${tab.id}/${b.id}/${e.label}`, text: e.text, line: !!e.line, test: e.test, nox: e.nox, raw: e.raw, cx: e.cx, skip: e.skip });
        }
    }
    return out;
}
