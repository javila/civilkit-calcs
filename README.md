# CivilKit Calcs

CivilKit Calcs is a browser-based engineering calculation sheet. You write a worksheet in the
CalcpadCE language (equations with units, vectors and matrices, plots, input forms, text and
tables), and the page calculates it, shows a formatted report, and exports it to PDF (by
printing), Word and Excel. Everything runs in the page: the calculation engine is Rust compiled to
WebAssembly, and no calculation request leaves the browser.

* Live demo: https://civilkit.io/calcs/ (also at https://mageaustralia.github.io/civilkit-calcs/)
* Source: https://github.com/mageaustralia/civilkit-calcs
* Version: see [CHANGELOG.md](CHANGELOG.md). Releases are numbered YY.M.N (26.10.1 is the first release of
  October 2026) and tagged `v26.10.1`.

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
* **Known deviations** from CalcpadCE's behaviour are listed one by one, with the test that pins
  each, in [DEVIATIONS.md](DEVIATIONS.md). Not supported: raster (SkiaSharp) plots,
  `#ProjectPath`/`#LibraryPath` path roots, and compiled `.cpdz` worksheets.

What has and has not been checked for the exports: the Excel workbook was checked by opening it in
openpyxl and in LibreOffice; it has not been opened in Microsoft Excel. The Word document was
checked in LibreOffice only; it has not been opened in Microsoft Word. Printing to PDF goes through
the browser's print dialog and has the browser's header and footer.

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
* `web/`: the web app. `web/calcpad-frontend` is the shared frontend, `web/calcpad-web` the page
  (Vue, Monaco), `web/e2e` the browser checks.
* `tools/`: generators, the test runner, `build_wasm.sh`, `set_version.mjs`.
* `docs/`: [DEPLOY.md](docs/DEPLOY.md) and the model-bindings specification.

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
node ../tools/set_version.mjs --check
```

## Checked at 26.10.1

`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean; `cargo test` ran 5234 tests with
none failing (3 ignored); `web/e2e/browser-check.mjs` passed all 122 checks against the build served at `/` and against
the `CALCS_BASE=/calcs/` build; `web/e2e/ribbon-check.mjs` and `web/e2e/verify-ribbon-snippets.mjs` pass (every library
snippet except two that need a data list was run through the engine without an error).

## Contributing

Issues and pull requests are welcome. Keep changes to the engine behind a test; where behaviour
differs from CalcpadCE, record it in DEVIATIONS.md with the test that pins it. Run the gates above
before sending a change. To bump the version, see [docs/DEPLOY.md](docs/DEPLOY.md).

## Licence

MIT. See [LICENSE](LICENSE) (the upstream licence text and copyright lines are kept, with ours
added), [NOTICE](NOTICE) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md). Bundled fonts,
the spreadsheet widget and the sample worksheets carry their own licence files next to them.
