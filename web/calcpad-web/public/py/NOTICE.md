# Python runtime for `#python` blocks

- `micropython/micropython.wasm` and `micropython/micropython.mjs`: MicroPython v1.24.1 (build dated 2024-11-30) compiled to
  WebAssembly with Emscripten, with its JavaScript loader. https://micropython.org, https://github.com/micropython/micropython.
  The version is the string embedded in the wasm. MicroPython is MIT licensed:

  > The MIT License (MIT)
  >
  > Copyright (c) 2013-2024 Damien P. George and contributors
  >
  > Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation
  > files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy,
  > modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
  > Software is furnished to do so, subject to the following conditions:
  >
  > The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
  >
  > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
  > WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
  > COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
  > ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

  The loader also carries Emscripten's runtime glue (MIT or University of Illinois/NCSA, as Emscripten is licensed).
  The upstream licence text should be checked against the MicroPython repository before this is published.

- `lib/numpy/` and `lib/matplotlib/`: small stand-ins for the parts of numpy and matplotlib.pyplot that calculation scripts
  use. They are not the real packages and say so when a script reaches for something they lack (see docs/python-blocks.md).
  `lib/calcpad_py.py` is the host side of the block runner.
