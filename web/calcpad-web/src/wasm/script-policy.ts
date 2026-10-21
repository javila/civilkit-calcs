// calcpad-rs: what a worksheet's own scripts may do in the preview. Upstream Calcpad runs every
// script a worksheet carries; here (a deliberate deviation, DEVIATIONS.md) each rendered page
// gets a Content-Security-Policy:
//
//   script-src   'unsafe-inline' (the worksheet's glue code) and the EXACT file URLs on the
//                library list, never a whole host (a CDN like jsDelivr serves any package)
//   connect-src  blob: only, form-action 'none': a script cannot send data anywhere
//   img/style/font/media: data:, blob:, inline styles and this app's own origin
//
// <script src> tags for listed files get integrity= and crossorigin="anonymous"; module files
// get their hashes through an import map. A <script src> that is not on the list is replaced
// by a note naming the URL, and anything else the policy stops (a module import, a script
// added at run time, an image or request elsewhere) is reported by a note too.
//
// The page is edited as a string, so a page with no scripts keeps its exact bytes apart from
// the head additions.

import type { ScriptLibrary } from './library-list';

const NOTE_STYLE = 'margin:4pt 0;padding:4pt 8pt;border-left:3pt solid #c00;background:#fff3f3;color:#900;font:10pt sans-serif';

function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** The note shown where a script did not load. */
export function blockedNote(url: string): string {
    return `<p class="cprs-blocked" style="${NOTE_STYLE}">Script not loaded: <code>${escapeHtml(url)}</code>`
        + ' is not in the library list (Settings &gt; Libraries).</p>';
}

/** Reports what the policy stops while the page runs (module imports, later loads, requests). */
const VIOLATION_REPORTER = `<script>(function(){var seen={};document.addEventListener('securitypolicyviolation',function(e){
var u=e.blockedURI||'';var d=e.effectiveDirective||e.violatedDirective||'';if(!u||u==='inline'||u==='eval'||seen[u+d])return;seen[u+d]=1;
var isScript=d.indexOf('script')===0;function show(){var p=document.createElement('p');p.className='cprs-blocked';p.setAttribute('style','${NOTE_STYLE}');
var c=document.createElement('code');c.textContent=u;p.appendChild(document.createTextNode(isScript?'Script not loaded: ':'Blocked ('+d+'): '));p.appendChild(c);
p.appendChild(document.createTextNode(isScript?' is not in the library list (Settings > Libraries).':' Worksheets may not make network requests or load files from other sites.'));
var b=document.body||document.documentElement;b.insertBefore(p,b.firstChild);}
if(document.body)show();else document.addEventListener('DOMContentLoaded',show);});})();</script>`.replace(/\n/g, '');

/** CSP source for an exact URL: the query string is not part of CSP matching. */
function cspSource(url: string): string {
    return url.split('#')[0].split('?')[0].replace(/;/g, '%3B').replace(/,/g, '%2C').replace(/\s/g, '%20');
}

export function contentSecurityPolicy(libs: ScriptLibrary[], origin: string): string {
    const urls = [...new Set(libs.flatMap(l => l.files.map(f => cspSource(f.url))))];
    return [
        "default-src 'none'",
        `script-src 'unsafe-inline' ${urls.join(' ')}`.trim(),
        `style-src 'unsafe-inline' data: ${origin}`,
        `img-src data: blob: ${origin}`,
        `font-src data: ${origin}`,
        'media-src data: blob:',
        // blob: is data the page made itself (dxf-viewer reads a drawing it built that way); it
        // cannot reach another host.
        'connect-src blob:',
        "form-action 'none'",
        "base-uri 'none'",
        "object-src 'none'",
        "frame-src 'none'",
    ].join('; ');
}

/**
 * HTML comments (ended as the HTML parser ends them: -->, --!>, or at once by <!--> and
 * <!--->) or script elements. Comments are skipped, since a worksheet's comment text can
 * mention "<script>" without being one.
 */
const COMMENT_OR_SCRIPT = /<!--(?:->|>|[\s\S]*?--!?>)|<script\b([^>]*)>([\s\S]*?)<\/script\s*>/gi;
const SRC_ATTR = /\bsrc\s*=\s*("([^"]*)"|'([^']*)'|([^\s"'>]+))/i;
const STRIP_ATTRS = /\s(integrity|crossorigin)\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi;

/** `html` with the policy applied (see the top of this file). */
export function applyScriptPolicy(html: string, libs: ScriptLibrary[], origin: string): string {
    const byUrl = new Map<string, string>();
    for (const l of libs) for (const f of l.files) byUrl.set(f.url, f.integrity);

    const body = html.replace(COMMENT_OR_SCRIPT, (tag, attrs: string | undefined, inner: string) => {
        if (attrs === undefined) return tag;
        const m = SRC_ATTR.exec(attrs);
        if (!m) return tag;
        const url = (m[2] ?? m[3] ?? m[4] ?? '').trim().replace(/&amp;/g, '&');
        const integrity = byUrl.get(url);
        if (!integrity) return blockedNote(url);
        const kept = attrs.replace(STRIP_ATTRS, '');
        return `<script${kept} integrity="${integrity}" crossorigin="anonymous">${inner}</script>`;
    });

    const moduleHashes: Record<string, string> = {};
    for (const l of libs) for (const f of l.files) if (f.module) moduleHashes[f.url] = f.integrity;
    const importMap = Object.keys(moduleHashes).length
        ? `<script type="importmap">${JSON.stringify({ integrity: moduleHashes }).replace(/</g, '\\u003c')}</script>`
        : '';
    const head = `<meta http-equiv="Content-Security-Policy" content="${escapeHtml(contentSecurityPolicy(libs, origin))}">`
        + VIOLATION_REPORTER + importMap;

    // After <meta charset> when there is one: it has to stay in the first 1024 bytes of a saved page.
    const open = /<meta\s+charset\s*=[^>]*>/i.exec(body) ?? /<head\b[^>]*>/i.exec(body);
    if (open) return body.slice(0, open.index + open[0].length) + head + body.slice(open.index + open[0].length);
    return head + body;
}
