# Deliberate deviations from CalcpadCE

The port reproduces CalcpadCE output exactly unless a behaviour is listed here. Each entry
names the behaviour, why it differs, the test that pins it, and the conformance stubs
(`*.html.stub`) whose expected output it changes.

## D1. A failing `#if` / `#else if` condition runs no branch

**CalcpadCE:** when the condition expression throws (for example a misspelled variable,
`#if v > v_typo`), `ExpressionParser.ParseTokens` catches the error and `ParseLine` then
calls `Condition.Check(_parser.Result)`. `MathParser.Parse` reset the result to zero, so
the condition is treated as false and the `#else` branch runs. In calculation mode the
condition line is not part of the output (`isOutput` is false when the keyword length is
non-zero), so the error is not shown at all: the worksheet silently takes the `#else`
branch. With

```
#if v > v_typo
'NOT OK
#else
'OK
#end if
```

CalcpadCE prints `OK` and nothing else.

**calcpad-rs:** the block is left unresolved. Neither the branch of the failing condition
nor any later `#else if` / `#else` of the same chain runs (`Condition.check_failed` marks
the chain as found and false), and an error paragraph is written in place of the block:

```
Error in "v > v_typo" on line [n]: Undefined variable or units: "v_typo". The condition
could not be evaluated, so the block is unresolved and none of its branches were run.
```

Valid conditions, `#while` conditions and condition errors outside calculation mode are
unchanged.

**Tests:** `calcpad-core/tests/deviations.rs`
(`d1_failed_if_condition_runs_neither_branch`,
`d1_failed_else_if_condition_skips_the_rest_of_the_chain`,
`valid_conditions_are_unchanged`).

**Stubs changed:** none. One out-of-scope worksheet reaches the path, because of an
earlier out-of-scope failure: `Examples/Isaiah/UI Mode Test.cpd` (`useSteel` is defined by
an unsupported `#UI` line, so `#if useSteel ≡ 1` fails). The vector case listed in phase 1
(`Rayleigh Quotient Iteration.cpd`) now evaluates and passes.

## D2. Plot grid loops stop after 100,000 lines

**CalcpadCE:** the grid loops of `$Plot`/`$Map` step from the lower to the upper limit;
a NaN step never terminates (the worksheet hangs).

**calcpad-rs:** the loops stop after 100,000 iterations. Finite grids have far fewer lines,
so the output is unchanged for every terminating case.

**Tests:** none needed for finite cases; the guard only replaces a hang.

**Stubs changed:** none.

## D3. Worksheet scripts run only from a library list (browser build)

CalcpadCE passes HTML in comments through to the output, including `<script>`, and runs every
script a worksheet carries, from any address. The browser build gives each rendered page a
Content-Security-Policy instead (`web/calcpad-web/src/wasm/script-policy.ts`):

* `script-src` allows inline code (worksheet glue) and only the exact file URLs on the library
  list, never a whole host. Each listed `<script src>` gets `integrity=` (sha384) and
  `crossorigin="anonymous"`; module files are checked through an import map.
* A `<script src>` that is not on the list is not loaded and is replaced by a note naming its
  URL; anything else the policy stops while the page runs (a module import, a script added
  later, an image from another site) is reported by a note as well. There is no prompt.
* `connect-src blob:` and `form-action 'none'`: a script cannot send data anywhere. `blob:`
  is allowed because it is data the page made itself (the DXF examples hand a drawing built in
  the page to dxf-viewer that way); it cannot reach another host.
* Images, fonts and styles come only from `data:`, `blob:`, inline styles and the app's origin.

The list starts with the eight libraries the upstream examples use (jQuery 3.6.3, D3 3.5.17,
Plotly 2.26.0, three.js 0.145 with OrbitControls, three.js 0.160.0 as a module, dxf-viewer
1.0.46 with its module dependencies, dxf-writer 1.18.4, model-viewer 3.0.1), hashed by
`tools/gen_library_defaults.mjs` from the files as served. Settings > Libraries removes and
restores entries and adds new ones after showing the fetched file's size and hash. Effects on
the upstream examples: `Toc.cpd`'s relative `./toc.js` is not a library and is blocked, the
texture image in `Parametric 3D Drawing of RC Beam.cpd` comes from another site and is
blocked, and `Bolt 3D Model.cpd` cannot fetch its `.glb` model.

Because the added `integrity`/`crossorigin` attributes are part of the page (Save HTML keeps
them too), `3D Plot with Plotly.cpd` in the browser sample no longer matches its stub byte for
byte: the browser comparison reads 27 of 30, the third difference being those two attributes.

The print frame has no scripts at all: the page is stripped of script, frame, object and
embed elements, `on*` attributes and `javascript:` URLs, and the frame is sandboxed with
`allow-same-origin allow-modals` (`print.ts`).

## Not deviations (scope and known differences)

These are not deliberate behaviour changes, but they can make output differ:

* **Out of scope** (Word/OpenXml export, raster `$Plot` output with `VectorGraphics=false`
  or `ImagePath` set): the engine reports `... is not supported by calcpad-rs yet ...` as an
  ordinary expression error and records the feature so the harness classifies the worksheet
  as OUT-OF-SCOPE. `#ProjectPath`/`#LibraryPath` roots are accepted and ignored.
* **`#write` line endings:** CSV rows end with `\n` (.NET on Linux and macOS); Calcpad on
  Windows writes `\r\n`.
* **Excel workbooks written by `#write`** are built by `calcpad-core/src/openxml` rather than
  the DocumentFormat.OpenXml SDK. They hold the same cells (numbers as numbers, other text as
  inline strings) and read back the same, but the package parts are laid out differently;
  writing over a cell also drops what it held before (the SDK keeps an old value element
  beside a new inline string).
* **Browser build, `#write`:** a written file is kept in the page's file library and
  downloaded when its content differs from the library copy, rather than overwritten on disk
  on every render. Writes run only where upstream's write mode allows them (by default, the
  report).
* **Browser build, PDF:** printed by the browser (`@page` size, orientation and margins from
  the PDF settings); Calcpad.Server's header and footer (title, date, page numbers) are
  replaced by the browser's own print header and footer.
* **`$Map` PNG encoding:** CalcpadCE encodes the colour map with SkiaSharp (libpng and
  Chromium's zlib, level 4). calcpad-rs computes identical pixels and writes the same chunk
  layout, but its deflate stream differs, so the base64 text differs. `tools/compare_png.py`
  and `tools/classify_failures.py` report these worksheets as PNG-PIXEL (pixel-equal).
* **Floating point library:** `f64::sin`, `exp`, `pow` and friends come from the platform
  libm (macOS here); CalcpadCE uses the .NET runtime's. Last digit differences are absorbed
  by the harness except where a stub prints an exact integer (the reconciliation ignores
  integers): `Tests/Others/Integration tests.cpd` (`I_191`, classified NOISE) and
  `Examples/Engineering/Getting Started/Functions.cpd`, where `pow(1.4999999999999998, 2)`
  is one ulp off on macOS, the plot minimum becomes exactly 1.75 and one extra grid line is
  drawn (a structural difference, so it stays FAIL).
* **Browser build follows Calcpad.Server, the stubs follow Calcpad.Cli:** the CLI reads a
  worksheet through `CalcpadReader.Read`, whose `EnumerateLines` drops the final line break,
  while the server (and so `calcpad-wasm`) gives the editor text to the macro parser as it
  is. A worksheet ending in several blank lines therefore renders one more empty `<p>` in
  the browser than in its stub (`Tests/HP/Matrices/Functions/eigen functions.cpd`,
  `Tests/Matrices/Operators/scalar-operator-matrix/diagonal.cpd` in the browser sample).
  Removing one trailing newline gives the stub's output.
* **Browser build, report fonts:** Calcpad.Server embeds its bundled fonts in every page as
  data URLs (`BundledFonts`). `calcpad-wasm` does not compile them in (they would add about
  640 KB to the module); the page fetches them once from `report-fonts/` and passes the same
  `<style>` and `window.__calcpadFonts` markup with each request, so the HTML matches.
* **Browser build, code view:** the unwrapped/macro-error listing is not syntax highlighted,
  which is Calcpad.Server's own fallback when the Calcpad.Highlighter tokenizer fails.
