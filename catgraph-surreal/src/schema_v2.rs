/// SurrealQL DDL for the V2 RELATE-based graph persistence schema.
///
/// V2 uses first-class graph_node records connected by RELATE edges,
/// plus hub-node reification for n-ary hyperedges (cospans/spans).
/// Coexists with V1 embedded-array tables (different table names).
pub const SCHEMA_V2_DDL: &str = r#"
-- First-class graph vertices
DEFINE TABLE IF NOT EXISTS graph_node SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON graph_node TYPE string;
DEFINE FIELD IF NOT EXISTS kind ON graph_node TYPE string;
DEFINE FIELD IF NOT EXISTS labels ON graph_node TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS properties ON graph_node TYPE object FLEXIBLE DEFAULT {};
DEFINE FIELD IF NOT EXISTS created_at ON graph_node TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_node_kind ON graph_node FIELDS kind;
DEFINE INDEX IF NOT EXISTS idx_node_name ON graph_node FIELDS name;

-- Pairwise RELATE edges between graph_node records
DEFINE TABLE IF NOT EXISTS graph_edge SCHEMAFULL TYPE RELATION FROM graph_node TO graph_node;
DEFINE FIELD IF NOT EXISTS kind ON graph_edge TYPE string;
DEFINE FIELD IF NOT EXISTS weight ON graph_edge TYPE option<float> DEFAULT NONE;
DEFINE FIELD IF NOT EXISTS properties ON graph_edge TYPE object FLEXIBLE DEFAULT {};
DEFINE FIELD IF NOT EXISTS created_at ON graph_edge TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_edge_kind ON graph_edge FIELDS kind;

-- Hub record for n-ary hyperedge reification
DEFINE TABLE IF NOT EXISTS hyperedge_hub SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS kind ON hyperedge_hub TYPE string;
DEFINE FIELD IF NOT EXISTS properties ON hyperedge_hub TYPE object FLEXIBLE DEFAULT {};
DEFINE FIELD IF NOT EXISTS source_count ON hyperedge_hub TYPE int;
DEFINE FIELD IF NOT EXISTS target_count ON hyperedge_hub TYPE int;
DEFINE FIELD IF NOT EXISTS created_at ON hyperedge_hub TYPE datetime DEFAULT time::now();

-- Source participation: graph_node -> hyperedge_hub (with ordered position)
DEFINE TABLE IF NOT EXISTS source_of SCHEMAFULL TYPE RELATION FROM graph_node TO hyperedge_hub;
DEFINE FIELD IF NOT EXISTS position ON source_of TYPE int;

-- Target participation: hyperedge_hub -> graph_node (with ordered position)
DEFINE TABLE IF NOT EXISTS target_of SCHEMAFULL TYPE RELATION FROM hyperedge_hub TO graph_node;
DEFINE FIELD IF NOT EXISTS position ON target_of TYPE int;

-- Record references for composition provenance
DEFINE FIELD IF NOT EXISTS parent_hubs ON hyperedge_hub
    TYPE option<array<record<hyperedge_hub>>> REFERENCE ON DELETE UNSET;

-- Composition relation: tracks which hubs were composed to produce a child hub
DEFINE TABLE IF NOT EXISTS composed_from SCHEMAFULL
    TYPE RELATION FROM hyperedge_hub TO hyperedge_hub;
DEFINE FIELD IF NOT EXISTS operation ON composed_from TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON composed_from TYPE datetime DEFAULT time::now();

-- Computed provenance flag (evaluated only when selected, v3.0.5)
DEFINE FIELD IF NOT EXISTS has_provenance ON hyperedge_hub TYPE bool
    COMPUTED parent_hubs IS NOT NONE AND array::len(parent_hubs) > 0;
"#;
