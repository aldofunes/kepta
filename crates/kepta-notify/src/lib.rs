pub mod email;
pub mod push;
pub mod webhook;

use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Notification {
    pub event_type: String,
    pub household_id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub payload: Value,
}

#[async_trait::async_trait]
pub trait NotificationChannel: Send + Sync {
    async fn send(&self, notification: &Notification) -> Result<()>;
}
