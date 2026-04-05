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
DEFINE FIELD IF NOT EXISTS embedding ON graph_node TYPE option<array<float>> DEFAULT NONE;

DEFINE INDEX IF NOT EXISTS idx_node_kind ON graph_node FIELDS kind;
DEFINE INDEX IF NOT EXISTS idx_node_name ON graph_node FIELDS name;

-- Full-text search on node names with prefix autocomplete
DEFINE ANALYZER IF NOT EXISTS node_name_analyzer
    TOKENIZERS blank, class
    FILTERS lowercase, edgengram(2, 10);

DEFINE INDEX IF NOT EXISTS ft_node_name ON graph_node
    FIELDS name FULLTEXT
    ANALYZER node_name_analyzer
    BM25 HIGHLIGHTS;

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
DEFINE FIELD IF NOT EXISTS weight ON source_of TYPE option<decimal> DEFAULT NONE;

-- Target participation: hyperedge_hub -> graph_node (with ordered position)
DEFINE TABLE IF NOT EXISTS target_of SCHEMAFULL TYPE RELATION FROM hyperedge_hub TO graph_node;
DEFINE FIELD IF NOT EXISTS position ON target_of TYPE int;
DEFINE FIELD IF NOT EXISTS weight ON target_of TYPE option<decimal> DEFAULT NONE;

-- COUNT indexes for efficient participation counting
DEFINE INDEX IF NOT EXISTS cnt_source_of_out ON source_of COUNT;
DEFINE INDEX IF NOT EXISTS cnt_target_of_in ON target_of COUNT;

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

-- Petri net topology
DEFINE TABLE IF NOT EXISTS petri_net SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON petri_net TYPE string;
DEFINE FIELD IF NOT EXISTS label_type ON petri_net TYPE string;
DEFINE FIELD IF NOT EXISTS properties ON petri_net TYPE object FLEXIBLE DEFAULT {};
DEFINE FIELD IF NOT EXISTS created_at ON petri_net TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_petri_name ON petri_net FIELDS name;

-- Petri net place: linked to a petri_net record
DEFINE TABLE IF NOT EXISTS petri_place SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS net ON petri_place TYPE record<petri_net>;
DEFINE FIELD IF NOT EXISTS position ON petri_place TYPE int;
DEFINE FIELD IF NOT EXISTS label ON petri_place TYPE string;
DEFINE FIELD IF NOT EXISTS label_type ON petri_place TYPE string;
DEFINE FIELD IF NOT EXISTS properties ON petri_place TYPE object FLEXIBLE DEFAULT {};
DEFINE INDEX IF NOT EXISTS idx_place_net ON petri_place FIELDS net;

-- Petri net transition: linked to a petri_net record
DEFINE TABLE IF NOT EXISTS petri_transition SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS net ON petri_transition TYPE record<petri_net>;
DEFINE FIELD IF NOT EXISTS position ON petri_transition TYPE int;
DEFINE FIELD IF NOT EXISTS properties ON petri_transition TYPE object FLEXIBLE DEFAULT {};
DEFINE INDEX IF NOT EXISTS idx_transition_net ON petri_transition FIELDS net;

-- Pre-arc: place -> transition with weight
DEFINE TABLE IF NOT EXISTS pre_arc SCHEMAFULL TYPE RELATION FROM petri_place TO petri_transition;
DEFINE FIELD IF NOT EXISTS weight ON pre_arc TYPE decimal;

-- Post-arc: transition -> place with weight
DEFINE TABLE IF NOT EXISTS post_arc SCHEMAFULL TYPE RELATION FROM petri_transition TO petri_place;
DEFINE FIELD IF NOT EXISTS weight ON post_arc TYPE decimal;

-- Marking snapshot: token distribution at a point in time
DEFINE TABLE IF NOT EXISTS petri_marking SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS net ON petri_marking TYPE record<petri_net>;
DEFINE FIELD IF NOT EXISTS label ON petri_marking TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS tokens ON petri_marking TYPE object FLEXIBLE DEFAULT {};
DEFINE FIELD IF NOT EXISTS step ON petri_marking TYPE option<int> DEFAULT NONE;
DEFINE FIELD IF NOT EXISTS created_at ON petri_marking TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_marking_net ON petri_marking FIELDS net;
"#;

/// Generate DDL for an HNSW index on graph_node.embedding with configurable dimension.
///
/// The dimension must match the embedding vectors stored on graph_node records.
/// Call this once after `init_schema_v2`, typically via `FingerprintEngine::init_index`.
pub fn hnsw_index_ddl(dimension: u32) -> String {
    format!(
        "DEFINE INDEX IF NOT EXISTS hnsw_fingerprint ON graph_node \
         FIELDS embedding HNSW DIMENSION {dimension} DIST EUCLIDEAN EFC 150 M 16;"
    )
}
