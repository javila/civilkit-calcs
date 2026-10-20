# calcpad-rs

A Rust port of the [CalcpadCE](https://github.com/imartincei/CalcpadCE) calculation engine
(Calcpad.Core), ported from commit `0cf438f`. It covers the scalar engine (real and complex
values, units, operators, functions, custom functions, solve blocks, the directives and
settings), vectors and matrices including the high performance (HP) types and their
solvers, `$Plot`/`$Map` SVG plots and charts, and the markdown modes (a port of the Markdig
subset that CalcpadCE uses).

Not ported: `#UI` forms, `#read`/`#write`/`#append`, Word/OpenXml export and raster
(SkiaSharp) plot output. These report "not supported" errors.

## Layout

* `calcpad-core`: the engine. No I/O, no threads; `#include` goes through the
  `parsers::macro_parser::FileProvider` trait. Module names follow the Calcpad.Core folders
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
regenerated with `tools/transcribe_all.sh`) and the deviation tests.

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
cd web && npx playwright install chromium && node e2e/browser-check.mjs
cd .. && .venv/bin/python -I tools/compare_browser.py --ref ../calcpad-ref
```

In the browser: the editor, preview, unwrapped view, input form and report views, the
Variables and TOC panels, Save HTML. Not available, with a message saying so: PDF and Word
export, compiled `.cpdz` and portable packages (they need Calcpad.Server). Not ported, so
the editor runs without them: linting, semantic highlighting, go-to-definition/rename,
prettify and the snippet library (all from Calcpad.Highlighter). `#UI` forms and
`#read`/`#write` report "not supported" as in the CLI.

## Status

* xunit: 4692 transcribed tests pass (Scalars, Vectors, Matrices incl. HP, MathParser,
  Macros, ExpressionParser and the others); 2 raster plot tests are ignored.
* Stubs (369 worksheets): 343 PASS, 16 PNG-PIXEL (`$Map` rasters with equal pixels but a
  different deflate stream), 1 NOISE, 1 FAIL (libm last digit), 8 OUT-OF-SCOPE (`#UI`,
  `#read`). See `tools/classify_failures.py`, PORT_LOG.md and DEVIATIONS.md.
* Browser: 30 worksheets through the UI, 28 match the stubs; the other two differ only by
  one trailing empty paragraph, a Calcpad.Server versus Calcpad.Cli input difference (see
  DEVIATIONS.md). The `.wasm` is 1.16 MB (458 KB gzipped).

## Licence

MIT, see LICENSE and NOTICE. The original work is
Copyright (c) 2026 CalcpadCE Contributors and Copyright (c) 2014-2026 Nedelcho Ganchovski.
