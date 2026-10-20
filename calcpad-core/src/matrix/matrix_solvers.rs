//! The numerical members of Matrix/Matrix.cs and its subclasses: LU, QR and SVD
//! decompositions, solving, inverse, determinant, condition numbers, rank, adjoint,
//! cofactor, Cholesky and LDLT, and the eigenvalue solvers of the symmetric and diagonal
//! matrices.
use super::Matrix;
use crate::base_types::real_value::RealValue;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::vector::Vector;

fn not_ported(name: &str) -> MathParserException {
    MathParserException::new(format!(
        "Matrix function \"{}\" is not supported by calcpad-rs yet.",
        name
    ))
}

impl Matrix {
    /// `Determinant()` (virtual)
    pub fn determinant(&self) -> CResult<RealValue> {
        Err(not_ported("det"))
    }
    /// `Invert()` (virtual)
    pub fn invert(&self) -> CResult<Matrix> {
        Err(not_ported("inverse"))
    }
    /// `LSolve(Vector v)` (virtual)
    pub fn lsolve(&self, _v: &Vector) -> CResult<Vector> {
        Err(not_ported("lsolve"))
    }
    /// `MSolve(Matrix M)` (virtual)
    pub fn msolve(&self, _m: &Matrix) -> CResult<Matrix> {
        Err(not_ported("msolve"))
    }
    /// `LUDecomposition(out int[] indexes)` (virtual)
    pub fn lu_decomposition(&self) -> CResult<(Matrix, Vec<i32>)> {
        Err(not_ported("lu"))
    }
    /// `QRDecomposition()` (virtual)
    pub fn qr_decomposition(&self) -> CResult<Matrix> {
        Err(not_ported("qr"))
    }
    /// `SVDDecomposition()` (virtual)
    pub fn svd_decomposition(&self) -> CResult<Matrix> {
        Err(not_ported("svd"))
    }
    /// `L2Norm()` (virtual)
    pub fn l2_norm(&self) -> CResult<RealValue> {
        Err(not_ported("mnorm_2"))
    }
    /// `Cond1()` (virtual)
    pub fn cond1(&self) -> CResult<RealValue> {
        Err(not_ported("cond_1"))
    }
    /// `Cond2()` (virtual)
    pub fn cond2(&self) -> CResult<RealValue> {
        Err(not_ported("cond_2"))
    }
    /// `CondFrob()` (virtual)
    pub fn cond_frob(&self) -> CResult<RealValue> {
        Err(not_ported("cond_e"))
    }
    /// `CondInf()` (virtual)
    pub fn cond_inf(&self) -> CResult<RealValue> {
        Err(not_ported("cond_i"))
    }
    /// `Rank()` (virtual)
    pub fn rank(&self) -> CResult<RealValue> {
        Err(not_ported("rank"))
    }
    /// `Adjoint()` (virtual)
    pub fn adjoint(&self) -> CResult<Matrix> {
        Err(not_ported("adj"))
    }
    /// `Cofactor()` (virtual)
    pub fn cofactor(&self) -> CResult<Matrix> {
        Err(not_ported("cofactor"))
    }
    /// `CholeskyDecomposition()` (SymmetricMatrix and DiagonalMatrix)
    pub fn cholesky_decomposition(&self) -> CResult<Matrix> {
        Err(not_ported("cholesky"))
    }
    /// `EigenValues(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen_values(&self, _count: i64) -> CResult<Vector> {
        Err(not_ported("eigenvals"))
    }
    /// `EigenVectors(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen_vectors(&self, _count: i64) -> CResult<Matrix> {
        Err(not_ported("eigenvecs"))
    }
    /// `Eigen(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen(&self, _count: i64) -> CResult<Matrix> {
        Err(not_ported("eigen"))
    }
    /// `ClSolve(Vector v)` (SymmetricMatrix)
    pub fn cl_solve(&self, _v: &Vector) -> CResult<Vector> {
        Err(not_ported("clsolve"))
    }
}
