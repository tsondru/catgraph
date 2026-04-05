//! V2 persistence for [`WiringDiagram`] via hub-node reification.
//!
//! Decomposes the underlying [`NamedCospan`] into a hyperedge hub,
//! storing inner-circle port metadata `(Dir, InterCircle, IntraCircle)`
//! and outer-circle port metadata `(Dir, IntraCircle)` in hub properties.
//!
//! [`Dir`] is not serde-serializable (catgraph core has no serde dependency),
//! so the store converts it to/from string representation manually.

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use catgraph::cospan::Cospan;
use catgraph::named_cospan::NamedCospan;
use catgraph::wiring_diagram::{Dir, WiringDiagram};

use crate::error::PersistError;
use crate::hyperedge::HyperedgeStore;
use crate::persist::Persistable;
use crate::types_v2::HyperedgeHubRecord;

/// V2 persistence for [`WiringDiagram`] via hub-node reification.
///
/// Saves the underlying cospan structure through [`HyperedgeStore::decompose_cospan`],
/// with port name metadata serialized in the hub's `properties` JSON.
pub struct WiringDiagramStore<'a> {
    hyperedge_store: HyperedgeStore<'a>,
    db: &'a Surreal<Db>,
}

impl<'a> WiringDiagramStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self {
            hyperedge_store: HyperedgeStore::new(db),
            db,
        }
    }

    /// Save a [`WiringDiagram`] by decomposing its inner [`NamedCospan`].
    ///
    /// Port name metadata is stored in the hub's properties JSON.
    /// `Dir` is serialized as `"In"`, `"Out"`, or `"Undirected"`.
    pub async fn save<Lambda, InterCircle, IntraCircle>(
        &self,
        diagram: &WiringDiagram<Lambda, InterCircle, IntraCircle>,
        name: &str,
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
        InterCircle: Eq + Clone + Debug + Serialize,
        IntraCircle: Eq + Clone + Debug + Serialize,
    {
        let inner = diagram.inner();

        let left_names_json = serialize_left_names(inner.left_names())?;
        let right_names_json = serialize_right_names(inner.right_names())?;

        let props = serde_json::json!({
            "diagram_name": name,
            "left_port_names": left_names_json,
            "right_port_names": right_names_json,
        });

        let hub_id = self
            .hyperedge_store
            .decompose_cospan(inner.cospan(), "wiring_diagram", props, |l| {
                format!("{l:?}")
            })
            .await?;

        Ok(hub_id)
    }

    /// Reconstruct a [`WiringDiagram`] from a stored hub record.
    ///
    /// Rebuilds the underlying cospan via [`HyperedgeStore::reconstruct_cospan`]
    /// and restores port name metadata from hub properties.
    pub async fn load<Lambda, InterCircle, IntraCircle>(
        &self,
        hub_id: &RecordId,
    ) -> Result<WiringDiagram<Lambda, InterCircle, IntraCircle>, PersistError>
    where
        Lambda: Persistable + Copy,
        InterCircle: Eq + Clone + Debug + DeserializeOwned,
        IntraCircle: Eq + Clone + Debug + DeserializeOwned,
    {
        let hub = self.hyperedge_store.get_hub(hub_id).await?;
        if hub.kind != "wiring_diagram" {
            return Err(PersistError::InvalidData(format!(
                "hub kind '{}' is not 'wiring_diagram'",
                hub.kind
            )));
        }

        let cospan: Cospan<Lambda> = self.hyperedge_store.reconstruct_cospan(hub_id).await?;

        let left_names = deserialize_left_names(&hub.properties)?;
        let right_names = deserialize_right_names(&hub.properties)?;

        let named = NamedCospan::new(
            cospan.left_to_middle().to_vec(),
            cospan.right_to_middle().to_vec(),
            cospan.middle().to_vec(),
            left_names,
            right_names,
        );

        Ok(WiringDiagram::new(named))
    }

    /// Get the hub record for a stored wiring diagram.
    pub async fn get_hub(&self, hub_id: &RecordId) -> Result<HyperedgeHubRecord, PersistError> {
        self.hyperedge_store.get_hub(hub_id).await
    }

    /// Delete a stored wiring diagram and its participation edges.
    pub async fn delete(&self, hub_id: &RecordId) -> Result<(), PersistError> {
        self.hyperedge_store.delete_hub(hub_id).await
    }

    /// List all wiring diagram hubs.
    pub async fn list(&self) -> Result<Vec<HyperedgeHubRecord>, PersistError> {
        let mut result = self
            .db
            .query("SELECT * FROM hyperedge_hub WHERE kind = 'wiring_diagram'")
            .await?;
        let hubs: Vec<HyperedgeHubRecord> = result.take(0)?;
        Ok(hubs)
    }
}

// ---------------------------------------------------------------------------
// Dir ↔ JSON helpers (catgraph's Dir has no serde derives)
// ---------------------------------------------------------------------------

fn dir_to_str(d: &Dir) -> &'static str {
    match d {
        Dir::In => "In",
        Dir::Out => "Out",
        Dir::Undirected => "Undirected",
    }
}

fn dir_from_str(s: &str) -> Result<Dir, PersistError> {
    match s {
        "In" => Ok(Dir::In),
        "Out" => Ok(Dir::Out),
        "Undirected" => Ok(Dir::Undirected),
        other => Err(PersistError::InvalidData(format!(
            "unknown Dir variant: '{other}'"
        ))),
    }
}

/// Serialize left port names `Vec<(Dir, InterCircle, IntraCircle)>` to JSON.
fn serialize_left_names<InterCircle, IntraCircle>(
    names: &[(Dir, InterCircle, IntraCircle)],
) -> Result<serde_json::Value, PersistError>
where
    InterCircle: Serialize,
    IntraCircle: Serialize,
{
    let entries: Vec<serde_json::Value> = names
        .iter()
        .map(|(d, inter, intra)| {
            Ok(serde_json::json!({
                "dir": dir_to_str(d),
                "inter": serde_json::to_value(inter)
                    .map_err(|e| PersistError::InvalidData(format!("serialize inter: {e}")))?,
                "intra": serde_json::to_value(intra)
                    .map_err(|e| PersistError::InvalidData(format!("serialize intra: {e}")))?,
            }))
        })
        .collect::<Result<_, PersistError>>()?;
    Ok(serde_json::Value::Array(entries))
}

/// Serialize right port names `Vec<(Dir, IntraCircle)>` to JSON.
fn serialize_right_names<IntraCircle>(
    names: &[(Dir, IntraCircle)],
) -> Result<serde_json::Value, PersistError>
where
    IntraCircle: Serialize,
{
    let entries: Vec<serde_json::Value> = names
        .iter()
        .map(|(d, intra)| {
            Ok(serde_json::json!({
                "dir": dir_to_str(d),
                "intra": serde_json::to_value(intra)
                    .map_err(|e| PersistError::InvalidData(format!("serialize intra: {e}")))?,
            }))
        })
        .collect::<Result<_, PersistError>>()?;
    Ok(serde_json::Value::Array(entries))
}

/// Deserialize left port names from hub properties JSON.
fn deserialize_left_names<InterCircle, IntraCircle>(
    properties: &serde_json::Value,
) -> Result<Vec<(Dir, InterCircle, IntraCircle)>, PersistError>
where
    InterCircle: DeserializeOwned,
    IntraCircle: DeserializeOwned,
{
    let arr = properties
        .get("left_port_names")
        .and_then(|v| v.as_array())
        .ok_or_else(|| PersistError::InvalidData("missing 'left_port_names' array".into()))?;

    arr.iter()
        .map(|entry| {
            let dir_str = entry
                .get("dir")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PersistError::InvalidData("port entry missing 'dir'".into()))?;
            let dir = dir_from_str(dir_str)?;

            let inter_val = entry
                .get("inter")
                .ok_or_else(|| PersistError::InvalidData("port entry missing 'inter'".into()))?;
            let inter: InterCircle = serde_json::from_value(inter_val.clone())
                .map_err(|e| PersistError::InvalidData(format!("deserialize inter: {e}")))?;

            let intra_val = entry
                .get("intra")
                .ok_or_else(|| PersistError::InvalidData("port entry missing 'intra'".into()))?;
            let intra: IntraCircle = serde_json::from_value(intra_val.clone())
                .map_err(|e| PersistError::InvalidData(format!("deserialize intra: {e}")))?;

            Ok((dir, inter, intra))
        })
        .collect()
}

/// Deserialize right port names from hub properties JSON.
fn deserialize_right_names<IntraCircle>(
    properties: &serde_json::Value,
) -> Result<Vec<(Dir, IntraCircle)>, PersistError>
where
    IntraCircle: DeserializeOwned,
{
    let arr = properties
        .get("right_port_names")
        .and_then(|v| v.as_array())
        .ok_or_else(|| PersistError::InvalidData("missing 'right_port_names' array".into()))?;

    arr.iter()
        .map(|entry| {
            let dir_str = entry
                .get("dir")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PersistError::InvalidData("port entry missing 'dir'".into()))?;
            let dir = dir_from_str(dir_str)?;

            let intra_val = entry
                .get("intra")
                .ok_or_else(|| PersistError::InvalidData("port entry missing 'intra'".into()))?;
            let intra: IntraCircle = serde_json::from_value(intra_val.clone())
                .map_err(|e| PersistError::InvalidData(format!("deserialize intra: {e}")))?;

            Ok((dir, intra))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_roundtrip() {
        for d in [Dir::In, Dir::Out, Dir::Undirected] {
            let s = dir_to_str(&d);
            assert_eq!(dir_from_str(s).unwrap(), d);
        }
    }

    #[test]
    fn dir_invalid() {
        assert!(dir_from_str("bogus").is_err());
    }

    #[test]
    fn left_names_roundtrip() {
        let names = vec![
            (Dir::In, 0_i32, 10_i32),
            (Dir::Out, 1, 20),
            (Dir::Undirected, 2, 30),
        ];
        let json = serialize_left_names(&names).unwrap();
        let props = serde_json::json!({ "left_port_names": json });
        let restored: Vec<(Dir, i32, i32)> = deserialize_left_names(&props).unwrap();
        assert_eq!(names, restored);
    }

    #[test]
    fn right_names_roundtrip() {
        let names = vec![(Dir::Out, 0_usize), (Dir::In, 1)];
        let json = serialize_right_names(&names).unwrap();
        let props = serde_json::json!({ "right_port_names": json });
        let restored: Vec<(Dir, usize)> = deserialize_right_names(&props).unwrap();
        assert_eq!(names, restored);
    }
}
