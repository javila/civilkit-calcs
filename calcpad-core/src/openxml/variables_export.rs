//! The Excel export of a calculated worksheet (calcpad-rs's own feature; CalcpadCE has none):
//! the variables an `ExpressionParser` with `capture_values` recorded, written with the
//! workbook writer that `#write` uses.
//!
//! * "Variables": every variable in the order it was first assigned, with its last value: name,
//!   value, unit. A vector or a matrix names the sheet that holds it.
//! * One sheet per vector or matrix: its elements, each row with its unit.
//! * "Inputs": the variables set by input fields (`?`) and `#UI` controls.
//!
//! Numbers are written as numbers (full precision) and units as text in their own column.
use std::collections::HashSet;

use super::excel_data;
use crate::base_types::unit::OptUnit;
use crate::base_types::value::Value;
use crate::matrix::Matrix;
use crate::netfmt::double_to_string;
use crate::parsers::expression_parser::CapturedValue;
use crate::vector::Vector;

fn unit_text(u: &OptUnit) -> String {
    u.as_ref().map(|u| u.text()).unwrap_or_default()
}

fn number(d: f64) -> String {
    double_to_string(d)
}

fn complex_text(a: f64, b: f64) -> String {
    if b < 0.0 {
        format!("{} - {}i", number(a), number(-b))
    } else {
        format!("{} + {}i", number(a), number(b))
    }
}

/// Excel's sheet name rules: at most 31 characters, none of `[]:*?/\`, unique ignoring case.
fn sheet_name(name: &str, used: &mut HashSet<String>) -> String {
    let base: String = name
        .chars()
        .map(|c| if "[]:*?/\\'".contains(c) { '_' } else { c })
        .take(31)
        .collect();
    let base = if base.trim().is_empty() {
        "Sheet".to_string()
    } else {
        base
    };
    let mut candidate = base.clone();
    let mut n = 2;
    while used.contains(&candidate.to_lowercase()) {
        let suffix = format!(" ({})", n);
        let keep: String = base.chars().take(31 - suffix.chars().count()).collect();
        candidate = format!("{}{}", keep, suffix);
        n += 1;
    }
    used.insert(candidate.to_lowercase());
    candidate
}

fn col_name(mut n: usize) -> String {
    let mut s = Vec::new();
    while n > 0 {
        let r = (n - 1) % 26;
        s.push((b'A' + r as u8) as char);
        n = (n - 1) / 26;
    }
    s.iter().rev().collect()
}

fn add_sheet(book: Option<Vec<u8>>, name: &str, rows: &[Vec<String>]) -> Result<Vec<u8>, String> {
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
    let end = format!("{}{}", col_name(cols), rows.len().max(1));
    let rows: Vec<Vec<String>> = if rows.is_empty() {
        vec![vec![String::new()]]
    } else {
        rows.to_vec()
    };
    excel_data::write(book.as_deref(), name, "A1", &end, &rows, book.is_some())
}

fn vector_rows(v: &Vector) -> Vec<Vec<String>> {
    let hp_units = if v.is_hp() { v.hp_units() } else { None };
    let mut rows = vec![vec!["Index".to_string(), "Value".to_string(), "Unit".to_string()]];
    for i in 0..v.len() {
        let e = v.get(i);
        let u = if v.is_hp() {
            unit_text(&hp_units)
        } else {
            unit_text(&e.units)
        };
        rows.push(vec![(i + 1).to_string(), number(e.d), u]);
    }
    rows
}

fn matrix_rows(m: &Matrix) -> Vec<Vec<String>> {
    let (nr, nc) = (m.row_count(), m.col_count());
    let hp_units = if m.is_hp() {
        crate::matrix::hp_matrix::units(m)
    } else {
        None
    };
    let mut header = vec![String::new()];
    header.extend((1..=nc).map(|j| j.to_string()));
    header.push("Unit".to_string());
    let mut rows = vec![header];
    for i in 0..nr {
        let mut row = vec![(i + 1).to_string()];
        let mut units: Vec<String> = Vec::new();
        for j in 0..nc {
            let e = m.get(i, j);
            row.push(number(e.d));
            let u = if m.is_hp() {
                unit_text(&hp_units)
            } else {
                unit_text(&e.units)
            };
            if !units.contains(&u) {
                units.push(u);
            }
        }
        // The row's unit; elements with different units list each once.
        row.push(
            units
                .into_iter()
                .filter(|u| !u.is_empty())
                .collect::<Vec<_>>()
                .join("; "),
        );
        rows.push(row);
    }
    rows
}

/// A scalar's value and unit cells; `None` for a vector or matrix.
fn scalar_cells(v: &Value) -> Option<(String, String)> {
    match v {
        Value::Real(r) => Some((number(r.d), unit_text(&r.units))),
        Value::Complex(c) => Some((complex_text(c.a, c.b), unit_text(&c.units))),
        _ => None,
    }
}

/// One sheet of the export: its name and rows of text (numbers as their text). A vector or
/// matrix sheet also names the variable it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    pub name: String,
    pub rows: Vec<Vec<String>>,
    pub variable: Option<String>,
}

/// The sheets for the variables a parse recorded: "Variables", one per vector or matrix, "Inputs".
pub fn sheets(values: &[CapturedValue]) -> Vec<Sheet> {
    let mut used: HashSet<String> = ["variables", "inputs", "report"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let header = || vec!["Name".to_string(), "Value".to_string(), "Unit".to_string()];
    let mut variables = vec![header()];
    let mut extra: Vec<Sheet> = Vec::new();
    for c in values {
        match scalar_cells(&c.value) {
            Some((value, unit)) => variables.push(vec![c.name.clone(), value, unit]),
            None => {
                let sheet = sheet_name(&c.name, &mut used);
                let (rows, what) = match &c.value {
                    Value::Vector(v) => (vector_rows(v), format!("vector of {} (sheet \"{}\")", v.len(), sheet)),
                    Value::Matrix(m) => (
                        matrix_rows(m),
                        format!(
                            "matrix {} \u{00D7} {} (sheet \"{}\")",
                            m.row_count(),
                            m.col_count(),
                            sheet
                        ),
                    ),
                    _ => continue,
                };
                variables.push(vec![c.name.clone(), what, String::new()]);
                extra.push(Sheet {
                    name: sheet,
                    rows,
                    variable: Some(c.name.clone()),
                });
            }
        }
    }
    let mut inputs = vec![header()];
    for c in values.iter().filter(|c| c.is_input) {
        match scalar_cells(&c.value) {
            Some((value, unit)) => inputs.push(vec![c.name.clone(), value, unit]),
            None => inputs.push(vec![c.name.clone(), "(see Variables)".to_string(), String::new()]),
        }
    }
    let mut out = vec![Sheet {
        name: "Variables".into(),
        rows: variables,
        variable: None,
    }];
    out.extend(extra);
    out.push(Sheet {
        name: "Inputs".into(),
        rows: inputs,
        variable: None,
    });
    out
}

/// The sheets as JSON, for a page that hands them to another module.
pub fn sheets_to_json(sheets: &[Sheet]) -> serde_json::Value {
    serde_json::Value::Array(
        sheets
            .iter()
            .map(|s| serde_json::json!({ "name": s.name, "rows": s.rows, "variable": s.variable }))
            .collect(),
    )
}

/// The inverse of [`sheets_to_json`]; anything that does not fit is skipped.
pub fn sheets_from_json(v: &serde_json::Value) -> Vec<Sheet> {
    let Some(a) = v.as_array() else { return Vec::new() };
    a.iter()
        .filter_map(|s| {
            let name = s.get("name")?.as_str()?.to_string();
            let rows = s
                .get("rows")?
                .as_array()?
                .iter()
                .map(|r| {
                    r.as_array()
                        .map(|c| c.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
                        .unwrap_or_default()
                })
                .collect();
            let variable = s.get("variable").and_then(|x| x.as_str()).map(str::to_string);
            Some(Sheet { name, rows, variable })
        })
        .collect()
}

/// The workbook for the variables a parse recorded.
pub fn workbook(values: &[CapturedValue]) -> Result<Vec<u8>, String> {
    let mut book: Option<Vec<u8>> = None;
    for s in sheets(values) {
        book = Some(add_sheet(book, &s.name, &s.rows)?);
    }
    book.ok_or_else(|| "Nothing to export.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_names_follow_excel_rules() {
        let mut used: HashSet<String> = ["variables"].iter().map(|s| s.to_string()).collect();
        assert_eq!(sheet_name("a/b[1]", &mut used), "a_b_1_");
        assert_eq!(sheet_name("Variables", &mut used), "Variables (2)");
        assert_eq!(sheet_name("A_B_1_", &mut used), "A_B_1_ (2)");
        assert_eq!(sheet_name(&"x".repeat(40), &mut used).chars().count(), 31);
        assert_eq!(
            (col_name(1), col_name(26), col_name(27), col_name(703)),
            ("A".into(), "Z".into(), "AA".into(), "AAA".into())
        );
    }
}
