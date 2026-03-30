use catgraph::span::Span;
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use crate::error::PersistError;
use crate::persist::Persistable;
use crate::types::SpanRecord;

/// Typed store for `Span<Lambda>` persistence in SurrealDB.
pub struct SpanStore<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> SpanStore<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Save a Span, returning the generated RecordId.
    pub async fn save<Lambda: Persistable + Copy>(
        &self,
        span: &Span<Lambda>,
    ) -> Result<RecordId, PersistError> {
        let record = SpanRecord {
            id: None,
            left_labels: span
                .left()
                .iter()
                .map(|l| l.to_json_value().to_string())
                .collect(),
            right_labels: span
                .right()
                .iter()
                .map(|l| l.to_json_value().to_string())
                .collect(),
            middle_pairs: span
                .middle_pairs()
                .iter()
                .map(|&(l, r)| vec![l as i64, r as i64])
                .collect(),
            label_type: Lambda::type_name().to_string(),
            is_left_id: span.is_left_identity(),
            is_right_id: span.is_right_identity(),
        };

        let created: Option<SpanRecord> = self.db.create("span").content(record).await?;
        let created = created
            .ok_or_else(|| PersistError::InvalidData("failed to create span record".into()))?;
        created
            .id
            .ok_or_else(|| PersistError::InvalidData("created record has no id".into()))
    }

    /// Load a Span by RecordId.
    pub async fn load<Lambda: Persistable + Copy>(
        &self,
        id: &RecordId,
    ) -> Result<Span<Lambda>, PersistError> {
        let record: Option<SpanRecord> = self.db.select(id).await?;
        let record = record.ok_or_else(|| PersistError::NotFound(format!("{id:?}")))?;

        if record.label_type != Lambda::type_name() {
            return Err(PersistError::TypeMismatch {
                expected: Lambda::type_name().into(),
                got: record.label_type,
            });
        }

        let left: Vec<Lambda> = record
            .left_labels
            .iter()
            .map(|s| {
                let v: serde_json::Value = serde_json::from_str(s)
                    .map_err(|e| PersistError::InvalidData(e.to_string()))?;
                Lambda::from_json_value(&v)
            })
            .collect::<Result<_, _>>()?;

        let right: Vec<Lambda> = record
            .right_labels
            .iter()
            .map(|s| {
                let v: serde_json::Value = serde_json::from_str(s)
                    .map_err(|e| PersistError::InvalidData(e.to_string()))?;
                Lambda::from_json_value(&v)
            })
            .collect::<Result<_, _>>()?;

        let middle: Vec<(usize, usize)> = record
            .middle_pairs
            .iter()
            .map(|pair| {
                if pair.len() != 2 {
                    return Err(PersistError::InvalidData(format!(
                        "expected pair of length 2, got {}",
                        pair.len()
                    )));
                }
                let l = usize::try_from(pair[0])
                    .map_err(|_| PersistError::InvalidData(format!("negative index {} in middle_pair", pair[0])))?;
                let r = usize::try_from(pair[1])
                    .map_err(|_| PersistError::InvalidData(format!("negative index {} in middle_pair", pair[1])))?;
                Ok((l, r))
            })
            .collect::<Result<_, _>>()?;

        Ok(Span::new(left, right, middle))
    }

    /// Delete a Span by RecordId.
    pub async fn delete(&self, id: &RecordId) -> Result<(), PersistError> {
        let _: Option<SpanRecord> = self.db.delete(id).await?;
        Ok(())
    }

    /// List all Span RecordIds.
    pub async fn list(&self) -> Result<Vec<RecordId>, PersistError> {
        let records: Vec<SpanRecord> = self.db.select("span").await?;
        Ok(records.into_iter().filter_map(|r| r.id).collect())
    }
}
