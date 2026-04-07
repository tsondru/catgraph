use catgraph::named_cospan::NamedCospan;
use surrealdb::engine::local::Db;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

use crate::cospan_store::CospanStore;
use crate::error::PersistError;
use crate::persist::Persistable;
use crate::types::NamedCospanRecord;

/// Typed store for `NamedCospan<Lambda, L, R>` persistence in `SurrealDB`.
///
/// Named cospans are stored as a reference to the underlying cospan record
/// plus the port name arrays. The cospan is saved/loaded via `CospanStore`.
pub struct NamedCospanStore<'a> {
    db: &'a Surreal<Db>,
    cospan_store: CospanStore<'a>,
}

impl<'a> NamedCospanStore<'a> {
    #[must_use] 
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self {
            db,
            cospan_store: CospanStore::new(db),
        }
    }

    /// Save a `NamedCospan`, persisting both the underlying cospan and the name arrays.
    /// Returns the `named_cospan` `RecordId`.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::InvalidData`] if the underlying cospan or name
    /// record cannot be created.
    /// Returns [`PersistError::Surreal`] on database communication errors.
    pub async fn save<Lambda>(
        &self,
        nc: &NamedCospan<Lambda, String, String>,
    ) -> Result<RecordId, PersistError>
    where
        Lambda: Persistable + Copy,
    {
        let cospan_id = self.cospan_store.save(nc.cospan()).await?;

        let record = NamedCospanRecord {
            id: None,
            cospan_ref: cospan_id,
            left_names: nc.left_names().clone(),
            right_names: nc.right_names().clone(),
        };

        let created: Option<NamedCospanRecord> =
            self.db.create("named_cospan").content(record).await?;
        let created = created.ok_or_else(|| {
            PersistError::InvalidData("failed to create named_cospan record".into())
        })?;
        created
            .id
            .ok_or_else(|| PersistError::InvalidData("created record has no id".into()))
    }

    /// Load a `NamedCospan` by its `RecordId`.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::NotFound`] if the record does not exist.
    /// Returns [`PersistError::TypeMismatch`] if `Lambda` does not match the
    /// stored label type. Returns [`PersistError::Surreal`] on database errors.
    pub async fn load<Lambda>(
        &self,
        id: &RecordId,
    ) -> Result<NamedCospan<Lambda, String, String>, PersistError>
    where
        Lambda: Persistable + Copy,
    {
        let record: Option<NamedCospanRecord> = self.db.select(id).await?;
        let record = record.ok_or_else(|| PersistError::NotFound(format!("{id:?}")))?;

        let cospan = self.cospan_store.load::<Lambda>(&record.cospan_ref).await?;

        let left = cospan.left_to_middle().to_vec();
        let right = cospan.right_to_middle().to_vec();
        let middle = cospan.middle().to_vec();

        Ok(NamedCospan::new(
            left,
            right,
            middle,
            record.left_names,
            record.right_names,
        ))
    }

    /// Delete a `NamedCospan` and its underlying cospan atomically.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] if the database operation fails.
    pub async fn delete(&self, id: &RecordId) -> Result<(), PersistError> {
        let record: Option<NamedCospanRecord> = self.db.select(id).await?;
        if let Some(record) = record {
            self.db
                .query("BEGIN TRANSACTION; DELETE $cospan_id; DELETE $nc_id; COMMIT TRANSACTION;")
                .bind(("cospan_id", record.cospan_ref))
                .bind(("nc_id", id.clone()))
                .await?;
        }
        Ok(())
    }

    /// List all `NamedCospan` `RecordIds`.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError::Surreal`] if the database operation fails.
    pub async fn list(&self) -> Result<Vec<RecordId>, PersistError> {
        let records: Vec<NamedCospanRecord> = self.db.select("named_cospan").await?;
        Ok(records.into_iter().filter_map(|r| r.id).collect())
    }
}
