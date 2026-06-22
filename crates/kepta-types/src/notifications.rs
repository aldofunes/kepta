use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Push,
    Webhook,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NotificationPreference {
    pub channel: NotificationChannel,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateNotificationPreferenceRequest {
    pub channel: NotificationChannel,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Webhook {
    pub id: Uuid,
    pub household_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateWebhookRequest {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

pub mod events {
    pub const MAINTENANCE_DUE: &str = "maintenance.due";
    pub const MAINTENANCE_OVERDUE: &str = "maintenance.overdue";
    pub const DOCUMENT_UPLOADED: &str = "document.uploaded";
    pub const MEMBER_INVITED: &str = "member.invited";
    pub const MEMBER_JOINED: &str = "member.joined";
    pub const WARRANTY_EXPIRING: &str = "warranty.expiring";
    pub const SUBSCRIPTION_EXPIRING: &str = "subscription.expiring";
}
