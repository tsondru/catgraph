//! Shared helper types and functions used across multiple store modules.

use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;

/// Format a [`RecordId`] as a `"table:key"` string.
///
/// `RecordId` does not implement `Display` or `ToString`, so we match
/// on the key variant and produce a stable string that `RecordId::parse_simple`
/// can round-trip.
pub(crate) fn format_record_id(id: &RecordId) -> String {
    use surrealdb::types::RecordIdKey;
    let table = id.table.as_str();
    match &id.key {
        RecordIdKey::String(s) => format!("{table}:{s}"),
        RecordIdKey::Number(n) => format!("{table}:{n}"),
        RecordIdKey::Uuid(u) => format!("{table}:{u}"),
        other => format!("{table}:{other:?}"),
    }
}

/// Helper for extracting just the `RecordId` from `RELATE ... RETURN id`.
#[derive(Debug, serde::Deserialize, SurrealValue)]
pub(crate) struct IdOnly {
    pub(crate) id: RecordId,
}

/// Helper struct for extracting `out` `RecordId` from edge query results.
#[derive(Debug, serde::Deserialize, SurrealValue)]
pub(crate) struct OutRef {
    pub(crate) out: RecordId,
}

/// Helper struct for extracting source (`in`) `RecordId` from edge query results.
///
/// Uses `src` alias because `SurrealValue` derive does not support `#[serde(rename)]`.
/// The query must use `` SELECT `in` AS src FROM ... ``.
#[derive(Debug, serde::Deserialize, SurrealValue)]
pub(crate) struct InRef {
    pub(crate) src: RecordId,
}

/// Helper struct for extracting both `in` (as `src`) and `out` `RecordId` from edge
/// query results. Used by `shortest_path` to track parent-child relationships.
///
/// The query must alias `in` as `src`: `` SELECT `in` AS src, out FROM ... ``.
#[derive(Debug, serde::Deserialize, SurrealValue)]
pub(crate) struct InOutRef {
    pub(crate) src: RecordId,
    pub(crate) out: RecordId,
}
