pub mod error;
pub mod persist;
pub mod schema;
pub mod types;
pub mod cospan_store;
pub mod named_cospan_store;
pub mod span_store;

// V2 RELATE-based persistence layer
pub mod schema_v2;
pub mod types_v2;
pub mod node_store;
pub mod edge_store;
pub mod hyperedge;
pub use hyperedge as hyperedge_store;
pub mod query;
pub mod petri_net_store;
pub mod wiring_store;
pub mod fingerprint;

use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use error::PersistError;

/// Initialize the V1 catgraph persistence schema (embedded arrays).
pub async fn init_schema(db: &Surreal<Db>) -> Result<(), PersistError> {
    db.query(schema::SCHEMA_DDL).await?;
    Ok(())
}

/// Initialize the V2 RELATE-based graph persistence schema.
///
/// Can be called alongside `init_schema()` — V1 and V2 use different table names.
pub async fn init_schema_v2(db: &Surreal<Db>) -> Result<(), PersistError> {
    db.query(schema_v2::SCHEMA_V2_DDL).await?;
    Ok(())
}
