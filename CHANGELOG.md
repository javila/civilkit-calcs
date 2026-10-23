# Changelog

All notable changes are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions are numbered YY.M.N with no zero padding (26.10.1 is the first release of October 2026; N counts the releases
of the month and restarts at 1), which is valid semver for Cargo and npm. Tags are `v26.10.2` and so on.
CalcpadCE's own version is tracked separately in NOTICE.

The app changes are listed under their YY.M.N version below. The three Rust library crates have their own entries
under "Crates" at the end, on plain 0.x semver, and are tagged `<crate-name>-v<version>`
(for example `civilkit-calcs-engine-v0.1.0`).

## [Unreleased]

### Changed
- Faster calculation of the heaviest worksheets, with identical output. Soil Stress and Deformations runs on 37 %
  of the instructions in the command line tool (72 s of CPU time became 33 s on a busy machine) and in 60 % of the
  CPU time in the browser; Rectangular Slab FEA on 67 % of the instructions in the command line tool. Units are one
  allocation instead of about six, the command line tool uses a small-block allocator, and the result cache of
  custom functions hashes with a multiply. The engine wasm is 2 KB smaller.

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
