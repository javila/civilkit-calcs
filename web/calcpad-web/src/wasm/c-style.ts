// The C-style operator shortcuts a worksheet may be written with, converted as the desktop app
// and the CLI convert them when they open a file (CalcpadReader.ReplaceCStyleOperators, in Rust
// calcpad_core::extension_methods::replace_c_style_operators). Outside comments: == != >= <=
// become ≡ ≠ ≥ ≤, %% && || << become ⦼ ∧ ∨ ∠, and <* becomes ←. The server API does not do
// this, so a sheet written with "%%" failed there (DEVIATIONS D5). Plain TypeScript with no
// imports, so tools/gen_samples.mjs and e2e/c-style-check.mjs can load it under Node; the
// parity check holds it to the Rust function over every upstream example.

/** One line. */
export function replaceCStyleLine(s: string): string {
    const out: string[] = [];
    const chars = Array.from(s);
    let i = 0;
    while (i < chars.length) {
        const c = chars[i];
        if (c === "'" || c === '"') {
            // A comment runs to the matching quote (inclusive) or the end of the line.
            let j = i + 1;
            while (j < chars.length && chars[j] !== c) j++;
            const stop = Math.min(j + 1, chars.length);
            for (let k = i; k < stop; k++) out.push(chars[k]);
            i = stop;
            continue;
        }
        const last = out.length ? out[out.length - 1] : undefined;
        const shortcut = (replace: string) => {
            if (last === c) out[out.length - 1] = replace;
            else out.push(c);
        };
        switch (c) {
            case '=':
                if (last === '=') out[out.length - 1] = '≡';
                else if (last === '!') out[out.length - 1] = '≠';
                else if (last === '>') out[out.length - 1] = '≥';
                else if (last === '<') out[out.length - 1] = '≤';
                else out.push(c);
                break;
            case '%': shortcut('⦼'); break;
            case '&': shortcut('∧'); break;
            case '|': shortcut('∨'); break;
            case '<': shortcut('∠'); break;
            case '*':
                if (last === '<') out[out.length - 1] = '←';
                else out.push(c);
                break;
            default: out.push(c);
        }
        i++;
    }
    return out.join('');
}

/** A whole worksheet as opened from a file: every line, except the saved input values that
 *  follow a vertical tab (the desktop's form data), which are left as they are. */
export function replaceCStyleOperators(text: string): string {
    return text.split('\n').map((line) => {
        const v = line.indexOf('\u000B');
        if (v < 0) return replaceCStyleLine(line);
        return replaceCStyleLine(line.slice(0, v)) + line.slice(v);
    }).join('\n');
}
