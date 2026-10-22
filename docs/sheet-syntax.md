# Sheet syntax

A sheet is a friendlier way to write a worksheet: Markdown prose, plain `name: expression`
lines and LaTeX, compiled to the Calcpad worksheet language and run by the same engine. There
is no second engine. Real unit algebra, vectors and matrices, plots and the Word and Excel
exports all work, because the engine sees an ordinary Calcpad worksheet. The compiler is the
crate `civilkit-calcs-sheet` (`calcpad-sheet/`), a deterministic function from sheet text to
Calcpad text plus a source map. This is our addition and not part of CalcpadCE.

A file is a sheet when its first non-blank line is

```
<!--sheet-->
```

or when its name ends in `.sheet` (the editor adds the marker line when it opens one, so the
text and the editor mode always agree). **New sheet (Markdown style)** in the File menu or the
Resources tab starts one. Use **Show compiled** to see the Calcpad a sheet becomes (its line
numbers are the sheet's lines) and **Convert to Calcpad** to replace the sheet with that text.

## Example

```
<!--sheet-->
# Beam bending check

A simply supported beam. The moment **comes out in kN·m**.

L: 6 m | span
w: 12 kN/m | design load
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
Ms: 300 MPa * 1.23e6 mm³ -> kN*m
utilisation: Mstar / (0.9 * Ms)
if utilisation <= 1: "OK" else: "Fails"
```

## Prose

Anything that is not one of the line kinds below is Markdown prose: `#` to `######` headings,
paragraphs, `**bold**`, `*italic*`, `-` and numbered lists, pipe tables, links,
`![alt](file)` images (library files, as in Calcpad), `` `code` `` and fenced code blocks. The
compiler wraps each run of prose in a `#markdown ... #end markdown` block, so the engine's own
Markdown renderer does the work.

A line that would read as a quantity but is prose (`Note: see the drawing`) is prose already,
because its right side is not an expression. To force prose, start the line with a backslash.

### LaTeX

`$...$` is an inline formula and `$$...$$` a display formula, in prose, in notes and in the
text of an `if` line. A `$$` alone on a line opens and closes a multi-line formula. `\$` is a
dollar sign, and a `$` before a digit ("$5") is one too. The preview renders formulas with
KaTeX, which is bundled with the app and loaded the first time a sheet has a formula; the
rendered page needs no script and the content security policy is unchanged. For Word and Excel
a formula is written as its TeX source in italics.

## Quantity lines

```
name: expression [-> unit] [| note]
name = expression
a: 1; b: 2 | note for b
f(x): x^2 + 1
```

* The line shows what a Calcpad equation line shows: the name, the expression, the values
  substituted and the result with its units. The note follows in a muted style.
* **Names** are letters, digits, `_` and primes. Greek spellings become Greek letters:
  `phi` is φ, `sigma_c` is σ with subscript c, `Delta_T` is Δ with subscript T, and a Greek
  spelling followed by a capital or a digit (`phiMs`, `sigma2`) is the letter and the rest.
  Capitals are used only where they differ from a Latin capital (`Gamma`, `Delta`, `Theta`,
  `Lambda`, `Xi`, `Pi`, `Sigma`, `Upsilon`, `Phi`, `Psi`, `Omega`). A name ending in `star`
  after a capital or digit (`Mstar`) is that name with subscript `star`. The same rule renames
  every use, so a name is spelled one way throughout the sheet.
* **Units are real.** Write a unit after a number: `300 MPa`, `9.81 m/s²`, `5 kN*m`. The engine
  checks and combines them, so `fy * Ze` in MPa and mm³ is a moment. A unit written after a
  number is never renamed (`3 psi` is a pressure and `psi` alone is ψ).
* **Conversion:** `-> unit` converts the result: `Ms: fy * Ze -> kN*m` (Calcpad's `|kN*m`). It
  must come last, before any note.
* **Notes:** the first `|` outside brackets starts the note. `||` is the logical or. Use
  `abs(x)` for an absolute value.
* **Several per line:** separate with `;`. A note belongs to the last one.
* **Numbers:** `1.23e6` is written as `1230000`, since the engine has no exponent notation.
* **Operators:** `+ - * / ^ %`, `< > <= >= == != and or`, and the Unicode forms `≤ ≥ ≠ ≡ ∧ ∨ × · ÷ −`.
  In a condition a single `=` also means equal. `²` and `³` after a name or unit are `^2` and `^3`.
* **Calls** take commas: `max(a, b)`; the compiler writes the engine's `;`.
* **Vectors and matrices:** `[1; 2; 3]` and `[1, 2, 3]` are vectors. `[1, 2; 3, 4]` is a
  matrix: columns are separated by commas and rows by semicolons. (Calcpad writes
  `[1; 2 | 3; 4]`.) Rows must have equal length. Elements are expressions. An element is
  reached with `v.2` and `A.(1, 2)`.
* A quantity cannot have a bare unit after a bracket (`[1, 2] kN`); put the unit on the elements.

## Conditions

```
if utilisation <= 1: "OK" else: "Fails"
if a == 2: b: 1
```

One line: the branches are a quoted text or quantities. For anything longer use a block.

## Blocks, loops and conditions with a body

A line ending in `:` that starts with `if`, `else if`, `else`, `for`, `while` or `repeat`
opens a block. The body is the lines indented more than the header (a tab counts as four
spaces); blank lines inside are fine; a line indented no deeper than the header closes the
block. A body can hold anything: quantities, prose, LaTeX, `>>` lines and nested blocks. A
line that mixes tabs and spaces in its indentation is an error on that line.

```
for i in 1..5:
    P: 10 kN * i | load at stage $i$
    if i >= 3:
        Heavy stage.
    else:
        Light stage.

for x in [1, 2, 3]:
    y: x^2
while n > 0:
    n: n - 1
```

| Sheet | Calcpad |
| --- | --- |
| `if c:` / `else if c:` / `else:` | `#if c` / `#else if c` / `#else`, closed by `#end if` |
| `for i in a..b:` | `#for i = a : b`, closed by `#loop` |
| `for x in v:` | an index loop over the vector `v` (a name or a literal), hidden helper lines set `x` |
| `while c:` | `#while c` ... `#loop` |
| `repeat n:` | `#repeat n` ... `#loop` |

The range has no step. Quantity lines in a loop give exactly the report a Calcpad loop gives
(a test compares them). A header whose condition is not an expression (`If the load is
large:`) is prose, with a warning if it starts with a keyword.

## Plots

```
plot M(x) for x in 0..L
plot sin(x), cos(x) for x in 0..2*π
plot y = f(x) for x in 0..10
```

becomes `$Plot{ M(x) @ x = 0 : L }`; several expressions are drawn together. An optional
`name =` before the expressions is dropped. A malformed plot line is reported in the report and
on its sheet line.

## Calcpad pass-through

Every engine feature stays reachable. A line starting with `>>` is passed to the engine as
written, and so is everything between a `calcpad` fence:

````
>> #UI 'Span -'L = 6'm
```calcpad
#for i = 1 : n
    s = s + i*phi
#loop
```
````

Sheet lines and raw lines share one worksheet, so they alternate freely: a raw block reads
the variables defined by sheet lines above it and defines variables that sheet lines below it
use, and prose, quantities, LaTeX and raw Calcpad can come in any order. To make that work
without remembering renames, an identifier in a raw line that the sheet defines, or that a raw
line assigns (`sigma = 3`), is spelled the way the sheet spells it: `phi` in a raw line is
`φ`. (Text after a `'` or `"` in a raw line is left alone. `Show compiled` shows the result.)

A raw block inside a sheet block keeps the block's indentation rules like any other line.

## Errors

The engine's errors, the report's "on line [n]" links, the error banner, the scroll anchors and
the editor's lint markers all report the sheet line, whatever kind of line it was (quantity,
raw, inside a block). The text of an engine message still quotes the compiled Calcpad line,
which is what Show compiled displays. The compiler's own problems (mixed indentation, an
`else` with no `if`, an empty block, an unclosed `$$`, a bad plot line) are markers on their
lines too. "Unused variable" hints are dropped for sheets, since every quantity is shown.

## Not supported, and the limits

* No `for` step (`1..10..2`); no `break`, `continue` or `repeat` without a count (use `>>`).
* No units on vector or matrix literals; no nested brackets; no symbolic algebra (the engine
  is numeric, as Calcpad is).
* The prettifier, symbol lookup and semantic highlighting of Calcpad do not apply to a
  sheet; the sheet has its own colours and completions (names defined in the sheet, Greek
  names, block snippets).
* A formula in Word or Excel is its TeX source, not Office Math.
* A name that is a Calcpad function (`gamma`, `beta`) as a variable keeps working only when
  written the Calcpad way; function calls are not renamed unless the sheet defines the function.
* `Convert to Calcpad` is one way: the Calcpad text is not turned back into a sheet.

## Ribbon and the Insert tab

In a sheet the ribbon and the sidebar's Insert tab insert the sheet form: Heading is
`## Heading`, Text is a prose line, Table is a pipe table, Plot is a `plot` line, the
conditions and loops are the indented blocks, a definition such as `c = 299792458m/s` is
`c: 299792458 m/s`. Functions, operators, Greek letters, units and constants are the same in
both syntaxes (with the argument and matrix punctuation above). Anything with no sheet form
(`#UI` forms, `$Map`, directives, macros) inserts its Calcpad text as `>>` lines, and its
button carries a small `>>` mark and a tooltip saying so. Each open file has its own mode and
the ribbon follows the active tab.

`web/e2e/verify-ribbon-sheet.mjs` runs the sheet form of every ribbon button and palette entry
through the compiler and the engine.
