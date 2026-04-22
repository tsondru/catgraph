//! Prop presentations (F&S Def 5.33): equations quotienting `Free(G)`.
//!
//! A presentation `(G, s, t, E)` consists of a signature `G` with arity maps
//! `s, t` (provided via [`super::PropSignature`]) and a set `E` of equations,
//! each a pair `(lhs, rhs)` of [`super::PropExpr<G>`] with matching arity. The
//! presented prop is `Free(G)` quotiented by the smallest congruence
//! containing `E` plus the SMC axioms.
//!
//! # Implementation
//!
//! Bounded-depth (default 32) term rewriting with:
//! 1. A fixed set of 8 **SMC-canonical-form rules** applied first (interchange,
//!    unitors, associator, compose-identity, compose-associator,
//!    braid-involution). This closes the F&S Def 5.30 PARTIAL gap (the
//!    syntactic quotient by SMC axioms is now explicit).
//! 2. User equations `E` applied left-to-right thereafter.
//!
//! ## SMC rules
//!
//! 1. **Interchange**: `(f1 ⊗ g1) ; (f2 ⊗ g2) → (f1 ; f2) ⊗ (g1 ; g2)` when all composable.
//! 2. **Left unitor**: `Identity(0) ⊗ f → f`.
//! 3. **Right unitor**: `f ⊗ Identity(0) → f`.
//! 4. **Associator (right-bias)**: `(f ⊗ g) ⊗ h → f ⊗ (g ⊗ h)`.
//! 5. **Compose-identity (left)**: `Identity(n) ; f → f` when `n` matches `f`'s source.
//! 6. **Compose-identity (right)**: `f ; Identity(n) → f` when `n` matches `f`'s target.
//! 7. **Compose-associator (right-bias)**: `(f ; g) ; h → f ; (g ; h)`.
//! 8. **Braid-involution**: `Braid(m,n) ; Braid(n,m) → Identity(m+n)`.
//!
//! # Confluence
//!
//! The 8 fixed rules are confluent on non-overlapping user equations. For
//! overlapping user equations the rewriter may yield false `eq_mod` negatives
//! — a conservative answer. Knuth-Bendix completion is out of scope.

use super::{PropExpr, PropSignature};
use catgraph::errors::CatgraphError;

/// Result of [`Presentation::normalize`]. Distinguishes "fully reduced"
/// from "hit depth bound" so callers can decide how to handle partial results.
///
/// v0.5.1 API change: replaces the v0.5.0 `Result<PropExpr<G>>` return type.
/// See [`Presentation::normalize`] migration notes.
#[derive(Debug, Clone)]
#[must_use]
pub struct NormalizeResult<G: PropSignature> {
    /// The (possibly partial) normalized expression.
    pub expr: PropExpr<G>,
    /// `true` iff normalization reached a fixpoint before the depth bound.
    pub converged: bool,
    /// Number of rewrite iterations performed (≤ `rewrite_depth`).
    pub steps_taken: usize,
}

/// A presentation of a prop: generators `G` with arity maps plus equations `E`.
#[derive(Debug, Clone)]
pub struct Presentation<G: PropSignature> {
    equations: Vec<(PropExpr<G>, PropExpr<G>)>,
    rewrite_depth: usize,
}

impl<G: PropSignature> Default for Presentation<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: PropSignature> Presentation<G> {
    /// New empty presentation with default `rewrite_depth = 32`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            equations: Vec::new(),
            rewrite_depth: 32,
        }
    }

    /// New empty presentation with a custom rewrite-depth bound.
    #[must_use]
    pub fn with_depth(rewrite_depth: usize) -> Self {
        Self {
            equations: Vec::new(),
            rewrite_depth,
        }
    }

    /// Add an equation `lhs = rhs`. Both sides must have matching arity.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError::Presentation`] if the two sides have
    /// different source or target arities.
    pub fn add_equation(
        &mut self,
        lhs: PropExpr<G>,
        rhs: PropExpr<G>,
    ) -> Result<(), CatgraphError> {
        let ls = lhs.source();
        let lt = lhs.target();
        let rs = rhs.source();
        let rt = rhs.target();
        if ls != rs || lt != rt {
            return Err(CatgraphError::Presentation {
                message: format!("arity mismatch: lhs ({ls} → {lt}), rhs ({rs} → {rt})"),
            });
        }
        self.equations.push((lhs, rhs));
        Ok(())
    }

    /// Borrow the equation list (LHS-RHS pairs) for external inspection.
    ///
    /// Primarily intended for soundness/faithfulness testing: callers can
    /// iterate every `(lhs, rhs)` pair and assert a chosen semantic
    /// interpretation (e.g. matrix equality under a functor) holds on every
    /// equation.
    #[must_use]
    pub fn equations(&self) -> &[(PropExpr<G>, PropExpr<G>)] {
        &self.equations
    }

    /// Normalize `expr` to canonical form under the SMC rules + user equations.
    ///
    /// Termination is always guaranteed by the depth bound; on a cyclic
    /// equation set the result is whichever representative was reached when
    /// the bound was hit.
    ///
    /// Returns a [`NormalizeResult`] exposing `.expr` (the possibly-partial
    /// normalized expression), `.converged` (`true` iff a fixpoint was reached
    /// before the depth bound), and `.steps_taken` (the number of rewrite
    /// iterations performed).
    ///
    /// v0.5.1 API change: previously returned `Result<PropExpr<G>, _>`.
    /// Callers that only need the expression can write
    /// `p.normalize(&e)?.expr`.
    ///
    /// # Errors
    ///
    /// Currently infallible, but returns [`CatgraphError::Presentation`] for
    /// forward-compatibility (future well-formedness checks may fire during
    /// rewriting).
    pub fn normalize(&self, expr: &PropExpr<G>) -> Result<NormalizeResult<G>, CatgraphError> {
        let mut current = expr.clone();
        for step in 0..self.rewrite_depth {
            let after_smc = apply_smc_rules(&current);
            let after_user = self.apply_user_equations(&after_smc);
            if after_user == current {
                return Ok(NormalizeResult {
                    expr: current,
                    converged: true,
                    // `step` is 0-indexed but a complete iteration (one SMC
                    // pass + one user-equations pass) runs BEFORE the
                    // fixpoint check, so the number of iterations performed
                    // is `step + 1`. Matches the rustdoc contract and the
                    // depth-bound branch (which returns `self.rewrite_depth`,
                    // the count of full iterations run).
                    steps_taken: step + 1,
                });
            }
            current = after_user;
        }
        // Depth bound reached; return whatever we have.
        Ok(NormalizeResult {
            expr: current,
            converged: false,
            steps_taken: self.rewrite_depth,
        })
    }

    /// Equality modulo this presentation.
    ///
    /// Returns `Ok(Some(true))` if both sides converge and normalize to the same
    /// expression; `Ok(Some(false))` if both converge to different expressions;
    /// `Ok(None)` if at least one side hit the depth bound before converging
    /// (the answer is unknown — increase `rewrite_depth`, or accept ambiguity).
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError::Presentation`] if normalization fails for either
    /// side (currently unreachable; future-proofing).
    pub fn eq_mod(
        &self,
        a: &PropExpr<G>,
        b: &PropExpr<G>,
    ) -> Result<Option<bool>, CatgraphError> {
        let na = self.normalize(a)?;
        let nb = self.normalize(b)?;
        if !na.converged || !nb.converged {
            return Ok(None);
        }
        Ok(Some(na.expr == nb.expr))
    }

    fn apply_user_equations(&self, expr: &PropExpr<G>) -> PropExpr<G> {
        let mut current = expr.clone();
        for (lhs, rhs) in &self.equations {
            current = rewrite_once_top(&current, lhs, rhs);
        }
        current
    }
}

/// Apply the 8 fixed SMC-axiom rules once bottom-up, recursing into Compose/Tensor.
fn apply_smc_rules<G: PropSignature>(expr: &PropExpr<G>) -> PropExpr<G> {
    // First, recurse into children (bottom-up).
    let expr = match expr {
        PropExpr::Compose(f, g) => {
            let f_norm = apply_smc_rules(f);
            let g_norm = apply_smc_rules(g);
            PropExpr::Compose(Box::new(f_norm), Box::new(g_norm))
        }
        PropExpr::Tensor(f, g) => {
            let f_norm = apply_smc_rules(f);
            let g_norm = apply_smc_rules(g);
            PropExpr::Tensor(Box::new(f_norm), Box::new(g_norm))
        }
        other => other.clone(),
    };

    // Now apply top-level rules. Order matters — more-specific rules first
    // (identity reductions and braid-involution) before associators, which
    // only rebalance structure.
    match expr {
        // Rule 5: Identity(n) ; f → f
        PropExpr::Compose(ref f, ref g) if matches!(f.as_ref(), PropExpr::Identity(_)) => {
            if let PropExpr::Identity(n) = f.as_ref()
                && *n == g.source()
            {
                return apply_smc_rules(g);
            }
            PropExpr::Compose(f.clone(), g.clone())
        }
        // Rule 6: f ; Identity(n) → f
        PropExpr::Compose(ref f, ref g) if matches!(g.as_ref(), PropExpr::Identity(_)) => {
            if let PropExpr::Identity(n) = g.as_ref()
                && *n == f.target()
            {
                return apply_smc_rules(f);
            }
            PropExpr::Compose(f.clone(), g.clone())
        }
        // Rule 8: Braid(m,n) ; Braid(n,m) → Identity(m+n)
        PropExpr::Compose(ref f, ref g)
            if matches!(f.as_ref(), PropExpr::Braid(_, _))
                && matches!(g.as_ref(), PropExpr::Braid(_, _)) =>
        {
            if let (PropExpr::Braid(m1, n1), PropExpr::Braid(m2, n2)) = (f.as_ref(), g.as_ref())
                && *m1 == *n2
                && *n1 == *m2
            {
                return PropExpr::Identity(m1 + n1);
            }
            PropExpr::Compose(f.clone(), g.clone())
        }
        // Rule 1: Interchange (f1 ⊗ g1) ; (f2 ⊗ g2) → (f1 ; f2) ⊗ (g1 ; g2)
        PropExpr::Compose(ref left, ref right)
            if matches!(left.as_ref(), PropExpr::Tensor(_, _))
                && matches!(right.as_ref(), PropExpr::Tensor(_, _)) =>
        {
            if let (PropExpr::Tensor(f1, g1), PropExpr::Tensor(f2, g2)) =
                (left.as_ref(), right.as_ref())
            {
                // Composability check: f1.target == f2.source and g1.target == g2.source.
                if f1.target() == f2.source() && g1.target() == g2.source() {
                    let f12 = PropExpr::Compose(f1.clone(), f2.clone());
                    let g12 = PropExpr::Compose(g1.clone(), g2.clone());
                    return apply_smc_rules(&PropExpr::Tensor(Box::new(f12), Box::new(g12)));
                }
            }
            PropExpr::Compose(left.clone(), right.clone())
        }
        // Rule 7: (f ; g) ; h → f ; (g ; h)
        PropExpr::Compose(ref outer_left, ref outer_right)
            if matches!(outer_left.as_ref(), PropExpr::Compose(_, _)) =>
        {
            if let PropExpr::Compose(f, g) = outer_left.as_ref() {
                let inner = PropExpr::Compose(g.clone(), outer_right.clone());
                return apply_smc_rules(&PropExpr::Compose(f.clone(), Box::new(inner)));
            }
            PropExpr::Compose(outer_left.clone(), outer_right.clone())
        }
        // Rule 2: Identity(0) ⊗ f → f
        PropExpr::Tensor(ref f, ref g) if matches!(f.as_ref(), PropExpr::Identity(0)) => {
            apply_smc_rules(g)
        }
        // Rule 3: f ⊗ Identity(0) → f
        PropExpr::Tensor(ref f, ref g) if matches!(g.as_ref(), PropExpr::Identity(0)) => {
            apply_smc_rules(f)
        }
        // Rule 4: (f ⊗ g) ⊗ h → f ⊗ (g ⊗ h)
        PropExpr::Tensor(ref outer_left, ref outer_right)
            if matches!(outer_left.as_ref(), PropExpr::Tensor(_, _)) =>
        {
            if let PropExpr::Tensor(f, g) = outer_left.as_ref() {
                let inner = PropExpr::Tensor(g.clone(), outer_right.clone());
                return apply_smc_rules(&PropExpr::Tensor(f.clone(), Box::new(inner)));
            }
            PropExpr::Tensor(outer_left.clone(), outer_right.clone())
        }
        other => other,
    }
}

/// Rewrite `expr`: if the whole tree matches `lhs` structurally, return
/// `rhs.clone()`; otherwise recurse into Compose/Tensor children so equations
/// can match subterms.
fn rewrite_once_top<G: PropSignature>(
    expr: &PropExpr<G>,
    lhs: &PropExpr<G>,
    rhs: &PropExpr<G>,
) -> PropExpr<G> {
    if expr == lhs {
        rhs.clone()
    } else {
        match expr {
            PropExpr::Compose(f, g) => PropExpr::Compose(
                Box::new(rewrite_once_top(f, lhs, rhs)),
                Box::new(rewrite_once_top(g, lhs, rhs)),
            ),
            PropExpr::Tensor(f, g) => PropExpr::Tensor(
                Box::new(rewrite_once_top(f, lhs, rhs)),
                Box::new(rewrite_once_top(g, lhs, rhs)),
            ),
            other => other.clone(),
        }
    }
}

/// A presented prop: wraps a [`Presentation`] with methods for operating on
/// equivalence classes. v0.5.0 surfaces only [`PresentedProp::presentation`]
/// and [`PresentedProp::quotient_representative`].
#[derive(Debug, Clone)]
pub struct PresentedProp<G: PropSignature> {
    presentation: Presentation<G>,
}

impl<G: PropSignature> PresentedProp<G> {
    /// Wrap a presentation as a presented prop.
    #[must_use]
    pub fn new(presentation: Presentation<G>) -> Self {
        Self { presentation }
    }

    /// Borrow the underlying presentation.
    #[must_use]
    pub fn presentation(&self) -> &Presentation<G> {
        &self.presentation
    }

    /// Returns the canonical representative of the equivalence class of `expr`.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError::Presentation`] on normalize failure
    /// (currently unreachable).
    pub fn quotient_representative(
        &self,
        expr: &PropExpr<G>,
    ) -> Result<NormalizeResult<G>, CatgraphError> {
        self.presentation.normalize(expr)
    }
}
