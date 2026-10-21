# calcpad-rs

A Rust port of the [CalcpadCE](https://github.com/imartincei/CalcpadCE) calculation engine
(Calcpad.Core), ported from commit `0cf438f`. It covers the scalar engine (real and complex
values, units, operators, functions, custom functions, solve blocks, the directives and
settings), vectors and matrices including the high performance (HP) types and their
solvers, `$Plot`/`$Map` SVG plots and charts, the markdown modes (a port of the Markdig
subset that CalcpadCE uses), `#UI` input forms, and `#read`/`#write`/`#append` with CSV,
text and Excel (.xlsx/.xlsm) files.

Word export (calcpad-docx, a port of Calcpad.OpenXml's Word path) is in the CLI
(`Cli input.cpd output.docx`) and the browser (Export > Save Word). Not ported: raster
(SkiaSharp) plot output, `#ProjectPath` and `#LibraryPath` path roots, and compiled `.cpdz`
worksheets. Raster plots report "not supported" errors.

## Layout

* `calcpad-core`: the engine. No threads; `#include` goes through the
  `parsers::macro_parser::FileProvider` trait and `#read`/`#write` through
  `data_files::DataFiles` (the disk natively, the page's files in the browser).
  `openxml` reads and writes the cell ranges of Excel workbooks (zip and XML, no SDK). Module names follow the Calcpad.Core folders
  (`base_types`, `calculator`, `output`, `parsers::math_parser`, `parsers::expression_parser`,
  ...), so a C# file can be found next to its port.
* `calcpad-cli`: the `Cli` binary, compatible with the batch mode of Calcpad.Cli
  (`Cli input.cpd output.html -b -s`).
* `calcpad-wasm`: WebAssembly bindings (wasm-bindgen). `convert` answers what
  Calcpad.Server's `POST /api/calcpad/convert` answers (the HTML page and the
  `X-Calcpad-Errors` list); `#include` files come from an in-memory map the page fills.
* `web/`: the CalcpadCE web editor (`Calcpad.Web/frontend`, MIT, see `web/LICENSE`), with
  its HTTP transport answered in the page by the wasm engine
  (`web/calcpad-web/src/wasm/transport.ts`). No calculation request leaves the page.
* `tools/`: generators for messages and exception constructors, the xunit transcriber, the
  stub comparison harness and `build_wasm.sh`.

## Gates

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`cargo test` runs the transcribed CalcpadCE xunit tests (`calcpad-core/tests/xunit_*.rs`,
regenerated with `tools/transcribe_all.sh`; `ui_directive.rs` and `data_exchange.rs` are
transcribed by hand) and the deviation tests. The HP comparison classes are heavy, so on a
busy machine run the suite through `python3 -I tools/run_tests.py [--minutes N]`, which
stops every test process when the time limit passes and reports a run that ended early.

Conformance against the committed `*.html.stub` files (uses the upstream
`compare_renderings.py` for discovery, the denylist and number reconciliation):

```sh
python3 -m venv .venv && .venv/bin/pip install beautifulsoup4==4.14.3
cargo build --release
.venv/bin/python -I tools/compare_stubs.py --ref ../calcpad-ref [--filter path] [--diff]
```

## Browser build

Needs the `wasm32-unknown-unknown` target, the wasm-bindgen CLI 0.2.122 (the version the
crate pins), node 22 and, optionally, `wasm-opt`.

```sh
tools/build_wasm.sh                       # web/calcpad-web/public/wasm
cd web && npm install && npm run build    # web/calcpad-web/dist
npm run preview -- --port 4173            # then open http://localhost:4173/
```

`npm run dev` serves the same with hot reload. The end-to-end check (headless Chromium via
Playwright: typing, a 30 worksheet sample through Export > Save HTML, network capture, the
server-only exports) and its stub comparison:

```sh
cd web && npx playwright install chromium && node e2e/browser-check.mjs && node e2e/c-style-check.mjs
cd .. && .venv/bin/python -I tools/compare_browser.py --ref ../calcpad-ref
```

The production build (`web/calcpad-web/dist`) is static files only and runs from any
folder of any static host; nothing it calculates goes over the network.

What works in the browser:

* the editor, preview, unwrapped view, report view, Variables and TOC panels;
* `#UI` input forms (entries, dropdowns, radio buttons, checkboxes and datagrids): enter
  values in the Input view, the report uses them, Save values writes them into the document;
* Samples (File > Samples, or the link on an empty editor): 101 of CalcpadCE's structural and
  maths examples (`tools/gen_samples.mjs`, attribution in `public/samples/NOTICE.md`) with
  groups, search, a design code filter and a preview; Open makes an untitled copy and puts the
  sample's images and data files into the library;
* File menu (menu bar, top): new, open (picker or drag and drop), save and save as (in
  place where the browser has the File System Access API, otherwise a download), recent
  worksheets, Ctrl+O / Ctrl+S / Ctrl+Shift+S;
* a file library for `#include` and `#read`: every opened worksheet and every added or
  dropped file (CSV, text, .xlsx/.xlsm) is kept in the browser (IndexedDB) and resolved at
  "/<name>";
* `#read` from CSV, text and Excel files, `#write`/`#append` to CSV and Excel: the written
  file joins the library and downloads whenever its content changes;
* PDF by printing: Print PDF and Export > Save PDF print the report through the browser's
  print dialog, with the paper size, orientation and margins of the PDF settings;
* Save HTML.
* worksheet scripts (Calcpad passes `<script>` in comments through) load only the libraries
  on Settings > Libraries, from their exact CDN addresses with SRI hashes, and cannot make
  network requests; anything else is blocked with a note in the preview (DEVIATIONS.md, D3).
  Insert > Libraries adds a listed library's script tag.

* File > Export Excel saves the variables as a workbook: all variables with values and units,
  a sheet per vector or matrix, and the inputs (calcpad-rs's own, DEVIATIONS.md, D7; also
  `Cli input.cpd output.xlsx`).
* Save Word writes the .docx in the page: the engine renders the equations as Office Math and
  calcpad-docx-wasm, a separate module loaded on the first export (about 600 KB gzipped, with
  DejaVu Sans fetched for the text of plots), builds the document (DEVIATIONS.md, D6).

What does not: compiled `.cpdz` worksheets and portable packages need Calcpad.Server and say
so. Linting, semantic highlighting, go-to-definition/rename, prettify
and the snippet library come from Calcpad.Highlighter, which is not ported, so the editor
runs without them. The PDF has the browser's header and footer, not Calcpad.Server's.

## Status

* Tests: 4828 pass (the transcribed xunit classes, UiDirectiveTests and the Data tests, the
  deviation and unit tests); 2 raster plot tests are ignored.
* Stubs (369 worksheets): 351 PASS, 16 PNG-PIXEL (`$Map` rasters with equal pixels but a
  different deflate stream), 1 NOISE, 1 FAIL (libm last digit), none out of scope. See
  `tools/classify_failures.py`, PORT_LOG.md and DEVIATIONS.md.
* Browser: 30 worksheets through the UI, 28 match the stubs; the other two differ only by
  one trailing empty paragraph, a Calcpad.Server versus Calcpad.Cli input difference (see
  DEVIATIONS.md). The `.wasm` is 1.30 MB (517 KB gzipped).

## Licence

MIT, see LICENSE and NOTICE. The original work is
Copyright (c) 2026 CalcpadCE Contributors and Copyright (c) 2014-2026 Nedelcho Ganchovski.
