// calcpad-rs: the head markup Calcpad.Server's BundledFonts adds to every rendered page:
// @font-face rules (GetFontFaceStyleTag) and window.__calcpadFonts (GetInjectionScript),
// both with data URLs, since the report template's CSP allows fonts only from data: and
// https:. The font files are static assets in public/report-fonts (licences alongside).

type Face = { family: string; weight?: string; style?: string };

const FONT_FACES: Record<string, Face[]> = {
    'Jost-100-Hairline.otf': [{ family: 'Jost* Hairline' }],
    'Jost-200-Thin.otf': [{ family: 'Jost* Thin' }],
    'DejaVuSerifCondensed.woff2': [{ family: 'DejaVu Serif Condensed', weight: 'normal', style: 'normal' }],
    'DejaVuSerifCondensed-Italic.woff2': [{ family: 'DejaVu Serif Condensed', weight: 'normal', style: 'italic' }],
    'DejaVuSerifCondensed-Bold.woff2': [{ family: 'DejaVu Serif Condensed', weight: 'bold', style: 'normal' }],
    'DejaVuSerifCondensed-BoldItalic.woff2': [{ family: 'DejaVu Serif Condensed', weight: 'bold', style: 'italic' }],
    'CalcpadCEVector.woff2': [{ family: 'CalcpadCE Vector', weight: 'normal', style: 'normal' }],
};

const cssEscape = (s: string) => s.replace(/\\/g, '\\\\').replace(/"/g, '\\"');

async function dataUrl(url: string, mime: string): Promise<string | null> {
    const res = await fetch(url);
    if (!res.ok) return null;
    const bytes = new Uint8Array(await res.arrayBuffer());
    let bin = '';
    for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    return `data:${mime};base64,${btoa(bin)}`;
}

let cached: Promise<string> | null = null;

/** The markup to insert before </head>; empty if the fonts cannot be loaded. */
export function reportHeadMarkup(base: string): Promise<string> {
    cached ??= (async () => {
        const urls: [string, string][] = [];
        await Promise.all(Object.keys(FONT_FACES).map(async (file) => {
            const otf = file.endsWith('.otf');
            const url = await dataUrl(`${base}report-fonts/${file}`, otf ? 'font/otf' : 'font/woff2').catch(() => null);
            if (url) urls.push([file, url]);
        }));
        if (!urls.length) return '';
        urls.sort((a, b) => Object.keys(FONT_FACES).indexOf(a[0]) - Object.keys(FONT_FACES).indexOf(b[0]));
        let css = '';
        for (const [file, url] of urls) {
            const format = file.endsWith('.otf') ? 'opentype' : 'woff2';
            for (const f of FONT_FACES[file]) {
                css += `@font-face{font-family:"${cssEscape(f.family)}";src:url(${url}) format("${format}")`;
                if (f.weight) css += `;font-weight:${f.weight}`;
                if (f.style) css += `;font-style:${f.style}`;
                css += ';}';
            }
        }
        const dict = Object.fromEntries(urls);
        return `<style>${css}</style><script>window.__calcpadFonts=${JSON.stringify(dict)};</script>`;
    })();
    return cached;
}
