# Importing foreign worksheets (spike)

Crate: `calcpad-import-xmcd` (package `civilkit-calcs-import-xmcd`, not published). It reads the
`.xmcd` and `.mcdx` worksheet files of a commercial calculation tool, the spreadsheet-like worksheet format with
text, equations and plots placed freely on a page, and writes **Calcpad worksheet text** for the existing
engine. It evaluates nothing itself: one engine, one language. Every number in the output is a literal
that was in the source.

It is a spike. It answers two questions: how much of real files converts, and which missing features
come first. The two file formats share one translator; see [The .mcdx front door](#the-mcdx-front-door).

## In the app

`File > Import worksheet...`, the **Import worksheet** button on the Document tab, `File > Open...` with a `.xmcd` or
`.mcdx` file, or dropping one anywhere on the page (a dashed hint shows while a file is dragged over it).

1. The file is read in the page and converted by the importer module (`calcpad_import_xmcd_wasm`, in its own web
   worker, loaded on first use; the engine module is not involved). **Nothing is uploaded**: the file never leaves
   the browser.
2. The picture files go into the library under `/imported/<file name>/` (letters, digits, `_`, `-`; importing the
   same file name again replaces that folder).
3. The converted worksheet opens as a new unsaved tab named `<file name>.cpd`.
4. A dialog says how many regions converted live, how many are shown as static pictures and how many need
   attention, and lists the top reasons in plain words ("calls to a function from an add-in we cannot run",
   "embedded objects", ...). Two toggles re-run the import in the same tab: **Pictures of unconverted regions**
   (default on) and **Show saved results as static values** (default off).

Errors are messages, never crashes: a file that is not `.xmcd` ("... is not a worksheet this importer reads"),
malformed XML or a DTD ("not well-formed XML"), text that is not UTF-8, and a `.mcdx` zip that cannot be read
("the file is a zip package that cannot be read").

**Pictures and Save.** The `.cpd` text refers to pictures by library path (`<img src="imported/<name>/R12.svg">`).
The report, the preview and the HTML, Word and Excel exports resolve those paths against the library and inline
the pictures (a test checks the rendered page and the Word package). Save writes the `.cpd` and keeps the pictures in
the library, which is stored in the browser: the pictures stay available as long as the library does. A `.cpd`
opened later from disk shows its pictures only while the library still holds them (it does not embed them);
exporting to HTML or Word first gives a self-contained file.

**Limits.** The module is built for the browser, which has no threads and no unwinding. Metafile conversion is bound
by a record-chain check (a file with more than 200 000 records, or a record chain that does not add up, is refused
and the raster picture used) plus the size caps below. If the module traps anyway, the page starts a fresh worker and
retries once with vector pictures off, so raster pictures are used instead. Input over 64 MiB, XML nested deeper than
256, a DTD, pictures over 16 MiB each or 192 MiB in all are refused or skipped as described under Security.

Size of the module: 1,152,164 bytes raw, 473,567 bytes gzipped (`tools/build_wasm.sh import`); the `.mcdx` reader added about 21 KB.

## Use

```text
cargo run -p civilkit-calcs-import-xmcd --example import -- <file.xmcd>   # text on stdout, report on stderr
```

```rust
let r = calcpad_import_xmcd::import_xmcd(&xml);      // r.calcpad: String, r.report: ImportReport
println!("{} of {} regions need attention",
         r.report.regions_needing_attention(), r.report.regions_total());
```

The report has, per region kind (`text`, `math`, `plot`, `image`, `ole`, `component`, `reference`,
`other`), the totals, how many converted, how many converted with a note, and how many need attention.
Page breaks are layout and are not counted. `report.issues` lists every issue with a stable code, a
short reason, the `region-id` and the source line. `report.ranked_gaps()` ranks the missing features by
the number of regions they block.

### The placeholder rule

A region that does not convert is never dropped and never guessed. The output gets one line for it:

```text
'[import-gap R41 range-variable] range variable (a..b) or an expression iterating over one; ... | i = 1 .. 10
```

The line starts with `'` (a Calcpad text line, so it shows in the report and runs as text), then
`[import-gap`, the region id, the code(s), a fixed reason, and for math regions a bounded,
escaped rendering of what the importer understood. Input text reaches the output only through three
doors: escaped prose in a `'` line, names that pass the Calcpad identifier check, and numbers that
parse as finite numbers (plus the mapping tables). A test plants a payload in every kind of
unconverted region and checks that it only ever appears inside a placeholder line.

## What converts

| Source construct | Calcpad output |
| --- | --- |
| Text region | One `'` text line per paragraph. Styles `Heading 1..3` become `<h1>..<h3>`; bold, italic, underline, sub and superscript are kept; colour and font are dropped. Text is HTML-escaped and quote marks doubled, so prose cannot start a directive. Empty paragraphs are skipped. |
| Math nested inside a text region | Becomes its own region, ordered by its own position. |
| `x := expr` | `x = expr` |
| `x ≡ expr` (global) | `x = expr` at its reading-order position, with note `global-definition` |
| `expr =` | the bare expression |
| `x := expr =` | `x = expr` (a display unit on the result is dropped) |
| `expr = ` with a display unit | `(expr)\|unit`, note `unit-override` |
| `f(a, b) := expr` | `f(a; b) = expr` |
| `+ - * /`, powers, unary minus, factorial, comparison, and/or/xor | the same operators, with parentheses added from the tree, never from the source |
| `sqrt`, `\|x\|` | `sqrt(x)`, `abs(x)` (note `abs-of-array` when the operand is a known matrix or vector) |
| `x_i`, `ρ_tp` (literal subscripts), Greek names | `x_i`, `ρ_tp` |
| Vector, matrix literal | `[a; b; c]`, `[a; b\|c; d]`, a row vector as `transp([a\|b\|c])`. The file stores matrices column by column; the importer reads them that way. |
| `v[i]`, `M[i, j]` | `v.(i + 1)`, `M.(i + 1; j + 1)` for an array origin of 0; unchanged for 1; a literal index is folded. Note `array-origin`. |
| `ORIGIN := n` (integer literal) | Later indexes use `n`. Note `origin-assigned`. |
| Units | See below. |
| Trigonometric functions | The output starts with `#rad` (the source default is radians); note `angle-mode`. |
| Numbers with an exponent (`1.5e-3`) | Plain digits (`0.0015`), by moving the decimal point; `(m*10^(e))` beyond 20 places. Calcpad does not read `e` notation. |
| A variable applied like a function (`k(2)` where `k` is a variable) | `k*2`, note `call-as-multiplication` |

### Reading order

The source evaluates regions top to bottom, then left to right. Regions are sorted by the row they
sit on, then by their left edge; position is used only for this and then dropped.

* The row key is the region's **baseline** (`align-y`) when the file gives one, otherwise its top.
  A tall region next to a short one shares the short one's baseline while its top edge is higher, so
  sorting by top alone puts a use before the definition to its left. Real files do this.
* Regions whose keys are within 4 points of the first region of a row are one row.

### Array origin

The `array-origin` setting is read from the file. The default in the source is 0 and Calcpad counts
from 1, so every index is shifted (`v[i]` becomes `v.(i + 1)`). A literal index is folded.

### Functions

The mapping table lives in one place, `src/tables.rs` (`FUNCTIONS`). Calls with the wrong number of
arguments are `arity-mismatch` gaps. Caveats are notes: `log(x, b)` becomes `(ln(x)/ln(b))`,
`round(x, n)` becomes `(round((x)*10^(n))/10^(n))`.

| Source | Calcpad |
| --- | --- |
| sin cos tan sec csc cot asin acos atan atan2 acot asec acsc | same name (angle mode: see above) |
| sinh cosh tanh sech csch coth asinh acosh atanh | same name |
| ln, exp, sqrt, abs, sign, signum, floor, trunc, mod, min, max, gcd, lcm, if, identity, matrix, det, rank, augment, stack, sort, reverse, mean | same name (`signum` becomes `sign`) |
| log(x) | `log(x)` (base 10) |
| ceil | `ceiling` |
| Re, Im, arg | `re`, `im`, `phase` |
| rnd | `random` |
| rows, cols, length | `n_rows`, `n_cols`, `len` |
| norm2 | `norm_2` |

A call to a function that is not defined in the sheet is one of three codes, ranked separately:

* `builtin-missing`: a built-in of the source tool that the engine lacks (`stdev`, `lookup`, `erf`,
  `cumsum`, ...). The table is `BUILTIN_MISSING` in `src/tables.rs`; together with `FUNCTIONS` it is the
  list of built-in names the importer knows.
* `reference-function`: any other undefined name, in a sheet that has worksheet-reference regions, which may
  define it.
* `addin-function`: any other undefined name, in a sheet with no reference regions. The file has no
  structural marker for an add-in (the files only mention add-ins in prose), so this is a heuristic:
  not built in, not defined, nothing to define it.

Solve-block names (`Given`, `Find`, ...) are `solve-block`.

### Units

A name is a unit when it is not defined as a variable or function earlier in reading order and it is in
the table. Units are written with `*` (`1200*psi`). A name that is neither defined nor in a table is
`undefined-variable`, so an undefined variable whose name spells a unit (`m`, `h`, `T`) is read as the
unit: this cannot be told apart without types.

Same spelling in both: m km dm cm mm μm nm pm in ft yd mi nmi th ly AU, g kg mg μg lb oz slug gr t kt,
s ms μs ns min h d w y Hz kHz MHz GHz rpm, A mA μA kA V mV kV Ω kΩ F μF H mH C S T Wb, K °C °F °R mol cd
lm lx, N kN MN GN daN kNm Nm lbf kgf kip dyn, Pa kPa MPa GPa hPa μPa bar mbar atm psi ksi psf ksf mmHg
inHg Torr, J kJ MJ GJ mJ cal kcal BTU erg eV Wh kWh MWh W kW MW GW mW hp, L mL μL ha, rad deg ° grad rev %,
mph knot. The micro sign (U+00B5) is read as Greek mu (U+03BC).

Renamed: sec to s, hr and hour to h, day to d, yr to y, gm to g, tonne to t, lbm and pound to lb, liter
and litre to L, ml to mL, gal to gal_US, qt to qt_US, pt to pt_US, mile to mi, torr to Torr, dyne to dyn,
ohm to Ω, tonf to tonf_US, ton to ton_US, mil to th, acre to ac, nmile to nmi, degC/degF/degR to
°C/°F/°R, foot to ft, inch to in, International_BTU to BTU, pounds_per_sq_inch to psi, and the spelled-out
SI names (meter, gram, kilogram, second, minute, newton, pascal, joule, watt, volt, ampere, kelvin,
hertz, coulomb, farad, henry, tesla, weber, siemens, radian, percent).

Known to the source but missing in Calcpad (`unmapped-unit`): kmol, mmol, lbmol, gmol, kgmol, angstrom,
gal_imp, dB, Bq, Gy, Sv.

A test checks that every Calcpad unit and function name in the tables exists in the engine.

The built-in gravitational acceleration `g` becomes `(9.80665*m/s^2)`, because Calcpad's `g` is the gram.

## Pictures and fallbacks

Options (`ImportOptions`, `import_xmcd_opts`): `pictures` (default on), `saved_results` (default off),
`asset_prefix` (default `imported`), reader `limits`, `svg_limits`, `max_total_asset_bytes`.

`ImportResult.assets` is a list of `Asset { path, mime, bytes }`. The Calcpad text refers to each by exactly
its `path` in `<img src="...">`, relative to the library root with `/` separators (for example
`imported/R1795.svg`). The caller stores each asset in the page's file library under that path: the web
app resolves a relative `<img src>` against the library (root `/`) and inlines it, the same way the bundled
samples show their images. Give every imported worksheet its own `asset_prefix`; the prefix is cleaned to
letters, digits, `_ - . /` and cannot climb out of the library. Region ids are made unique within a prefix.
The batch runner writes the assets under the output folder as `<worksheet name>/R<id>.svg|png`.

* **Picture regions.** A picture region carries a metafile and/or a PNG. The metafile (base64, gzip when the item
  says so, WMF or EMF) is converted to SVG with the `emf-core` crate and written as `R<id>.svg`; the
  text gets `'<img style="width:Npt;max-width:100%;" src="..." alt="R<id>">`. If the conversion fails, is
  refused by a guard, or the SVG is too large, the PNG (the picture's own `png` item, else the region's saved
  rendering) is used instead, with note `svg-fallback`. If neither exists the region keeps its placeholder
  (`image`). A shown picture counts as converted.
* **Fallback pictures.** For every region that does not convert (math, embedded object, plot, component,
  reference) and has a saved rendering (a PNG), the output shows that picture, followed on the same text line
  by a caption: `static picture of the original, not live (R1795 ole-object)`. The grey `[import-gap` line
  is not written for such a region; the report keeps the gap codes and adds the note `static-picture`. The
  region still counts as needing attention (it is not live); `report.regions_showing()` counts converted plus
  pictured regions and `KindStats.pictured` the pictured ones. With `pictures` off there are no assets and
  every unconverted region is a placeholder line, as before.
* **Saved results.** An evaluation stores its last calculated result in the file. A single number (with a
  unit) is read; strings, matrices and anything else are ignored. For an unconverted math region that has
  one, `report.pictures.saved_result_available` counts it. With `saved_results` on, and no picture shown for
  the region, the output adds `'<i>saved result from the original, not live (R12): 571.9 International_BTU/(pound)</i>`
  after the placeholder line. Units are written as the source spells them; they are not converted. Default off.

Guards (every byte here is untrusted): base64 text over 32 MiB is refused; a gzip item may not inflate past
16 MiB (`max_item_bytes`) and all items together are capped at 192 MiB; a metafile over 4 MiB is not converted; the
converter runs on its own thread with a 32 MiB stack and a 10 s limit (a panic is caught, a run past the limit is
abandoned and its thread left to finish, so a hostile file costs a stuck thread, not the caller); an SVG over
3 MiB, or one containing script, foreign content, entity declarations or event attributes, is refused and the PNG is
used; a worksheet may produce at most 128 MiB of assets. Tests cut a hand-made metafile short at every
length, corrupt it at every byte and feed 300 random buffers.

## What does not convert

Reported with a code, counted, and replaced by a placeholder. Codes are stable; the UI and the batch
summary key on them.

| Code | Meaning |
| --- | --- |
| `range-variable` | `i := 1..10`, and anything that iterates over one. **Not built in this spike**: the engine has no range variable. Names it defines block later regions (`depends-on-gap`, counted as downstream of this code). |
| `program-block` | Multi-line programs (local definitions, loops, conditions) |
| `solve-block` | Given/Find and relatives |
| `symbolic-evaluation` | Symbolic evaluation |
| `calculus-operator` | Derivative and integral operators (the engine has `$Derivative` and `$Integral`; a mapping needs a verified sample of the operand layout) |
| `summation-operator`, `vectorize-operator`, `matrix-operator` | Sum/product/limit, vectorize, column/row/vector-sum operators |
| `string-value` | Strings; Calcpad has no string variables |
| `bare-expression` | An expression with no definition or evaluation operator (documentation, or a solve-block constraint) |
| `element-assignment`, `disabled-region` | `v[i] := x`; a region switched off in the source |
| `plot`, `ole-object`, `component`, `worksheet-reference`, `other-region` | Counted only, never converted; shown as a static picture when the file has one (see Pictures) |
| `header-footer`, `rich-text-element`, `text-unreadable` | `.mcdx` only: a page header or footer; an element of a text region that would hide content (a table, a picture, a floating block; the element name is the detail); a text region whose prose could not be read from the package |
| `image` | A picture region with nothing usable (no metafile, no raster picture) |
| `unknown-element` | An element name the reader does not know (the name is the detail) |
| `unmapped-unit`, `builtin-missing`, `addin-function`, `reference-function`, `undefined-variable` | See above |
| `depends-on-gap` | Uses a name that an earlier unconverted region defines; attributed to the root cause in `ranked_gaps()` |
| `invalid-name`, `bad-number`, `arity-mismatch`, `bad-structure`, `too-deep` | Structure the importer will not guess |

Notes (the region converted, with a caveat): `global-definition`, `angle-mode`, `array-origin`,
`origin-assigned`, `log-base`, `round-digits`, `abs-of-array`, `call-as-multiplication`, `unit-override`.
Whole-file problems: `xml-error`, `not-a-worksheet`, `input-too-large`, `unsupported-container` (a zip that is damaged,
encrypted or uses a zip feature the reader refuses).

### Element names: seen in real files, or assumed

The reader was written against 31 real `.xmcd` files and then tested on synthetic snippets. These
element names were **seen in real files**: `id` (with `subscript`), `real`, `str`, `apply` (with
`fixity="postfix"`), `plus`, `minus`, `neg`, `mult`, `div`, `pow`, `equal`, `lessThan`, `absval`,
`parens`, `sequence`, `define`, `globalDefine`, `localDefine`, `function`, `boundVars`, `lambda`,
`matrix` (with `rows`, `cols`), `indexer`, `range`, `eval` (with `unitOverride` and a cached `result`),
`provenance`, `derivative`, `integral`, `degree`, `program`, `ifThen`, `for`, `otherwise`, `matcol`,
`vectorSum`. These are **assumed** from the schema family and only exercised by synthetic tests:
`sqrt`, `factorial`, `transpose`, `notEqual`, `greaterThan`, `lessOrEqual`, `greaterOrEqual`, `and`,
`or`, `xor`, `not`, `symEval`. A wrong guess fails safe (an `unknown-element` gap), never as a wrong
number. `nthRoot` is deliberately **not** read: its operand order is unverified, so it is reported.

The cached `result` of an evaluation is ignored. Comparing the engine's value with it would be a
strong check and is the obvious next step for validating numbers.

## Batch runner

```text
cargo build --release -p calcpad-cli                       # the engine command line
cargo run --release -p civilkit-calcs-import-xmcd --example batch -- \
    <dir with .xmcd files> <output dir outside the repo> target/release/Cli
```

For each file it imports, writes `<name>.cpd` to the output directory, runs the engine on it
(`Cli x.cpd x.html -b -s`), and counts the engine's `Error in "..."` messages in the HTML. It prints
and saves `summary.txt`: a per-file and total table (regions, text and math counts, converted share
of all regions and of math regions, regions needing attention, engine errors), the regions by kind and
by math shape, the ranked missing features with their most frequent details, the notes, and the engine
errors grouped by message. Top-level directories whose name starts with `build`, `doc` or `includes`
are skipped. It only reads the inputs. Input files are never copied into this repository.

### What the spike found (31 worksheets of a thermodynamic property handbook)

* 1315 content regions: 562 text (all converted), 578 math, 132 embedded objects, 28 pictures, 11 plots,
  2 components, 2 worksheet references. 48 page breaks are not counted.
* 79 of 578 math regions convert (13.7 %). Live-convertible regions overall: 669 of 1315 (50.9 %), 641
  (48.7 %) when the 28 pictures are not counted. With pictures on, every region shows something: the 669 live
  plus 646 static pictures of the original. Engine errors after import: 0 in all 31 files, because anything
  that would fail is a placeholder or a picture.
* Pictures: 13 of 13 metafiles became an SVG (1.4 MB in total), 15 pictures came from a PNG item, and 646
  unconverted regions got their saved rendering (1.3 MB of PNG). Saved scalar results exist on 26 unconverted
  math regions.
* The corpus is dominated by calls into an add-in (the sheets are wrappers around a native library), so
  the top gaps are `addin-function` (408 regions including 82 downstream), embedded objects (132),
  strings that feed those calls (107 including downstream), `bare-expression` (94, documentation
  that shows call syntax), `reference-function` (18) and, in this corpus, no `builtin-missing` at all. Range variables, programs and calculus together are under 30 regions here.
  A corpus of ordinary structural or mechanical sheets will rank differently; this one does not tell
  how common range variables are in the wild.

## Security

The XML is untrusted. A DTD is refused (no entity definitions, so no entity expansion and no external
entities); the input size (64 MiB), node count (4 million) and element nesting (256, checked by a
pre-scan before the parser runs) are capped; expression and region nesting is followed only 64 levels
deep (deeper becomes a `too-deep` gap); an n-ary operator with more than 100 operands is a `too-deep`
gap so that its left-leaning tree stays shallow. Tests cover malformed input, a billion-laughs DTD, an
external entity, 200 000 nested elements, 200 nested expressions, a 5000-term sum and odd attribute values.
The crate forbids `unsafe`. Dependencies: `roxmltree` (MIT OR Apache-2.0), `miniz_oxide` (MIT OR Zlib OR
Apache-2.0) and `emf-core` 0.1.1 (MIT, default features off, so no `tracing`), which brings `wmf-core` (MIT),
`snafu` and `snafu-derive` (MIT OR Apache-2.0), `strum` and `strum_macros` (MIT), `heck` and `embedded-io`
(MIT OR Apache-2.0), `base64` (MIT OR Apache-2.0), `codepage` (Apache-2.0 OR MIT), `encoding_rs`
((Apache-2.0 OR MIT) AND BSD-3-Clause), `multiversion_no_op`, `scopeguard`, `simdutf8`, `unicode-segmentation`
and `unicode-width` (all MIT or Apache-2.0 alternatives). Nothing non-permissive; `THIRD_PARTY_LICENSES.md`
is regenerated. The importer never executes anything from the file.

A `.mcdx` is a zip, which adds its own guards (`src/zip.rs`): the entry count (20 000) and the declared size of
each entry are capped before anything is allocated; inflate stops at the declared size, so an entry that lies
about it fails instead of growing; the total of decoded bytes is capped (192 MiB, the same budget as pictures);
encryption, zip64 and compression methods other than stored and deflate are refused; entries are looked up by
name inside the archive and never written to disk, and a relationship target is folded (`..` cannot climb out).
The CRC is not checked. A flow document is a second, nested zip and gets the same treatment. The XML parts obey
the same DTD, depth and node limits as an `.xmcd`.

## The .mcdx front door

`.mcdx` is the newer container of the same worksheet language. `import_xmcd_bytes` looks at the first bytes: a zip
signature goes to `mcdx::read` (also callable as `import_mcdx`), anything else is read as the plain XML. Both
produce the same `ir::Sheet`, so the translator, the report and the placeholder rule are shared. The mcdx
reader is `src/mcdx.rs`; the zip walker `src/zip.rs`; the math reader of `xmcd.rs` is reused (namespaces are matched
by family, so `math30` and `math50` are both `math`).

What the reader does with the package:

* The worksheet part is found through the package relationships (any part called `worksheet.xml` as a fallback).
  Its siblings are read relative to it: `result.xml` (saved results, joined to a math region by `resultRef`),
  `settings/calculation.xml` (the array origin; 0 when absent) and the header and footer parts.
* **Text.** A text region names a XAML flow document through a relationship. That part is a nested zip holding
  `Xaml/Document.xaml`. Older files carry the flow document inline, as a child of the text element; both forms are read. Paragraphs, runs, bold, italic, underline, sub and superscript, line breaks, hyperlinks
  (the text stays, the address does not), bulleted lists and decimal lists (numbered "1. ", "2. ") convert.
  Colours, fonts and sizes are dropped, as for `.xmcd`. Math nested in a text region becomes its own region at the
  text's position, as before.
* **Math.** The tree is the same family as `.xmcd`. What differs: a value followed by its unit is a `scale`
  application (read as a product); an identifier's subscript is a styled span inside it (`T` + subscript `max`
  reads as `T_max`); a call with one argument has no `sequence` wrapper; an empty slot is a `placeholder`
  (an empty unit override means "no unit"; a placeholder anywhere else is an `unknown-element` gap). Matrices are
  stored column by column, as in `.xmcd` (read off a 3 x 2 matrix in the sample files, a column of names then a column of numbers).
* **Programs** (`if`, `for`, `while`, `return`, try/catch, local definitions) are `program-block` gaps, as in
  `.xmcd`; `summation` is a `summation-operator` gap; derivatives and integrals are `calculus-operator` gaps.
* **Collapsible areas** only group regions: their regions are read as if on the page, positioned relative to
  the area.
* **Pictures** with a PNG convert like `.xmcd` pictures (the PNG is stored in the library). The package carries no
  cached rendering of a region, so an unconverted region is a placeholder line, never a static picture.

Skipped, with a code and a visible placeholder line each (nothing is dropped silently):

| Part | Code |
| --- | --- |
| Embedded object of another application (`oleObject`) | `ole-object` |
| Spreadsheet and chart components, with the regions inside them | `component` |
| Reference to another worksheet | `worksheet-reference` (and `reference-function` for calls it may define) |
| Plot | `plot` |
| Page header and footer (when they hold regions) | `header-footer` |
| Table, picture or floating block inside a text region | `rich-text-element` (the element name is the detail) |
| A text region whose flow document is missing or unreadable | `text-unreadable` |
| Any other region kind | `other-region` |
| Calls into an external library | `addin-function` (as in `.xmcd`) |

Two wrong-number traps are closed for both formats: after an element assignment (`v[i] := x`) or a definition
whose target is a matrix of names, the names involved are marked as defined by an unconverted region, so a later
use is a `depends-on-gap` instead of the old value (or, for a name such as `L` that is also a unit, the unit).

Not read: the unit system setting, strings-use-origin and the other calculation settings; the regions inside a
component (they belong to the skipped component); a definition whose target is a matrix of names
(`bad-structure`).

`cargo run --release -p civilkit-calcs-import-xmcd --example engine_check -- file.mcdx out` imports a file, runs
the result through the engine and writes the text, the engine's HTML and a list of saved results, so converted
lines can be compared with the numbers the original stored.

Checked against seven worksheets of a thermodynamic property handbook, which are mostly text and calls into an
external library, so they exercise the reader and the skip reports far more than the maths. Their counts are kept
out of the repository with the files. The synthetic packages in `tests/mcdx.rs` cover assignment, arithmetic with
units, a function, a matrix, a program, a range variable, text, areas, skipped parts, saved results and zip abuse;
each math case is built once and imported through both front doors, and the two must agree.

## Not covered

Range variables (top engine gap to close, see the ranked list), programs, solve blocks, symbolic
evaluation, plots, pictures, embedded objects, data tables and components, strings, derivative and integral
operators, worksheet references and includes, text colours and fonts, tables of results. Types
are not tracked, so `|x|` of a vector or matrix is only noted when the operand was defined from a matrix
literal.
