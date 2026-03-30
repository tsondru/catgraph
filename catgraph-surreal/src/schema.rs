/// SurrealQL DDL for the catgraph persistence schema.
/// Uses SCHEMAFULL tables with embedded arrays for O(1) reconstruction.
pub const SCHEMA_DDL: &str = r#"
DEFINE TABLE IF NOT EXISTS vertex SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS label ON vertex TYPE string;
DEFINE FIELD IF NOT EXISTS label_type ON vertex TYPE string;

DEFINE TABLE IF NOT EXISTS cospan SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS left_map ON cospan TYPE array<int>;
DEFINE FIELD IF NOT EXISTS right_map ON cospan TYPE array<int>;
DEFINE FIELD IF NOT EXISTS middle_labels ON cospan TYPE array<string>;
DEFINE FIELD IF NOT EXISTS label_type ON cospan TYPE string;
DEFINE FIELD IF NOT EXISTS is_left_id ON cospan TYPE bool;
DEFINE FIELD IF NOT EXISTS is_right_id ON cospan TYPE bool;
DEFINE FIELD IF NOT EXISTS created_at ON cospan TYPE datetime DEFAULT time::now();

DEFINE TABLE IF NOT EXISTS named_cospan SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS cospan_ref ON named_cospan TYPE record<cospan>;
DEFINE FIELD IF NOT EXISTS left_names ON named_cospan TYPE array<string>;
DEFINE FIELD IF NOT EXISTS right_names ON named_cospan TYPE array<string>;
DEFINE FIELD IF NOT EXISTS created_at ON named_cospan TYPE datetime DEFAULT time::now();

DEFINE TABLE IF NOT EXISTS span SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS left_labels ON span TYPE array<string>;
DEFINE FIELD IF NOT EXISTS right_labels ON span TYPE array<string>;
DEFINE FIELD IF NOT EXISTS middle_pairs ON span TYPE array<array<int>>;
DEFINE FIELD IF NOT EXISTS label_type ON span TYPE string;
DEFINE FIELD IF NOT EXISTS is_left_id ON span TYPE bool;
DEFINE FIELD IF NOT EXISTS is_right_id ON span TYPE bool;
DEFINE FIELD IF NOT EXISTS created_at ON span TYPE datetime DEFAULT time::now();
"#;
