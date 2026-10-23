# Changelog

All notable changes are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions are numbered YY.M.N with no zero padding (26.10.1 is the first release of October 2026; N counts the releases
of the month and restarts at 1), which is valid semver for Cargo and npm. Tags are `v26.10.2` and so on.
CalcpadCE's own version is tracked separately in NOTICE.

The app changes are listed under their YY.M.N version below. The three Rust library crates have their own entries
under "Crates" at the end, on plain 0.x semver, and are tagged `<crate-name>-v<version>`
(for example `civilkit-calcs-engine-v0.1.0`).

## [26.10.4] - 2026-10-23

### Added
- Python blocks. A worksheet can hold `#python` ... `#end python` blocks, and a sheet a fenced ```python block, that run on
  MicroPython in a worker before the engine and become part of the worksheet: `print` output as text, matplotlib figures as
  inline SVG, and numbers handed back with `export(name=value)` as Calcpad definitions (a number, a vector, a matrix).
  `export_data` defines values without printing them, for a chart to read. The runtime (a 0.4 MB wasm, MIT licensed,
  with small numpy and matplotlib.pyplot stand-ins, about 240 KB gzipped in all) loads only when a worksheet has a block.
  The first run in each tab asks "This worksheet runs Python code"; a script has no network or file access, runs at most 10 s
  and has capped output. Preview, Report, Print PDF, Word and Excel carry the results. Python cannot read Calcpad variables
  yet. Design, sandbox limits and a compatibility table: docs/python-blocks.md.
- The editor treats a block as Python: no Calcpad errors on its lines, names it exports are defined for the lines after it,
  the prettifier and Format Document leave it alone. In a sheet the ribbon inserts a `python` fence.
- Four Python samples: "Beam Explorer: Python Solve, Plotly Charts" (a three-span beam solved by the stiffness method in
  numpy, with an interactive Plotly chart driven by the exported results; also as a sheet), a beam moment diagram, a function
  plot, and section properties from numpy arrays. Math > Programming has `#python` entries.

### Changed
- The Samples browser shows the source of a Python sample instead of running it.

## [26.10.3] - 2026-10-23

### Added
- Sheet syntax, now the default way to start a worksheet: Markdown prose, `name: expression -> unit | note` lines,
  LaTeX in prose (`$...$`, `$$...$$`, rendered with KaTeX, bundled and loaded on first use), vectors and matrices,
  conditions, plots, indented `for` (with `step`), `while`, `repeat` and `if` / `else if` / `else` blocks with
  `break` and `continue`, and `>>` lines and `calcpad` fences that pass raw Calcpad through. It compiles to the
  Calcpad language and runs on the same engine, so units, vectors, plots and the Word and Excel exports work.
  A sheet starts with `<!--sheet-->` or is a `.sheet` file. The compiler is a new crate (`calcpad-sheet`, not
  published) in its own WebAssembly module (84 KB, 39 KB gzipped, loaded on first use). Grammar: docs/sheet-syntax.md.
- Editor support for sheets: colours, name completions, Format Document, go to definition, find references, hover,
  semantic highlighting of names; engine errors, report links and lint markers report the sheet line.
- Show compiled Calcpad and Convert to Calcpad (Resources tab) for a sheet.
- In a sheet the ribbon, the Text Formatting tab (Markdown bold, italic, headings, lists, links, code) and the
  sidebar Insert tab insert the sheet form of each item; items with no sheet form insert raw Calcpad as `>>` lines
  and are marked.
- File > New and the quick-access New button open a new sheet (Ctrl+N too in the desktop app); New Calcpad worksheet opens the Calcpad
  syntax. The first tab of a fresh start is a sheet that mentions both.
- Eight sheet samples at the top of the Samples dialog, including sheet versions of the bundled beam deflection
  and cantilever examples.
- Changelog button on the Resources tab and a What's new link in About (GitHub, new tab).
- Checks: `web/e2e/sheet-check.mjs` and `web/e2e/verify-ribbon-sheet.mjs` (every ribbon insert, in sheet form,
  through the compiler and the engine).

### Changed
- The compiled Calcpad of a sheet reads as hand-written: headings are heading lines, a paragraph of plain text is
  one text line, Markdown blocks are used only for prose that needs them (lists, tables, emphasis, links, code),
  notes are plain comments in the compiled view (a style class in the page), and blank lines between sections
  are kept.

## [26.10.2] - 2026-10-23

### Added
- Import worksheet: File menu, a Document-tab ribbon button and drag and drop bring in a worksheet of the
  spreadsheet-like format of another calculation tool (`.xmcd`). It is read and converted in the page by a third
  WebAssembly module (loaded on first use, 1.1 MB, 0.46 MB gzipped; the engine module is unchanged), opens as a
  new unsaved worksheet, keeps its pictures in the library under `/imported/<name>/`, and reports what converted
  live, what is shown as a static picture and what needs attention, with toggles for the pictures and for saved
  results. Prime `.mcdx` files are refused with a message.
- While files are dragged over the page, a full-window hint says whether a drop opens a worksheet (`.cpd`) or imports
  one (`.xmcd`).
- Library: several files can be written with a single store write and one sync (an import brings in hundreds of
  pictures).
- `calcpad-import-xmcd` (not published): the converter behind Import worksheet, with a batch runner that reports
  what converted over a folder of worksheets; mapping tables and limits are in docs/worksheet-import.md.
- THIRD_PARTY_LICENSES.md lists the 16 additional Rust crates the importer pulls in (94 in the build graph).

### Fixed
- Relative `#include` paths resolve for a worksheet that has a source path (the Frames samples): the active sheet
  is given a path in the editor's requests.

## [26.10.1] - 2026-10-22

First public release, named CivilKit Calcs. It is a derivative of CalcpadCE (MIT): its web frontend and
worksheet language, with the engine, highlighter and Word and Excel export reimplemented in Rust.

### Added
- Rust calculation engine compiled to WebAssembly, in place of the .NET engine and server; no calculation
  request leaves the page.
- A second WebAssembly module for editor services (highlighting, linting, prettifier, definitions, the 1166-snippet
  library), loaded on first use.
- Word export (equations as Office Math) and Excel export (a Report sheet with text, equations, tables, pictures and
  native charts, plus Variables, Inputs and Plot data sheets), both built in the page.
- `#UI` input forms with select, radio and checkbox values carried into the report; dark theme for the preview.
- Samples browser with 101 worksheets from CalcpadCE's examples, their input fields rewritten as `#UI`.
- Ribbon interface with ten tabs, palette popovers with a Common section above the full snippet library, and icons in
  light and dark.
- Model bindings (`civilkit: bind` comments): specification, parser and in-app display; connecting a model is not
  implemented yet.
- Worksheet scripts limited to a library list under a Content Security Policy.
- The version (YY.M.N) shown in the page (About, status bar, `<meta name="version">`) and returned by the engine's
  `version()`; `tools/set_version.mjs` keeps every manifest in step.

### Changed
- The product is named CivilKit Calcs; CalcpadCE is credited in the About text, README, LICENSE and NOTICE.

### Known limits
- Raster (SkiaSharp) plots, `#ProjectPath`/`#LibraryPath` path roots and compiled `.cpdz` worksheets are not supported.
- Exports are untested in Microsoft Office: Excel files were checked in openpyxl and LibreOffice, Word files in
  LibreOffice.
- Deviations from CalcpadCE are listed in DEVIATIONS.md.

## Crates

Published on crates.io. The API follows the needs of the web app and may change between 0.x minor versions.

### civilkit-calcs-engine 0.1.0, civilkit-calcs-highlighter 0.1.0, civilkit-calcs-docx 0.1.0 - 2026-10-22

First release of the three libraries, from the same commit as the 26.10.1 release.
- `civilkit-calcs-engine`: the calculation engine (a Rust port of the CalcpadCE worksheet engine).
- `civilkit-calcs-highlighter`: tokenizer, linter, content resolution, prettifier and snippet library. Depends on the engine.
- `civilkit-calcs-docx`: Word and Excel export. Depends on the engine.
- The Rust crate names are unchanged (`calcpad_core`, `calcpad_highlighter`, `calcpad_docx`).
