//! `SurrealDB` persistence stub for multiway evolution graphs.
//!
//! Schema DDL (in [`schema_v2`]) defines `multiway_node` and `multiway_edge`
//! tables. Record types (in [`types_v2`]) provide [`MultiwayNodeRecord`] and
//! [`MultiwayEdgeRecord`] for typed deserialization.
//!
//! Full `save`/`load` implementation is deferred until `MultiwayEvolutionGraph`
//! serialization is available in catgraph core.
//!
//! [`schema_v2`]: crate::schema_v2
//! [`types_v2`]: crate::types_v2
//! [`MultiwayNodeRecord`]: crate::types_v2::MultiwayNodeRecord
//! [`MultiwayEdgeRecord`]: crate::types_v2::MultiwayEdgeRecord

use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Persistence layer for multiway evolution graphs.
///
/// Stores [`MultiwayEvolutionGraph`] nodes and edges in `SurrealDB`'s
/// `multiway_node` and `multiway_edge` tables. Each node records a
/// `(branch_id, step)` pair and a human-readable state label; edges
/// are typed as `"evolution"` (parent-to-child within a branch) or
/// `"branchial"` (same-step cross-branch).
///
/// # Status
///
/// Design and stub only. Schema DDL and record types are ready.
/// Full `save`/`load` implementation is deferred until
/// `MultiwayEvolutionGraph` serialization is available.
///
/// [`MultiwayEvolutionGraph`]: catgraph::multiway::MultiwayEvolutionGraph
pub struct MultiwayEvolutionStore<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> MultiwayEvolutionStore<'a> {
    /// Creates a new persistence handle.
    ///
    /// Requires V2 schema to be initialized via [`init_schema_v2`].
    ///
    /// [`init_schema_v2`]: crate::init_schema_v2
    #[must_use]
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Returns a reference to the underlying database connection.
    #[must_use]
    pub fn db(&self) -> &Surreal<Db> {
        self.db
    }
}
