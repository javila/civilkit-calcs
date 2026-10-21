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

## D4. Model binding comments are marked in the browser output (addition)

A comment line `'<!--civilkit: bind <variable> = <path> [@ <selector>] -->` before an input
is a model binding (docs/model-bindings.md). The browser build (calcpad-wasm) adds a "model"
tag with the binding path at the end of the bound input's line, or the error for a malformed
binding (`calcpad_core::model_bindings`). Upstream passes the comment through unseen, and so
does the CLI here, so CLI output and the stub comparison are unchanged. No model is connected
yet: the typed values are used.

## D5. The browser converts C-style operators when it opens a sheet

The desktop app and the CLI read a worksheet file through `CalcpadReader`, which turns the
C-style shortcuts outside comments into Calcpad's operators: `==` `!=` `>=` `<=` into
≡ ≠ ≥ ≤, `%%` `&&` `||` `<<` into ⦼ ∧ ∨ ∠ and `<*` into ←. Upstream's server API does
not, so a sheet written with them (Simply Supported Beams/Concentrated Forces.cpd uses `%%`)
fails there. The browser build is the desktop app here, so it converts a sheet when it opens
one (File > Open, drag and drop, recent files, Samples), in the editor text as the desktop
does. Included files are not converted, as in the CLI. The conversion is
`calcpad_core::extension_methods::replace_c_style_operators` (moved from the CLI), with a
TypeScript copy in the browser (`c-style.ts`); `web/e2e/c-style-check.mjs` holds the two equal
over every line of the upstream repository.

## D6. Word export: no upstream oracle, and the differences from Calcpad.OpenXml

`calcpad-docx` ports the Word path of Calcpad.OpenXml (OpenXmlWriter, TableBuilder,
StyleDefinitionsWriter, ImageWriter/ImageProcessor, CssParser) and calcpad-core ports
Output/XmlWriter.cs for the equations (`ExpressionParser::capture_open_xml`). The package is
written with calcpad-core's own zip and XML code, not the OpenXml SDK.

There is no upstream oracle. The live API at api.calcpad-ce.org answers `/convert` but has no
`/docx` (404) and ignores an `outputFormat`; .NET is not available here. The port is checked by:
tests that follow the C# rules (`calcpad-core/tests/open_xml.rs`, `calcpad-docx/tests/docx.rs`,
mutation-checked); `tools/docx_check.py`, which writes every sample as .docx, checks the XML,
counts the equations against the HTML and has LibreOffice open each document; and the browser
check (Save Word).

Differences from upstream's writer, each deliberate:

* **SVG images** (plots, drawings) are rasterised to PNG with resvg at twice their size and the
  SVG is kept beside the PNG as an `svgBlip`, so Word versions without SVG support show the
  PNG. Upstream embeds the SVG alone. A plot's label alignment, which Calcpad sets with
  `.plot text.end` style rules that SVG renderers in word processors ignore, is written as
  `text-anchor` attributes. The browser draws SVG text with DejaVu Sans; the CLI uses the
  installed fonts.
* **Ordered lists are numbered.** Upstream gives each list the highest numbering instance + 1
  without creating it, so Word shows no numbers; each ordered list gets its own instance here.
* **A unit power with a format** (`#format F2`, then `m^2`): upstream's FixPowerFormatting for
  Office Math keeps the number's first character in front of the reformatted number ("22");
  the number is replaced whole.
* **Valid nesting.** A paragraph that the HTML puts inside another (an image in a text line)
  goes into it inline instead of as a nested `<w:p>`, and a table cell always ends with a
  paragraph.
* **Unknown elements keep their content.** Upstream drops any element it has no case for
  together with everything in it, so a `<blockquote>` (Numerical Differentiation puts its
  error lines in one) or an `<abbr>` vanished from the document; here such an element passes
  its content on. Scripts, styles, form buttons and other non-content elements are still left
  out. The HTML is read as browsers read it in one more respect: a `<select>` closes a
  `<select>` left open (the I-Section Properties samples do this), where HtmlAgilityPack would
  take the rest of the form into the first list.
* **Images** come from data: URLs (the browser turns library images into these) or, in the
  CLI, files next to the worksheet. Web addresses are not fetched (upstream downloads them
  with a 2 s timeout); a missing image is reported in the document as upstream does.
* **No validation pass.** Upstream runs the SDK's validator and returns its messages; the
  checks above stand in for it.
* The Office Math is written without the indentation upstream's verbatim strings carry.

## D7. Excel export of the variables (addition)

CalcpadCE has no such export. File > Export Excel (and `Cli input.cpd output.xlsx`) saves the
calculated worksheet's variables as a workbook, written with the writer `#write` uses
(`openxml::variables_export`, recorded by `ExpressionParser::capture_values`):

* **Variables:** every variable in the order it was first assigned, with its last value, as
  name, value, unit. A vector or matrix names the sheet that holds it.
* **One sheet per vector or matrix:** the elements, each row with its unit.
* **Inputs:** the variables set by input fields (`?`) and `#UI` controls.

Numbers are numeric cells at full precision; units are text in their own column. A complex
value is written as text ("3 + 2i"). The report is calculated as for print, with the input
values and `#UI` entries the sheet holds. It is checked by `calcpad-core/tests/excel_export.rs`
(read back with the reader `#read` uses), by `tools/xlsx_check.py` (every sample, read with
openpyxl), and by the browser check.

Found on the way: the workbook writer ordered a row's cells by comparing their references as
text, so a 27th column ("AA1") went in front of "B1" and the sheet was invalid. It now compares
column numbers, which also fixes `#write` past column Z.

## Not deviations (scope and known differences)

These are not deliberate behaviour changes, but they can make output differ:

* **Out of scope** (raster `$Plot` output with `VectorGraphics=false`
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
