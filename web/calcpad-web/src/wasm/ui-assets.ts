// calcpad-rs: Calcpad.Server's BundledUiAssets in the page. The datagrid libraries
// (jsuites and jspreadsheet-ce 5, MIT, see public/ui-assets/README.md) are inlined into
// <head> only when a rendered form contains a datagrid, in the order the server uses.
// They are fetched the first time a grid is shown and kept for the session.

const STYLE_SHEETS = ['jsuites.min.css', 'jspreadsheet.min.css'];
const SCRIPTS = ['jsuites.min.js', 'jspreadsheet.min.js'];
/** Calcpad's own datagrid rules, inlined last so they override the library's. */
const CUSTOM_STYLE_SHEET = 'calcpad-datagrid.css';
/** jSuites binds its document listeners once per window; the preview reuses its window. */
const RESET_STATE_CONTROL = '<script>delete window.jSuitesStateControl;</script>';

let cached: Promise<string> | null = null;

async function load(base: string, file: string): Promise<string | null> {
    try {
        const res = await fetch(`${base}ui-assets/${file}`);
        return res.ok ? await res.text() : null;
    } catch {
        return null;
    }
}

/** The markup for <head>; empty when a library is missing, since half a widget is worse than none. */
export function uiAssetsMarkup(base: string): Promise<string> {
    cached ??= (async () => {
        let markup = RESET_STATE_CONTROL;
        for (const [file, tag] of [...STYLE_SHEETS.map(f => [f, 'style']), ...SCRIPTS.map(f => [f, 'script'])]) {
            const content = await load(base, file);
            if (content == null) {
                console.warn(`[calcpad-rs] ${file} is missing, datagrids will not render`);
                return '';
            }
            markup += `<${tag}>${content}</${tag}>`;
        }
        const custom = await load(base, CUSTOM_STYLE_SHEET);
        if (custom != null) markup += `<style>${custom}</style>`;
        return markup;
    })();
    return cached;
}
