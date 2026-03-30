use catgraph::cospan::Cospan;
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use crate::error::PersistError;
use crate::persist::Persistable;
use crate::types::CospanRecord;

/// Typed store for `Cospan<Lambda>` persistence in SurrealDB.
pub struct CospanStore<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> CospanStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Save a Cospan, returning the generated RecordId.
    pub async fn save<Lambda: Persistable + Copy>(
        &self,
        cospan: &Cospan<Lambda>,
    ) -> Result<RecordId, PersistError> {
        let record = CospanRecord {
            id: None,
            left_map: cospan.left_to_middle().iter().map(|&i| i as i64).collect(),
            right_map: cospan.right_to_middle().iter().map(|&i| i as i64).collect(),
            middle_labels: cospan
                .middle()
                .iter()
                .map(|l| l.to_json_value().to_string())
                .collect(),
            label_type: Lambda::type_name().to_string(),
            is_left_id: cospan.is_left_identity(),
            is_right_id: cospan.is_right_identity(),
        };

        let created: Option<CospanRecord> = self.db.create("cospan").content(record).await?;
        let created = created
            .ok_or_else(|| PersistError::InvalidData("failed to create cospan record".into()))?;
        created
            .id
            .ok_or_else(|| PersistError::InvalidData("created record has no id".into()))
    }

    /// Load a Cospan by RecordId.
    pub async fn load<Lambda: Persistable + Copy>(
        &self,
        id: &RecordId,
    ) -> Result<Cospan<Lambda>, PersistError> {
        let record: Option<CospanRecord> = self.db.select(id).await?;
        let record = record.ok_or_else(|| PersistError::NotFound(format!("{id:?}")))?;

        if record.label_type != Lambda::type_name() {
            return Err(PersistError::TypeMismatch {
                expected: Lambda::type_name().into(),
                got: record.label_type,
            });
        }

        let left: Vec<usize> = record
            .left_map
            .iter()
            .map(|&i| {
                usize::try_from(i)
                    .map_err(|_| PersistError::InvalidData(format!("negative index {i} in left_map")))
            })
            .collect::<Result<_, _>>()?;
        let right: Vec<usize> = record
            .right_map
            .iter()
            .map(|&i| {
                usize::try_from(i)
                    .map_err(|_| PersistError::InvalidData(format!("negative index {i} in right_map")))
            })
            .collect::<Result<_, _>>()?;
        let middle: Vec<Lambda> = record
            .middle_labels
            .iter()
            .map(|s| {
                let v: serde_json::Value = serde_json::from_str(s)
                    .map_err(|e| PersistError::InvalidData(e.to_string()))?;
                Lambda::from_json_value(&v)
            })
            .collect::<Result<_, _>>()?;

        Ok(Cospan::new(left, right, middle))
    }

    /// Delete a Cospan by RecordId.
    pub async fn delete(&self, id: &RecordId) -> Result<(), PersistError> {
        let _: Option<CospanRecord> = self.db.delete(id).await?;
        Ok(())
    }

    /// List all Cospan RecordIds.
    pub async fn list(&self) -> Result<Vec<RecordId>, PersistError> {
        let records: Vec<CospanRecord> = self.db.select("cospan").await?;
        Ok(records.into_iter().filter_map(|r| r.id).collect())
    }
}
