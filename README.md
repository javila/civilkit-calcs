# calcpad-rs

A Rust port of the [CalcpadCE](https://github.com/imartincei/CalcpadCE) calculation engine
(Calcpad.Core), ported from commit `0cf438f`. Phase 1 covers the scalar engine: real and
complex values, the unit system with target units and custom units, operators, the scalar
function library, variables, custom functions with keyword and default arguments, solve
blocks (`$Root`, `$Find`, `$Sup`, `$Inf`, `$Area`, `$Integral`, `$Slope`, `$Derivative`,
`$Sum`, `$Product`, `$Repeat`, `$While`, `$Block`, `$Inline`), comments and headings, HTML
passthrough, `#if`/`#else if`/`#else`, `#repeat`/`#for`/`#while` loops with
`#break`/`#continue`, `#def` macros, `#include`, the visibility and output directives and
the settings directives.

Vectors, matrices, high performance types, plots, `#UI` forms, `#read`/`#write` and
markdown are out of scope for phase 1 and are reported as such.

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

* xunit: 879/879 scalar tests (Calcpad.Tests/Scalars) and 134/134 in-scope tests of the
  other scalar classes pass; 17 transcribed tests need out-of-scope features and are
  `#[ignore]`d with the reason.
* Stubs: 97 of the 98 in-scope worksheets match; 271 worksheets use out-of-scope
  features. See PORT_LOG.md for the line counts and DEVIATIONS.md for deliberate changes.

## Licence

MIT, see LICENSE and NOTICE. The original work is
Copyright (c) 2026 CalcpadCE Contributors and Copyright (c) 2014-2026 Nedelcho Ganchovski.
