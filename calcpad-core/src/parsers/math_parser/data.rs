//! Port of the `#read`/`#write` members of Calcpad.Core/Parsers/MathParser/MathParser.cs:
//! `GetMatrix`, `SetMatrix`, `SetVector` and `ParseValue`.
use super::MathParser;
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::Unit;
use crate::base_types::value::Value;
use crate::base_types::variable::Variable;
use crate::exceptions;
use crate::extension_methods::is_letter;
use crate::math_parser_exception::CResult;
use crate::matrix::{
    column_matrix, diagonal_matrix, hp_matrix, lower_triangular_matrix, symmetric_matrix, upper_triangular_matrix,
    Matrix, MatrixType,
};
use crate::netfmt;
use crate::parsers::units_parser;
use crate::validator;
use crate::vector::{hp_vector, large_vector};

/// `RealValue.ToString()`
fn real_to_string(v: &RealValue) -> String {
    if v.is_unit {
        return v.units.as_ref().map(|u| u.text()).unwrap_or_default();
    }
    let s = v.units.as_ref().map(|u| u.text()).unwrap_or_default();
    format!("{}{}", netfmt::double_to_string(v.d), s)
}

/// `double.TryParse(s, provider, out d)`: NumberStyles.Float | AllowThousands.
pub(crate) fn try_parse_double(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let (sign, body) = match t.strip_prefix('-') {
        Some(b) => (-1.0, b),
        None => (1.0, t.strip_prefix('+').unwrap_or(t)),
    };
    if body.eq_ignore_ascii_case("infinity") {
        return Some(sign * f64::INFINITY);
    }
    if body.eq_ignore_ascii_case("nan") {
        return Some(f64::NAN);
    }
    let mantissa_end = body.find(['e', 'E']).unwrap_or(body.len());
    let (mantissa, exponent) = body.split_at(mantissa_end);
    let int_end = mantissa.find('.').unwrap_or(mantissa.len());
    // Group separators are allowed in the integer part only.
    if mantissa[int_end..].contains(',') || exponent.contains(',') || mantissa.starts_with(',') {
        return None;
    }
    let cleaned: String = format!("{}{}", mantissa.replace(',', ""), exponent);
    if !cleaned
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '-' | '+'))
    {
        return None;
    }
    let digits = mantissa.chars().filter(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    cleaned.parse::<f64>().ok().map(|d| sign * d)
}

impl MathParser {
    fn assign_value(&self, name: &str, value: Value) {
        let existing = self.variables.borrow().get(name).cloned();
        match existing {
            Some(v) => v.assign(value),
            None => {
                self.variables
                    .borrow_mut()
                    .insert(name.to_string(), Variable::with_value(value));
                self.defined_variables.borrow_mut().insert(name.to_string());
            }
        }
    }

    /// `GetMatrix`: the cells of a vector or matrix variable as text, for `#write`.
    pub(crate) fn get_matrix(&self, name: &str, ty: char) -> CResult<Vec<Vec<String>>> {
        let value = {
            let vars = self.variables.borrow();
            let v = vars.get(name).ok_or_else(|| exceptions::VariableNotExist(name))?;
            v.value()
        };
        if ty != 'N' && ty != 'Y' {
            return Err(exceptions::InvalidType(ty));
        }
        let matrix = match value {
            Some(Value::Vector(vec)) => {
                if ty == 'Y' {
                    Matrix::from_row_vector(&vec)?
                } else {
                    column_matrix::from_vector(&vec)?
                }
            }
            Some(Value::Matrix(m)) => m,
            _ => return Err(exceptions::VariableNotMatrix(name)),
        };
        let rows = matrix.row_count();
        let cols = matrix.col_count();
        let zero = RealValue::ZERO;
        if ty == 'Y' {
            match matrix.mtype() {
                MatrixType::Column => {
                    return Ok(vec![(0..rows).map(|i| real_to_string(&matrix.get(i, 0))).collect()]);
                }
                MatrixType::Diagonal => {
                    return Ok(vec![(0..rows).map(|i| real_to_string(&matrix.get(i, i))).collect()]);
                }
                MatrixType::LowerTriangular => {
                    let mut result = Vec::with_capacity(rows);
                    for i in 0..rows {
                        let size = (0..=i).rev().find(|&j| !matrix.get(i, j).equals(&zero)).unwrap_or(0);
                        result.push((0..=size).map(|j| real_to_string(&matrix.get(i, j))).collect());
                    }
                    return Ok(result);
                }
                MatrixType::UpperTriangular | MatrixType::Symmetric => {
                    let mut result = Vec::with_capacity(rows);
                    for i in 0..rows {
                        let size = (i..cols).rev().find(|&j| !matrix.get(i, j).equals(&zero)).unwrap_or(i);
                        result.push((i..=size).map(|j| real_to_string(&matrix.get(i, j))).collect());
                    }
                    return Ok(result);
                }
                MatrixType::Full => {}
            }
        }
        let mut result = Vec::with_capacity(rows);
        for i in 0..rows {
            let size = (0..cols).rev().find(|&j| !matrix.get(i, j).equals(&zero)).unwrap_or(0);
            result.push((0..=size).map(|j| real_to_string(&matrix.get(i, j))).collect());
        }
        Ok(result)
    }

    /// `SetMatrix`: assigns the cells read by `#read` as a matrix of the given type.
    pub(crate) fn set_matrix(&self, name: &str, data: &[Vec<String>], ty: char, is_hp: bool) -> CResult<()> {
        if data.is_empty() || data[0].is_empty() {
            return Ok(());
        }
        let rows = data.len();
        let cols = data.iter().map(|v| v.len()).max().unwrap_or(0);
        let n = rows.max(cols);
        if (ty == 'C' || ty == 'D') && cols != 1 && rows != 1 {
            return Err(exceptions::IndexOutOfRange(&format!("{}, {}", rows, cols)));
        }
        let matrix = if is_hp {
            match ty {
                'R' => hp_matrix::new(rows, cols, None)?,
                'C' => hp_matrix::new_column(n, None)?,
                'D' => hp_matrix::new_diagonal(n, None)?,
                'L' => hp_matrix::new_lower(rows, None)?,
                'U' => hp_matrix::new_upper(rows, None)?,
                'S' => hp_matrix::new_symmetric(rows, None)?,
                _ => return Err(exceptions::InvalidType(ty)),
            }
        } else {
            match ty {
                'R' => Matrix::new(rows, cols)?,
                'C' => column_matrix::new_checked(n)?,
                'D' => diagonal_matrix::new_checked(n)?,
                'L' => lower_triangular_matrix::new_checked(rows)?,
                'U' => upper_triangular_matrix::new_checked(rows)?,
                'S' => symmetric_matrix::new_checked(rows)?,
                _ => return Err(exceptions::InvalidType(ty)),
            }
        };
        match ty {
            'C' => {
                if rows == 1 {
                    for j in 0..cols {
                        matrix.set(j, 0, self.parse_value(&data[0][j])?)?;
                    }
                } else {
                    for (i, row) in data.iter().enumerate() {
                        matrix.set(i, 0, self.parse_value(row.first().map_or("", String::as_str))?)?;
                    }
                }
            }
            'D' => {
                if rows == 1 {
                    for j in 0..cols {
                        matrix.set(j, j, self.parse_value(&data[0][j])?)?;
                    }
                } else {
                    for (i, row) in data.iter().enumerate() {
                        matrix.set(i, i, self.parse_value(row.first().map_or("", String::as_str))?)?;
                    }
                }
            }
            'L' => {
                for (i, row) in data.iter().enumerate() {
                    for j in 0..row.len().min(i + 1) {
                        matrix.set(i, j, self.parse_value(&row[j])?)?;
                    }
                }
            }
            'U' | 'S' => {
                for (i, row) in data.iter().enumerate() {
                    for j in 0..row.len().min(rows - i) {
                        matrix.set(i, i + j, self.parse_value(&row[j])?)?;
                    }
                }
            }
            _ => {
                for (i, row) in data.iter().enumerate() {
                    for (j, cell) in row.iter().enumerate() {
                        matrix.set(i, j, self.parse_value(cell)?)?;
                    }
                }
            }
        }
        self.assign_value(name, Value::Matrix(matrix));
        Ok(())
    }

    /// `SetVector`: assigns the cells read by `#read ... type=V`, row by row.
    pub(crate) fn set_vector(&self, name: &str, data: &[Vec<String>], is_hp: bool) -> CResult<()> {
        if data.is_empty() || data[0].is_empty() {
            return Ok(());
        }
        let mut values = Vec::with_capacity(data.iter().map(|r| r.len()).sum());
        for row in data {
            for cell in row {
                values.push(self.parse_value(cell)?);
            }
        }
        let vector = if is_hp {
            hp_vector::from_values(&values)?
        } else {
            large_vector::from_values(values)?
        };
        self.assign_value(name, Value::Vector(vector));
        Ok(())
    }

    /// `ParseValue`: a number with an optional unit, as written in a data file.
    pub(crate) fn parse_value(&self, s: &str) -> CResult<RealValue> {
        if s.is_empty() {
            return Ok(RealValue::ZERO);
        }
        let chars: Vec<char> = s.chars().collect();
        let mut is_unit_char = true;
        let mut n = chars.iter().position(|&c| c == '^').unwrap_or(chars.len());
        while n > 0 {
            let c = chars[n - 1];
            if c.is_ascii_digit() || c == '.' {
                if is_unit_char {
                    break;
                }
                is_unit_char = false;
            } else {
                is_unit_char = is_letter(c) || validator::is_unit_char(c);
            }
            n -= 1;
        }
        let number: String = chars[..n].iter().collect();
        let mut d = 1.0;
        if n > 0 {
            d = try_parse_double(&number).ok_or_else(|| exceptions::InvalidNumber(&number))?;
        }
        let mut u = None;
        if n < chars.len() {
            let unit: String = chars[n..].iter().collect();
            let is_us = self.is_us.get();
            u = match Unit::try_get(&unit, is_us) {
                Some(u) => Some(u),
                None => units_parser::parse(&unit, None, is_us)?,
            };
        }
        Ok(if n == 0 {
            RealValue::unit(u)
        } else {
            RealValue::new(d, u)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::try_parse_double;

    #[test]
    fn parses_like_dotnet_float_with_thousands() {
        assert_eq!(try_parse_double("7.5e-06"), Some(7.5e-6));
        assert_eq!(try_parse_double("1,000.5"), Some(1000.5));
        assert_eq!(try_parse_double(" -2 "), Some(-2.0));
        assert_eq!(try_parse_double("1.5,2"), None);
        assert_eq!(try_parse_double("abc"), None);
        assert_eq!(try_parse_double("."), None);
    }
}
