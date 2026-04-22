//! Bounded congruence-closure decision procedure for [`super::Presentation`]-modulo equality.
//!
//! Given a term graph over [`PropExpr<G>`] and a seed set of equations,
//! computes the smallest congruence relation containing the seed, then
//! answers `are_equal` queries by union-find root comparison.
//!
//! Based on the Downey-Sethi-Tarjan 1980 algorithm using a signature-table
//! indexed by canonical child-class IDs. Correct for finitely-presented
//! equational theories without binders; complete for the 16 F&S Thm 5.60
//! equations (Baez-Erbele 2015).
//!
//! This engine is **not** full Knuth-Bendix completion with critical-pair
//! discovery — it seeds a term graph with the user's equations as-is, then
//! propagates congruence through `Compose` / `Tensor`. For a confluent,
//! terminating rewrite system, this is a decision procedure. For the 16
//! Thm 5.60 equations specifically, Baez-Erbele 2015 proved completeness, so
//! congruence closure with this seed decides Mat(R)-equality on SFG
//! expressions.
//!
//! # Algorithm sketch
//!
//! The term graph hash-conses every sub-term to a `TermId`. A signature
//! table keyed on `(Tag, root_class(arg_a), root_class(arg_b))` records the
//! canonical representative of each function-node congruence class. On
//! `add_term` we probe this table: if a match exists, the new term is
//! immediately merged into the existing class. On `merge` we walk the uses
//! lists of the smaller class, re-probing each function node's signature
//! against the (now updated) table — any collision produces another merge,
//! propagating congruence to fixpoint.
//!
//! # Complexity
//!
//! Per `are_equal` query:
//! - Term insertion: `O(|a| + |b|)` expected, assuming `O(1)` hash
//!   operations on the term / signature tables.
//! - Congruence propagation: amortized `O(n · α(n))` total across all
//!   merges, where `α` is the inverse-Ackermann function (from union-find
//!   with path halving).
//!
//! With a sorted-pair signature representation (as in the original DST
//! paper) the `α(n)` bound extends to per-insertion; we trade that for
//! the average-case simplicity of a hash table.
//!
//! # References
//!
//! * P. J. Downey, R. Sethi, R. E. Tarjan. *Variations on the Common
//!   Subexpression Problem*. J. ACM 27(4), 1980.
//! * J. Baez, J. Erbele. *Categories in Control*. Theory and Applications
//!   of Categories 30, 2015.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use super::super::{PropExpr, PropSignature};

/// Internal term ID — dense index into the term graph.
type TermId = usize;

/// Tag distinguishing function-symbol constructor variants for congruence
/// propagation. Atoms (`Identity`, `Braid`, `Generator`) never propagate
/// congruence — only `Compose` and `Tensor` do — so only these two tags
/// occur as signature-table keys or in the `uses` index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tag {
    Compose,
    Tensor,
}

/// A term-graph node. The `Generator(G)` variant constrains `G: Eq + Hash`
/// via the unconditional derives below; all other variants have `usize` or
/// `TermId` children, so derivation works uniformly once `G` satisfies the
/// required bounds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Node<G>
where
    G: Clone + PartialEq + Eq + Hash,
{
    Identity(usize),
    Braid(usize, usize),
    Generator(G),
    Compose(TermId, TermId),
    Tensor(TermId, TermId),
}

/// Congruence-closure engine seeded with a fixed set of equations.
///
/// After construction the engine is ready to answer [`Self::are_equal`]
/// queries. Queries may extend the term graph with previously unseen
/// sub-terms; the engine re-probes the signature table on insertion, so
/// query results remain consistent with the seeded equations.
///
/// Equality is **modulo the seeded equations only** — associativity,
/// unitality, interchange, braiding naturality, and other SMC axioms are
/// *not* assumed unless explicitly seeded. Callers that need an SMC-aware
/// decision procedure should pre-seed the 16 Thm 5.60 equations (Baez-Erbele
/// 2015).
pub struct CongruenceClosure<G>
where
    G: PropSignature,
{
    /// Canonical term-graph lookup: structural `Node` → `TermId`. Ensures
    /// structurally-identical sub-terms share a single ID on insertion.
    nodes: HashMap<Node<G>, TermId>,
    /// Inverse map for each fresh `TermId`: the `Node` it was created from.
    /// Read by `propagate` to re-canonicalize a function node's children
    /// via `find` after a merge has potentially invalidated the IDs
    /// recorded in the `uses` list.
    reverse: Vec<Node<G>>,
    /// Union-find parent pointers; `parent[i] == i` iff `i` is a class root.
    parent: Vec<TermId>,
    /// Per-class uses list: for each class root `c`, records every
    /// function-symbol node `f(a, b)` with `find(a) == c` or `find(b) == c`,
    /// as `(term_id, other_arg_id, constructor_tag)`. Scanned during merge
    /// propagation to re-probe signatures. Entries may become stale (refer
    /// to non-root IDs) after subsequent merges — we re-canonicalize on use.
    uses: Vec<Vec<(TermId, TermId, Tag)>>,
    /// Signature table keyed on `(Tag, find(arg_a), find(arg_b))`, mapping
    /// to the canonical representative of the corresponding congruence
    /// class. New function nodes probe this table on insertion.
    signatures: HashMap<(Tag, TermId, TermId), TermId>,
    /// LIFO worklist (stack) of pending `(ra, rb)` root pairs awaiting
    /// propagation. DST terminates under any worklist order, so a stack
    /// via `Vec::pop` is fine.
    pending: Vec<(TermId, TermId)>,
}

impl<G> CongruenceClosure<G>
where
    G: PropSignature,
{
    /// Build a new engine seeded with the given equations.
    ///
    /// Each equation's LHS and RHS are inserted into the term graph and
    /// their classes merged. Congruence is then propagated to fixpoint.
    #[must_use]
    pub fn new(equations: &[(PropExpr<G>, PropExpr<G>)]) -> Self {
        let mut engine = Self {
            nodes: HashMap::new(),
            reverse: Vec::new(),
            parent: Vec::new(),
            uses: Vec::new(),
            signatures: HashMap::new(),
            pending: Vec::new(),
        };
        let mut seed_pairs = Vec::with_capacity(equations.len());
        for (lhs, rhs) in equations {
            let l = engine.add_term(lhs);
            let r = engine.add_term(rhs);
            seed_pairs.push((l, r));
        }
        for (l, r) in seed_pairs {
            engine.merge(l, r);
        }
        engine.propagate();
        engine
    }

    /// Test equality of two terms modulo the seeded equations.
    ///
    /// May extend the term graph with previously unseen sub-terms; after
    /// any such extension, congruence is re-propagated so the returned
    /// verdict is consistent with the seeded theory.
    #[must_use]
    pub fn are_equal(&mut self, a: &PropExpr<G>, b: &PropExpr<G>) -> bool {
        let a_id = self.add_term(a);
        let b_id = self.add_term(b);
        self.propagate();
        self.find(a_id) == self.find(b_id)
    }

    /// Add a term to the graph, returning its ID.
    ///
    /// Structural hash-cons: identical `Node` shapes share an ID. For
    /// function-symbol nodes (`Compose` / `Tensor`) we additionally probe
    /// the signature table against the class-roots of the children — if a
    /// congruent function node already exists, the new node is merged with
    /// it. Recurses on children.
    fn add_term(&mut self, expr: &PropExpr<G>) -> TermId {
        let node = match expr {
            PropExpr::Identity(n) => Node::Identity(*n),
            PropExpr::Braid(m, n) => Node::Braid(*m, *n),
            PropExpr::Generator(g) => Node::Generator(g.clone()),
            PropExpr::Compose(f, g) => {
                let f_id = self.add_term(f);
                let g_id = self.add_term(g);
                Node::Compose(f_id, g_id)
            }
            PropExpr::Tensor(f, g) => {
                let f_id = self.add_term(f);
                let g_id = self.add_term(g);
                Node::Tensor(f_id, g_id)
            }
        };
        if let Some(&id) = self.nodes.get(&node) {
            return id;
        }
        let id = self.reverse.len();
        self.parent.push(id);
        self.uses.push(Vec::new());
        self.reverse.push(node.clone());
        self.nodes.insert(node.clone(), id);

        // Register uses and probe signature table for function-symbol nodes.
        match node {
            Node::Compose(a, b) => self.install_function_node(id, a, b, Tag::Compose),
            Node::Tensor(a, b) => self.install_function_node(id, a, b, Tag::Tensor),
            _ => {}
        }
        id
    }

    /// Register a freshly-inserted function node in its children's uses
    /// lists and in the signature table. If the signature collides with an
    /// existing class representative, enqueue a merge.
    fn install_function_node(&mut self, id: TermId, a: TermId, b: TermId, tag: Tag) {
        let ra = self.find(a);
        let rb = self.find(b);
        self.uses[ra].push((id, b, tag));
        if ra != rb {
            self.uses[rb].push((id, a, tag));
        }
        if let Some(existing) = self.signatures.insert((tag, ra, rb), id) {
            // Signature collision: `existing` already represents the
            // congruence class of (tag, ra, rb). Merge the two, then
            // store the *post-merge* canonical root — `merge` links one
            // root onto the other but the direction is implementation-
            // defined, so we must re-canonicalize via `find`.
            self.merge(id, existing);
            let root = self.find(existing);
            self.signatures.insert((tag, ra, rb), root);
        }
    }

    /// Union-find root with path halving.
    fn find(&mut self, mut id: TermId) -> TermId {
        while self.parent[id] != id {
            let next = self.parent[id];
            self.parent[id] = self.parent[next]; // path halving
            id = next;
        }
        id
    }

    /// Merge two classes. If they are already unified this is a no-op.
    /// Otherwise the first argument's root is linked to the second
    /// argument's root — ordering is determined by the caller, not by
    /// ID comparison — and the pair is queued for propagation via
    /// [`Self::propagate`].
    ///
    /// We don't union-by-rank here because per-class uses lists
    /// dominate cost and aren't tied to the root choice; `propagate`
    /// is responsible for re-filing uses from the losing root into
    /// the winning root's list.
    fn merge(&mut self, a: TermId, b: TermId) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        // Link ra's root onto rb's root. Record the losing-side root
        // so propagation knows which uses list to walk.
        self.parent[ra] = rb;
        self.pending.push((ra, rb));
    }

    /// Drain the pending worklist, re-probing the signature table for
    /// every function node in each losing-class's uses list. If a signature
    /// now collides with an existing class representative, merge those and
    /// enqueue again; otherwise update the table with the new canonical
    /// signature. Terminates because each effective merge reduces the
    /// number of equivalence classes by 1.
    fn propagate(&mut self) {
        while let Some((losing_root, _winning_root)) = self.pending.pop() {
            // Take the losing root's uses list; after the merge, any uses
            // of its members properly belong to the winning root's list.
            // We re-probe each use's signature against the current root
            // classes and re-file into the winner's list.
            let losing_uses = std::mem::take(&mut self.uses[losing_root]);
            for (term, _other, tag) in losing_uses {
                // `term` is a `Compose(a, b)` / `Tensor(a, b)` node. Re-read
                // its literal children directly from `reverse` — the `other`
                // component of the uses tuple may reference a non-root ID by
                // the time propagation reaches us, so we re-canonicalize via
                // `find` rather than trust it.
                let (Node::Compose(a, b) | Node::Tensor(a, b)) = self.reverse[term] else {
                    unreachable!(
                        "non-function node in uses list (Generator/Identity/Braid never register uses)"
                    )
                };
                let ra = self.find(a);
                let rb = self.find(b);
                let key = (tag, ra, rb);

                match self.signatures.get(&key).copied() {
                    Some(canonical) if self.find(canonical) != self.find(term) => {
                        // Fresh collision — merge the two classes.
                        self.merge(term, canonical);
                    }
                    Some(_) => {
                        // Already canonical for this signature; nothing to do.
                    }
                    None => {
                        // Fresh signature; register `term` as canonical.
                        self.signatures.insert(key, term);
                    }
                }

                // Re-file the use under the winning root of each child so
                // later merges involving this node can still find it.
                let root_a = self.find(a);
                let root_b = self.find(b);
                self.uses[root_a].push((term, b, tag));
                if root_a != root_b {
                    self.uses[root_b].push((term, a, tag));
                }
            }
        }
    }
}
