# civilkit-calcs-highlighter

Editor services for CalcpadCE worksheets, a Rust port of Calcpad.Highlighter: tokenizer, linter, content
resolution (includes and macros), prettifier and the snippet library. Depends on `civilkit-calcs-engine`.

Part of [CivilKit Calcs](https://github.com/mageaustralia/civilkit-calcs), a browser-based engineering calculation
sheet built on [CalcpadCE](https://github.com/imartincei/CalcpadCE) (MIT) by Nedelcho Ganchovski (Proektsoft) and
contributors. This crate is a Rust reimplementation that follows CalcpadCE's module structure; it is not affiliated
with the CalcpadCE authors.

Status: early (0.x). The API follows the needs of the web app and may change between minor versions.
The differences from CalcpadCE's behaviour are listed in
[DEVIATIONS.md](https://github.com/mageaustralia/civilkit-calcs/blob/main/DEVIATIONS.md).

Licence: MIT. The upstream copyright lines are kept in the LICENSE file next to this README.
