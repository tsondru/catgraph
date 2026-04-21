//! Matrix prop `Mat(R)` over a rig R (F&S 2018 Def 5.50).
//!
//! Objects are ℕ. A morphism `m → n` is an `m × n` matrix, i.e. **rows =
//! domain arity, cols = codomain arity** (per Def 5.50 + Remark 5.49 row-vector
//! convention). Composition is `v ; A` row-major matmul:
//! `(M ; N)(i, c) = Σ_b M(i, b) · N(b, c)`. Monoidal tensor is block-diagonal
//! sum.
//!
//! This module does NOT use nalgebra — `Mat(R)` over an arbitrary rig may fail
//! nalgebra's field-like trait bounds (`Tropical`, `BoolRig`, any semiring
//! without subtraction). An nalgebra bridge specialized to `F64Rig` may be
//! added in a later release behind a feature flag.

use catgraph::{
    category::{Composable, HasIdentity},
    errors::CatgraphError,
    monoidal::{Monoidal, MonoidalMorphism, SymmetricMonoidalMorphism},
};
use permutations::Permutation;

use crate::rig::Rig;

/// An `m × n` matrix over a rig `R`, representing a morphism `m → n` of
/// `Mat(R)` (F&S Def 5.50).
///
/// Row-major layout: `entries[i][j]` is the entry `M(i, j)`.
#[derive(Clone, Debug, PartialEq)]
pub struct MatR<R: Rig> {
    rows: usize,
    cols: usize,
    entries: Vec<Vec<R>>,
}

impl<R: Rig> MatR<R> {
    /// Construct a matrix, validating that `entries.len() == rows` and each
    /// inner `entries[i].len() == cols`.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError::Composition`] on shape mismatch.
    pub fn new(rows: usize, cols: usize, entries: Vec<Vec<R>>) -> Result<Self, CatgraphError> {
        if entries.len() != rows {
            return Err(CatgraphError::Composition {
                message: format!("expected {rows} rows, got {}", entries.len()),
            });
        }
        for (i, row) in entries.iter().enumerate() {
            if row.len() != cols {
                return Err(CatgraphError::Composition {
                    message: format!("row {i} has {} cols, expected {cols}", row.len()),
                });
            }
        }
        Ok(Self { rows, cols, entries })
    }

    /// The `n × n` identity matrix.
    #[must_use]
    pub fn identity(n: usize) -> Self {
        let mut entries = vec![vec![R::zero(); n]; n];
        for (i, row) in entries.iter_mut().enumerate().take(n) {
            row[i] = R::one();
        }
        Self { rows: n, cols: n, entries }
    }

    /// The all-zeros `rows × cols` matrix.
    #[must_use]
    pub fn zero_matrix(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            entries: vec![vec![R::zero(); cols]; rows],
        }
    }

    /// Matrix multiplication: `self ; other` where `self: m × n` and
    /// `other: n × p`, producing an `m × p` matrix.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError::CompositionSizeMismatch`] if
    /// `self.cols != other.rows`.
    pub fn matmul(&self, other: &Self) -> Result<Self, CatgraphError> {
        if self.cols != other.rows {
            return Err(CatgraphError::CompositionSizeMismatch {
                expected: self.cols,
                actual: other.rows,
            });
        }
        let mut result = vec![vec![R::zero(); other.cols]; self.rows];
        for (i, out_row) in result.iter_mut().enumerate().take(self.rows) {
            for (c, out_cell) in out_row.iter_mut().enumerate().take(other.cols) {
                let mut sum = R::zero();
                for b in 0..self.cols {
                    sum = sum + (self.entries[i][b].clone() * other.entries[b][c].clone());
                }
                *out_cell = sum;
            }
        }
        Ok(Self {
            rows: self.rows,
            cols: other.cols,
            entries: result,
        })
    }

    /// Block-diagonal sum: `[[self, 0], [0, other]]`.
    #[must_use]
    pub fn block_diagonal(&self, other: &Self) -> Self {
        let new_rows = self.rows + other.rows;
        let new_cols = self.cols + other.cols;
        let mut entries = vec![vec![R::zero(); new_cols]; new_rows];
        for (i, src_row) in self.entries.iter().enumerate() {
            for (j, src_cell) in src_row.iter().enumerate() {
                entries[i][j] = src_cell.clone();
            }
        }
        for (i, src_row) in other.entries.iter().enumerate() {
            for (j, src_cell) in src_row.iter().enumerate() {
                entries[self.rows + i][self.cols + j] = src_cell.clone();
            }
        }
        Self {
            rows: new_rows,
            cols: new_cols,
            entries,
        }
    }

    /// Matrix from a permutation: `n × n` matrix with `entries[i][p(i)] = 1`.
    #[must_use]
    pub fn permutation_matrix(p: &Permutation) -> Self {
        let n = p.len();
        let mut entries = vec![vec![R::zero(); n]; n];
        for (i, row) in entries.iter_mut().enumerate().take(n) {
            row[p.apply(i)] = R::one();
        }
        Self { rows: n, cols: n, entries }
    }

    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[must_use]
    pub fn cols(&self) -> usize {
        self.cols
    }

    #[must_use]
    pub fn entries(&self) -> &[Vec<R>] {
        &self.entries
    }
}

// ---- Category / monoidal trait impls ----

impl<R: Rig> HasIdentity<Vec<()>> for MatR<R> {
    fn identity(on_this: &Vec<()>) -> Self {
        Self::identity(on_this.len())
    }
}

impl<R: Rig> Composable<Vec<()>> for MatR<R> {
    fn compose(&self, other: &Self) -> Result<Self, CatgraphError> {
        self.matmul(other)
    }

    fn domain(&self) -> Vec<()> {
        vec![(); self.rows]
    }

    fn codomain(&self) -> Vec<()> {
        vec![(); self.cols]
    }
}

impl<R: Rig> Monoidal for MatR<R> {
    fn monoidal(&mut self, other: Self) {
        *self = self.block_diagonal(&other);
    }
}

impl<R: Rig> MonoidalMorphism<Vec<()>> for MatR<R> {}

impl<R: Rig> SymmetricMonoidalMorphism<()> for MatR<R> {
    fn from_permutation(
        p: Permutation,
        _types: &[()],
        _types_as_on_domain: bool,
    ) -> Result<Self, CatgraphError> {
        Ok(Self::permutation_matrix(&p))
    }

    fn permute_side(&mut self, p: &Permutation, of_codomain: bool) {
        // Right-mul by P permutes columns (codomain side); left-mul by P^T
        // permutes rows (domain side). Length-mismatch is defensive no-op
        // to match the trait's non-fallible signature.
        let expected = if of_codomain { self.cols } else { self.rows };
        if p.len() != expected {
            return;
        }
        let perm_mat = Self::permutation_matrix(p);
        if of_codomain {
            if let Ok(result) = self.matmul(&perm_mat) {
                *self = result;
            }
        } else {
            // P^T has entries[p(i)][i] = 1; equivalently, the transpose of P.
            let n = p.len();
            let mut entries = vec![vec![R::zero(); n]; n];
            for i in 0..n {
                entries[p.apply(i)][i] = R::one();
            }
            let p_transpose = Self { rows: n, cols: n, entries };
            if let Ok(result) = p_transpose.matmul(self) {
                *self = result;
            }
        }
    }
}
