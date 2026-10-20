//! Port of Matrix/HpMatrix: the high performance matrices (doubles with one unit).
//!
//! The storage is shared with `Matrix` (`MatData.hp == true`, rows are HpVectors, the
//! unit is `MatData.units`). This module holds the constructors and the HpMatrix
//! overrides of the virtual `Matrix` members.
use super::{Matrix, MatrixType, MAX_SIZE};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::OptUnit;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::vector::{hp_vector, Relation, Vector};

pub(crate) fn hp_not_ported() -> MathParserException {
    MathParserException::new("HP matrices are not supported by calcpad-rs yet.")
}

/// `new HpMatrix(int rows, int cols, Unit units)`
pub fn new(rows: usize, cols: usize, units: OptUnit) -> CResult<Matrix> {
    if rows > MAX_SIZE || cols > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let mut r = Vec::with_capacity(rows);
    for _ in 0..rows {
        r.push(hp_vector::new_length(cols, units.clone())?);
    }
    Ok(Matrix::from_parts(MatrixType::Full, true, rows, cols, r, units))
}

/// `new HpColumnMatrix(int length, Unit units)`
pub fn new_column(length: usize, units: OptUnit) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatrixDimensions());
    }
    let r = vec![hp_vector::new_length(length, units.clone())?];
    Ok(Matrix::from_parts(MatrixType::Column, true, length, 1, r, units))
}

/// `new HpDiagonalMatrix(int length, Unit units)`
pub fn new_diagonal(length: usize, units: OptUnit) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let r = vec![hp_vector::new_length(length, units.clone())?];
    Ok(Matrix::from_parts(MatrixType::Diagonal, true, length, length, r, units))
}

/// `new HpLowerTriangularMatrix(int length, Unit units)`
pub fn new_lower(length: usize, units: OptUnit) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let mut r = Vec::with_capacity(length);
    for i in 0..length {
        r.push(hp_vector::new_length(i + 1, units.clone())?);
    }
    Ok(Matrix::from_parts(
        MatrixType::LowerTriangular,
        true,
        length,
        length,
        r,
        units,
    ))
}

/// `new HpUpperTriangularMatrix(int length, Unit units)`
pub fn new_upper(length: usize, units: OptUnit) -> CResult<Matrix> {
    if length > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let mut r = Vec::with_capacity(length);
    for i in 0..length {
        r.push(hp_vector::new_length(length - i, units.clone())?);
    }
    Ok(Matrix::from_parts(
        MatrixType::UpperTriangular,
        true,
        length,
        length,
        r,
        units,
    ))
}

/// `new HpSymmetricMatrix(int length, Unit units)`
pub fn new_symmetric(length: usize, units: OptUnit) -> CResult<Matrix> {
    let m = new_upper(length, units)?;
    m.data_mut().mtype = MatrixType::Symmetric;
    Ok(m)
}

/// `new HpColumnMatrix(Vector v)` (also the implicit conversion of an HpVector)
pub fn hp_column_from_vector(v: &Vector) -> CResult<Matrix> {
    let len = v.len();
    if len > MAX_SIZE {
        return Err(exceptions::MatrixDimensions());
    }
    let hp_v = if v.is_hp() {
        v.clone()
    } else {
        hp_vector::from_vector(v, None)?
    };
    let units = hp_v.hp_units();
    Ok(Matrix::from_parts(MatrixType::Column, true, len, 1, vec![hp_v], units))
}

/// `HpMatrix.Units`
pub fn units(m: &Matrix) -> OptUnit {
    m.data().units.clone()
}

/// `HpMatrix.Units = value` (virtual setter: sets the units of all rows)
pub fn set_units_prop(m: &Matrix, units: OptUnit) {
    m.data_mut().units = units.clone();
    for r in m.rows().iter().rev() {
        r.set_hp_units(units.clone());
    }
}

/// `Scale(double d)`
pub fn scale(m: &Matrix, d: f64) {
    for r in m.rows() {
        r.hp_scale(d);
    }
}

pub(super) fn clone_m(m: &Matrix) -> Matrix {
    let (t, rc, cc, u) = {
        let d = m.data();
        (d.mtype, d.row_count, d.col_count, d.units.clone())
    };
    match t {
        MatrixType::Full => new(rc, cc, u),
        MatrixType::Column => new_column(rc, u),
        MatrixType::Diagonal => new_diagonal(rc, u),
        MatrixType::LowerTriangular => new_lower(rc, u),
        MatrixType::UpperTriangular => new_upper(rc, u),
        MatrixType::Symmetric => new_symmetric(rc, u),
    }
    .expect("valid dimensions")
}

/// `CloneAsMatrix()` (virtual)
pub(super) fn clone_as_matrix(m: &Matrix) -> Matrix {
    let (t, rc, cc) = {
        let d = m.data();
        (d.mtype, d.row_count, d.col_count)
    };
    match t {
        MatrixType::Column => super::column_matrix::new(rc),
        _ => Matrix::full(rc, cc),
    }
}

pub(super) fn equals(_m: &Matrix, _other: &Matrix) -> bool {
    false
}
pub(super) fn copy(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn resize(_m: &Matrix, _rows: usize, _cols: usize) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn set_units(m: &Matrix, units: &OptUnit) -> CResult<()> {
    m.data_mut().units = units.clone();
    for r in m.rows() {
        r.set_units(units)?;
    }
    Ok(())
}
pub(super) fn row(_m: &Matrix, _index: i64) -> CResult<Vector> {
    Err(hp_not_ported())
}
pub(super) fn col(_m: &Matrix, _index: i64) -> CResult<Vector> {
    Err(hp_not_ported())
}
pub(super) fn submatrix(_m: &Matrix, _i1: i64, _i2: i64, _j1: i64, _j2: i64) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn extract_rows(_m: &Matrix, _v: &Vector) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn extract_cols(_m: &Matrix, _v: &Vector) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn fill(_m: &Matrix, _value: &RealValue) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn fill_row(_m: &Matrix, _i: i64, _value: &RealValue) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn fill_col(_m: &Matrix, _j: i64, _value: &RealValue) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn search(_m: &Matrix, _value: &RealValue, _i: usize, _j: usize) -> CResult<Vector> {
    Err(hp_not_ported())
}
pub(super) fn count(_m: &Matrix, _value: &RealValue) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn find_all(_m: &Matrix, _value: &RealValue, _rel: Relation) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn h_lookup(_m: &Matrix, _value: &RealValue, _s: i64, _r: i64, _rel: Relation) -> CResult<Vector> {
    Err(hp_not_ported())
}
pub(super) fn v_lookup(_m: &Matrix, _value: &RealValue, _s: i64, _r: i64, _rel: Relation) -> CResult<Vector> {
    Err(hp_not_ported())
}
pub(super) fn sort_rows(_m: &Matrix, _j: i64, _reverse: bool) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn reorder_rows(_m: &Matrix, _indexes: &[i32]) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn sort_cols(_m: &Matrix, _i: i64, _reverse: bool) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn reorder_cols(_m: &Matrix, _indexes: &[i32]) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn order_rows(_m: &Matrix, _j: i64, _reverse: bool) -> CResult<Vector> {
    Err(hp_not_ported())
}
pub(super) fn order_cols(_m: &Matrix, _i: i64, _reverse: bool) -> CResult<Vector> {
    Err(hp_not_ported())
}
pub(super) fn min(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn max(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn sum(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn sum_sq(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn srss(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn average(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn product(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn mean(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn transpose(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn trace(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn l1_norm(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn inf_norm(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}
pub(super) fn copy_to(_m: &Matrix, _target: &Matrix, _i: i64, _j: i64) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn add_to(_m: &Matrix, _target: &Matrix, _i: i64, _j: i64) -> CResult<Matrix> {
    Err(hp_not_ported())
}
pub(super) fn diagonal(_m: &Matrix) -> CResult<Vector> {
    Err(hp_not_ported())
}

/// The HpMatrix operators, dispatched from `Value::binary`. HpVector operands are
/// converted to HpColumnMatrix by the caller.
pub mod ops {
    use super::hp_not_ported;
    use crate::base_types::real_value::RealValue;
    use crate::base_types::value::BinOp;
    use crate::math_parser_exception::CResult;
    use crate::matrix::Matrix;

    /// `-HpMatrix`
    pub fn neg(_a: &Matrix) -> CResult<Matrix> {
        Err(hp_not_ported())
    }

    /// `HpMatrix op RealValue`
    pub fn op_mr(_op: BinOp, _a: &Matrix, _b: &RealValue) -> CResult<Matrix> {
        Err(hp_not_ported())
    }

    /// `RealValue op HpMatrix` (only the operators that do not commute)
    pub fn op_rm(_op: BinOp, _a: &RealValue, _b: &Matrix) -> CResult<Matrix> {
        Err(hp_not_ported())
    }

    /// `HpMatrix op HpMatrix`
    pub fn op_mm(_op: BinOp, _a: &Matrix, _b: &Matrix) -> CResult<Matrix> {
        Err(hp_not_ported())
    }
}
