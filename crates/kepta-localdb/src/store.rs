use anyhow::Result;
use chrono::{DateTime, Utc};
use kepta_types::sync::{PendingWrite, SyncEntityType};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait LocalStore: Send + Sync {
    async fn last_sync_at(&self, household_id: Uuid) -> Result<Option<DateTime<Utc>>>;
    async fn set_last_sync_at(&self, household_id: Uuid, at: DateTime<Utc>) -> Result<()>;

    async fn get_pending_writes(&self, household_id: Uuid) -> Result<Vec<PendingWrite>>;
    async fn mark_writes_accepted(&self, ids: &[Uuid]) -> Result<()>;

    async fn upsert_raw(&self, entity_type: &SyncEntityType, data: serde_json::Value) -> Result<()>;
    async fn delete_entity(&self, entity_type: &SyncEntityType, id: Uuid) -> Result<()>;
}
