use anyhow::Result;
use chrono::Utc;
use kepta_localdb::LocalStore;
use kepta_types::sync::SyncPush;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;

pub struct SyncEngine {
    store: Arc<dyn LocalStore>,
    client: Client,
    base_url: String,
    access_token: String,
}

impl SyncEngine {
    pub fn new(
        store: Arc<dyn LocalStore>,
        base_url: String,
        access_token: String,
    ) -> Self {
        Self {
            store,
            client: Client::new(),
            base_url,
            access_token,
        }
    }

    pub async fn sync(&self, household_id: Uuid) -> Result<()> {
        let since = self.store.last_sync_at(household_id).await?;

        // Pull changes from server
        let mut url = format!("{}/api/v1/households/{}/sync", self.base_url, household_id);
        if let Some(since) = since {
            url = format!("{}?since={}", url, since.to_rfc3339());
        }

        let pull = self.client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .error_for_status()?
            .json::<kepta_types::sync::SyncPull>()
            .await?;

        // Apply pulled records locally
        for item in pull.items {
            self.store.upsert_raw(&kepta_types::sync::SyncEntityType::Item, item).await?;
        }
        for room in pull.rooms {
            self.store.upsert_raw(&kepta_types::sync::SyncEntityType::Room, room).await?;
        }
        for schedule in pull.schedules {
            self.store.upsert_raw(&kepta_types::sync::SyncEntityType::MaintenanceSchedule, schedule).await?;
        }
        for contact in pull.contacts {
            self.store.upsert_raw(&kepta_types::sync::SyncEntityType::ServiceProviderContact, contact).await?;
        }
        for deleted in &pull.deleted_ids {
            self.store.delete_entity(&deleted.entity_type, deleted.id).await?;
        }

        // Push local pending writes
        let pending = self.store.get_pending_writes(household_id).await?;
        if !pending.is_empty() {
            let result = self.client
                .post(format!("{}/api/v1/households/{}/sync", self.base_url, household_id))
                .bearer_auth(&self.access_token)
                .json(&SyncPush { writes: pending })
                .send()
                .await?
                .error_for_status()?
                .json::<kepta_types::sync::SyncResult>()
                .await?;

            self.store.mark_writes_accepted(&result.accepted).await?;
        }

        self.store.set_last_sync_at(household_id, pull.server_time).await?;
        Ok(())
    }
}
