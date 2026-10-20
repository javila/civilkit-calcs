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
* `tools/`: generators for messages and exception constructors, the xunit transcriber and
  the stub comparison harness.

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

## Status

* xunit: 4692 transcribed tests pass (Scalars, Vectors, Matrices incl. HP, MathParser,
  Macros, ExpressionParser and the others); 2 raster plot tests are ignored.
* Stubs (369 worksheets): 343 PASS, 16 PNG-PIXEL (`$Map` rasters with equal pixels but a
  different deflate stream), 1 NOISE, 1 FAIL (libm last digit), 8 OUT-OF-SCOPE (`#UI`,
  `#read`). See `tools/classify_failures.py`, PORT_LOG.md and DEVIATIONS.md.

## Licence

MIT, see LICENSE and NOTICE. The original work is
Copyright (c) 2026 CalcpadCE Contributors and Copyright (c) 2014-2026 Nedelcho Ganchovski.
