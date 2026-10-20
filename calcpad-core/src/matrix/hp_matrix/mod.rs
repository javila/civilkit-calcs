//! Port of Matrix/HpMatrix: the high performance matrices (doubles with one unit).
//!
//! The storage is shared with `Matrix` (`MatData.hp == true`, rows are HpVectors, the
//! unit is `MatData.units`). This module holds the constructors and the HpMatrix
//! overrides of the virtual `Matrix` members.
use super::{Matrix, MatrixType, MAX_SIZE, PARALLEL_THRESHOLD};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::calculator::{self, RealFn, RealOp};
use crate::exceptions;
use crate::extension_methods::almost_equals;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::vector::{hp_vector, HpData, Relation, VecData, Vector};

pub mod hp_column_matrix;
pub mod hp_diagonal_matrix;
pub mod hp_eigen_solver;
pub mod hp_fft;
pub mod hp_linear_solver;
pub mod hp_matmul_winograd;
pub mod hp_svd_solver;
pub mod hp_symmetric_matrix;
pub mod hp_triangular_matrix;
pub mod numerics;
pub mod vectorized;

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

/// `new HpVector(length, units)` for a length known to be valid.
pub(crate) fn hpv(length: usize, units: OptUnit) -> Vector {
    Vector::from_data(VecData::Hp(HpData {
        values: None,
        size: 0,
        capacity: 0,
        length,
        units,
    }))
}

/// Wraps an HP storage into a vector object.
pub(crate) fn hpd(d: HpData) -> Vector {
    Vector::from_data(VecData::Hp(d))
}

// ----- typed element access ------------------------------------------------------------------

/// `GetValue(int row, int col)` (virtual in C#, by matrix type)
pub fn get_value(m: &Matrix, row: usize, col: usize) -> f64 {
    let (v, j) = {
        let d = m.data();
        match d.mtype {
            MatrixType::Full => (d.rows[row].clone(), col),
            MatrixType::Column => (d.rows[0].clone(), row),
            MatrixType::Diagonal => {
                if row != col {
                    return 0.0;
                }
                (d.rows[0].clone(), row)
            }
            MatrixType::LowerTriangular => {
                if row < col {
                    return 0.0;
                }
                (d.rows[row].clone(), col)
            }
            MatrixType::UpperTriangular => {
                if row > col {
                    return 0.0;
                }
                (d.rows[row].clone(), col - row)
            }
            MatrixType::Symmetric => {
                if row <= col {
                    (d.rows[row].clone(), col - row)
                } else {
                    (d.rows[col].clone(), row - col)
                }
            }
        }
    };
    v.with_hp(|h| h.get_value(j))
}

/// `SetValue(double value, int row, int col)` (virtual in C#, by matrix type)
pub fn set_value(m: &Matrix, value: f64, row: usize, col: usize) -> CResult<()> {
    let (v, j) = {
        let d = m.data();
        match d.mtype {
            MatrixType::Full => (d.rows[row].clone(), col),
            MatrixType::Column => (d.rows[0].clone(), row),
            MatrixType::Diagonal => {
                if row != col {
                    return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row + 1, col + 1)));
                }
                (d.rows[0].clone(), row)
            }
            MatrixType::LowerTriangular => {
                if row < col {
                    return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row + 1, col + 1)));
                }
                (d.rows[row].clone(), col)
            }
            MatrixType::UpperTriangular => {
                if row > col {
                    return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row, col)));
                }
                (d.rows[row].clone(), col - row)
            }
            MatrixType::Symmetric => {
                if row <= col {
                    (d.rows[row].clone(), col - row)
                } else {
                    (d.rows[col].clone(), row - col)
                }
            }
        }
    };
    v.with_hp_mut(|h| h.set_value(value, j));
    Ok(())
}

/// A copy of the rows and dimensions of an HP matrix for the numerical algorithms, with
/// the same typed element access as the matrix.
#[derive(Clone)]
pub(crate) struct HpMat {
    pub mtype: MatrixType,
    pub rows: Vec<HpData>,
    pub row_count: usize,
    pub col_count: usize,
    pub units: OptUnit,
}

impl HpMat {
    pub fn from(m: &Matrix) -> HpMat {
        let d = m.data();
        HpMat {
            mtype: d.mtype,
            rows: d.rows.iter().map(hp_vector::snap).collect(),
            row_count: d.row_count,
            col_count: d.col_count,
            units: d.units.clone(),
        }
    }

    /// An empty HP matrix of the given type (the C# constructors).
    pub fn empty(mtype: MatrixType, rc: usize, cc: usize, units: OptUnit) -> HpMat {
        let blank = |len: usize| HpData {
            values: None,
            size: 0,
            capacity: 0,
            length: len,
            units: units.clone(),
        };
        let rows = match mtype {
            MatrixType::Full => (0..rc).map(|_| blank(cc)).collect(),
            MatrixType::Column | MatrixType::Diagonal => vec![blank(rc)],
            MatrixType::LowerTriangular => (0..rc).map(|i| blank(i + 1)).collect(),
            MatrixType::UpperTriangular | MatrixType::Symmetric => (0..rc).map(|i| blank(rc - i)).collect(),
        };
        HpMat {
            mtype,
            rows,
            row_count: rc,
            col_count: cc,
            units,
        }
    }

    pub fn into_matrix(self) -> Matrix {
        let rows = self.rows.into_iter().map(hpd).collect();
        Matrix::from_parts(self.mtype, true, self.row_count, self.col_count, rows, self.units)
    }

    #[inline]
    pub fn get_value(&self, row: usize, col: usize) -> f64 {
        match self.mtype {
            MatrixType::Full => self.rows[row].get_value(col),
            MatrixType::Column => self.rows[0].get_value(row),
            MatrixType::Diagonal => {
                if row == col {
                    self.rows[0].get_value(row)
                } else {
                    0.0
                }
            }
            MatrixType::LowerTriangular => {
                if row >= col {
                    self.rows[row].get_value(col)
                } else {
                    0.0
                }
            }
            MatrixType::UpperTriangular => {
                if row <= col {
                    self.rows[row].get_value(col - row)
                } else {
                    0.0
                }
            }
            MatrixType::Symmetric => {
                if row <= col {
                    self.rows[row].get_value(col - row)
                } else {
                    self.rows[col].get_value(row - col)
                }
            }
        }
    }

    pub fn set_value(&mut self, value: f64, row: usize, col: usize) -> CResult<()> {
        match self.mtype {
            MatrixType::Full => self.rows[row].set_value(value, col),
            MatrixType::Column => self.rows[0].set_value(value, row),
            MatrixType::Diagonal => {
                if row != col {
                    return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row + 1, col + 1)));
                }
                self.rows[0].set_value(value, row)
            }
            MatrixType::LowerTriangular => {
                if row < col {
                    return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row + 1, col + 1)));
                }
                self.rows[row].set_value(value, col)
            }
            MatrixType::UpperTriangular => {
                if row > col {
                    return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row, col)));
                }
                self.rows[row].set_value(value, col - row)
            }
            MatrixType::Symmetric => {
                if row <= col {
                    self.rows[row].set_value(value, col - row)
                } else {
                    self.rows[col].set_value(value, row - col)
                }
            }
        }
        Ok(())
    }
}

// ----- constructors used by MatrixCalculator --------------------------------------------------

/// `new HpMatrix(double[][] M, Unit units)`
pub(crate) fn from_jagged(m: Vec<Vec<f64>>, units: OptUnit) -> CResult<Matrix> {
    let rc = m.len();
    let cc = m
        .first()
        .map(|r| r.len())
        .ok_or_else(crate::vector::index_was_outside)?;
    let mut rows = Vec::with_capacity(rc);
    for r in m {
        rows.push(hp_vector::from_doubles(r, units.clone())?);
    }
    Ok(Matrix::from_parts(MatrixType::Full, true, rc, cc, rows, units))
}

/// `HpMatrix(Matrix matrix)` base constructor: the rows of `matrix` converted to HP.
fn rows_from_matrix(matrix: &Matrix, mtype: MatrixType) -> CResult<Matrix> {
    let (rc, cc) = (matrix.row_count(), matrix.col_count());
    if rc == 0 || cc == 0 {
        let rows = (0..rc).map(|_| hpv(0, None)).collect();
        return Ok(Matrix::from_parts(mtype, true, rc, cc, rows, None));
    }
    let units = matrix.get(0, 0).units;
    let src = matrix.rows();
    let mut rows = Vec::with_capacity(rc);
    for i in 0..rc {
        let r = src.get(i).ok_or_else(crate::vector::index_was_outside)?;
        let hr = hp_vector::from_vector(r, units.clone())?;
        let ru = hr.hp_units();
        if units.is_none() {
            if let Some(ru) = &ru {
                return Err(exceptions::InconsistentUnits(&Unit::get_text_of(&None), &ru.text()));
            }
        }
        rows.push(hr);
    }
    // the C# constructor fills the array from the last row
    Ok(Matrix::from_parts(mtype, true, rc, cc, rows, units))
}

/// The `hp()` conversion of a non-HP matrix: `new HpColumnMatrix(cm)`, `new
/// HpDiagonalMatrix(dm)`, `new HpLowerTriangularMatrix(ltm)`, ... or `new HpMatrix(matrix)`.
pub fn from_matrix(m: &Matrix) -> CResult<Matrix> {
    match m.mtype() {
        MatrixType::Column => hp_column_from_vector(&m.row_ref(0)),
        MatrixType::Diagonal => diagonal_from_vector(&m.row_ref(0)),
        t => rows_from_matrix(m, t),
    }
}

/// `new HpMatrix(Vector v)`: a one row HP matrix.
pub fn from_row_vector(v: &Vector) -> CResult<Matrix> {
    let hp_v = if v.is_hp() {
        v.clone()
    } else {
        hp_vector::from_vector(v, None)?
    };
    let units = hp_v.hp_units();
    Ok(Matrix::from_parts(
        MatrixType::Full,
        true,
        1,
        v.len(),
        vec![hp_v],
        units,
    ))
}

/// `new HpDiagonalMatrix(Vector v)`
pub fn diagonal_from_vector(v: &Vector) -> CResult<Matrix> {
    let len = v.len();
    if len > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let hp_v = if v.is_hp() {
        v.clone()
    } else {
        hp_vector::from_vector(v, None)?
    };
    let units = hp_v.hp_units();
    Ok(Matrix::from_parts(
        MatrixType::Diagonal,
        true,
        len,
        len,
        vec![hp_v],
        units,
    ))
}

/// `new HpDiagonalMatrix(int length, in RealValue value)`
pub fn diagonal_with_value(n: usize, value: &RealValue) -> CResult<Matrix> {
    let m = new_diagonal(n, value.units.clone())?;
    m.row_ref(0).hp_fill(value)?;
    Ok(m)
}

/// `new HpColumnMatrix(int length, RealValue value)`
pub fn column_with_value(n: usize, value: &RealValue) -> CResult<Matrix> {
    let m = new_column(n, value.units.clone())?;
    m.row_ref(0).hp_fill(value)?;
    Ok(m)
}

/// `Identity(int n)` (HpMatrix)
pub(crate) fn identity(n: usize) -> HpMat {
    let mut m = HpMat::empty(MatrixType::Full, n, n, None);
    for i in 0..n {
        m.rows[i].set_value(1.0, i);
    }
    m
}

/// `CheckBounds(HpMatrix a, HpMatrix b)`
fn check_bounds(a: &Matrix, b: &Matrix) -> CResult<()> {
    if a.row_count() != b.row_count() || a.col_count() != b.col_count() {
        return Err(exceptions::MatrixDimensions());
    }
    Ok(())
}

// ----- virtual members -----------------------------------------------------------------------

/// `Clone()` (virtual)
pub(super) fn clone_m(m: &Matrix) -> Matrix {
    let d = m.data();
    HpMat::empty(d.mtype, d.row_count, d.col_count, d.units.clone()).into_matrix()
}

/// `CloneAsMatrix()` (virtual): the non-HP matrix of the same class.
pub(super) fn clone_as_matrix(m: &Matrix) -> Matrix {
    let (t, rc, cc) = {
        let d = m.data();
        (d.mtype, d.row_count, d.col_count)
    };
    match t {
        MatrixType::Full => Matrix::full(rc, cc),
        MatrixType::Column => super::column_matrix::new(rc),
        MatrixType::Diagonal => super::diagonal_matrix::new(rc),
        MatrixType::LowerTriangular => super::lower_triangular_matrix::new(rc),
        MatrixType::UpperTriangular => super::upper_triangular_matrix::new(rc),
        MatrixType::Symmetric => super::symmetric_matrix::new(rc),
    }
}

/// `Equals(object)` of HpMatrix
pub(super) fn equals(m: &Matrix, other: &Matrix) -> bool {
    if Matrix::ptr_eq(m, other) {
        return true;
    }
    if !other.is_hp() {
        return false;
    }
    if m.mtype() != other.mtype() || m.row_count() != other.row_count() || m.col_count() != other.col_count() {
        return false;
    }
    let (ra, rb) = (m.rows(), other.rows());
    for i in (0..ra.len()).rev() {
        if !ra[i].equals(&rb[i]) {
            return false;
        }
    }
    true
}

/// `Copy()` (protected virtual): a full HP matrix.
pub(super) fn copy(m: &Matrix) -> CResult<Matrix> {
    Ok(copy_mat(&HpMat::from(m)).into_matrix())
}

pub(crate) fn copy_mat(a: &HpMat) -> HpMat {
    let (rc, cc) = (a.row_count, a.col_count);
    let mut mm = HpMat::empty(MatrixType::Full, rc, cc, a.units.clone());
    match a.mtype {
        MatrixType::Full => {
            for i in 0..rc {
                mm.rows[i] = a.rows[i].copy();
            }
        }
        MatrixType::LowerTriangular => {
            for i in 0..rc {
                let mut row = a.rows[i].copy();
                row.resize(cc);
                mm.rows[i] = row;
            }
        }
        _ => {
            for i in (0..rc).rev() {
                for j in (0..cc).rev() {
                    let v = a.get_value(i, j);
                    mm.rows[i].set_value(v, j);
                }
            }
        }
    }
    mm
}

/// `RawCopy()`: the elements as a jagged array of full rows.
pub(crate) fn raw_copy(a: &HpMat) -> Vec<Vec<f64>> {
    let (rc, cc) = (a.row_count, a.col_count);
    let mut out = Vec::with_capacity(rc);
    if a.mtype == MatrixType::Full || a.mtype == MatrixType::LowerTriangular {
        for i in 0..rc {
            out.push(a.rows[i].raw_copy(Some(cc)));
        }
    } else {
        for i in 0..rc {
            let mut row = vec![0.0; cc];
            for (j, v) in row.iter_mut().enumerate().rev() {
                *v = a.get_value(i, j);
            }
            out.push(row);
        }
    }
    out
}

/// `Resize(int m, int n)` (virtual)
pub(super) fn resize(m: &Matrix, rows: usize, cols: usize) -> CResult<Matrix> {
    let (t, rc, cc, units) = {
        let d = m.data();
        (d.mtype, d.row_count, d.col_count, d.units.clone())
    };
    if rows == rc && cols == cc {
        return Ok(m.clone());
    }
    if rows > MAX_SIZE || cols > MAX_SIZE {
        return Err(exceptions::MatrixDimensions());
    }
    let m1 = rows.min(rc);
    if t == MatrixType::Full {
        if rows != rc {
            let mut d = m.data_mut();
            d.rows.truncate(rows);
            for _ in rc..rows {
                d.rows.push(hpv(cols, units.clone()));
            }
            d.row_count = rows;
        }
        if cols != cc {
            for i in 0..m1 {
                m.row_ref(i).hp_resize(cols)?;
            }
            m.data_mut().col_count = cols;
        }
        return Ok(m.clone());
    }
    if cols == 1 && t == MatrixType::Column {
        if rows != rc {
            m.row_ref(0).hp_resize(rows)?;
            m.data_mut().row_count = rows;
        }
        return Ok(m.clone());
    }
    if rows == cols {
        let n = rows;
        match t {
            MatrixType::Diagonal => {
                if n != rc {
                    m.row_ref(0).hp_resize(n)?;
                    m.set_dims(n, n);
                }
                return Ok(m.clone());
            }
            MatrixType::Symmetric | MatrixType::UpperTriangular => {
                if n != rc {
                    let n1 = n.min(rc);
                    {
                        let mut d = m.data_mut();
                        d.rows.truncate(n);
                        for i in rc..n {
                            d.rows.push(hpv(n - i, units.clone()));
                        }
                    }
                    for i in 0..n1 {
                        m.row_ref(i).hp_resize(n - i)?;
                    }
                    m.set_dims(n, n);
                }
                return Ok(m.clone());
            }
            MatrixType::LowerTriangular => {
                if n != rc {
                    let mut d = m.data_mut();
                    d.rows.truncate(n);
                    for i in rc..n {
                        d.rows.push(hpv(i + 1, units.clone()));
                    }
                    d.row_count = n;
                    d.col_count = n;
                }
                return Ok(m.clone());
            }
            _ => {}
        }
    }
    // ResizeGeneral
    let a = HpMat::from(m);
    let mut mm = HpMat::empty(MatrixType::Full, rows, cols, units);
    let n1 = cols.min(cc);
    for i in (0..m1).rev() {
        for j in (0..n1).rev() {
            mm.rows[i].set_value(a.get_value(i, j), j);
        }
    }
    Ok(mm.into_matrix())
}

/// `SetUnits(Unit units)` of an HP matrix (the base Matrix method on the rows).
pub(super) fn set_units(m: &Matrix, units: &OptUnit) -> CResult<()> {
    for r in m.rows().iter().rev() {
        r.set_units(units)?;
    }
    Ok(())
}

/// `Row(int index)`
pub(super) fn row(m: &Matrix, index: i64) -> CResult<Vector> {
    if index < 1 || index > m.row_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&index.to_string()));
    }
    let i = (index - 1) as usize;
    let t = m.mtype();
    if t == MatrixType::Full || t == MatrixType::LowerTriangular {
        return Ok(m.row_ref(i).hp_copy());
    }
    Ok(row_values(m, i))
}

/// `RowRef(int i)`
pub(crate) fn row_ref(m: &Matrix, i: usize) -> Vector {
    let t = m.mtype();
    if t == MatrixType::Full || t == MatrixType::LowerTriangular {
        return m.row_ref(i);
    }
    row_values(m, i)
}

fn row_values(m: &Matrix, i: usize) -> Vector {
    let n = m.col_count();
    let mut row = HpData {
        values: None,
        size: 0,
        capacity: 0,
        length: n,
        units: units(m),
    };
    for j in (0..n).rev() {
        row.set_value(get_value(m, i, j), j);
    }
    hpd(row)
}

/// `Col(int index)`
pub(super) fn col(m: &Matrix, index: i64) -> CResult<Vector> {
    if index < 1 || index > m.col_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&index.to_string()));
    }
    let j = (index - 1) as usize;
    if m.mtype() == MatrixType::Column {
        return Ok(m.row_ref(j).hp_copy());
    }
    hp_vector::from_doubles(raw_col(m, j), units(m))
}

/// `RawCol(int index)`
pub(crate) fn raw_col(m: &Matrix, j: usize) -> Vec<f64> {
    let rc = m.row_count();
    (0..rc).map(|i| get_value(m, i, j)).collect()
}

/// `SetRow(int index, HpVector row)`
pub(crate) fn set_row(m: &Matrix, index: usize, row: &Vector) -> CResult<()> {
    let r = row.hp_resize(m.col_count())?;
    m.set_row_ref(index, r);
    Ok(())
}

/// `SetCol(int index, HpVector col)`
pub(crate) fn set_col(m: &Matrix, index: usize, col: &Vector) -> CResult<()> {
    let h = hp_vector::snap(col);
    let n = h.size.min(m.row_count());
    let values = h.raw_all();
    for i in (0..n).rev() {
        set_value(m, values[i], i, index)?;
    }
    Ok(())
}

/// `Submatrix(int i1, int i2, int j1, int j2)`
pub(super) fn submatrix(m: &Matrix, i1: i64, i2: i64, j1: i64, j2: i64) -> CResult<Matrix> {
    let (mut i1, mut i2, mut j1, mut j2) = (i1, i2, j1, j2);
    if i2 < i1 {
        std::mem::swap(&mut i1, &mut i2);
    }
    if j2 < j1 {
        std::mem::swap(&mut j1, &mut j2);
    }
    if i1 < 1 {
        return Err(exceptions::IndexOutOfRange(&i1.to_string()));
    }
    if i2 > m.row_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&i2.to_string()));
    }
    if j1 < 1 {
        return Err(exceptions::IndexOutOfRange(&j1.to_string()));
    }
    if j2 > m.col_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&j2.to_string()));
    }
    let rows = (i2 - i1) as usize;
    let cols = (j2 - j1) as usize;
    let mm = new(rows + 1, cols + 1, units(m))?;
    let i1 = (i1 - 1) as usize;
    let t = m.mtype();
    if t == MatrixType::Full || t == MatrixType::LowerTriangular {
        for i in (0..=rows).rev() {
            mm.set_row_ref(i, m.row_ref(i1 + i).hp_slice(j1, j2)?);
        }
    } else {
        let j1 = (j1 - 1) as usize;
        for i in (0..=rows).rev() {
            let row = mm.row_ref(i);
            for j in (0..=cols).rev() {
                let v = get_value(m, i1 + i, j1 + j);
                row.with_hp_mut(|h| h.set_value(v, j));
            }
        }
    }
    Ok(mm)
}

fn as_hp_vector(v: &Vector) -> CResult<Vector> {
    if v.is_hp() {
        Ok(v.clone())
    } else {
        hp_vector::from_vector(v, None)
    }
}

/// `ExtractRows(Vector vector)`: shares the rows of a full matrix.
pub(super) fn extract_rows(m: &Matrix, vector: &Vector) -> CResult<Matrix> {
    let hv = as_hp_vector(vector)?;
    let row_count = hv.len();
    let n = m.col_count();
    let mm = new(row_count, n, units(m))?;
    let t = m.mtype();
    for i in (0..row_count).rev() {
        let row_index = hv.get(i).d as i64;
        if row_index < 1 || row_index > m.row_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&row_index.to_string()));
        }
        let ri = (row_index - 1) as usize;
        if t == MatrixType::Full || t == MatrixType::LowerTriangular {
            mm.set_row_ref(i, m.row_ref(ri));
        } else {
            let row = mm.row_ref(i);
            for j in (0..n).rev() {
                let v = get_value(m, ri, j);
                row.with_hp_mut(|h| h.set_value(v, j));
            }
        }
    }
    Ok(mm)
}

/// `ExtractCols(Vector vector)`
pub(super) fn extract_cols(m: &Matrix, vector: &Vector) -> CResult<Matrix> {
    let hv = as_hp_vector(vector)?;
    let col_count = hv.len();
    let rc = m.row_count();
    let mm = new(rc, col_count, units(m))?;
    for j in (0..col_count).rev() {
        let col_index = hv.get(j).d as i64;
        if col_index < 1 || col_index > m.col_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&col_index.to_string()));
        }
        for i in (0..rc).rev() {
            mm.set(i, j, m.get(i, (col_index - 1) as usize))?;
        }
    }
    Ok(mm)
}

/// `Fill(RealValue value)`: fills in place.
pub(super) fn fill(m: &Matrix, value: &RealValue) -> CResult<Matrix> {
    for r in m.rows().iter().rev() {
        r.hp_fill(value)?;
    }
    set_units_prop(m, value.units.clone());
    Ok(m.clone())
}

/// `FillRow(int i, in RealValue value)`
pub(super) fn fill_row(m: &Matrix, i: i64, value: &RealValue) -> CResult<Matrix> {
    if i > m.row_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&i.to_string()));
    }
    let i = (i - 1) as usize;
    let u = units(m);
    let d = Unit::convert(&u, &value.units, ',')?;
    let v = RealValue::new(value.d * d, u);
    match m.mtype() {
        MatrixType::Full | MatrixType::Symmetric | MatrixType::UpperTriangular | MatrixType::LowerTriangular => {
            m.row_ref(i).hp_fill(&v)?;
        }
        _ => m.row_ref(0).set(i, v)?,
    }
    Ok(m.clone())
}

/// `FillCol(int j, in RealValue value)`
pub(super) fn fill_col(m: &Matrix, j: i64, value: &RealValue) -> CResult<Matrix> {
    if j > m.col_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&j.to_string()));
    }
    let j = (j - 1) as usize;
    let u = units(m);
    let d = Unit::convert(&u, &value.units, ',')?;
    let v = RealValue::new(value.d * d, u);
    let rc = m.row_count();
    match m.mtype() {
        MatrixType::Full => {
            for i in (0..rc).rev() {
                set_value(m, v.d, i, j)?;
            }
        }
        MatrixType::Column | MatrixType::Symmetric => {
            m.row_ref(j).hp_fill(&v)?;
        }
        MatrixType::Diagonal => m.row_ref(0).set(j, v)?,
        MatrixType::LowerTriangular => {
            for i in (j..rc).rev() {
                set_value(m, v.d, i, j)?;
            }
        }
        MatrixType::UpperTriangular => {
            for i in (0..=j).rev() {
                set_value(m, v.d, i, j)?;
            }
        }
    }
    Ok(m.clone())
}

/// `HpMatrix.Augment(params HpMatrix[] matrices)`
pub fn augment(matrices: &[Matrix]) -> CResult<Matrix> {
    let (mut m, mut n) = (0, 0);
    for matrix in matrices {
        n += matrix.col_count();
        m = m.max(matrix.row_count());
    }
    let u0 = units(matrices.first().ok_or_else(crate::vector::index_was_outside)?);
    let mut mm = HpMat::empty(MatrixType::Full, m, n, u0.clone());
    if m > MAX_SIZE || n > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let mut c = 0;
    for matrix in matrices {
        let a = HpMat::from(matrix);
        let d = Unit::convert(&u0, &a.units, ',')?;
        for i in (0..a.row_count).rev() {
            for j in (0..a.col_count).rev() {
                mm.rows[i].set_value(a.get_value(i, j) * d, c + j);
            }
        }
        c += a.col_count;
    }
    Ok(mm.into_matrix())
}

/// `HpMatrix.Stack(params HpMatrix[] matrices)`
pub fn stack(matrices: &[Matrix]) -> CResult<Matrix> {
    let (mut m, mut n) = (0, 0);
    for matrix in matrices {
        n = n.max(matrix.col_count());
        m += matrix.row_count();
    }
    let u0 = units(matrices.first().ok_or_else(crate::vector::index_was_outside)?);
    if m > MAX_SIZE || n > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let mut mm = HpMat::empty(MatrixType::Full, m, n, u0.clone());
    let mut r = 0;
    for matrix in matrices {
        let a = HpMat::from(matrix);
        let d = Unit::convert(&u0, &a.units, ',')?;
        for i in (0..a.row_count).rev() {
            for j in (0..a.col_count).rev() {
                mm.rows[r + i].set_value(a.get_value(i, j) * d, j);
            }
        }
        r += a.row_count;
    }
    Ok(mm.into_matrix())
}

/// `Search(in RealValue value, int iStart, int jStart)`
pub(super) fn search(m: &Matrix, value: &RealValue, i_start: usize, j_start: usize) -> CResult<Vector> {
    let t = m.mtype();
    let (rc, cc) = (m.row_count(), m.col_count());
    if t == MatrixType::Full || t == MatrixType::LowerTriangular {
        let mut j_start = j_start;
        for i in i_start.saturating_sub(1)..rc {
            let j_value = m.row_ref(i).hp_search(value, j_start)?.d;
            if j_value != 0.0 {
                return hp_vector::from_doubles(vec![(i + 1) as f64, j_value], None);
            }
            j_start = 1;
        }
    } else {
        let u = units(m);
        if Unit::is_consistent_opt(&u, &value.units) {
            let d = value.d * Unit::convert(&u, &value.units, ',')?;
            let mut j_start = j_start.saturating_sub(1);
            for i in i_start.saturating_sub(1)..rc {
                for j in j_start..cc {
                    if almost_equals(get_value(m, i, j), d) {
                        return hp_vector::from_doubles(vec![(i + 1) as f64, (j + 1) as f64], None);
                    }
                }
                j_start = 0;
            }
        }
    }
    Ok(hpv(2, None))
}

/// `Count(RealValue value)` (note the C# local function that adds to `count` and returns
/// it, so the counts accumulate)
pub(super) fn count(m: &Matrix, value: &RealValue) -> CResult<RealValue> {
    if m.mtype() == MatrixType::Symmetric {
        return hp_symmetric_matrix::count(m, value);
    }
    let t = m.mtype();
    let mut count: i64 = 0;
    if t == MatrixType::Full || t == MatrixType::LowerTriangular || !value.equals(&RealValue::ZERO) {
        let rows = m.rows();
        let len = rows.len();
        if len > PARALLEL_THRESHOLD {
            let mut sum = 0.0;
            for r in &rows {
                sum += r.hp_count(value, 1)?.d;
            }
            count = sum as i64;
        } else {
            for r in &rows {
                count += r.hp_count(value, 1)?.d as i64;
            }
        }
    } else {
        let u = units(m);
        if !Unit::is_consistent_opt(&u, &value.units) {
            return Ok(RealValue::ZERO);
        }
        let d = value.d * Unit::convert(&u, &value.units, ',')?;
        let (rc, cc) = (m.row_count(), m.col_count());
        let a = HpMat::from(m);
        let count_row = |i: usize, count: &mut i64| -> i64 {
            for j in 0..cc {
                if almost_equals(a.get_value(i, j), d) {
                    *count += 1;
                }
            }
            *count
        };
        if rc > PARALLEL_THRESHOLD {
            let mut sum = 0.0;
            for i in 0..rc {
                sum += count_row(i, &mut count) as f64;
            }
            count = sum as i64;
        } else {
            for i in 0..rc {
                // count += CountRow(i, d): the left operand is read before the call
                let before = count;
                let r = count_row(i, &mut count);
                count = before + r;
            }
        }
    }
    Ok(RealValue::num(count as i32 as f64))
}

fn find_all_indexes(m: &Matrix, value: &RealValue, rel: Relation) -> CResult<Vec<(usize, usize)>> {
    let mut indexes = Vec::new();
    let u = units(m);
    if !Unit::is_consistent_opt(&u, &value.units) {
        return Ok(indexes);
    }
    let d = value.d * Unit::convert(&u, &value.units, ',')?;
    let a = HpMat::from(m);
    for i in 0..a.row_count {
        for j in 0..a.col_count {
            if hp_vector::relate(a.get_value(i, j), d, rel) {
                indexes.push((i + 1, j + 1));
            }
        }
    }
    Ok(indexes)
}

/// `FindAll(in RealValue value, Relation rel)`
pub(super) fn find_all(m: &Matrix, value: &RealValue, rel: Relation) -> CResult<Matrix> {
    let indexes = find_all_indexes(m, value, rel)?;
    let n = indexes.len();
    if n == 0 {
        return new(2, 1, None);
    }
    let mut mm = HpMat::empty(MatrixType::Full, 2, n, None);
    for (k, &(i, j)) in indexes.iter().enumerate() {
        mm.rows[0].set_value(i as f64, k);
        mm.rows[1].set_value(j as f64, k);
    }
    Ok(mm.into_matrix())
}

/// `HLookup(in RealValue value, int searchRow, int returnRow, Relation rel)`
pub(super) fn h_lookup(
    m: &Matrix,
    value: &RealValue,
    search_row: i64,
    return_row: i64,
    rel: Relation,
) -> CResult<Vector> {
    let rc = m.row_count() as i64;
    if search_row < 1 || search_row > rc {
        return Err(exceptions::IndexOutOfRange(&search_row.to_string()));
    }
    if return_row < 1 || return_row > rc {
        return Err(exceptions::IndexOutOfRange(&return_row.to_string()));
    }
    let u = units(m);
    if !Unit::is_consistent_opt(&u, &value.units) {
        return Ok(hpv(1, u));
    }
    let d = value.d * Unit::convert(&u, &value.units, ',')?;
    let i = (search_row - 1) as usize;
    let mut indexes = Vec::new();
    for j in 0..m.col_count() {
        if hp_vector::relate(get_value(m, i, j), d, rel) {
            indexes.push(j);
        }
    }
    let n = indexes.len();
    let mut v = HpData {
        values: None,
        size: 0,
        capacity: 0,
        length: n,
        units: u,
    };
    let i = (return_row - 1) as usize;
    for (j, &k) in indexes.iter().enumerate() {
        v.set_value(get_value(m, i, k), j);
    }
    Ok(hpd(v))
}

/// `VLookup(in RealValue value, int searchCol, int returnCol, Relation rel)`
pub(super) fn v_lookup(
    m: &Matrix,
    value: &RealValue,
    search_col: i64,
    return_col: i64,
    rel: Relation,
) -> CResult<Vector> {
    let cc = m.col_count() as i64;
    if search_col > cc {
        return Err(exceptions::IndexOutOfRange(&search_col.to_string()));
    }
    if return_col > cc {
        return Err(exceptions::IndexOutOfRange(&return_col.to_string()));
    }
    let u = units(m);
    if !Unit::is_consistent_opt(&u, &value.units) {
        return Ok(hpv(1, u));
    }
    let d = value.d * Unit::convert(&u, &value.units, ',')?;
    if search_col < 1 || return_col < 1 {
        return Err(crate::vector::index_was_outside());
    }
    let j = (search_col - 1) as usize;
    let mut indexes = Vec::new();
    for i in 0..m.row_count() {
        if hp_vector::relate(get_value(m, i, j), d, rel) {
            indexes.push(i);
        }
    }
    let n = indexes.len();
    let mut v = HpData {
        values: None,
        size: 0,
        capacity: 0,
        length: n,
        units: u,
    };
    let j = (return_col - 1) as usize;
    for (i, &k) in indexes.iter().enumerate() {
        v.set_value(get_value(m, k, j), i);
    }
    Ok(hpd(v))
}

/// `SortRows(int j, bool reverse)`
pub(super) fn sort_rows(m: &Matrix, j: i64, reverse: bool) -> CResult<Matrix> {
    let col = if m.mtype() == MatrixType::Column {
        m.row_ref(0)
    } else {
        col(m, j)?
    };
    let indexes = col.hp_get_order_indexes(reverse, None)?;
    reorder_rows(m, &indexes)
}

/// `ReorderRows(int[] indexes)`: shares the rows of a full matrix.
pub(super) fn reorder_rows(m: &Matrix, indexes: &[i32]) -> CResult<Matrix> {
    let (rc, cc) = (m.row_count(), m.col_count());
    let mm = new(rc, cc, units(m))?;
    let t = m.mtype();
    if t == MatrixType::Full || t == MatrixType::LowerTriangular {
        for i in (0..rc).rev() {
            mm.set_row_ref(i, m.row_ref(indexes[i] as usize));
        }
    } else {
        for i in (0..rc).rev() {
            let row = mm.row_ref(i);
            for k in (0..cc).rev() {
                let v = get_value(m, indexes[i] as usize, k);
                row.with_hp_mut(|h| h.set_value(v, k));
            }
        }
    }
    Ok(mm)
}

/// `SortCols(int i, bool reverse)`
pub(super) fn sort_cols(m: &Matrix, i: i64, reverse: bool) -> CResult<Matrix> {
    if i > m.row_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&i.to_string()));
    }
    if i < 1 {
        return Err(crate::vector::index_was_outside());
    }
    let indexes = row_ref(m, (i - 1) as usize).hp_get_order_indexes(reverse, Some(m.col_count()))?;
    reorder_cols(m, &indexes)
}

/// `ReorderCols(int[] indexes)`
pub(super) fn reorder_cols(m: &Matrix, indexes: &[i32]) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (rc, cc) = (a.row_count, a.col_count);
    let mut mm = HpMat::empty(MatrixType::Full, rc, cc, a.units.clone());
    for k in (0..rc).rev() {
        for j in (0..cc).rev() {
            let v = a.get_value(k, indexes[j] as usize);
            mm.rows[k].set_value(v, j);
        }
    }
    Ok(mm.into_matrix())
}

/// `OrderRows(int j, bool reverse)`
pub(super) fn order_rows(m: &Matrix, j: i64, reverse: bool) -> CResult<Vector> {
    if j > m.col_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&j.to_string()));
    }
    let col = if m.mtype() == MatrixType::Column {
        m.row_ref(0)
    } else {
        col(m, j)?
    };
    col.hp_order(reverse)
}

/// `OrderCols(int i, bool reverse)`
pub(super) fn order_cols(m: &Matrix, i: i64, reverse: bool) -> CResult<Vector> {
    if i > m.row_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&i.to_string()));
    }
    if i < 1 {
        return Err(crate::vector::index_was_outside());
    }
    row_ref(m, (i - 1) as usize).hp_order(reverse)
}

/// LINQ `Enumerable.Min(double[])`: returns the first NaN.
fn linq_min(v: &[f64]) -> f64 {
    let mut value = v[0];
    if value.is_nan() {
        return value;
    }
    for &x in &v[1..] {
        if x < value {
            value = x;
        } else if x.is_nan() {
            return x;
        }
    }
    value
}

/// LINQ `Enumerable.Max(double[])`: skips leading NaNs.
fn linq_max(v: &[f64]) -> f64 {
    let mut i = 0;
    while i < v.len() && v[i].is_nan() {
        i += 1;
    }
    if i == v.len() {
        return v[i - 1];
    }
    let mut value = v[i];
    while i < v.len() {
        if v[i] > value {
            value = v[i];
        }
        i += 1;
    }
    value
}

fn rows_data(m: &Matrix) -> Vec<HpData> {
    m.rows().iter().map(hp_vector::snap).collect()
}

/// `Min()`
pub(super) fn min(m: &Matrix) -> CResult<RealValue> {
    let rows = rows_data(m);
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let len = rows.len();
    let mut min = hp_vector::min_data(&rows[0]).d;
    if len > PARALLEL_THRESHOLD {
        let mut v = vec![min];
        v.extend(rows[1..].iter().map(|r| hp_vector::min_data(r).d));
        min = linq_min(&v);
    } else {
        for r in &rows[1..] {
            let row_min = hp_vector::min_data(r).d;
            if row_min < min {
                min = row_min;
            }
        }
    }
    if !m.is_structurally_consistent_type() && min > 0.0 {
        min = 0.0;
    }
    Ok(RealValue::new(min, units(m)))
}

/// `Max()`
pub(super) fn max(m: &Matrix) -> CResult<RealValue> {
    let rows = rows_data(m);
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let len = rows.len();
    let mut max = hp_vector::max_data(&rows[0]).d;
    if len > PARALLEL_THRESHOLD {
        let mut v = vec![max];
        v.extend(rows[1..].iter().map(|r| hp_vector::max_data(r).d));
        max = linq_max(&v);
    } else {
        for r in &rows[1..] {
            let b = hp_vector::max_data(r).d;
            if b > max {
                max = b;
            }
        }
    }
    if !m.is_structurally_consistent_type() && max < 0.0 {
        max = 0.0;
    }
    Ok(RealValue::new(max, units(m)))
}

/// `rowValues.Sum()` (LINQ: a plain loop starting from 0)
fn linq_sum(v: &[f64]) -> f64 {
    let mut s = 0.0;
    for &x in v {
        s += x;
    }
    s
}

/// `Sum()`
pub(super) fn sum(m: &Matrix) -> CResult<RealValue> {
    if m.mtype() == MatrixType::Symmetric {
        return hp_symmetric_matrix::sum(m);
    }
    let rows = rows_data(m);
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let mut sum = hp_vector::sum_data(&rows[0]).d;
    if rows.len() > PARALLEL_THRESHOLD {
        let mut v = vec![sum];
        v.extend(rows[1..].iter().map(|r| hp_vector::sum_data(r).d));
        sum = linq_sum(&v);
    } else {
        for r in &rows[1..] {
            sum += hp_vector::sum_data(r).d;
        }
    }
    Ok(RealValue::new(sum, units(m)))
}

/// `SumSq()`
pub(super) fn sum_sq(m: &Matrix) -> CResult<RealValue> {
    if m.mtype() == MatrixType::Symmetric {
        return hp_symmetric_matrix::sum_sq(m);
    }
    let rows = rows_data(m);
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let mut sumsq = hp_vector::sum_sq_data(&rows[0]).d;
    if rows.len() > PARALLEL_THRESHOLD {
        let mut v = vec![sumsq];
        v.extend(rows[1..].iter().map(|r| hp_vector::sum_sq_data(r).d));
        sumsq = linq_sum(&v);
    } else {
        for r in &rows[1..] {
            sumsq += hp_vector::sum_sq_data(r).d;
        }
    }
    let u = units(m);
    Ok(RealValue::new(sumsq, u.and_then(|u| u.pow(2.0))))
}

/// `Srss()`
pub(super) fn srss(m: &Matrix) -> CResult<RealValue> {
    Ok(RealValue::new(sum_sq(m)?.d.sqrt(), units(m)))
}

/// `Average()`
pub(super) fn average(m: &Matrix) -> CResult<RealValue> {
    let n = m.row_count() * m.col_count();
    if n == 0 {
        return Ok(RealValue::ZERO);
    }
    Ok(RealValue::new(sum(m)?.d / n as f64, units(m)))
}

/// `Product()`
pub(super) fn product(m: &Matrix) -> CResult<RealValue> {
    if m.mtype() == MatrixType::Symmetric {
        return hp_symmetric_matrix::product(m);
    }
    let n = m.row_count() * m.col_count();
    let u = units(m).and_then(|u| u.pow(n as f32));
    if !m.is_structurally_consistent_type() {
        return Ok(RealValue::new(0.0, u));
    }
    let rows = rows_data(m);
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let mut product = hp_vector::product_data(&rows[0]).d;
    if rows.len() > PARALLEL_THRESHOLD {
        let mut v = vec![product];
        v.extend(rows[1..].iter().map(|r| hp_vector::product_data(r).d));
        let len = v.len();
        product = hp_vector::product_data(&HpData {
            values: Some(v),
            size: len,
            capacity: 0,
            length: len,
            units: None,
        })
        .d;
    } else {
        for r in &rows[1..] {
            product *= hp_vector::product_data(r).d;
        }
    }
    Ok(RealValue::new(product, u))
}

/// `Mean()`
pub(super) fn mean(m: &Matrix) -> CResult<RealValue> {
    let n = m.row_count() * m.col_count();
    if n == 0 {
        return Ok(RealValue::ZERO);
    }
    Ok(RealValue::new(product(m)?.d.powf(1.0 / n as f64), units(m)))
}

/// `And()`
pub fn and_all(m: &Matrix) -> RealValue {
    if !m.is_structurally_consistent_type() {
        return RealValue::ZERO;
    }
    for r in m.rows() {
        if r.hp_and().d.abs() < crate::base_types::real_value::LOGICAL_ZERO {
            return RealValue::ZERO;
        }
    }
    RealValue::ONE
}

/// `Or()`
pub fn or_all(m: &Matrix) -> RealValue {
    for r in m.rows() {
        if r.hp_or().d.abs() >= crate::base_types::real_value::LOGICAL_ZERO {
            return RealValue::ONE;
        }
    }
    RealValue::ZERO
}

/// `Xor()`
pub fn xor_all(m: &Matrix) -> CResult<RealValue> {
    let rows = m.rows();
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let lz = crate::base_types::real_value::LOGICAL_ZERO;
    let mut b = rows[0].hp_xor().d.abs() >= lz;
    for r in &rows[1..] {
        b = b != (r.hp_xor().d.abs() >= lz);
    }
    Ok(if b { RealValue::ONE } else { RealValue::ZERO })
}

/// `Gcd()`
pub fn gcd(m: &Matrix) -> CResult<RealValue> {
    let rows = m.rows();
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let v = rows[0].hp_gcd()?;
    let mut a = calculator::as_long(v.d)?;
    let u = v.units.clone();
    for r in &rows[1..] {
        let v = r.hp_gcd()?;
        let b = calculator::as_long(v.d * Unit::convert(&u, &v.units, ',')?)?;
        a = calculator::gcd_l(a, b);
    }
    Ok(RealValue::num(a as f64))
}

/// `Lcm()`
pub fn lcm(m: &Matrix) -> CResult<RealValue> {
    let rows = m.rows();
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let v = rows[0].hp_lcm()?;
    let mut a = calculator::as_long(v.d)?;
    let u = v.units.clone();
    for r in &rows[1..] {
        let v = r.hp_lcm()?;
        let b = calculator::as_long(v.d * Unit::convert(&u, &v.units, ',')?)?;
        let g = calculator::gcd_l(a, b);
        if g == 0 {
            return Err(MathParserException::unexpected("Attempted to divide by zero."));
        }
        a = a.wrapping_mul(b) / g;
    }
    Ok(RealValue::num(a as f64))
}

fn out_of_limits(m: &Matrix, dx: f64, dy: f64) -> bool {
    let (rc, cc) = (m.row_count() as f64, m.col_count() as f64);
    !dy.is_normal()
        || dy < calculator::DELTA_MINUS
        || dy > rc * calculator::DELTA_PLUS
        || !dx.is_normal()
        || dx < calculator::DELTA_MINUS
        || dx > cc * calculator::DELTA_PLUS
}

/// `Take(in RealValue x, in RealValue y)`
pub fn take(m: &Matrix, x: &RealValue, y: &RealValue) -> CResult<RealValue> {
    let dx = x.d.round();
    let dy = y.d.round();
    if out_of_limits(m, dx, dy) {
        return Ok(RealValue::NAN);
    }
    m.try_get(dy as usize - 1, dx as usize - 1)
}

/// `Line(in RealValue x, in RealValue y)` (C# converts x and y to the matrix units)
pub fn line(m: &Matrix, x: &RealValue, y: &RealValue) -> CResult<RealValue> {
    let u = units(m);
    let dx = x.d * Unit::convert(&u, &x.units, '\u{2501}')?;
    let dy = y.d * Unit::convert(&u, &y.units, '\u{2501}')?;
    if out_of_limits(m, dx, dy) {
        return Ok(RealValue::NAN);
    }
    let (rc, cc) = (m.row_count() as f64, m.col_count() as f64);
    let j = dx.floor() as usize;
    let i = dy.floor() as usize;
    let a = HpMat::from(m);
    let z11 = a.get_value(i - 1, j - 1);
    if j as f64 == dx || dx >= cc {
        if i as f64 == dy || dy >= rc {
            return Ok(RealValue::new(z11, u));
        }
        let z21 = a.get_value(i, j - 1);
        return Ok(RealValue::new(z11 + (z21 - z11) * (dy - i as f64), u));
    }
    if i as f64 == dy || dy >= rc {
        let z12 = a.get_value(i - 1, j);
        return Ok(RealValue::new(z11 + (z12 - z11) * (dx - j as f64), u));
    }
    let z12 = a.get_value(i - 1, j);
    let z21 = a.get_value(i, j - 1);
    let z22 = a.get_value(i, j);
    let z1 = z11 + (z12 - z11) * (dx - j as f64);
    let z2 = z21 + (z22 - z21) * (dx - j as f64);
    Ok(RealValue::new(z1 + (z2 - z1) * (dy - i as f64), u))
}

/// `Spline(in RealValue x, in RealValue y)`
pub fn spline(m: &Matrix, x: &RealValue, y: &RealValue) -> CResult<RealValue> {
    let u = units(m);
    let dx = x.d * Unit::convert(&u, &x.units, '~')?;
    let dy = y.d * Unit::convert(&u, &y.units, '~')?;
    if out_of_limits(m, dx, dy) {
        return Ok(RealValue::NAN);
    }
    let a = HpMat::from(m);
    let rc = a.row_count as i64;
    let j = dx.floor() as i64 - 1;
    let i = dy.floor() as i64 - 1;
    let sign = crate::base_types::complex::math_sign;
    let z0 = spline_row(&a, i, j, dx)?;
    if i as f64 == dy || dy >= rc as f64 {
        return Ok(RealValue::new(z0, u));
    }
    let z1 = spline_row(&a, i + 1, j, dx)?;
    let mut dz = z1 - z0;
    let mut aa = dz;
    let mut b = dz;
    dz = sign(dz)? as f64;
    if i > 0 {
        let z2 = spline_row(&a, i - 1, j, dx)?;
        aa = (z1 - z2) * if sign(z0 - z2)? as f64 == dz { 0.5 } else { 0.25 };
    }
    if i < rc - 2 {
        let z2 = spline_row(&a, i + 2, j, dx)?;
        b = (z2 - z0) * if sign(z2 - z1)? as f64 == dz { 0.5 } else { 0.25 };
    }
    if i == 0 {
        aa += (aa - b) / 2.0;
    }
    if i == rc - 2 {
        b += (b - aa) / 2.0;
    }
    let t = dy - i as f64 - 1.0;
    let z = z0 + ((z1 - z0) * (3.0 - 2.0 * t) * t + ((aa + b) * t - aa) * (t - 1.0)) * t;
    Ok(RealValue::new(z, u))
}

fn spline_row(a: &HpMat, i: i64, j: i64, dx: f64) -> CResult<f64> {
    let cc = a.col_count as i64;
    let at = |i: i64, j: i64| -> CResult<f64> {
        if i < 0 || j < 0 || i as usize >= a.row_count.max(1) && a.mtype != MatrixType::Column {
            return Err(crate::vector::index_was_outside());
        }
        Ok(a.get_value(i as usize, j as usize))
    };
    let y0 = at(i, j)?;
    if j as f64 == dx || dx >= cc as f64 {
        return Ok(y0);
    }
    let y1 = at(i, j + 1)?;
    let mut dy = y1 - y0;
    let mut aa = dy;
    let mut b = dy;
    let sign = crate::base_types::complex::math_sign;
    dy = sign(dy)? as f64;
    if j > 0 {
        let y2 = at(i, j - 1)?;
        aa = (y1 - y2) * if sign(y0 - y2)? as f64 == dy { 0.5 } else { 0.25 };
    }
    if j < cc - 2 {
        let y2 = at(i, j + 2)?;
        b = (y2 - y0) * if sign(y2 - y1)? as f64 == dy { 0.5 } else { 0.25 };
    }
    if j == 0 {
        aa += (aa - b) / 2.0;
    }
    if j == cc - 2 {
        b += (b - aa) / 2.0;
    }
    let t = dx - j as f64 - 1.0;
    Ok(y0 + ((y1 - y0) * (3.0 - 2.0 * t) * t + ((aa + b) * t - aa) * (t - 1.0)) * t)
}

/// `Transpose()` (virtual, with the overrides of the HP subclasses)
pub(super) fn transpose(m: &Matrix) -> CResult<Matrix> {
    let u = units(m);
    match m.mtype() {
        MatrixType::Column => {
            let c = new(1, m.row_count(), u)?;
            c.set_row_ref(0, m.row_ref(0).hp_copy());
            Ok(c)
        }
        MatrixType::Diagonal => {
            let c = new_diagonal(m.row_count(), u)?;
            c.set_row_ref(0, m.row_ref(0).hp_copy());
            Ok(c)
        }
        MatrixType::LowerTriangular => hp_triangular_matrix::lower_transpose(m),
        MatrixType::UpperTriangular => hp_triangular_matrix::upper_transpose(m),
        MatrixType::Symmetric => Ok(hp_symmetric_matrix::raw_copy(m)),
        MatrixType::Full => {
            if m.row_count() == 1 {
                return hp_column_from_vector(&m.row_ref(0));
            }
            let (rc, cc) = (m.row_count(), m.col_count());
            let mm = new(cc, rc, u)?;
            for i in (0..cc).rev() {
                mm.set_row_ref(i, col(m, i as i64 + 1)?);
            }
            Ok(mm)
        }
    }
}

/// `Trace()`
pub(super) fn trace(m: &Matrix) -> CResult<RealValue> {
    let n = m.row_count();
    if n != m.col_count() {
        return Err(exceptions::MatrixNotSquare());
    }
    let a = HpMat::from(m);
    if a.rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let mut trace = a.get_value(0, 0);
    for i in 1..n {
        trace += a.get_value(i, i);
    }
    Ok(RealValue::new(trace, units(m)))
}

/// `L1Norm()` (with the HpColumnMatrix and HpDiagonalMatrix overrides)
pub(super) fn l1_norm(m: &Matrix) -> CResult<RealValue> {
    match m.mtype() {
        MatrixType::Column => return m.row_ref(0).hp_l1_norm(),
        MatrixType::Diagonal => return m.row_ref(0).hp_inf_norm(),
        _ => {}
    }
    let cc = m.col_count();
    let mut norm = col(m, 1)?.hp_l1_norm()?.d;
    if cc > PARALLEL_THRESHOLD {
        let mut v = vec![norm];
        for j in 1..cc {
            v.push(col(m, j as i64 + 1)?.hp_l1_norm()?.d);
        }
        norm = linq_max(&v);
    } else {
        for j in 1..cc {
            let col_norm = col(m, j as i64 + 1)?.hp_l1_norm()?.d;
            if col_norm > norm {
                norm = col_norm;
            }
        }
    }
    Ok(RealValue::new(norm, units(m)))
}

/// `InfNorm()` (with the overrides of the HP subclasses)
pub(super) fn inf_norm(m: &Matrix) -> CResult<RealValue> {
    match m.mtype() {
        MatrixType::Column | MatrixType::Diagonal => return m.row_ref(0).hp_inf_norm(),
        MatrixType::Symmetric => return hp_symmetric_matrix::inf_norm(m),
        _ => {}
    }
    let rows = rows_data(m);
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let mut norm = hp_vector::l1_norm_data(&rows[0]).d;
    if rows.len() > PARALLEL_THRESHOLD {
        let mut v = vec![norm];
        v.extend(rows[1..].iter().map(|r| hp_vector::l1_norm_data(r).d));
        norm = linq_max(&v);
    } else {
        for r in &rows[1..] {
            let row_norm = hp_vector::l1_norm_data(r).d;
            if row_norm > norm {
                norm = row_norm;
            }
        }
    }
    Ok(RealValue::new(norm, units(m)))
}

fn is_target(t: MatrixType, i: i64, j: i64) -> bool {
    match t {
        MatrixType::Full | MatrixType::Column => true,
        MatrixType::Symmetric | MatrixType::UpperTriangular => j >= i,
        MatrixType::LowerTriangular => j <= i,
        MatrixType::Diagonal => i == j,
    }
}

/// `CopyTo(HpMatrix target, int i, int j)`
pub(super) fn copy_to(m: &Matrix, target: &Matrix, i: i64, j: i64) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (i, j) = (i - 1, j - 1);
    let mr = (a.row_count as i64).min(target.row_count() as i64 - i);
    let nr = (a.col_count as i64).min(target.col_count() as i64 - j);
    let d = Unit::convert(&units(target), &a.units, ',')?;
    let tt = target.mtype();
    for ii in 0..mr {
        let i1 = i + ii;
        for jj in 0..nr {
            let j1 = j + jj;
            if is_target(tt, i1, j1) {
                set_value(
                    target,
                    a.get_value(ii as usize, jj as usize) * d,
                    i1 as usize,
                    j1 as usize,
                )?;
            }
        }
    }
    Ok(target.clone())
}

/// `AddTo(HpMatrix target, int i, int j)`
pub(super) fn add_to(m: &Matrix, target: &Matrix, i: i64, j: i64) -> CResult<Matrix> {
    let a = HpMat::from(m);
    let (i, j) = (i - 1, j - 1);
    let mr = (a.row_count as i64).min(target.row_count() as i64 - i);
    let nr = (a.col_count as i64).min(target.col_count() as i64 - j);
    let d = Unit::convert(&units(target), &a.units, ',')?;
    let tt = target.mtype();
    for ii in 0..mr {
        let i1 = i + ii;
        for jj in 0..nr {
            let j1 = j + jj;
            if is_target(tt, i1, j1) {
                let v = get_value(target, i1 as usize, j1 as usize);
                let w = a.get_value(ii as usize, jj as usize) * d;
                if v == 0.0 {
                    set_value(target, w, i1 as usize, j1 as usize)?;
                } else {
                    set_value(target, v + w, i1 as usize, j1 as usize)?;
                }
            }
        }
    }
    Ok(target.clone())
}

/// `Diagonal()`
pub(super) fn diagonal(m: &Matrix) -> CResult<Vector> {
    let a = HpMat::from(m);
    let n = a.row_count.min(a.col_count);
    let mut v = HpData::new(n, n, a.units.clone());
    {
        let vals = v.values.as_mut().unwrap();
        for (i, x) in vals.iter_mut().enumerate() {
            *x = a.get_value(i, i);
        }
    }
    Ok(hpd(v))
}

/// `HpMatrix.CreateFromCols(HpVector[] cols, int m)`
pub fn create_from_cols(cols: &[Vector], m: usize) -> CResult<Matrix> {
    let n = cols.len();
    let u = cols.first().ok_or_else(crate::vector::index_was_outside)?.hp_units();
    let mm = new(m, n, u.clone())?;
    for j in (0..n).rev() {
        let d = Unit::convert(&u, &cols[j].hp_units(), ',')?;
        if d == 1.0 {
            set_col(&mm, j, &cols[j])?;
        } else {
            set_col(&mm, j, &hp_vector::mul_vr(&cols[j], &RealValue::num(d))?)?;
        }
    }
    Ok(mm)
}

/// `HpMatrix.CreateFromRows(HpVector[] rows, int n)`
pub fn create_from_rows(rows: &[Vector], n: usize) -> CResult<Matrix> {
    let m = rows.len();
    let u = rows.first().ok_or_else(crate::vector::index_was_outside)?.hp_units();
    let mm = new(m, n, u.clone())?;
    for i in (0..m).rev() {
        let d = Unit::convert(&u, &rows[i].hp_units(), ',')?;
        if d == 1.0 {
            set_row(&mm, i, &rows[i])?;
        } else {
            set_row(&mm, i, &hp_vector::mul_vr(&rows[i], &RealValue::num(d))?)?;
        }
    }
    Ok(mm)
}

/// `AsJaggedArray()`: the stored part of each row.
pub(crate) fn as_jagged_array(a: &HpMat) -> Vec<Vec<f64>> {
    a.rows.iter().map(|r| r.raw_all()[..r.size].to_vec()).collect()
}

/// `AsJaggedArrayTransposed()`
pub(crate) fn as_jagged_array_transposed(a: &HpMat) -> CResult<Vec<Vec<f64>>> {
    let m = a.rows.first().ok_or_else(crate::vector::index_was_outside)?.size;
    let n = a.rows.len();
    let mut out = vec![vec![0.0; n]; m];
    for (i, row) in a.rows.iter().enumerate() {
        let raw = row.raw_all();
        for j in 0..m {
            out[j][i] = *raw.get(j).ok_or_else(crate::vector::index_was_outside)?;
        }
    }
    Ok(out)
}

/// `FromJaggedArrayTransposed(double[][] jaggedArray, Unit units)`
pub(crate) fn from_jagged_array_transposed(x: &[Vec<f64>], units: OptUnit) -> CResult<Matrix> {
    let m = x.first().ok_or_else(crate::vector::index_was_outside)?.len();
    let n = x.len();
    let mut mm = HpMat::empty(MatrixType::Full, m, n, units);
    for i in 0..m {
        for j in 0..n {
            mm.rows[i].set_value(x[j][i], j);
        }
    }
    Ok(mm.into_matrix())
}

/// `FFT(bool inverse)`
pub fn fft(m: &Matrix, inverse: bool) -> CResult<Matrix> {
    let (rc, cc) = (m.row_count(), m.col_count());
    if rc == 0 || cc == 0 {
        return Ok(m.clone());
    }
    let power = (cc as f64).log2().ceil();
    let n = 2f64.powf(power) as usize;
    let mut re = vec![0.0; n];
    let mut im = vec![0.0; n];
    if rc == 1 || rc == 2 {
        let row = hp_vector::snap(&m.row_ref(0));
        re[..row.size].copy_from_slice(&row.raw_all()[..row.size]);
        if rc == 2 {
            let row = hp_vector::snap(&m.row_ref(1));
            im[..row.size].copy_from_slice(&row.raw_all()[..row.size]);
        }
    } else {
        return Err(exceptions::MatrixOneOrTwoRows());
    }
    hp_fft::transform(&mut re, &mut im, if inverse { -1 } else { 1 });
    let u = units(m);
    let result = new(2, n, u.clone())?;
    result.set_row_ref(0, hp_vector::from_doubles(re, u.clone())?);
    result.set_row_ref(1, hp_vector::from_doubles(im, u)?);
    if inverse {
        scale(&result, 1.0 / n as f64);
    }
    Ok(result)
}

// ----- operators -------------------------------------------------------------------------------

/// `c = a.Clone(); c._units = units;` followed by `c._hpRows[i] = f(a._hpRows[i])`
fn with_rows(a: &Matrix, units: OptUnit, f: &dyn Fn(&Vector, usize) -> CResult<Vector>) -> CResult<Matrix> {
    let c = clone_m(a);
    c.data_mut().units = units;
    let rows = a.rows();
    for i in (0..rows.len()).rev() {
        c.set_row_ref(i, f(&rows[i], i)?);
    }
    Ok(c)
}

/// `c = new HpMatrix(m, n, units)` filled element by element with `SetValue`.
fn elementwise(rc: usize, cc: usize, units: OptUnit, f: &dyn Fn(usize, usize) -> f64) -> Matrix {
    let mut c = HpMat::empty(MatrixType::Full, rc, cc, units);
    for i in (0..rc).rev() {
        for j in (0..cc).rev() {
            let v = f(i, j);
            c.rows[i].set_value(v, j);
        }
    }
    c.into_matrix()
}

/// `-HpMatrix`
pub fn neg(a: &Matrix) -> Matrix {
    let c = clone_m(a);
    let rows = a.rows();
    for i in (0..rows.len()).rev() {
        c.set_row_ref(i, hp_vector::neg(&rows[i]));
    }
    c
}

type VV = fn(&Vector, &Vector) -> CResult<Vector>;
type VR = fn(&Vector, &RealValue) -> CResult<Vector>;
type RV = fn(&RealValue, &Vector) -> CResult<Vector>;

/// The arithmetic operators of two HP matrices.
fn arith_mm(
    a: &Matrix,
    b: &Matrix,
    by_rows: bool,
    units: OptUnit,
    vf: VV,
    ef: &dyn Fn(f64, f64) -> f64,
) -> CResult<Matrix> {
    if by_rows {
        let br = b.rows();
        return with_rows(a, units, &|r, i| vf(r, &br[i]));
    }
    let (aa, bb) = (HpMat::from(a), HpMat::from(b));
    Ok(elementwise(aa.row_count, aa.col_count, units, &|i, j| {
        ef(aa.get_value(i, j), bb.get_value(i, j))
    }))
}

pub fn sub(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    check_bounds(a, b)?;
    if a.mtype() == b.mtype() {
        return arith_mm(a, b, true, units(a), hp_vector::sub, &|_, _| 0.0);
    }
    let d = Unit::convert(&units(a), &units(b), '-')?;
    arith_mm(a, b, false, units(a), hp_vector::sub, &|x, y| x - y * d)
}

pub fn add(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    check_bounds(a, b)?;
    if a.mtype() == b.mtype() {
        return arith_mm(a, b, true, units(a), hp_vector::add, &|_, _| 0.0);
    }
    let d = Unit::convert(&units(a), &units(b), '+')?;
    arith_mm(a, b, false, units(a), hp_vector::add, &|x, y| x + y * d)
}

pub fn div(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    check_bounds(a, b)?;
    let (unit, d) = Unit::divide(&units(a), &units(b), true);
    if a.mtype() == b.mtype() && a.is_structurally_consistent_type() {
        return arith_mm(a, b, true, unit, hp_vector::div, &|_, _| 0.0);
    }
    arith_mm(a, b, false, unit, hp_vector::div, &|x, y| x / y * d)
}

pub fn rem(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    if units(b).is_some() {
        return Err(exceptions::CannotEvaluateRemainder(
            &Unit::get_text_of(&units(a)),
            &Unit::get_text_of(&units(b)),
        ));
    }
    check_bounds(a, b)?;
    if a.mtype() == b.mtype() && a.is_structurally_consistent_type() {
        return arith_mm(a, b, true, units(a), hp_vector::rem, &|_, _| 0.0);
    }
    arith_mm(a, b, false, units(a), hp_vector::rem, &|x, y| x % y)
}

/// The relational operators of two HP matrices.
fn cmp_mm(a: &Matrix, b: &Matrix, need_consistent: bool, op: char, vf: VV, ef: fn(f64, f64) -> f64) -> CResult<Matrix> {
    check_bounds(a, b)?;
    if a.mtype() == b.mtype() && (!need_consistent || a.is_structurally_consistent_type()) {
        let br = b.rows();
        return with_rows(a, None, &|r, i| vf(r, &br[i]));
    }
    let d = Unit::convert(&units(a), &units(b), op)?;
    let (aa, bb) = (HpMat::from(a), HpMat::from(b));
    Ok(elementwise(aa.row_count, aa.col_count, None, &|i, j| {
        ef(aa.get_value(i, j), bb.get_value(i, j) * d)
    }))
}

pub fn eq(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    cmp_mm(a, b, true, '\u{2261}', hp_vector::eq, hp_vector::is_equal)
}
pub fn ne(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    cmp_mm(a, b, false, '\u{2260}', hp_vector::ne, hp_vector::is_not_equal)
}
pub fn lt(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    cmp_mm(a, b, false, '<', hp_vector::lt, hp_vector::is_less_than)
}
pub fn gt(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    cmp_mm(a, b, false, '>', hp_vector::gt, hp_vector::is_greater_than)
}
pub fn le(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    cmp_mm(a, b, true, '<', hp_vector::le, hp_vector::is_less_than_or_equal)
}
pub fn ge(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    cmp_mm(a, b, true, '>', hp_vector::ge, hp_vector::is_greater_than_or_equal)
}

fn logic_and(a: f64, b: f64) -> f64 {
    let lz = crate::base_types::real_value::LOGICAL_ZERO;
    if a.abs() < lz || b.abs() < lz {
        0.0
    } else {
        1.0
    }
}
fn logic_or(a: f64, b: f64) -> f64 {
    let lz = crate::base_types::real_value::LOGICAL_ZERO;
    if a.abs() >= lz || b.abs() >= lz {
        1.0
    } else {
        0.0
    }
}
fn logic_xor(a: f64, b: f64) -> f64 {
    let lz = crate::base_types::real_value::LOGICAL_ZERO;
    if (a.abs() >= lz) != (b.abs() >= lz) {
        1.0
    } else {
        0.0
    }
}

fn logic_mm(a: &Matrix, b: &Matrix, vf: fn(&Vector, &Vector) -> Vector, ef: fn(f64, f64) -> f64) -> CResult<Matrix> {
    check_bounds(a, b)?;
    if a.mtype() == b.mtype() {
        let br = b.rows();
        return with_rows(a, None, &|r, i| Ok(vf(r, &br[i])));
    }
    let (aa, bb) = (HpMat::from(a), HpMat::from(b));
    Ok(elementwise(aa.row_count, aa.col_count, None, &|i, j| {
        ef(aa.get_value(i, j), bb.get_value(i, j))
    }))
}

pub fn and(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    logic_mm(a, b, hp_vector::and, logic_and)
}
pub fn or(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    logic_mm(a, b, hp_vector::or, logic_or)
}
pub fn xor(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    logic_mm(a, b, hp_vector::xor, logic_xor)
}

fn elem_mr(a: &Matrix, units: OptUnit, ef: &dyn Fn(f64) -> f64) -> Matrix {
    let aa = HpMat::from(a);
    elementwise(aa.row_count, aa.col_count, units, &|i, j| ef(aa.get_value(i, j)))
}

pub fn sub_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    if a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO) {
        return with_rows(a, units(a), &|r, _| hp_vector::sub_vr(r, b));
    }
    let db = b.d * Unit::convert(&units(a), &b.units, '-')?;
    Ok(elem_mr(a, units(a), &|x| x - db))
}

pub fn add_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    if a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO) {
        return with_rows(a, units(a), &|r, _| hp_vector::add_vr(r, b));
    }
    let db = b.d * Unit::convert(&units(a), &b.units, '-')?;
    Ok(elem_mr(a, units(a), &|x| x + db))
}

pub fn mul_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let (unit, mut db) = Unit::multiply(&units(a), &b.units, true);
    db *= b.d;
    let c = clone_m(a);
    c.data_mut().units = unit;
    if db != 0.0 {
        let rows = a.rows();
        for i in (0..rows.len()).rev() {
            let r = hp_vector::snap(&rows[i]);
            c.set_row_ref(i, hpd(hp_vector::mul_d(&r, db)));
        }
    }
    Ok(c)
}

pub fn div_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let (u, mut db) = Unit::divide(&units(a), &b.units, true);
    if a.is_structurally_consistent_type() && !b.equals(&RealValue::ZERO) {
        return with_rows(a, u, &|r, _| hp_vector::div_vr(r, b));
    }
    db /= b.d;
    Ok(elem_mr(a, u, &|x| x * db))
}

pub fn rem_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    if b.equals(&RealValue::ZERO) {
        let m = new(a.row_count(), a.col_count(), None)?;
        return fill(&m, &RealValue::NAN);
    }
    if b.units.is_some() {
        return Err(exceptions::CannotEvaluateRemainder(
            &Unit::get_text_of(&units(a)),
            &Unit::get_text_of(&b.units),
        ));
    }
    if a.is_structurally_consistent_type() {
        return with_rows(a, units(a), &|r, _| hp_vector::rem_vr(r, b));
    }
    let db = b.d;
    Ok(elem_mr(a, units(a), &|x| x % db))
}

fn cmp_mr(a: &Matrix, b: &RealValue, rows_cond: bool, op: char, vf: VR, ef: fn(f64, f64) -> f64) -> CResult<Matrix> {
    if rows_cond {
        return with_rows(a, None, &|r, _| vf(r, b));
    }
    let db = Unit::convert(&units(a), &b.units, op)? * b.d;
    Ok(elem_mr(a, None, &|x| ef(x, db)))
}

pub fn eq_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let c = a.is_structurally_consistent_type();
    cmp_mr(a, b, c, '\u{2261}', hp_vector::eq_vr, hp_vector::is_equal)
}
pub fn ne_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let c = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
    cmp_mr(a, b, c, '\u{2260}', hp_vector::ne_vr, hp_vector::is_not_equal)
}
pub fn lt_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let c = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
    cmp_mr(a, b, c, '<', hp_vector::lt_vr, hp_vector::is_less_than)
}
pub fn gt_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let c = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
    cmp_mr(a, b, c, '>', hp_vector::gt_vr, hp_vector::is_greater_than)
}
pub fn le_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let c = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
    cmp_mr(a, b, c, '<', hp_vector::le_vr, hp_vector::is_less_than_or_equal)
}
pub fn ge_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    let c = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
    cmp_mr(a, b, c, '>', hp_vector::ge_vr, hp_vector::is_greater_than_or_equal)
}

fn logic_mr(
    a: &Matrix,
    b: &RealValue,
    vf: fn(&Vector, &RealValue) -> Vector,
    ef: fn(f64, f64) -> f64,
) -> CResult<Matrix> {
    if a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO) {
        return with_rows(a, None, &|r, _| Ok(vf(r, b)));
    }
    let db = b.d;
    Ok(elem_mr(a, None, &|x| ef(x, db)))
}

pub fn and_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    logic_mr(a, b, hp_vector::and_vr, logic_and)
}
pub fn or_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    logic_mr(a, b, hp_vector::or_vr, logic_or)
}
pub fn xor_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    logic_mr(a, b, hp_vector::xor_vr, logic_xor)
}

pub fn sub_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    if b.is_structurally_consistent_type() || a.equals(&RealValue::ZERO) {
        return with_rows(b, a.units.clone(), &|r, _| hp_vector::sub_rv(a, r));
    }
    let da = a.d;
    let k = Unit::convert(&a.units, &units(b), '-')?;
    Ok(elem_mr(b, a.units.clone(), &|x| da - x * k))
}

pub fn div_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    let (u, mut da) = Unit::divide(&a.units, &units(b), true);
    if b.is_structurally_consistent_type() {
        return with_rows(b, u, &|r, _| hp_vector::div_rv(a, r));
    }
    da *= a.d;
    Ok(elem_mr(b, u, &|x| da / x))
}

pub fn rem_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    if units(b).is_some() {
        return Err(exceptions::CannotEvaluateRemainder(
            &Unit::get_text_of(&a.units),
            &Unit::get_text_of(&units(b)),
        ));
    }
    if b.is_structurally_consistent_type() {
        return with_rows(b, units(b), &|r, _| hp_vector::rem_rv(a, r));
    }
    let da = a.d;
    Ok(elem_mr(b, a.units.clone(), &|x| da % x))
}

fn cmp_rm(a: &RealValue, b: &Matrix, op: char, vf: RV, ef: fn(f64, f64) -> f64) -> CResult<Matrix> {
    if b.is_structurally_consistent_type() || a.equals(&RealValue::ZERO) {
        return with_rows(b, None, &|r, _| vf(a, r));
    }
    let da = a.d;
    let k = Unit::convert(&a.units, &units(b), op)?;
    Ok(elem_mr(b, None, &|x| ef(da, x * k)))
}

pub fn lt_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    cmp_rm(a, b, '<', hp_vector::lt_rv, hp_vector::is_less_than)
}
pub fn gt_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    cmp_rm(a, b, '>', hp_vector::gt_rv, hp_vector::is_greater_than)
}
pub fn le_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    cmp_rm(a, b, '<', hp_vector::le_rv, hp_vector::is_less_than_or_equal)
}
pub fn ge_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    cmp_rm(a, b, '>', hp_vector::ge_rv, hp_vector::is_greater_than_or_equal)
}

/// `HpMatrix * HpMatrix`
pub fn mul(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    if a.col_count() != b.row_count() {
        return Err(exceptions::MatrixDimensions());
    }
    let (ta, tb) = (a.mtype(), b.mtype());
    if tb == MatrixType::Column {
        return match ta {
            MatrixType::Symmetric => hp_column_matrix::mul_symmetric(a, b),
            MatrixType::UpperTriangular => hp_column_matrix::mul_upper(a, b),
            _ => hp_column_matrix::mul(a, b),
        };
    }
    if ta == MatrixType::Diagonal {
        return match tb {
            MatrixType::Diagonal => hp_diagonal_matrix::mul_dd(a, b),
            MatrixType::LowerTriangular | MatrixType::UpperTriangular => hp_diagonal_matrix::mul_dt(a, b),
            MatrixType::Symmetric => hp_diagonal_matrix::mul_ds(a, b),
            _ => hp_diagonal_matrix::mul_dm(a, b),
        };
    } else if tb == MatrixType::Diagonal {
        return match ta {
            MatrixType::LowerTriangular => hp_diagonal_matrix::mul_ld(a, b),
            MatrixType::UpperTriangular => hp_diagonal_matrix::mul_ud(a, b),
            MatrixType::Symmetric => hp_diagonal_matrix::mul_sd(a, b),
            _ => hp_diagonal_matrix::mul_md(a, b),
        };
    }
    let m = a.row_count();
    if m >= 10 && m == a.col_count() && b.row_count() == b.col_count() {
        return hp_matmul_winograd::multiply(a, b);
    }
    let (unit, d) = Unit::multiply(&units(a), &units(b), true);
    let aa = HpMat::from(a);
    let bb = HpMat::from(b);
    let na = aa.col_count;
    let nb = bb.col_count;
    let mut c = HpMat::empty(MatrixType::Full, m, nb, unit.clone());
    if ta == MatrixType::Full || ta == MatrixType::LowerTriangular {
        if tb == MatrixType::Full || tb == MatrixType::LowerTriangular {
            for i in 0..m {
                c.rows[i] = HpData::new(nb, nb, unit.clone());
            }
            for i in 0..m {
                let a_i = &aa.rows[i];
                let ar = a_i.raw_all();
                let sc = c.rows[i].values.as_mut().unwrap();
                for k in 0..a_i.size {
                    vectorized::multiply_add(bb.rows[k].raw_all(), ar[k], sc);
                }
            }
            if d != 1.0 {
                for r in c.rows.iter_mut() {
                    r.scale(d);
                }
            }
        } else {
            for i in (0..m).rev() {
                let a_i = &aa.rows[i];
                let ar = a_i.raw_all();
                if a_i.size > 0 {
                    let last = a_i.size - 1;
                    for j in (0..nb).rev() {
                        let mut c_ij = ar[last] * bb.get_value(last, j) * d;
                        for k in (0..last).rev() {
                            c_ij += ar[k] * bb.get_value(k, j) * d;
                        }
                        c.rows[i].set_value(c_ij, j);
                    }
                }
            }
        }
    } else {
        if na == 0 {
            return Err(crate::vector::index_was_outside());
        }
        for i in (0..m).rev() {
            for j in (0..nb).rev() {
                let mut c_ij = aa.get_value(i, na - 1) * bb.get_value(na - 1, j) * d;
                for k in (0..na - 1).rev() {
                    c_ij += aa.get_value(i, k) * bb.get_value(k, j) * d;
                }
                c.rows[i].set_value(c_ij, j);
            }
        }
    }
    Ok(c.into_matrix())
}

// ----- EvaluateOperator / EvaluateFunction -----------------------------------------------------

/// `HpMatrix.EvaluateOperator(op, HpMatrix a, HpMatrix b, index)`
pub fn evaluate_operator(op: RealOp, a: &Matrix, b: &Matrix, index: i64) -> CResult<Matrix> {
    match calculator::get_operator_symbol(index) {
        '/' | '\u{00F7}' => div(a, b),
        '\u{29BC}' => rem(a, b),
        '*' => mul(a, b),
        '-' => sub(a, b),
        '+' => add(a, b),
        '<' => lt(a, b),
        '>' => gt(a, b),
        '\u{2264}' => le(a, b),
        '\u{2265}' => ge(a, b),
        '\u{2261}' => eq(a, b),
        '\u{2260}' => ne(a, b),
        '\u{2227}' => and(a, b),
        '\u{2228}' => or(a, b),
        '\u{2295}' => xor(a, b),
        _ => evaluate_operator2(op, a, b, index),
    }
}

/// `HpMatrix.EvaluateOperator(op, HpMatrix a, RealValue b, index)`
pub fn evaluate_operator_mr(op: RealOp, a: &Matrix, b: &RealValue, index: i64) -> CResult<Matrix> {
    match calculator::get_operator_symbol(index) {
        '/' | '\u{00F7}' => div_mr(a, b),
        '\u{29BC}' => rem_mr(a, b),
        '*' => mul_mr(a, b),
        '-' => sub_mr(a, b),
        '+' => add_mr(a, b),
        '<' => lt_mr(a, b),
        '>' => gt_mr(a, b),
        '\u{2264}' => le_mr(a, b),
        '\u{2265}' => ge_mr(a, b),
        '\u{2261}' => eq_mr(a, b),
        '\u{2260}' => ne_mr(a, b),
        '\u{2227}' => and_mr(a, b),
        '\u{2228}' => or_mr(a, b),
        '\u{2295}' => xor_mr(a, b),
        _ => evaluate_operator2_mr(op, a, b, index),
    }
}

/// `HpMatrix.EvaluateOperator(op, RealValue a, HpMatrix b, index)`
pub fn evaluate_operator_rm(op: RealOp, a: &RealValue, b: &Matrix, index: i64) -> CResult<Matrix> {
    match calculator::get_operator_symbol(index) {
        '/' | '\u{00F7}' => div_rm(a, b),
        '\u{29BC}' => rem_rm(a, b),
        '*' => mul_mr(b, a),
        '-' => sub_rm(a, b),
        '+' => add_mr(b, a),
        '<' => lt_rm(a, b),
        '>' => gt_rm(a, b),
        '\u{2264}' => le_rm(a, b),
        '\u{2265}' => ge_rm(a, b),
        '\u{2261}' => eq_mr(b, a),
        '\u{2260}' => ne_mr(b, a),
        '\u{2227}' => and_mr(b, a),
        '\u{2228}' => or_mr(b, a),
        '\u{2295}' => xor_mr(b, a),
        _ => evaluate_operator2_rm(op, a, b, index),
    }
}

fn evaluate_operator2(op: RealOp, a: &Matrix, b: &Matrix, index: i64) -> CResult<Matrix> {
    check_bounds(a, b)?;
    let units = op.apply(&a.try_get(0, 0)?, &b.try_get(0, 0)?)?.units;
    if a.mtype() == b.mtype() && (a.is_structurally_consistent_type() || calculator::is_zero_preserving_operator(index))
    {
        let br = b.rows();
        return with_rows(a, units, &|r, i| hp_vector::evaluate_operator(op, r, &br[i], index));
    }
    let (rc, cc) = (a.row_count(), a.col_count());
    let mut c = HpMat::empty(MatrixType::Full, rc, cc, units);
    for i in (0..rc).rev() {
        for j in (0..cc).rev() {
            let v = op.apply(&a.get(i, j), &b.get(i, j))?.d;
            c.rows[i].set_value(v, j);
        }
    }
    Ok(c.into_matrix())
}

fn evaluate_operator2_mr(op: RealOp, a: &Matrix, b: &RealValue, index: i64) -> CResult<Matrix> {
    let units = op.apply(&a.try_get(0, 0)?, b)?.units;
    // C# compares the matrix with RealValue.Zero here (always false).
    let _ = index;
    if a.is_structurally_consistent_type() {
        return with_rows(a, units, &|r, _| hp_vector::evaluate_operator_vr(op, r, b, index));
    }
    let (rc, cc) = (a.row_count(), a.col_count());
    let mut c = HpMat::empty(MatrixType::Full, rc, cc, units);
    for i in (0..rc).rev() {
        for j in (0..cc).rev() {
            let v = op.apply(&a.get(i, j), b)?.d;
            c.rows[i].set_value(v, j);
        }
    }
    Ok(c.into_matrix())
}

fn evaluate_operator2_rm(op: RealOp, a: &RealValue, b: &Matrix, index: i64) -> CResult<Matrix> {
    let units = op.apply(a, &b.try_get(0, 0)?)?.units;
    if b.is_structurally_consistent_type()
        || calculator::is_zero_preserving_operator(index) && a.equals(&RealValue::ZERO)
    {
        return with_rows(b, units, &|r, _| hp_vector::evaluate_operator_rv(op, a, r, index));
    }
    let (rc, cc) = (b.row_count(), b.col_count());
    let mut c = HpMat::empty(MatrixType::Full, rc, cc, units);
    for i in (0..rc).rev() {
        for j in (0..cc).rev() {
            let v = op.apply(a, &b.get(i, j))?.d;
            c.rows[i].set_value(v, j);
        }
    }
    Ok(c.into_matrix())
}

/// `HpMatrix.EvaluateFunction(f, HpMatrix a, index)`
pub fn evaluate_function(f: RealFn, a: &Matrix, index: i64) -> CResult<Matrix> {
    let units = f.apply(&a.try_get(0, 0)?)?.units;
    if a.is_structurally_consistent_type() || calculator::is_zero_preserving_function(index) {
        return with_rows(a, units, &|r, _| hp_vector::evaluate_function(f, r));
    }
    let (rc, cc) = (a.row_count(), a.col_count());
    let mut c = HpMat::empty(MatrixType::Full, rc, cc, units);
    for i in (0..rc).rev() {
        for j in (0..cc).rev() {
            let v = f.apply(&a.get(i, j))?.d;
            c.rows[i].set_value(v, j);
        }
    }
    Ok(c.into_matrix())
}

/// `HpMatrix.Hadamard(a, b)`
pub fn hadamard(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    check_bounds(a, b)?;
    let (u, d) = Unit::multiply(&units(a), &units(b), true);
    if a.mtype() == b.mtype() {
        let br = b.rows();
        return with_rows(a, u, &|r, i| hp_vector::mul(r, &br[i]));
    }
    let (aa, bb) = (HpMat::from(a), HpMat::from(b));
    let (rc, cc) = (aa.row_count, aa.col_count);
    let mut c = if aa.mtype == MatrixType::Column && bb.mtype == MatrixType::Column {
        HpMat::empty(MatrixType::Column, rc, 1, u)
    } else {
        HpMat::empty(MatrixType::Full, rc, cc, u)
    };
    for i in (0..rc).rev() {
        for j in (0..cc).rev() {
            let v = aa.get_value(i, j) * bb.get_value(i, j) * d;
            // `c._hpRows[i].SetValue(v, j)` (row i of the result)
            c.rows[i].set_value(v, j);
        }
    }
    Ok(c.into_matrix())
}

/// `HpMatrix.Kronecker(a, b)`
pub fn kronecker(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    let (aa, bb) = (HpMat::from(a), HpMat::from(b));
    let (m, n) = (aa.row_count, aa.col_count);
    let (p, q) = (bb.row_count, bb.col_count);
    let (u, d) = Unit::multiply(&aa.units, &bb.units, true);
    if m * p > MAX_SIZE || n * q > MAX_SIZE {
        return Err(exceptions::MatirixSizeLimit());
    }
    let mut c = HpMat::empty(MatrixType::Full, m * p, n * q, u);
    for i in (0..m).rev() {
        let ip = i * p;
        for j in (0..n).rev() {
            let jq = j * q;
            for k in (0..p).rev() {
                let ipk = ip + k;
                for l in (0..q).rev() {
                    let v = aa.get_value(i, j) * bb.get_value(k, l) * d;
                    c.set_value(v, ipk, jq + l)?;
                }
            }
        }
    }
    Ok(c.into_matrix())
}

/// `HpMatrix.Frobenius(a, b)`
pub fn frobenius(a: &Matrix, b: &Matrix) -> CResult<RealValue> {
    check_bounds(a, b)?;
    let (ra, rb) = (rows_data(a), rows_data(b));
    let m = ra.len();
    let mut sum = 0.0;
    if m > PARALLEL_THRESHOLD {
        let v: Vec<f64> = (0..m).map(|i| hp_vector::raw_dot_product(&ra[i], &rb[i])).collect();
        sum = linq_sum(&v);
    } else {
        for i in (0..m).rev() {
            sum += hp_vector::raw_dot_product(&ra[i], &rb[i]);
        }
    }
    let (u, _) = Unit::multiply(&units(a), &units(b), true);
    Ok(RealValue::new(sum, u))
}

/// `HpMatmulWinograd.Multiply(a, b)`
pub fn matmul(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    hp_matmul_winograd::multiply(a, b)
}

/// The HpMatrix operators, dispatched from `Value::binary`. HpVector operands are
/// converted to HpColumnMatrix by the caller.
pub mod ops {
    use crate::base_types::real_value::RealValue;
    use crate::base_types::value::BinOp;
    use crate::math_parser_exception::CResult;
    use crate::matrix::Matrix;

    /// `-HpMatrix`
    pub fn neg(a: &Matrix) -> CResult<Matrix> {
        Ok(super::neg(a))
    }

    /// `HpMatrix op RealValue`
    pub fn op_mr(op: BinOp, a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        use super::*;
        match op {
            BinOp::Add => add_mr(a, b),
            BinOp::Sub => sub_mr(a, b),
            BinOp::Mul => mul_mr(a, b),
            BinOp::Div => div_mr(a, b),
            BinOp::Rem => rem_mr(a, b),
            BinOp::Lt => lt_mr(a, b),
            BinOp::Gt => gt_mr(a, b),
            BinOp::Le => le_mr(a, b),
            BinOp::Ge => ge_mr(a, b),
            BinOp::Eq => eq_mr(a, b),
            BinOp::Ne => ne_mr(a, b),
            BinOp::And => and_mr(a, b),
            BinOp::Or => or_mr(a, b),
            BinOp::Xor => xor_mr(a, b),
        }
    }

    /// `RealValue op HpMatrix` (the commuting operators are evaluated as `b op a`)
    pub fn op_rm(op: BinOp, a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        use super::*;
        match op {
            BinOp::Sub => sub_rm(a, b),
            BinOp::Div => div_rm(a, b),
            BinOp::Rem => rem_rm(a, b),
            BinOp::Lt => lt_rm(a, b),
            BinOp::Gt => gt_rm(a, b),
            BinOp::Le => le_rm(a, b),
            BinOp::Ge => ge_rm(a, b),
            _ => op_mr(op, b, a),
        }
    }

    /// `HpMatrix op HpMatrix`
    pub fn op_mm(op: BinOp, a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        use super::*;
        match op {
            BinOp::Add => add(a, b),
            BinOp::Sub => sub(a, b),
            BinOp::Mul => mul(a, b),
            BinOp::Div => div(a, b),
            BinOp::Rem => rem(a, b),
            BinOp::Lt => lt(a, b),
            BinOp::Gt => gt(a, b),
            BinOp::Le => le(a, b),
            BinOp::Ge => ge(a, b),
            BinOp::Eq => eq(a, b),
            BinOp::Ne => ne(a, b),
            BinOp::And => and(a, b),
            BinOp::Or => or(a, b),
            BinOp::Xor => xor(a, b),
        }
    }
}
