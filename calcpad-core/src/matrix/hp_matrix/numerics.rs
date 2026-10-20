//! The HpMatrix overrides of the numerical matrix members (HpMatrix.cs and its subclasses,
//! HpLinearSolver, HpEigenSolver, HpSvdSolver). Called from `matrix_solvers.rs` for HP
//! matrices.
use super::hp_not_ported;
use crate::base_types::real_value::RealValue;
use crate::math_parser_exception::CResult;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// `Determinant()` (virtual)
pub fn determinant(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}

/// `Invert()` (virtual)
pub fn invert(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `LSolve(Vector v)` (virtual)
pub fn lsolve(_m: &Matrix, _v: &Vector) -> CResult<Vector> {
    Err(hp_not_ported())
}

/// `MSolve(Matrix M)` (virtual)
pub fn msolve(_m: &Matrix, _b: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `LUDecomposition(out int[] indexes)` (virtual)
pub fn lu_decomposition(_m: &Matrix) -> CResult<(Matrix, Vec<i32>)> {
    Err(hp_not_ported())
}

/// `QRDecomposition()` (virtual)
pub fn qr_decomposition(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `SVDDecomposition()` (virtual)
pub fn svd_decomposition(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `L2Norm()` (virtual)
pub fn l2_norm(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}

/// `Cond1()` (virtual)
pub fn cond1(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}

/// `Cond2()` (virtual)
pub fn cond2(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}

/// `CondFrob()` (virtual)
pub fn cond_frob(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}

/// `CondInf()` (virtual)
pub fn cond_inf(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}

/// `Rank()` (virtual)
pub fn rank(_m: &Matrix) -> CResult<RealValue> {
    Err(hp_not_ported())
}

/// `Adjoint()` (virtual)
pub fn adjoint(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `Cofactor()` (virtual)
pub fn cofactor(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `CholeskyDecomposition()` (SymmetricMatrix and DiagonalMatrix)
pub fn cholesky_decomposition(_m: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `EigenValues(int count)` (SymmetricMatrix and DiagonalMatrix)
pub fn eigen_values(_m: &Matrix, _count: i64, _tol: f64) -> CResult<Vector> {
    Err(hp_not_ported())
}

/// `EigenVectors(int count)` (SymmetricMatrix and DiagonalMatrix)
pub fn eigen_vectors(_m: &Matrix, _count: i64, _tol: f64) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `Eigen(int count)` (SymmetricMatrix and DiagonalMatrix)
pub fn eigen(_m: &Matrix, _count: i64, _tol: f64) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `ClSolve(Vector v)` (SymmetricMatrix)
pub fn cl_solve(_m: &Matrix, _v: &Vector) -> CResult<Vector> {
    Err(hp_not_ported())
}
/// `CmSolve(HpMatrix M)` (HpSymmetricMatrix)
pub fn cm_solve(_m: &Matrix, _b: &Matrix) -> CResult<Matrix> {
    Err(hp_not_ported())
}

/// `SlSolve(HpVector b, double tol)` (HpSymmetricMatrix, iterative)
pub fn sl_solve(_m: &Matrix, _b: &Vector, _tol: f64) -> CResult<Vector> {
    Err(hp_not_ported())
}

/// `SmSolve(HpMatrix B, double tol)` (HpSymmetricMatrix, iterative)
pub fn sm_solve(_m: &Matrix, _b: &Matrix, _tol: f64) -> CResult<Matrix> {
    Err(hp_not_ported())
}
