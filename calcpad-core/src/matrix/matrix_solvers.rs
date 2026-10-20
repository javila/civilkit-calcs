//! The numerical members of Matrix/Matrix.cs and its subclasses: LU, QR and SVD
//! decompositions, solving, inverse, determinant, condition numbers, rank, adjoint,
//! cofactor, Cholesky and LDLT, and the eigenvalue solvers of the symmetric and diagonal
//! matrices.
use super::{hp_matrix, Matrix};
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
        if self.is_hp() {
            return hp_matrix::numerics::determinant(self);
        }
        Err(not_ported("det"))
    }
    /// `Invert()` (virtual)
    pub fn invert(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::invert(self);
        }
        Err(not_ported("inverse"))
    }
    /// `LSolve(Vector v)` (virtual)
    pub fn lsolve(&self, _v: &Vector) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::numerics::lsolve(self, _v);
        }
        Err(not_ported("lsolve"))
    }
    /// `MSolve(Matrix M)` (virtual)
    pub fn msolve(&self, _m: &Matrix) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::msolve(self, _m);
        }
        Err(not_ported("msolve"))
    }
    /// `LUDecomposition(out int[] indexes)` (virtual)
    pub fn lu_decomposition(&self) -> CResult<(Matrix, Vec<i32>)> {
        if self.is_hp() {
            return hp_matrix::numerics::lu_decomposition(self);
        }
        Err(not_ported("lu"))
    }
    /// `QRDecomposition()` (virtual)
    pub fn qr_decomposition(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::qr_decomposition(self);
        }
        Err(not_ported("qr"))
    }
    /// `SVDDecomposition()` (virtual)
    pub fn svd_decomposition(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::svd_decomposition(self);
        }
        Err(not_ported("svd"))
    }
    /// `L2Norm()` (virtual)
    pub fn l2_norm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::l2_norm(self);
        }
        Err(not_ported("mnorm_2"))
    }
    /// `Cond1()` (virtual)
    pub fn cond1(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond1(self);
        }
        Err(not_ported("cond_1"))
    }
    /// `Cond2()` (virtual)
    pub fn cond2(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond2(self);
        }
        Err(not_ported("cond_2"))
    }
    /// `CondFrob()` (virtual)
    pub fn cond_frob(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond_frob(self);
        }
        Err(not_ported("cond_e"))
    }
    /// `CondInf()` (virtual)
    pub fn cond_inf(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::cond_inf(self);
        }
        Err(not_ported("cond_i"))
    }
    /// `Rank()` (virtual)
    pub fn rank(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::numerics::rank(self);
        }
        Err(not_ported("rank"))
    }
    /// `Adjoint()` (virtual)
    pub fn adjoint(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::adjoint(self);
        }
        Err(not_ported("adj"))
    }
    /// `Cofactor()` (virtual)
    pub fn cofactor(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::cofactor(self);
        }
        Err(not_ported("cofactor"))
    }
    /// `CholeskyDecomposition()` (SymmetricMatrix and DiagonalMatrix)
    pub fn cholesky_decomposition(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::cholesky_decomposition(self);
        }
        Err(not_ported("cholesky"))
    }
    /// `EigenValues(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen_values(&self, _count: i64, _tol: f64) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::numerics::eigen_values(self, _count, _tol);
        }
        Err(not_ported("eigenvals"))
    }
    /// `EigenVectors(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen_vectors(&self, _count: i64, _tol: f64) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::eigen_vectors(self, _count, _tol);
        }
        Err(not_ported("eigenvecs"))
    }
    /// `Eigen(int count)` (SymmetricMatrix and DiagonalMatrix)
    pub fn eigen(&self, _count: i64, _tol: f64) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::eigen(self, _count, _tol);
        }
        Err(not_ported("eigen"))
    }
    /// `ClSolve(Vector v)` (SymmetricMatrix)
    pub fn cl_solve(&self, _v: &Vector) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::numerics::cl_solve(self, _v);
        }
        Err(not_ported("clsolve"))
    }
    /// `CmSolve(Matrix M)` (SymmetricMatrix)
    pub fn cm_solve(&self, _m: &Matrix) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::numerics::cm_solve(self, _m);
        }
        Err(not_ported("cmsolve"))
    }
}
