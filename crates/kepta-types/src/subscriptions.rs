use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Plan {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub price_cents: i32,
    pub household_limit: Option<i32>,
    pub storage_limit_mb: Option<i32>,
    pub document_limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
    PastDue,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Subscription {
    pub id: Uuid,
    pub plan: Plan,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CheckoutSession {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BillingPortalSession {
    pub url: String,
}
