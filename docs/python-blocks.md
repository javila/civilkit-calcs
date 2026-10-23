# Python blocks (spike)

A worksheet can hold `#python` blocks. The page runs them on MicroPython (a 0.4 MB wasm) in a worker and mixes their results into
the worksheet: printed text, matplotlib figures as inline SVG, and numbers handed back as Calcpad variables. There is no 27 MB
full-Python download: MicroPython and small numpy and matplotlib stand-ins load only when a worksheet contains `#python`.

This is a spike on the `python-block` branch. It works end to end; the sections below say what it does not do.

## Syntax

```
"Beam
#python
import numpy as np
import matplotlib.pyplot as plt

x = np.linspace(0, 6, 61)
M = 12.0 * x * (6 - x) / 2
print("Maximum moment", float(M.max()))
plt.plot(x, M)
plt.show()

export(M_max=float(M.max()))
#end python
M_Ed = M_max*1kN*m
```

- `#python` on a line by itself (any case, blanks around are fine) opens a block. `#end python` on a line by itself closes it.
  No nesting. A block left open is an error line.
- All blocks of a worksheet run in order in one interpreter, so a later block sees the earlier blocks' variables and imports.
- Blocks run **before** the engine and unconditionally: a `#python` inside an `#if` still runs, and a block in an `#include`d file
  is not seen. Python is run only on the worksheet text itself.

### What a block produces

| In the block | In the worksheet |
|---|---|
| `print(...)` | a monospace paragraph, HTML-escaped |
| `plt.show()` or `plt.savefig(...)` | an inline `<svg>` figure (SVG, never a raster image), drawn at the point where the figure was shown |
| `export(name=value, ...)` or `export("name", value)` | a Calcpad definition line per name: `name = 5.5`, `name = [1; 2; 3]` (a list or 1-D array), `name = [1; 2\| 3; 4]` (a list of lists or 2-D array) |
| an exception | one red error line naming the Python line and the exception type, and an entry in the error list on that line; the rest of the worksheet still calculates |

Variables are exported **explicitly**, not automatically: loop variables and arrays used for plotting would otherwise flood the
worksheet and collide with Calcpad names and units (`m`, `s`, `L`). `export` is a builtin; no import is needed. Values must be
numbers, bools, lists of numbers or numeric arrays; `nan` and `inf` are refused. Names must be letters (any script), digits and
underscores, not starting with a digit or underscore. Exported numbers are unitless: write `L = L_m*1m` after the block.
Several names in one `export` call arrive in an unspecified order (MicroPython does not keep keyword order); use separate calls
where the order matters.

### One way only

Python cannot read Calcpad variables yet. Doing it cheaply is possible: run the engine on the text before the block, read the
numeric variables from its answer, and put them into the block's globals. It costs one extra engine pass per block and needs
the engine to return variable values for a partial worksheet, so it is not built.

## The editor

`calcpad-highlighter` treats a block as not Calcpad (`src/python_blocks.rs`): its lines are not tokenized, linted or prettified;
only the `#python` and `#end python` lines are marked as keywords. A name the block exports is defined for the lines after the
block, at the line of its `export` call (the linter sees `name = 0` there), so later lines are not flagged as undefined. Names
built dynamically (`export(**d)`) cannot be seen and will be flagged. An unclosed block is an error (`CPD-3801`). The prettifier
leaves the block byte for byte.

## How it runs

```
editor -> /api/calcpad/convert (also docx, xlsx)
          transport.ts: find blocks -> ask once per tab -> python-runner.ts -> python-worker.ts (MicroPython)
          replace each block by its results -> engine -> map line numbers back -> page
```

- A block's replacement keeps the block's place in the worksheet, but not its line count. The engine's answer is renumbered
  afterwards (`data-source-line`, `data-ui-line`, error links, the error list) so click-to-source, `#UI` write-back and error
  markers use the worksheet's own line numbers (`remapPage`, `remapErrors` in `python-blocks.ts`).
- Preview, Report, Print PDF, Word and Excel all go through the same transport, so all of them see the substituted result, figures
  included. The Code ("Unwrapped") view lists the substituted text.
- Results are cached by the text of the blocks (last 8), so typing in Calcpad lines does not rerun Python. Each calculation that
  does run gets a fresh interpreter.
- Limits: 10 s per block (the worker is killed and restarted), 30 s per calculation, 100 000 characters of print output, 20 figures,
  3 MB of SVG, 500 exported names, 32 MB interpreter heap.

### Lazy loading

Nothing of Python is fetched for a worksheet without `#python` (`e2e/python-check.mjs` checks the network log). With one, after the
confirmation below: the worker chunk, `py/micropython/micropython.{mjs,wasm}` and the seven files under `py/lib`.

| File | Raw | Gzip |
|---|---|---|
| `micropython.wasm` | 389 485 | 181 710 |
| `micropython.mjs` | 104 975 | 30 027 |
| `lib/numpy/_array.py` + `__init__.py` | 56 513 | 15 537 |
| `lib/matplotlib/*`, `calcpad_py.py`, `_route.py` | 18 993 | 5 960 |
| `python-worker` chunk (includes the SVG chart code) | 16 030 | 6 263 |
| `python` chunk (runner, consent dialog, block text code) | 4 204 | 1 810 |
| **Total on first use** | **about 590 000 (0.56 MB)** | **about 240 000** |

(The seven Python files together gzip to 20 036 bytes. The sizes are of the build on this branch.)

## Consent and sandbox

Calculating a worksheet executes its blocks, so the first run in each open tab asks "This worksheet runs Python code" (Run Python
or Not now). Run is remembered for that tab. Not now is remembered until the worksheet's Python code changes; blocks then show a
"Python was not run" line and nothing loads. The Samples browser never runs a sample's Python: a Python sample's preview shows
its source, and opening it asks as above.

What stops a script:

- It runs in a Web Worker: no DOM, no cookies, no `localStorage`.
- MicroPython's own `js` and `jsffi` modules (which would hand it the JavaScript global object) are refused by an import guard
  installed before the script runs; the guard and the host object live in closures a script cannot reach. `_thread` and the
  module that carries the host object are refused too.
- After loading, the worker's `fetch` serves only the interpreter's wasm from memory, and `XMLHttpRequest`, `WebSocket`,
  `EventSource`, `indexedDB`, `caches`, `BroadcastChannel`, `Worker` and `SharedWorker` are removed, and `importScripts` refuses.
- Messages from the worker carry a per-run token; the files MicroPython can open are an in-memory file system holding only the stand-in modules.
- A runaway loop or allocation is stopped by the timeout or the heap limit.

What it does not do, honestly:

- It is not a security boundary against a determined attacker. The guard is a Python-level check on `import`; a way around it that
  we have not found would give a script the worker's global object. The worker is same-origin with the app, and the app page sets no
  Content-Security-Policy, so a worker CSP is not there to back this up.
- The existing CSP of the rendered worksheet is unchanged: inline SVG needs nothing from it. Worker and wasm are served from the app's
  own origin, no CDN. A `Content-Security-Policy: default-src 'none'` response header on `py/*` and the worker chunk would add a
  platform-level network block; that is a deployment setting and is not done.
- A script can use the CPU for its time budget and the memory for its heap.

## Compatibility with typical calculation scripts

Probed by running small scripts on the shipped stand-ins (24 probes; the numbers are what is in `public/py/lib`).

| Area | Status | Notes |
|---|---|---|
| numpy: `array`, `shape`, `T`, `sum`, `mean`, `max`, `argmax`, `cumsum`, `abs`, `clip`, `maximum`, `where`, `sqrt`, `exp`, `sin`, `cos` | works | numpy's answers; slices are views |
| `linspace`, `arange`, `zeros`, `ones`, `full`, element assignment, boolean masks | works | |
| array arithmetic and `**`, `@`, `np.dot`, scalar on either side | works | float on the left needed a fix (below) |
| `np.diff`, `np.trapz` / `trapezoid`, `np.eye` | works | added in the spike (about 25 lines) |
| `np.linalg` (`solve`, `inv`, `eig`) | missing | the biggest gap for structural scripts |
| `np.vstack`, `hstack`, `reshape`, `meshgrid`, `sort`, `unique`, `interp`, `polyfit`, `random`, `cross`, `outer` | missing | raise `NotImplementedError` naming the function |
| math, f-strings, comprehensions, lambdas, dicts, classes, `time`, `random` | works | MicroPython 1.24 |
| `float(0-d array)` | missing | removed so that `2.5 * array` works |
| matplotlib: `plot` (format strings, labels), `xlabel`, `ylabel`, `title`, `legend`, `grid`, `xlim`, `ylim`, `figure(figsize)`, `show`, `savefig` | works | |
| `scatter`, `text`, `axis("equal")`, log axes, `subplots`, `ax.plot`, `set_title`, `tight_layout`, `suptitle` | works | |
| `bar`, `fill_between`, `axhline`, `axvline` | works | added in the spike |
| `hist`, `pie`, `errorbar`, `step`, `annotate`, `imshow`, `contour`, `colorbar`, `set_xticks`, `semilogy`, `twinx`, 3-D, polar | missing | |
| `input()` | refused | there is no keyboard |
| `import js`, `jsffi`, network, host files | refused | see the sandbox |

Missing features by importance for calculation worksheets: (1) `numpy.linalg`; (2) `vstack`/`hstack`/`reshape`/`meshgrid`/`sort`;
(3) `hist`, `annotate`, `twinx`, `set_xticks`, `step`; (4) `interp`/`polyfit`; (5) `contour`/`imshow`/`colorbar`; (6) `errorbar`, `pie`.

## What was changed in the copied stand-ins

The numpy and matplotlib stand-ins are the author's own small subsets that mimic those packages' APIs (the real, MIT-licensed
packages are not copied). MicroPython's licence is in `THIRD_PARTY_LICENSES.md`. Changes made for this repository:

- error messages and docstrings reworded to stand alone;
- the figure recorder (`_chart.py`) now hands a plain specification to the page, whose own SVG code (`src/wasm/python-chart.ts`)
  draws it; contours and colour bars were removed from the recorder rather than left half-working; `bar`, `fill_between`, `axhline`,
  `axvline` and `suptitle` were added;
- `ndarray.__float__` and `__int__` were removed: MicroPython's float operators call them on the right operand and give up when
  they raise, so `2.5 * array` failed;
- `diff`, `trapezoid`/`trapz` and `eye` were added.

## Not done

- Python reading Calcpad variables.
- Automatic export of variables.
- A friendly-sheet ` ```python ` fence (that front end is not on this base).
- A Python-aware highlighter for the block body (it is shown as plain text).
- Auto-run is not persisted anywhere: opening a worksheet runs nothing until the editor calculates it, and the question comes first.
