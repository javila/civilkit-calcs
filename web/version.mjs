// The version of CivilKit Calcs: the workspace version in the root Cargo.toml, the single source of truth
// (tools/set_version.mjs updates every manifest and checks that they agree). The upstream frontend this
// copy was taken from is tracked separately.
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const UPSTREAM_VERSION = '8.0.0-beta3-preview';

export function resolveVersion() {
    if (process.env.CALCPAD_VERSION) return process.env.CALCPAD_VERSION;
    const toml = readFileSync(join(dirname(fileURLToPath(import.meta.url)), '..', 'Cargo.toml'), 'utf8');
    const m = /\[workspace\.package\][^\[]*?\nversion\s*=\s*"([^"]+)"/.exec(toml);
    if (!m) throw new Error('no [workspace.package] version in Cargo.toml');
    return m[1];
}
