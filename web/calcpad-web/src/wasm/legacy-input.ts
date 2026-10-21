// The deprecated stored input field `name = ? {default}` (CalcpadCE's linter warns CPD-3419:
// "declare the input with #UI instead") rewritten as the #UI declaration its migration guide
// gives (docs/migration-guide.md: `L = ?{6}m` becomes `#UI L = 6m`):
//
//   * the `? {`, `}` around the default go, the default stays where it was, so the variable
//     name, the value and any unit written after the brace are unchanged;
//   * `#UI ` is put at the start of the line (after its indentation), once. A #UI line may
//     carry comments before and between its assignments (docs/new-ui-mode.md: "Labels",
//     "Several controls on one line"), which is how upstream's sheets write their fields.
//
// Deterministic and conservative: a line is changed only when every field on it is a plain
// number (no exponent, which #UI rejects) assigned with a single `=` to a name, and the line is
// not another directive. Anything else is counted in `skipped` and left as written. Plain
// TypeScript with no imports, so tools/gen_samples.mjs (Node) and the editor share it.

export interface LegacyInputLine {
    line: string;
    /** Fields rewritten. */
    converted: number;
    /** Fields found but left alone (the whole line is then left as written). */
    skipped: number;
}

const NUMBER = /^[+-]?(\d+\.?\d*|\.\d+)$/;
const NAME_BEFORE_EQUALS = /[\p{L}\p{N}_,′″‴⁗°ø∡℧̀-ͯ₀-₉⁰-⁹]$/u;

/** One line (no line ending). */
export function convertLegacyInputsInLine(line: string): LegacyInputLine {
    const chars = Array.from(line);
    const fields: { start: number; end: number; value: string }[] = [];
    let skipped = 0;
    let quote = '';
    for (let i = 0; i < chars.length; i++) {
        const c = chars[i];
        if (quote) {
            if (c === quote) quote = '';
            continue;
        }
        if (c === "'" || c === '"') {
            quote = c;
            continue;
        }
        if (c !== '?') continue;
        let j = i + 1;
        while (j < chars.length && chars[j] === ' ') j++;
        if (chars[j] !== '{') continue;
        let end = j + 1;
        while (end < chars.length && chars[end] !== '}') end++;
        if (end >= chars.length) { skipped++; break; }
        const value = chars.slice(j + 1, end).join('').trim();
        // Preceded by `name =` (a single equals sign)?
        let k = i - 1;
        while (k >= 0 && chars[k] === ' ') k--;
        const isAssignment = k >= 0 && chars[k] === '=' && !'=!<>≡≠≤≥'.includes(chars[k - 1] ?? ' ');
        let n = k - 1;
        while (n >= 0 && chars[n] === ' ') n--;
        const named = isAssignment && n >= 0 && NAME_BEFORE_EQUALS.test(chars[n]);
        if (named && NUMBER.test(value)) fields.push({ start: i, end: end + 1, value });
        else skipped++;
        i = end;
    }
    if (skipped > 0 || fields.length === 0) return { line, converted: 0, skipped };
    const trimmed = line.trimStart();
    if (trimmed.startsWith('#')) return { line, converted: 0, skipped: fields.length };
    for (const f of [...fields].reverse()) chars.splice(f.start, f.end - f.start, ...Array.from(f.value));
    const indent = line.length - trimmed.length;
    const out = chars.join('');
    return { line: out.slice(0, indent) + '#UI ' + out.slice(indent), converted: fields.length, skipped: 0 };
}

export interface LegacyInputDocument {
    text: string;
    converted: number;
    skipped: number;
}

/**
 * A whole document: lines in `#html` and `#markdown` blocks are content, not code. `keep` holds
 * 1-based line numbers to leave as written (the generator puts back a line the engine rejects
 * as a #UI line, e.g. one that also assigns an expression).
 */
export function convertLegacyInputs(text: string, keep: ReadonlySet<number> = new Set()): LegacyInputDocument {
    let converted = 0;
    let skipped = 0;
    let block = '';
    const lines = text.split('\n').map((raw, index) => {
        const cr = raw.endsWith('\r') ? '\r' : '';
        const line = cr ? raw.slice(0, -1) : raw;
        const t = line.trim().toLowerCase();
        if (block) {
            if (t === `#end ${block}`) block = '';
            return raw;
        }
        if (t === '#html' || t === '#markdown') {
            block = t.slice(1);
            return raw;
        }
        if (keep.has(index + 1)) {
            skipped += convertLegacyInputsInLine(line).converted;
            return raw;
        }
        const r = convertLegacyInputsInLine(line);
        converted += r.converted;
        skipped += r.skipped;
        return r.line + cr;
    });
    return { text: lines.join('\n'), converted, skipped };
}
