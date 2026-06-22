use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntityType {
    Item,
    Room,
    MaintenanceSchedule,
    MaintenanceLog,
    Document,
    ServiceProviderContact,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SyncPull {
    pub since: Option<DateTime<Utc>>,
    pub items: Vec<Value>,
    pub rooms: Vec<Value>,
    pub schedules: Vec<Value>,
    pub logs: Vec<Value>,
    pub contacts: Vec<Value>,
    pub deleted_ids: Vec<DeletedRecord>,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeletedRecord {
    pub id: Uuid,
    pub entity_type: SyncEntityType,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SyncPush {
    pub writes: Vec<PendingWrite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PendingWrite {
    pub id: Uuid,
    pub entity_type: SyncEntityType,
    pub entity_id: Uuid,
    pub operation: WriteOperation,
    pub payload: Value,
    pub client_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WriteOperation {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SyncResult {
    pub accepted: Vec<Uuid>,
    pub rejected: Vec<RejectedWrite>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RejectedWrite {
    pub write_id: Uuid,
    pub reason: String,
}
