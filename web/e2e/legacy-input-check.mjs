// Unit checks of src/wasm/legacy-input.ts (the `name = ? {default}` to #UI rewrite).
//   node e2e/legacy-input-check.mjs
import { convertLegacyInputs, convertLegacyInputsInLine } from '../calcpad-web/src/wasm/legacy-input.ts';

let failed = 0;
const eq = (got, want, what) => {
    const ok = JSON.stringify(got) === JSON.stringify(want);
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}${ok ? '' : ` (got ${JSON.stringify(got)})`}`);
    if (!ok) failed++;
};
const line = (s) => convertLegacyInputsInLine(s).line;

eq(line('L = ?{6}m'), '#UI L = 6m', 'the migration guide example');
eq(line("'Load -'q = ? {5}'kN/m"), "#UI 'Load -'q = 5'kN/m", 'a label comment before, a unit comment after');
eq(line("'Distances -'a = ? {1}'m,'b = ? {2}'m"), "#UI 'Distances -'a = 1'm,'b = 2'm", 'several fields on one line');
eq(line("\tx_1 = ? {0.85}"), "\t#UI x_1 = 0.85", 'indentation stays in front');
eq(line("'text ? {5}' y = 2"), "'text ? {5}' y = 2", 'a ? { inside a comment is text');
eq(line("a = ? {1e3}"), "a = ? {1e3}", 'exponent notation is not valid in #UI: left alone');
eq(line("a = 2*? {3}"), "a = 2*? {3}", 'not a plain assignment: left alone');
eq(line("a = ? {3} + b"), "#UI a = 3 + b", 'text after the brace stays (the engine decides if #UI accepts it)');
eq(line("#if a = ? {1}"), "#if a = ? {1}", 'another directive: left alone');
eq(line("#UI a = 5"), "#UI a = 5", 'nothing to do');
eq(line("a = ? {-2.5}kN"), "#UI a = -2.5kN", 'sign and unit');
eq(line("a ≡ ? {1}"), "a ≡ ? {1}", 'a comparison is not an assignment');
eq(convertLegacyInputs("#html\nx = ? {1}\n#end html\ny = ? {2}\r\n").text, "#html\nx = ? {1}\n#end html\n#UI y = 2\r\n", 'html blocks are content; CRLF kept');
eq(convertLegacyInputs("a = ? {1}\nb = ? {2}", new Set([1])).text, "a = ? {1}\n#UI b = 2", 'a line in the keep set is left as written');
process.exit(failed ? 1 : 0);
