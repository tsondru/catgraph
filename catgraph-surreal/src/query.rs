use std::collections::HashSet;

use surrealdb::engine::local::Db;
use surrealdb::IndexedResults;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use crate::error::PersistError;
use crate::node_store::NodeStore;
use crate::types_v2::GraphNodeRecord;

/// Thin query helper for common SurrealQL graph traversal patterns.
pub struct QueryHelper<'a> {
    db: &'a Surreal<Db>,
    node_store: NodeStore<'a>,
}

impl<'a> QueryHelper<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self {
            db,
            node_store: NodeStore::new(db),
        }
    }

    /// Get outbound neighbors via edges of a specific kind.
    ///
    /// Queries the edge table directly to find target node IDs, then fetches
    /// each node. Avoids `serde_json::Value` intermediary which cannot
    /// deserialize `RecordId`.
    pub async fn outbound_neighbors(
        &self,
        node: &RecordId,
        edge_kind: &str,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT out FROM graph_edge WHERE in = $node AND kind = $kind")
            .bind(("node", node.clone()))
            .bind(("kind", edge_kind.to_string()))
            .await?;
        let refs: Vec<OutRef> = result.take(0)?;
        let mut nodes = Vec::with_capacity(refs.len());
        for r in &refs {
            nodes.push(self.node_store.get(&r.out).await?);
        }
        Ok(nodes)
    }

    /// Get inbound neighbors via edges of a specific kind.
    ///
    /// Queries the edge table directly to find source node IDs, then fetches
    /// each node.
    pub async fn inbound_neighbors(
        &self,
        node: &RecordId,
        edge_kind: &str,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT `in` AS src FROM graph_edge WHERE out = $node AND kind = $kind")
            .bind(("node", node.clone()))
            .bind(("kind", edge_kind.to_string()))
            .await?;
        let refs: Vec<InRef> = result.take(0)?;
        let mut nodes = Vec::with_capacity(refs.len());
        for r in &refs {
            nodes.push(self.node_store.get(&r.src).await?);
        }
        Ok(nodes)
    }

    /// Find all nodes reachable within `depth` hops via edges of a specific kind.
    ///
    /// Implements BFS traversal up to `depth` hops by querying the edge table
    /// iteratively. Each depth level issues a single batched query for all
    /// frontier nodes using `WHERE `in` IN $nodes`, giving O(depth) queries
    /// instead of O(frontier_size * depth).
    ///
    /// SurrealDB's native recursion syntax (`record.{N}->edge->table`)
    /// requires a literal record ID expression, not a bind parameter, and does not
    /// support edge-property filtering — so we do iterative expansion here.
    pub async fn reachable(
        &self,
        node: &RecordId,
        edge_kind: &str,
        depth: u32,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        // RecordId contains a regex cache (interior mutability) but Hash/Eq are stable.
        #[allow(clippy::mutable_key_type)]
        let mut visited: HashSet<RecordId> = HashSet::from([node.clone()]);
        let mut visited_ordered: Vec<RecordId> = vec![node.clone()];
        let mut frontier: Vec<RecordId> = vec![node.clone()];

        for _ in 0..depth {
            if frontier.is_empty() {
                break;
            }
            let mut result = self
                .db
                .query("SELECT out FROM graph_edge WHERE `in` IN $nodes AND kind = $kind")
                .bind(("nodes", frontier.clone()))
                .bind(("kind", edge_kind.to_string()))
                .await?;
            let refs: Vec<OutRef> = result.take(0)?;
            let mut next_frontier = Vec::new();
            for r in refs {
                if visited.insert(r.out.clone()) {
                    visited_ordered.push(r.out.clone());
                    next_frontier.push(r.out);
                }
            }
            frontier = next_frontier;
        }

        // Fetch all discovered nodes except the starting node
        let mut nodes = Vec::with_capacity(visited_ordered.len().saturating_sub(1));
        for id in &visited_ordered[1..] {
            nodes.push(self.node_store.get(id).await?);
        }
        Ok(nodes)
    }

    /// Find the shortest path between two nodes via edges of a specific kind.
    ///
    /// Returns the path as a sequence of `GraphNodeRecord` (start to end),
    /// or `None` if the target is unreachable within `max_depth` hops.
    /// When `from == to`, returns a single-element path containing just that node.
    ///
    /// Uses BFS with parent tracking — O(depth) queries, each batched over
    /// the frontier.
    pub async fn shortest_path(
        &self,
        from: &RecordId,
        to: &RecordId,
        edge_kind: &str,
        max_depth: u32,
    ) -> Result<Option<Vec<GraphNodeRecord>>, PersistError> {
        // Same node: trivial path of length 1.
        if from == to {
            let node = self.node_store.get(from).await?;
            return Ok(Some(vec![node]));
        }

        // BFS with parent map for path reconstruction.
        // RecordId contains a regex cache (interior mutability) but Hash/Eq are stable.
        #[allow(clippy::mutable_key_type)]
        let mut visited: HashSet<RecordId> = HashSet::from([from.clone()]);
        // Map child -> parent for path reconstruction.
        #[allow(clippy::mutable_key_type)]
        let mut parent: std::collections::HashMap<RecordId, RecordId> =
            std::collections::HashMap::new();
        let mut frontier: Vec<RecordId> = vec![from.clone()];
        let mut found = false;

        for _ in 0..max_depth {
            if frontier.is_empty() {
                break;
            }
            let mut result = self
                .db
                .query("SELECT `in` AS src, out FROM graph_edge WHERE `in` IN $nodes AND kind = $kind")
                .bind(("nodes", frontier.clone()))
                .bind(("kind", edge_kind.to_string()))
                .await?;
            let refs: Vec<InOutRef> = result.take(0)?;
            let mut next_frontier = Vec::new();
            for r in refs {
                if visited.insert(r.out.clone()) {
                    parent.insert(r.out.clone(), r.src.clone());
                    if r.out == *to {
                        found = true;
                        break;
                    }
                    next_frontier.push(r.out);
                }
            }
            if found {
                break;
            }
            frontier = next_frontier;
        }

        if !found {
            return Ok(None);
        }

        // Reconstruct path by walking parent chain backward.
        let mut path_ids = vec![to.clone()];
        let mut current = to.clone();
        while current != *from {
            let p = parent
                .get(&current)
                .ok_or_else(|| PersistError::InvalidData("BFS parent chain broken".into()))?;
            path_ids.push(p.clone());
            current = p.clone();
        }
        path_ids.reverse();

        // Fetch node records in path order.
        let mut path = Vec::with_capacity(path_ids.len());
        for id in &path_ids {
            path.push(self.node_store.get(id).await?);
        }
        Ok(Some(path))
    }

    /// Collect all unique nodes reachable within `max_depth` hops via edges of
    /// a specific kind, deduplicated.
    ///
    /// Delegates to [`reachable`](Self::reachable) — this is a convenience
    /// alias with clearer naming for the "collect all" use case.
    pub async fn collect_reachable(
        &self,
        node: &RecordId,
        edge_kind: &str,
        max_depth: u32,
    ) -> Result<Vec<GraphNodeRecord>, PersistError> {
        self.reachable(node, edge_kind, max_depth).await
    }

    /// Execute a raw SurrealQL query with bindings.
    pub async fn raw(
        &self,
        surql: &str,
        bindings: Vec<(String, serde_json::Value)>,
    ) -> Result<IndexedResults, PersistError> {
        let mut query = self.db.query(surql);
        for (key, val) in bindings {
            query = query.bind((key, val));
        }
        Ok(query.await?)
    }
}

/// Helper struct for extracting `out` RecordId from edge query results.
#[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
struct OutRef {
    out: RecordId,
}

/// Helper struct for extracting source (`in`) RecordId from edge query results.
///
/// Uses `src` alias because `SurrealValue` derive does not support `#[serde(rename)]`.
/// The query must use `SELECT `in` AS src FROM ...`.
#[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
struct InRef {
    src: RecordId,
}

/// Helper struct for extracting both `in` (as `src`) and `out` RecordId from edge
/// query results. Used by `shortest_path` to track parent→child relationships.
///
/// The query must alias `in` as `src`: `SELECT `in` AS src, out FROM ...`.
#[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
struct InOutRef {
    src: RecordId,
    out: RecordId,
}
