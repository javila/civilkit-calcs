// calcpad-rs: PDF through the browser. Calcpad.Server renders PDFs with a headless Chromium;
// here the report HTML is printed by the browser itself, whose print dialog saves as PDF.
// Paper size, orientation and margins come from the PDF settings as an @page rule. The
// server's own header and footer (title, date, page numbers) are not drawn: the browser's
// print dialog offers its own.

export interface PrintOptions {
    format?: string;
    orientation?: string;
    marginTop?: string;
    marginRight?: string;
    marginBottom?: string;
    marginLeft?: string;
    documentTitle?: string;
}

const CSS_LENGTH = /^\s*\d+(\.\d+)?\s*(mm|cm|in|pt|px|pc)?\s*$/i;

function length(v: string | undefined, def: string): string {
    return v && CSS_LENGTH.test(v) ? v.trim() : def;
}

function pageRule(o: PrintOptions): string {
    const size = /^[A-Za-z0-9-]+$/.test(o.format ?? '') ? o.format : 'A4';
    const orientation = o.orientation === 'landscape' ? 'landscape' : 'portrait';
    const margin = [
        length(o.marginTop, '0.75in'), length(o.marginRight, '0.5in'),
        length(o.marginBottom, '0.75in'), length(o.marginLeft, '0.5in'),
    ].join(' ');
    return `<style>@page{size:${size} ${orientation};margin:${margin};}</style>`;
}

function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** Prints `html` (a whole page) from a hidden frame and resolves once the dialog closes. */
export function printHtml(html: string, options: PrintOptions = {}): Promise<void> {
    let page = html;
    const head = pageRule(options) + (options.documentTitle ? `<title>${escapeHtml(options.documentTitle)}</title>` : '');
    const i = page.toLowerCase().indexOf('</head>');
    page = i >= 0 ? page.slice(0, i) + head + page.slice(i) : head + page;
    return new Promise<void>((resolve) => {
        const frame = document.createElement('iframe');
        frame.setAttribute('aria-hidden', 'true');
        frame.style.cssText = 'position:fixed;right:0;bottom:0;width:0;height:0;border:0;';
        let finished = false;
        const finish = () => {
            if (finished) return;
            finished = true;
            setTimeout(() => frame.remove(), 500);
            resolve();
        };
        frame.onload = () => {
            const w = frame.contentWindow;
            if (!w) return finish();
            w.addEventListener('afterprint', finish, { once: true });
            const fontsReady = (w.document as Document & { fonts?: FontFaceSet }).fonts?.ready ?? Promise.resolve();
            void fontsReady.then(() => {
                w.focus();
                w.print();
                // Browsers whose print() returns at once fire afterprint; the rest return when
                // the dialog closes, so either way the frame is cleaned up.
                setTimeout(finish, 1000);
            });
        };
        frame.srcdoc = page;
        document.body.appendChild(frame);
    });
}
