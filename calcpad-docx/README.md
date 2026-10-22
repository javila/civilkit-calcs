# civilkit-calcs-docx

Word (.docx) and Excel (.xlsx) export of CalcpadCE worksheet reports, a Rust port of Calcpad.OpenXml's Word path
plus an Excel writer. Equations are written as Office Math; the Excel workbook has a Report sheet, variable sheets and
native editable charts for plots. Depends on `civilkit-calcs-engine`. Checked in LibreOffice (Word and Excel) and
openpyxl (Excel); not yet opened in Microsoft Office.

Part of [CivilKit Calcs](https://github.com/mageaustralia/civilkit-calcs), a browser-based engineering calculation
sheet built on [CalcpadCE](https://github.com/imartincei/CalcpadCE) (MIT) by Nedelcho Ganchovski (Proektsoft) and
contributors. This crate is a Rust reimplementation that follows CalcpadCE's module structure; it is not affiliated
with the CalcpadCE authors.

Status: early (0.x). The API follows the needs of the web app and may change between minor versions.
The differences from CalcpadCE's behaviour are listed in
[DEVIATIONS.md](https://github.com/mageaustralia/civilkit-calcs/blob/main/DEVIATIONS.md).

Licence: MIT. The upstream copyright lines are kept in the LICENSE file next to this README.
