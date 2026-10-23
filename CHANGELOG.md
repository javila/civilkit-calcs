# Changelog

All notable changes are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions are numbered YY.M.N with no zero padding (26.10.1 is the first release of October 2026; N counts the releases
of the month and restarts at 1), which is valid semver for Cargo and npm. Tags are `v26.10.2` and so on.
CalcpadCE's own version is tracked separately in NOTICE.

The app changes are listed under their YY.M.N version below. The three Rust library crates have their own entries
under "Crates" at the end, on plain 0.x semver, and are tagged `<crate-name>-v<version>`
(for example `civilkit-calcs-engine-v0.1.0`).

## [26.10.6] - 2026-10-24

### Added
- The Plotly API in Python blocks, so an interactive chart needs no HTML and no JavaScript: `import plotly.graph_objects as go` (also
  `plotly.graph_objs`), `go.Figure`, the common trace classes (`Scatter`, `Bar`, `Heatmap`, `Scatter3d` and others), `add_trace`,
  `update_layout`, `update_xaxes`, `update_yaxes`, `update_traces`, `add_shape`, `add_vline`, `add_hline`, `fig.show()`, and
  `plotly.subplots.make_subplots` (rows, cols, shared axes, subplot titles). It is a thin pass-through: every keyword is written into the
  figure JSON as given and Plotly.js validates it; `line_color`-style names expand to nested dicts, and anything the shim does not provide
  (another trace type, `write_html`, `fig.data`, a named template, a `fig.show` option) fails by name. Menus and sliders are Plotly's own
  `updatemenus` and `sliders` with precomputed steps. The page draws the figure with Plotly.js from its CDN, loaded only when a figure is
  present through the library list entry that already holds the pinned URL and integrity hash (not bundled, policy unchanged), with a light and
  a dark theme; without a connection a note says so, and print, Word and Excel carry a paragraph naming the chart. The 24 KB shim is fetched
  only by a block that mentions plotly. docs/python-blocks.md.
- A "Beam Explorer: Plotly from Python" sample (also as a sheet): the same three-span beam, solved in Python and explored with a native load case
  menu, a load factor slider (0.50 to 2.00 in steps of 0.05) and a point load position slider, with an envelope band and support lines, in 190 kB of figure data.
  `web/e2e/python-check.mjs` checks its plotted hogging moments against a three-moment solution, and that the real menu and sliders change the plotted data.

### Changed
- The raw HTML and JavaScript beam explorer is kept as "Beam Explorer: Python Results in Raw JS Plotly" (the example of the script route; it
  can combine controls and compute a readout, which Plotly's own controls cannot), and the Plotly API sample comes first in the Python group.

## [26.10.5] - 2026-10-24

### Added
- A sheet accepts Calcpad's own `#python` ... `#end python` pair as a Python block, the same block as a `python` fence (a pasted block no longer turns into sheet lines).
- `numpy.linalg` and `scipy.linalg` in Python blocks, computed by the Calcpad engine (the code behind `lsolve`, `inverse`, `det`,
  `cholesky`, `eigenvals`, `svd` in Calcpad lines), so Python and Calcpad lines give one answer: `solve`, `inv`, `det`,
  `cholesky`, `eigh`, `eigvalsh`, `svd`, `matrix_rank`, `cond`, `pinv`, `norm`; `scipy.linalg.solve`, `cholesky`, `cho_factor`,
  `cho_solve`, `solve_triangular`. A singular matrix is a `LinAlgError` on its Python line; unsupported calls (`lstsq`, `qr`,
  `eig` of a non-symmetric matrix) say so in a `NotImplementedError`. A new `calcpad-linalg-wasm` module (345 KB, 138 KB
  gzipped) is downloaded only by a block that mentions linalg, scipy, dot, matmul or `@`. Large float matrix products
  (about 46 by 46 and up) go to the engine as well, with identical values and about 7 times faster.
  Solving a 120 by 120 system takes 87 ms instead of 1.5 s in a hand-written Python loop. docs/python-blocks.md.
- `tools/build_wasm.sh linalg` builds the new module.
- A "Plane Truss by the Stiffness Method" sample in the Python group: a statically determinate truss assembled in loops and solved with `np.linalg.solve`, member forces and reactions recovered and drawn with matplotlib (tension and compression coloured, deflected shape), and a Calcpad member check on the exported maxima. `web/e2e/python-check.mjs` checks its member forces, reactions and deflections against hand values (method of joints, virtual work), and now also covers a bare `#python` block in a sheet, exported names with an underscore or a capital in a sheet, the section properties calculation, and the Sections group in the Samples browser.

### Changed
- The "Section Properties from numpy Arrays" sample is now "Built-up Section Properties" in a new Sections group, written with vectors and `hprod` and no Python: the units carry through and the report shows every substitution. The Python group keeps the samples where Python is the better tool.
- docs/python-blocks.md records why ulab (a numpy-like MicroPython module) was measured and not shipped: it is 10 times faster
  than the engine for dense solves, but a second, different numpy (2-D only, no `@` or `.sum()`) and 430 KB more wasm.

### Fixed
- In a sheet, the names a Python block hands over with `export()` use the sheet's spelling, so `export(sigma_c=2)`, `phi` and `Mstar` are found by the lines below the block as `σ_c`, `φ` and `M_star`.

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
- Faster calculation of the heaviest worksheets, with identical output. Soil Stress and Deformations runs on 37 %
  of the instructions in the command line tool (72 s of CPU time became 33 s on a busy machine) and in 60 % of the
  CPU time in the browser; Rectangular Slab FEA on 67 % of the instructions in the command line tool. Units are one
  allocation instead of about six, the command line tool uses a small-block allocator, and the result cache of
  custom functions hashes with a multiply. The engine wasm is 2 KB smaller.

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
