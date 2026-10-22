# civilkit-calcs-engine

The calculation engine: a Rust port of the CalcpadCE worksheet engine (Calcpad.Core). It parses a worksheet
(equations with units, vectors and matrices, plots, `#UI` input forms, macros, includes and text), calculates it and
returns the HTML report. It compiles to WebAssembly as well as native code.

Part of [CivilKit Calcs](https://github.com/mageaustralia/civilkit-calcs), a browser-based engineering calculation
sheet built on [CalcpadCE](https://github.com/imartincei/CalcpadCE) (MIT) by Nedelcho Ganchovski (Proektsoft) and
contributors. This crate is a Rust reimplementation that follows CalcpadCE's module structure; it is not affiliated
with the CalcpadCE authors.

Status: early (0.x). The API follows the needs of the web app and may change between minor versions.
The differences from CalcpadCE's behaviour are listed in
[DEVIATIONS.md](https://github.com/mageaustralia/civilkit-calcs/blob/main/DEVIATIONS.md).

Licence: MIT. The upstream copyright lines are kept in the LICENSE file next to this README.
