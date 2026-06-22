use anyhow::Result;
use chrono::{DateTime, Utc};
use kepta_types::sync::{PendingWrite, SyncEntityType};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::store::LocalStore;

pub struct SqliteStore {
    db: SqlitePool,
}

impl SqliteStore {
    pub async fn new(path: &str) -> Result<Self> {
        let pool = SqlitePool::connect(path).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { db: pool })
    }
}

#[async_trait::async_trait]
impl LocalStore for SqliteStore {
    async fn last_sync_at(&self, household_id: Uuid) -> Result<Option<DateTime<Utc>>> {
        let id = household_id.to_string();
        let row = sqlx::query_scalar!(
            "SELECT last_sync_at FROM sync_state WHERE household_id = ?",
            id
        )
        .fetch_optional(&self.db)
        .await?;
        Ok(row.flatten().and_then(|s| s.parse().ok()))
    }

    async fn set_last_sync_at(&self, household_id: Uuid, at: DateTime<Utc>) -> Result<()> {
        let id = household_id.to_string();
        let at_str = at.to_rfc3339();
        sqlx::query!(
            "INSERT OR REPLACE INTO sync_state (household_id, last_sync_at) VALUES (?, ?)",
            id, at_str
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_pending_writes(&self, _household_id: Uuid) -> Result<Vec<PendingWrite>> {
        // TODO: query pending_writes table
        Ok(vec![])
    }

    async fn mark_writes_accepted(&self, _ids: &[Uuid]) -> Result<()> {
        Ok(())
    }

    async fn upsert_raw(&self, _entity_type: &SyncEntityType, _data: serde_json::Value) -> Result<()> {
        Ok(())
    }

    async fn delete_entity(&self, _entity_type: &SyncEntityType, _id: Uuid) -> Result<()> {
        Ok(())
    }
}
