// Draws the figure specification the light matplotlib stand-in records (public/py/lib/matplotlib/_chart.py) as
// one line of SVG text. The page puts that line into a worksheet as a text line, so it must hold no newline and no
// single quote: attributes use double quotes and every text is escaped.
//
// Supported: line, scatter, bar, fill_between, axhline, axvline and text items; titles, axis labels, limits, linear and
// log scales, equal aspect, grid, legend; a subplot grid; the default colour cycle, format strings ("r--", "o").
// Not supported (the recorder refuses these): contours, colour bars, images, polar and 3-D axes.

type Json = null | boolean | number | string | Json[] | { [k: string]: Json };
type Item = { [k: string]: any };
type AxesSpec = { index: number; items: Item[]; [k: string]: any };
type FigureSpec = {
    axes: AxesSpec[]; figsize?: number[]; nrows?: number; ncols?: number; suptitle?: string; [k: string]: any;
};

const CYCLE = ['#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd', '#8c564b', '#e377c2', '#7f7f7f', '#bcbd22', '#17becf'];
const LETTER: Record<string, string> = { b: '#1f77b4', g: '#2ca02c', r: '#d62728', c: '#17becf', m: '#bf00bf', y: '#bfbf00', k: '#000000', w: '#ffffff' };
const TAB: Record<string, string> = {
    blue: '#1f77b4', orange: '#ff7f0e', green: '#2ca02c', red: '#d62728', purple: '#9467bd', brown: '#8c564b',
    pink: '#e377c2', gray: '#7f7f7f', grey: '#7f7f7f', olive: '#bcbd22', cyan: '#17becf',
};
const DASH: Record<string, string> = { '--': '7 3.5', ':': '1.5 3.5', '-.': '7 3.5 1.5 3.5', dashed: '7 3.5', dotted: '1.5 3.5', dashdot: '7 3.5 1.5 3.5' };
const MARKERS = new Set(['o', 's', '^', 'v', '<', '>', 'x', '+', '.', '*', 'd', 'D', 'p', 'h', '|', '_']);
const FONT = 'sans-serif';

const r2 = (v: number) => Math.round(v * 100) / 100;
const esc = (s: unknown) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
const num = (v: unknown, d: number) => (typeof v === 'number' && Number.isFinite(v) ? v : d);

/** A colour as an SVG paint: hex and CSS names pass through, "C3" and "tab:blue" map to the cycle, "0.5" is a grey. */
function paint(c: unknown, fallback: string): string {
    if (typeof c !== 'string') return fallback;
    const s = c.trim();
    if (/^C\d$/.test(s)) return CYCLE[+s[1]];
    if (s.startsWith('tab:')) return TAB[s.slice(4)] ?? fallback;
    if (s.length === 1 && LETTER[s]) return LETTER[s];
    if (/^\d*\.?\d+$/.test(s)) { const g = Math.round(255 * Math.min(1, Math.max(0, +s))); return `rgb(${g},${g},${g})`; }
    if (/^#[0-9a-fA-F]{3,8}$/.test(s) || /^[a-zA-Z]+$/.test(s)) return s;
    return fallback;
}

/** "r--", "go", "k:", "b" split into colour, line style and marker. */
function parseFmt(fmt: string): { color?: string; style?: string; marker?: string } {
    const out: { color?: string; style?: string; marker?: string } = {};
    let s = fmt;
    for (const ls of ['--', '-.', '-', ':']) if (s.includes(ls)) { out.style = ls; s = s.replace(ls, ''); break; }
    for (const ch of s) {
        if (LETTER[ch] && !out.color) out.color = LETTER[ch];
        else if (MARKERS.has(ch) && !out.marker) out.marker = ch;
    }
    if (out.marker && !out.style) out.style = 'none';
    return out;
}

/** Tick positions with a 1, 2, 2.5, 5 step inside [lo, hi]; about `target` of them. */
function niceTicks(lo: number, hi: number, target: number): { ticks: number[]; step: number } {
    const range = hi - lo;
    if (!(range > 0) || !Number.isFinite(range)) return { ticks: [lo], step: 1 };
    const raw = range / Math.max(2, target);
    const mag = 10 ** Math.floor(Math.log10(raw));
    const norm = raw / mag;
    const step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 2.5 ? 2.5 : norm <= 5 ? 5 : 10) * mag;
    const out: number[] = [];
    for (let v = Math.ceil(lo / step - 1e-9) * step, i = 0; v <= hi + step * 1e-9 && i < 200; v += step, i++) out.push(Math.abs(v) < step * 1e-9 ? 0 : v);
    return { ticks: out, step };
}

function formatTick(v: number, step: number): string {
    if (v === 0) return '0';
    const a = Math.abs(v);
    if (a >= 1e6 || a < 1e-4) return v.toExponential(1).replace(/\.0e/, 'e').replace('e+', 'e');
    const dec = Math.min(8, Math.max(0, -Math.floor(Math.log10(step) + 1e-9)));
    return v.toFixed(dec);
}

interface Scale { lo: number; hi: number; log: boolean; px0: number; px1: number }
const tr = (s: Scale, v: number) => {
    const a = s.log ? Math.log10(Math.max(v, 1e-300)) : v;
    const lo = s.log ? Math.log10(Math.max(s.lo, 1e-300)) : s.lo;
    const hi = s.log ? Math.log10(Math.max(s.hi, 1e-300)) : s.hi;
    return s.px0 + ((a - lo) / (hi - lo || 1)) * (s.px1 - s.px0);
};

function marker(m: string, x: number, y: number, size: number, fill: string, edge: string): string {
    const r = size / 2;
    const f = `fill="${fill}" stroke="${edge}" stroke-width="1"`;
    switch (m) {
        case 's': return `<rect x="${r2(x - r)}" y="${r2(y - r)}" width="${r2(size)}" height="${r2(size)}" ${f}/>`;
        case '^': return `<path d="M${r2(x)} ${r2(y - r * 1.15)}L${r2(x + r * 1.1)} ${r2(y + r * 0.8)}L${r2(x - r * 1.1)} ${r2(y + r * 0.8)}Z" ${f}/>`;
        case 'v': return `<path d="M${r2(x)} ${r2(y + r * 1.15)}L${r2(x + r * 1.1)} ${r2(y - r * 0.8)}L${r2(x - r * 1.1)} ${r2(y - r * 0.8)}Z" ${f}/>`;
        case '<': return `<path d="M${r2(x - r * 1.15)} ${r2(y)}L${r2(x + r * 0.8)} ${r2(y - r * 1.1)}L${r2(x + r * 0.8)} ${r2(y + r * 1.1)}Z" ${f}/>`;
        case '>': return `<path d="M${r2(x + r * 1.15)} ${r2(y)}L${r2(x - r * 0.8)} ${r2(y - r * 1.1)}L${r2(x - r * 0.8)} ${r2(y + r * 1.1)}Z" ${f}/>`;
        case 'd': case 'D': return `<path d="M${r2(x)} ${r2(y - r * 1.2)}L${r2(x + r * 0.8)} ${r2(y)}L${r2(x)} ${r2(y + r * 1.2)}L${r2(x - r * 0.8)} ${r2(y)}Z" ${f}/>`;
        case 'x': return `<path d="M${r2(x - r)} ${r2(y - r)}L${r2(x + r)} ${r2(y + r)}M${r2(x - r)} ${r2(y + r)}L${r2(x + r)} ${r2(y - r)}" stroke="${fill}" stroke-width="1.6" fill="none"/>`;
        case '+': return `<path d="M${r2(x - r)} ${r2(y)}L${r2(x + r)} ${r2(y)}M${r2(x)} ${r2(y - r)}L${r2(x)} ${r2(y + r)}" stroke="${fill}" stroke-width="1.6" fill="none"/>`;
        case '|': return `<path d="M${r2(x)} ${r2(y - r)}L${r2(x)} ${r2(y + r)}" stroke="${fill}" stroke-width="1.6" fill="none"/>`;
        case '_': return `<path d="M${r2(x - r)} ${r2(y)}L${r2(x + r)} ${r2(y)}" stroke="${fill}" stroke-width="1.6" fill="none"/>`;
        case '*': return `<path d="M${r2(x)} ${r2(y - r * 1.3)}L${r2(x + r * 0.4)} ${r2(y - r * 0.4)}L${r2(x + r * 1.3)} ${r2(y - r * 0.3)}L${r2(x + r * 0.6)} ${r2(y + r * 0.3)}L${r2(x + r * 0.8)} ${r2(y + r * 1.2)}L${r2(x)} ${r2(y + r * 0.7)}L${r2(x - r * 0.8)} ${r2(y + r * 1.2)}L${r2(x - r * 0.6)} ${r2(y + r * 0.3)}L${r2(x - r * 1.3)} ${r2(y - r * 0.3)}L${r2(x - r * 0.4)} ${r2(y - r * 0.4)}Z" ${f}/>`;
        case '.': return `<circle cx="${r2(x)}" cy="${r2(y)}" r="${r2(size / 4)}" fill="${fill}"/>`;
        default: return `<circle cx="${r2(x)}" cy="${r2(y)}" r="${r2(r)}" ${f}/>`;
    }
}

let figureCounter = 0;

/** The SVG text of one figure. Throws on a specification it cannot draw. */
export function renderFigure(specText: string): string {
    const spec = JSON.parse(specText) as FigureSpec;
    const [fw, fh] = (spec.figsize && spec.figsize.length === 2 ? spec.figsize : [6.4, 4.8]).map((v) => Math.min(20, Math.max(1, num(v, 5))));
    const W = Math.round(fw * 100);
    const H = Math.round(fh * 100);
    const nrows = spec.nrows ?? 1;
    const ncols = spec.ncols ?? 1;
    const uid = `cpy${(++figureCounter).toString(36)}${Math.floor(Math.random() * 1e6).toString(36)}`;
    const parts: string[] = [];
    const top = spec.suptitle ? 30 : 0;
    if (spec.suptitle) parts.push(`<text x="${r2(W / 2)}" y="22" text-anchor="middle" font-family="${FONT}" font-size="18" fill="#000">${esc(spec.suptitle)}</text>`);
    const cellW = W / ncols;
    const cellH = (H - top) / nrows;
    const single = nrows * ncols === 1;
    for (const ax of spec.axes) {
        const row = Math.floor(ax.index / ncols);
        const col = ax.index % ncols;
        parts.push(drawAxes(ax, col * cellW, top + row * cellH, cellW, cellH, `${uid}-${ax.index}`, single));
    }
    return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" width="${W}" height="${H}" class="cprs-py-figure" style="max-width:100%;height:auto" role="img">`
        + `<rect width="${W}" height="${H}" fill="#fff"/>${parts.join('')}</svg>`;
}

function drawAxes(ax: AxesSpec, cx: number, cy: number, cw: number, ch: number, uid: string, single: boolean): string {
    const items = ax.items ?? [];
    const scaleFont = single ? 1 : Math.max(0.75, Math.min(1, cw / 420));
    const fs = { tick: 12.5 * scaleFont, label: 14 * scaleFont, title: 16 * scaleFont, legend: 12.5 * scaleFont };
    const ml = (ax.ylabel ? 62 : 50) * scaleFont, mr = 16 * scaleFont, mt = (ax.title ? 36 : 14) * scaleFont, mb = (ax.xlabel ? 52 : 36) * scaleFont;
    let px0 = cx + ml, px1 = cx + cw - mr, py0 = cy + mt, py1 = cy + ch - mb;
    const xlog = ax.xscale === 'log';
    const ylog = ax.yscale === 'log';

    // data bounds
    let xmin = Infinity, xmax = -Infinity, ymin = Infinity, ymax = -Infinity;
    let stickyY0 = false;
    const take = (xs: number[] | undefined, ys: number[] | undefined) => {
        for (const v of xs ?? []) if (Number.isFinite(v) && (!xlog || v > 0)) { xmin = Math.min(xmin, v); xmax = Math.max(xmax, v); }
        for (const v of ys ?? []) if (Number.isFinite(v) && (!ylog || v > 0)) { ymin = Math.min(ymin, v); ymax = Math.max(ymax, v); }
    };
    for (const it of items) {
        switch (it.type) {
            case 'line': case 'scatter': take(it.x, it.y); break;
            case 'fill_between': take(it.x, [...it.y1, ...it.y2]); break;
            case 'bar': {
                const lo = it.x.map((x: number, i: number) => it.align === 'edge' ? x : x - it.width[i] / 2);
                const hi = it.x.map((x: number, i: number) => it.align === 'edge' ? x + it.width[i] : x + it.width[i] / 2);
                take([...lo, ...hi], [...it.bottom, ...it.bottom.map((b: number, i: number) => b + it.height[i])]);
                if (it.bottom.every((b: number) => b === 0) && it.height.every((h: number) => h >= 0)) stickyY0 = true;
                break;
            }
            case 'hline': take(undefined, [it.y]); break;
            case 'vline': take([it.x], undefined); break;
            default: break;
        }
    }
    if (!Number.isFinite(xmin)) { xmin = 0; xmax = 1; }
    if (!Number.isFinite(ymin)) { ymin = 0; ymax = 1; }
    const pad = (lo: number, hi: number, log: boolean): [number, number] => {
        if (log) { const a = Math.log10(lo), b = Math.log10(hi); const d = (b - a || 1) * 0.05; return [10 ** (a - d), 10 ** (b + d)]; }
        if (hi === lo) return lo === 0 ? [-1, 1] : [lo - Math.abs(lo) * 0.05, hi + Math.abs(hi) * 0.05];
        const d = (hi - lo) * 0.05;
        return [lo - d, hi + d];
    };
    let [xlo, xhi] = pad(xmin, xmax, xlog);
    let [ylo, yhi] = pad(ymin, ymax, ylog);
    if (stickyY0 && !ylog && ymin >= 0) ylo = 0;
    if (Array.isArray(ax.xlim)) { if (ax.xlim[0] != null) xlo = ax.xlim[0]; if (ax.xlim[1] != null) xhi = ax.xlim[1]; }
    if (Array.isArray(ax.ylim)) { if (ax.ylim[0] != null) ylo = ax.ylim[0]; if (ax.ylim[1] != null) yhi = ax.ylim[1]; }

    if (ax.aspect === 'equal' && !xlog && !ylog) {
        const k = Math.min((px1 - px0) / (xhi - xlo), (py1 - py0) / (yhi - ylo));
        const w = k * (xhi - xlo), h = k * (yhi - ylo);
        const mx = (px1 - px0 - w) / 2, my = (py1 - py0 - h) / 2;
        px0 += mx; px1 -= mx; py0 += my; py1 -= my;
    }
    const sx: Scale = { lo: xlo, hi: xhi, log: xlog, px0, px1 };
    const sy: Scale = { lo: ylo, hi: yhi, log: ylog, px0: py1, px1: py0 };
    const X = (v: number) => r2(tr(sx, v));
    const Y = (v: number) => r2(tr(sy, v));

    const out: string[] = [];
    const clip = `${uid}-clip`;
    out.push(`<defs><clipPath id="${clip}"><rect x="${r2(px0)}" y="${r2(py0)}" width="${r2(px1 - px0)}" height="${r2(py1 - py0)}"/></clipPath></defs>`);

    // ticks and grid
    const ticksFor = (s: Scale, length: number, log: boolean) => {
        if (log) {
            const out: { v: number; label: string }[] = [];
            for (let e = Math.ceil(Math.log10(s.lo) - 1e-9); e <= Math.floor(Math.log10(s.hi) + 1e-9); e++) out.push({ v: 10 ** e, label: e >= -3 && e <= 4 ? String(10 ** e) : `1e${e}` });
            return out;
        }
        const { ticks, step } = niceTicks(s.lo, s.hi, Math.floor(length / 62));
        return ticks.map((v) => ({ v, label: formatTick(v, step) }));
    };
    const xt = ticksFor(sx, px1 - px0, xlog);
    const yt = ticksFor(sy, py1 - py0, ylog);
    const gridOn = !!ax.grid;
    const gridStyle = typeof ax.grid === 'object' && ax.grid ? ax.grid : {};
    const gridDash = gridStyle.linestyle && DASH[gridStyle.linestyle] ? ` stroke-dasharray="${DASH[gridStyle.linestyle]}"` : '';
    if (gridOn) {
        for (const t of xt) out.push(`<path d="M${X(t.v)} ${r2(py0)}V${r2(py1)}" stroke="#b0b0b0" stroke-width="0.8"${gridDash} fill="none"/>`);
        for (const t of yt) out.push(`<path d="M${r2(px0)} ${Y(t.v)}H${r2(px1)}" stroke="#b0b0b0" stroke-width="0.8"${gridDash} fill="none"/>`);
    }

    // data
    const data: string[] = [];
    let cycle = 0;
    const nextColor = () => CYCLE[cycle++ % CYCLE.length];
    const legend: { label: string; kind: 'line' | 'box' | 'marker'; color: string; dash?: string; width?: number; marker?: string }[] = [];
    for (const it of items) {
        if (it.type === 'line') {
            const f = typeof it.fmt === 'string' ? parseFmt(it.fmt) : {};
            const color = paint(it.color, f.color ?? nextColor());
            const style: string = it.linestyle ?? f.style ?? '-';
            const mk: string | undefined = it.marker ?? f.marker;
            const lw = num(it.linewidth, 1.6) * 1.0;
            const pts: string[] = [];
            const n = Math.min(it.x.length, it.y.length);
            let pen = false;
            let d = '';
            for (let i = 0; i < n; i++) {
                const xv = it.x[i], yv = it.y[i];
                if (!Number.isFinite(xv) || !Number.isFinite(yv)) { pen = false; continue; }
                d += `${pen ? 'L' : 'M'}${X(xv)} ${Y(yv)}`;
                pen = true;
                if (mk && mk !== 'None') pts.push(marker(mk, X(xv), Y(yv), num(it.markersize, 6) * 1.2, paint(it.markerfacecolor, color), paint(it.markeredgecolor, color)));
            }
            const dashed = style === 'none' || style === 'None' || style === '' || style === ' ';
            if (d && !dashed) data.push(`<path d="${d}" fill="none" stroke="${color}" stroke-width="${r2(lw)}" stroke-linejoin="round" stroke-linecap="round"${DASH[style] ? ` stroke-dasharray="${DASH[style]}"` : ''}${it.alpha != null ? ` opacity="${num(it.alpha, 1)}"` : ''}/>`);
            data.push(...pts);
            if (it.label) legend.push({ label: it.label, kind: dashed ? 'marker' : 'line', color, dash: DASH[style], width: lw, marker: mk });
        } else if (it.type === 'scatter') {
            const color = paint(it.color, nextColor());
            const mk = it.marker ?? 'o';
            const sz = Array.isArray(it.s) ? 0 : Math.sqrt(num(it.s, 36)) * 1.35;
            const n = Math.min(it.x.length, it.y.length);
            for (let i = 0; i < n; i++) {
                if (!Number.isFinite(it.x[i]) || !Number.isFinite(it.y[i])) continue;
                data.push(marker(mk, X(it.x[i]), Y(it.y[i]), sz || 8, color, color));
            }
            if (it.label) legend.push({ label: it.label, kind: 'marker', color, marker: mk });
        } else if (it.type === 'bar') {
            const color = paint(it.color, nextColor());
            for (let i = 0; i < it.x.length; i++) {
                const x0 = it.align === 'edge' ? it.x[i] : it.x[i] - it.width[i] / 2;
                const x1 = x0 + it.width[i];
                const y0 = it.bottom[i], y1 = it.bottom[i] + it.height[i];
                const xa = X(x0), xb = X(x1), ya = Y(y0), yb = Y(y1);
                data.push(`<rect x="${Math.min(xa, xb)}" y="${Math.min(ya, yb)}" width="${r2(Math.abs(xb - xa))}" height="${r2(Math.abs(yb - ya))}" fill="${color}"/>`);
            }
            if (it.label) legend.push({ label: it.label, kind: 'box', color });
        } else if (it.type === 'fill_between') {
            const color = paint(it.color, nextColor());
            const n = it.x.length;
            let d = '';
            for (let i = 0; i < n; i++) d += `${i ? 'L' : 'M'}${X(it.x[i])} ${Y(it.y1[i])}`;
            for (let i = n - 1; i >= 0; i--) d += `L${X(it.x[i])} ${Y(it.y2[i])}`;
            if (n) data.push(`<path d="${d}Z" fill="${color}" fill-opacity="${num(it.alpha, 1)}" stroke="none"/>`);
            if (it.label) legend.push({ label: it.label, kind: 'box', color });
        } else if (it.type === 'hline' || it.type === 'vline') {
            const color = paint(it.color, nextColor());
            const lw = num(it.linewidth, 1.6);
            const style: string = it.linestyle ?? '-';
            const dash = DASH[style] ? ` stroke-dasharray="${DASH[style]}"` : '';
            if (it.type === 'hline') data.push(`<path d="M${r2(px0)} ${Y(it.y)}H${r2(px1)}" stroke="${color}" stroke-width="${r2(lw)}"${dash} fill="none"/>`);
            else data.push(`<path d="M${X(it.x)} ${r2(py0)}V${r2(py1)}" stroke="${color}" stroke-width="${r2(lw)}"${dash} fill="none"/>`);
            if (it.label) legend.push({ label: it.label, kind: 'line', color, dash: DASH[style], width: lw });
        } else if (it.type === 'text') {
            const ha = it.ha === 'right' ? 'end' : it.ha === 'center' ? 'middle' : 'start';
            const va = it.va === 'top' ? 0.9 : it.va === 'center' ? 0.35 : 0;
            data.push(`<text x="${X(it.x)}" y="${r2(Y(it.y) + va * num(it.fontsize, 12) )}" text-anchor="${ha}" font-family="${FONT}" font-size="${num(it.fontsize, 12.5)}" fill="${paint(it.color, '#000')}">${esc(it.s)}</text>`);
        }
    }
    out.push(`<g clip-path="url(#${clip})">${data.join('')}</g>`);

    // frame, tick marks and labels
    out.push(`<rect x="${r2(px0)}" y="${r2(py0)}" width="${r2(px1 - px0)}" height="${r2(py1 - py0)}" fill="none" stroke="#000" stroke-width="1"/>`);
    for (const t of xt) {
        const x = X(t.v);
        out.push(`<path d="M${x} ${r2(py1)}v4" stroke="#000" stroke-width="1" fill="none"/><text x="${x}" y="${r2(py1 + 6 + fs.tick)}" text-anchor="middle" font-family="${FONT}" font-size="${r2(fs.tick)}" fill="#000">${esc(t.label)}</text>`);
    }
    for (const t of yt) {
        const y = Y(t.v);
        out.push(`<path d="M${r2(px0)} ${y}h-4" stroke="#000" stroke-width="1" fill="none"/><text x="${r2(px0 - 7)}" y="${r2(y + fs.tick * 0.35)}" text-anchor="end" font-family="${FONT}" font-size="${r2(fs.tick)}" fill="#000">${esc(t.label)}</text>`);
    }
    if (ax.title) out.push(`<text x="${r2((px0 + px1) / 2)}" y="${r2(py0 - 9 * scaleFont)}" text-anchor="middle" font-family="${FONT}" font-size="${r2(fs.title)}" fill="#000">${esc(ax.title)}</text>`);
    if (ax.xlabel) out.push(`<text x="${r2((px0 + px1) / 2)}" y="${r2(py1 + 14 + fs.tick + fs.label)}" text-anchor="middle" font-family="${FONT}" font-size="${r2(fs.label)}" fill="#000">${esc(ax.xlabel)}</text>`);
    if (ax.ylabel) {
        const lx = r2(cx + 8 * scaleFont + fs.label * 0.8);
        out.push(`<text transform="translate(${lx} ${r2((py0 + py1) / 2)}) rotate(-90)" text-anchor="middle" font-family="${FONT}" font-size="${r2(fs.label)}" fill="#000">${esc(ax.ylabel)}</text>`);
    }

    // legend
    const labels: unknown = typeof ax.legend === 'object' && ax.legend ? ax.legend.labels : undefined;
    if (ax.legend && legend.length) {
        const names = Array.isArray(labels) ? legend.map((l, i) => ({ ...l, label: String(labels[i] ?? l.label) })) : legend;
        const rowH = fs.legend + 6;
        const w = Math.max(...names.map((l) => l.label.length)) * fs.legend * 0.58 + 46;
        const h = names.length * rowH + 8;
        const loc = String(typeof ax.legend === 'object' ? (ax.legend.loc ?? 'best') : 'best');
        const right = !/left/.test(loc);
        const bottom = /lower/.test(loc);
        const lx0 = right ? px1 - w - 8 : px0 + 8;
        const ly0 = bottom ? py1 - h - 8 : py0 + 8;
        out.push(`<rect x="${r2(lx0)}" y="${r2(ly0)}" width="${r2(w)}" height="${r2(h)}" rx="3" fill="#fff" fill-opacity="0.85" stroke="#cccccc" stroke-width="1"/>`);
        names.forEach((l, i) => {
            const yy = ly0 + 4 + i * rowH + rowH / 2;
            if (l.kind === 'line') out.push(`<path d="M${r2(lx0 + 7)} ${r2(yy)}h22" stroke="${l.color}" stroke-width="${r2(l.width ?? 1.6)}"${l.dash ? ` stroke-dasharray="${l.dash}"` : ''} fill="none"/>`);
            else if (l.kind === 'box') out.push(`<rect x="${r2(lx0 + 7)}" y="${r2(yy - 5)}" width="22" height="10" fill="${l.color}"/>`);
            else out.push(marker(l.marker ?? 'o', lx0 + 18, yy, 7, l.color, l.color));
            out.push(`<text x="${r2(lx0 + 36)}" y="${r2(yy + fs.legend * 0.35)}" font-family="${FONT}" font-size="${r2(fs.legend)}" fill="#000">${esc(l.label)}</text>`);
        });
    }
    return out.join('');
}

export type { Json };
