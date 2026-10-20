//! Port of Calcpad.Core/Parsers/MathParser/MathParser.UI.cs: what a `#UI` datagrid reads
//! back from the parser.
use super::MathParser;
use crate::base_types::unit::OptUnit;
use crate::base_types::value::Value;
use crate::netfmt;

/// Grid cells and their units; a padded cell has no unit (None, not an empty one).
pub type UiGrid = (Vec<Vec<String>>, Vec<Vec<Option<String>>>);

impl MathParser {
    /// `GetVariableShape`
    pub(crate) fn get_variable_shape(&self, name: &str) -> (i32, i32) {
        let vars = self.variables.borrow();
        let Some(v) = vars.get(name) else {
            return (0, 0);
        };
        if !v.is_initialized() {
            return (0, 0);
        }
        match v.value() {
            Some(Value::Vector(vector)) => (1, vector.len() as i32),
            Some(Value::Matrix(m)) => (m.row_count() as i32, m.col_count() as i32),
            _ => (0, 0),
        }
    }

    /// `GetUiGridValues`: the cells of a vector or matrix variable padded to rows x cols.
    pub(crate) fn get_ui_grid_values(&self, name: &str, rows: i32, cols: i32) -> Option<UiGrid> {
        let value = {
            let vars = self.variables.borrow();
            let v = vars.get(name)?;
            if !v.is_initialized() {
                return None;
            }
            v.value()?
        };
        let (row_count, col_count) = match &value {
            Value::Vector(vector) => (1, vector.len()),
            Value::Matrix(m) => (m.row_count(), m.col_count()),
            _ => return None,
        };
        let decimals = self.settings().decimals();
        let mut values = Vec::with_capacity(rows.max(0) as usize);
        let mut units = Vec::with_capacity(rows.max(0) as usize);
        for i in 0..rows.max(0) as usize {
            let mut vr = Vec::with_capacity(cols.max(0) as usize);
            let mut ur = Vec::with_capacity(cols.max(0) as usize);
            for j in 0..cols.max(0) as usize {
                if i < row_count && j < col_count {
                    let cell = match &value {
                        Value::Vector(vector) => vector.get(j),
                        Value::Matrix(m) => m.get(i, j),
                        _ => unreachable!(),
                    };
                    vr.push(format_ui_cell(netfmt::round_even_digits(cell.d, decimals)));
                    ur.push(Some(unit_input_text(&cell.units)));
                } else {
                    vr.push("0".to_string());
                    ur.push(None);
                }
            }
            values.push(vr);
            units.push(ur);
        }
        Some((values, units))
    }

    /// The result's unit as the parser reads it back in, empty when it has none.
    pub(crate) fn result_units_input_text(&self) -> String {
        unit_input_text(&self.units())
    }
}

/// Plain decimal: the exponent form is read back as a unit, not as a number.
fn format_ui_cell(d: f64) -> String {
    netfmt::format_custom(d, "0.###############")
}

/// '·' and '∕' are written for display and are not input operators.
fn unit_input_text(units: &OptUnit) -> String {
    match units {
        None => String::new(),
        Some(u) => u.text().replace('\u{00B7}', "*").replace('\u{2215}', "/"),
    }
}
