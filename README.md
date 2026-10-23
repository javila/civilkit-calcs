# CivilKit Calcs

CivilKit Calcs is a browser-based engineering calculation sheet. You write a worksheet in the
CalcpadCE language (equations with units, vectors and matrices, plots, input forms, text and
tables), and the page calculates it, shows a formatted report, and exports it to PDF (by
printing), Word and Excel. Everything runs in the page: the calculation engine is Rust compiled to
WebAssembly, and no calculation request leaves the browser.

* Live demo: https://civilkit.io/calcs/ (also at https://mageaustralia.github.io/civilkit-calcs/)
* Source: https://github.com/mageaustralia/civilkit-calcs
* Version: see [CHANGELOG.md](CHANGELOG.md). Releases are numbered YY.M.N (26.10.1 was the first release of
  October 2026; this is 26.10.2) and tagged `v26.10.2`.

Status: early. The project is not affiliated with the authors of CalcpadCE.

## Credit: this is a derivative of CalcpadCE

CivilKit Calcs is built on [CalcpadCE](https://github.com/imartincei/CalcpadCE), the MIT-licensed
continuation of Calcpad by Nedelcho Ganchovski (Proektsoft) and contributors. This repository
contains:

* CalcpadCE's web frontend (`web/`, imported from `Calcpad.Web/frontend`, with changes), which
  provides the editor, sidebar, settings, the worksheet language's editor support and the report view;
* the CalcpadCE worksheet language and its example worksheets (the 101 bundled samples are copies
  of CalcpadCE's `Examples/`, see `web/calcpad-web/public/samples/NOTICE.md`);
* Rust reimplementations of CalcpadCE's calculation engine (Calcpad.Core), its editor language
  services (Calcpad.Highlighter: tokenizer, linter, prettifier, snippet library) and its Word
  export path (Calcpad.OpenXml). They follow the upstream module structure so that upstream
  changes can be followed file by file, and they are tested against upstream's own test classes
  and its expected-output files (see Tests below).

The upstream copyright and licence are kept in [LICENSE](LICENSE) and [NOTICE](NOTICE). The upstream
versions this work tracks are listed in NOTICE: the frontend 8.0.0-beta3-preview, the engine ported
from commit `0cf438f`. CalcpadCE's own version is tracked there, separately from the CivilKit Calcs version.

## Rust crates

The Rust libraries are published on crates.io (0.1.0, early; the API may change between minor versions):

* [`civilkit-calcs-engine`](https://crates.io/crates/civilkit-calcs-engine): the calculation engine.
* [`civilkit-calcs-highlighter`](https://crates.io/crates/civilkit-calcs-highlighter): tokenizer, linter, prettifier and snippets.
* [`civilkit-calcs-docx`](https://crates.io/crates/civilkit-calcs-docx): Word and Excel export.

Their Rust crate names are unchanged (`calcpad_core`, `calcpad_highlighter`, `calcpad_docx`), so `use calcpad_core::...` works
with `civilkit-calcs-engine = "0.1"` in `Cargo.toml`. The crates have their own 0.x semver, separate from the app's YY.M.N version.

## What we changed

* **Engine:** the Rust engine, compiled to WebAssembly, replaces the .NET engine and the server it
  runs in. The page answers the server API itself (`web/calcpad-web/src/wasm/transport.ts`).
  The engine wasm is 1.74 MB raw and 656 KB gzipped; it is built at opt-level `s`, which ran a heavy
  `$Map` sheet 1.5 times faster than the size-optimised build.
* **Editor services in a second wasm:** the highlighter, linter, prettifier, go-to-definition and
  the snippet library are a separate module (`calcpad-highlighter-wasm`, 351 KB gzipped) that loads
  on first use.
* **Word export:** `Export > Save Word` builds the .docx in the page (module `calcpad-docx-wasm`,
  loaded on first use). Equations are written as Office Math.
* **Excel export:** a Report sheet that mirrors the worksheet (headings and text, each equation as
  a row of name, expression, expression with values, value and unit, tables as cell ranges,
  `$Plot` as native editable charts with their data on a Plot data sheet, other drawings as
  pictures), then a Variables sheet, one sheet per vector or matrix, and an Inputs sheet.
* **`#UI` input forms:** entries, drop-downs, radio buttons, checkboxes and data grids. A chosen
  select, radio or checkbox is carried into the report, PDF and Word output.
* **Dark theme:** the preview keeps form controls, drawings and plots readable.
* **Import worksheet:** `File > Import worksheet...`, the Document tab's Import button, or dropping a `.xmcd` file on
  the page converts a worksheet of the spreadsheet-like format of another calculation tool into a new unsaved
  worksheet, locally in the page (its own WebAssembly module, loaded on first use). Text, definitions,
  evaluations, functions, matrices, units and pictures convert; what does not is shown as a static picture of the
  original when the file has one, and counted in a summary. See [docs/worksheet-import.md](docs/worksheet-import.md).
* **Samples browser:** 101 worksheets, all copied from CalcpadCE's examples (structural and maths),
  with groups, search and a preview. We rewrote their deprecated `? {default}` input fields as
  `#UI` declarations, and kept a conversion only where the engine renders it like the original.
* **Ribbon interface:** a tabbed ribbon (ten tabs, a quick-access strip) in place of the menu bar. Each
  palette opens a Common section above the whole snippet library, which the ribbon reads from the same
  highlighter call as the sidebar's Insert tab (1166 snippets, all reachable from the ribbon).
* **Model bindings:** a worksheet input can carry a `civilkit: bind` comment that ties it to a
  quantity of a structural model. The parser and the in-app display exist; connecting a model does
  not yet. The generic specification is in [docs/model-bindings.md](docs/model-bindings.md).
* **Worksheet scripts:** scripts in worksheets run only from a library list, under a
  Content Security Policy, with the print frame sandboxed (DEVIATIONS.md, D3).
* **Sheet syntax (our addition, not part of CalcpadCE):** the main way to write a worksheet here. Markdown
  prose, `name: expression -> unit | note` lines, LaTeX, indented `for` (with `step`), `while` and `if` blocks
  with `break` and `continue`, plots, and `>>` lines that pass raw Calcpad through. File > New and the quick-access New
  button open a sheet; New Calcpad worksheet opens the advanced syntax. A sheet compiles to the Calcpad worksheet language
  and runs on the same engine (a front end, not a second engine), so units, vectors, plots and the Word and
  Excel exports all work, and Show compiled / Convert to Calcpad give the Calcpad a person would write. The
  compiler is the crate `calcpad-sheet` in its own 39 KB gzipped wasm (`calcpad-sheet-wasm`); the preview
  renders LaTeX with the bundled KaTeX, loaded only when a sheet has a formula. The editor formats, completes,
  highlights, and goes to definition, finds references and hovers on a sheet's names; errors and lint markers
  are mapped back to sheet lines; the ribbon and Insert tab insert sheet forms. See
  [docs/sheet-syntax.md](docs/sheet-syntax.md).
* **Changelog:** a button on the Resources tab and a link in About open
  [CHANGELOG.md](CHANGELOG.md) on GitHub.
* **Known deviations** from CalcpadCE's behaviour are listed one by one, with the test that pins
  each, in [DEVIATIONS.md](DEVIATIONS.md). Not supported: raster (SkiaSharp) plots,
  `#ProjectPath`/`#LibraryPath` path roots, and compiled `.cpdz` worksheets.

Export testing: the Excel workbook opens cleanly in openpyxl and LibreOffice, but we have not tried it in
Microsoft Excel. The Word document has been checked in LibreOffice only, not in Microsoft Word. PDF output uses
the browser's print dialog, so it carries the browser's own header and footer.

## Layout

* `calcpad-core`: the engine. No threads; `#include` goes through a file-provider trait and
  `#read`/`#write` through `data_files::DataFiles` (the disk natively, the page's files in the
  browser). `openxml` reads and writes Excel cell ranges (zip and XML, no SDK). Module names follow
  the Calcpad.Core folders.
* `calcpad-cli`: the `Cli` binary, compatible with the batch mode of Calcpad.Cli
  (`Cli input.cpd output.html -b -s`, also `.docx` and `.xlsx` outputs).
* `calcpad-highlighter` and `calcpad-highlighter-wasm`: the editor language services and their
  WebAssembly binding. The snippet data is `calcpad-highlighter/src/snippets/snippets.json`.
* `calcpad-wasm`: the engine's WebAssembly binding (wasm-bindgen).
* `calcpad-docx` and `calcpad-docx-wasm`: the Word and Excel writers and their binding.
* `calcpad-import-xmcd` (package `civilkit-calcs-import-xmcd`): a spike that translates the simple subset of
  a foreign worksheet format (`.xmcd`) into Calcpad text, with a report of what did not convert.
  See [Importing foreign worksheets](#importing-foreign-worksheets-spike).
* `calcpad-sheet` and `calcpad-sheet-wasm`: the sheet syntax compiler and its binding.
* `web/`: the web app. `web/calcpad-frontend` is the shared frontend, `web/calcpad-web` the page
  (Vue, Monaco), `web/e2e` the browser checks.
* `tools/`: generators, the test runner, `build_wasm.sh`, `set_version.mjs`.
* `docs/`: [DEPLOY.md](docs/DEPLOY.md), the model-bindings specification and
  [the foreign worksheet importer](docs/worksheet-import.md).

## Importing foreign worksheets (spike)

`calcpad-import-xmcd` reads the `.xmcd` files of the spreadsheet-like worksheet format of a commercial
calculation tool and writes Calcpad worksheet text for this engine, so existing sheets can be brought
over. It evaluates nothing itself. Text, definitions, evaluations, functions, vectors and matrices,
indexing (with the array origin shifted to Calcpad's 1), units, the common built-in functions and picture regions
(metafile to SVG) convert.
Range variables, programs, solve blocks, symbolic evaluation, plots, pictures and embedded objects do
not: each becomes a static picture of the original when the file has one, else a visible placeholder line
carrying its region id, and the report counts them by code.
The newer `.mcdx` container is not read yet. Details, the mapping tables and how to run the batch
runner over a folder: [docs/worksheet-import.md](docs/worksheet-import.md).

```text
cargo run -p civilkit-calcs-import-xmcd --example import -- sheet.xmcd
```

## Build and run

Needs Rust with the `wasm32-unknown-unknown` target, the wasm-bindgen CLI 0.2.122 (the version the
crate pins), node 22 and, optionally, `wasm-opt`.

```sh
tools/build_wasm.sh                       # web/calcpad-web/public/wasm
cd web && npm install && npm run build    # web/calcpad-web/dist
npm run preview -- --port 4173            # then open http://localhost:4173/
```

`npm run dev` serves the same with hot reload. The production build is static files only; it
runs from any folder of any static host. To build for a fixed path, set the base:
`CALCS_BASE=/calcs/ npm run build` (see [docs/DEPLOY.md](docs/DEPLOY.md)).

The command line engine: `cargo build --release`, then `target/release/Cli input.cpd output.html`.

## Tests

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`cargo test` runs the xunit classes of CalcpadCE transcribed to Rust (`calcpad-core/tests/xunit_*.rs`,
regenerated with `tools/transcribe_all.sh`), the highlighter's transcribed tests and linter corpus,
and the deviation tests. The HP comparison classes are heavy; on a busy machine run
`python3 -I tools/run_tests.py [--minutes N]`, which stops every test process when the time limit
passes. Conformance against upstream's expected-output files (`*.html.stub`):

```sh
python3 -m venv .venv && .venv/bin/pip install beautifulsoup4==4.14.3
cargo build --release
.venv/bin/python -I tools/compare_stubs.py --ref ../calcpad-ref [--filter path] [--diff]
```

Browser checks (headless Chromium via Playwright, against the built `dist`):

```sh
cd web && npx playwright install chromium
node e2e/browser-check.mjs        # worksheets through the UI, input forms, files, exports, scripts, printing
node e2e/ribbon-check.mjs --url http://127.0.0.1:4173/ --lint
node e2e/verify-ribbon-snippets.mjs --cli ../target/release/Cli
node e2e/import-check.mjs         # Import worksheet: a synthetic .xmcd, pictures, toggles, error paths, drop, save, exports
node ../tools/set_version.mjs --check
```

## Checked at 26.10.3

`cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean; `cargo test` ran 5369 tests
with none failing (3 ignored); every wasm module was rebuilt (the sheet compiler is 95 KB, 44 KB gzipped; the engine
module is unchanged in size); `web/e2e/sheet-check.mjs` (sheet samples, LaTeX, error mapping, mixing with raw
Calcpad, the compiled view, Convert to Calcpad, Word and Excel, `.sheet` open, edit, save and reopen, prettify,
go to definition, hover, references, the ribbon, Text Formatting and Insert tab forms, New, the changelog link),
`web/e2e/verify-ribbon-sheet.mjs` (407 ribbon inserts in sheet form through the compiler and the engine),
`web/e2e/import-check.mjs` and `web/e2e/ribbon-check.mjs --lint` pass. `web/e2e/browser-check.mjs` passed 119 checks
on the build served at `/`; its mapped-controls step (`select-map-check.mjs`, real clicks in a preview that is
replaced by every render) failed intermittently on a machine under heavy load, and it does the same on the 26.10.2
build, so it is not counted as passed here.

## Contributing

Issues and pull requests are welcome. Keep changes to the engine behind a test; where behaviour
differs from CalcpadCE, record it in DEVIATIONS.md with the test that pins it. Run the gates above
before sending a change. To bump the version, see [docs/DEPLOY.md](docs/DEPLOY.md).

## Licence

MIT. See [LICENSE](LICENSE) (the upstream licence text and copyright lines are kept, with ours
added), [NOTICE](NOTICE) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md). Bundled fonts,
the spreadsheet widget and the sample worksheets carry their own licence files next to them.
