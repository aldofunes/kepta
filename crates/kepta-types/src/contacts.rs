use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Plumber,
    Electrician,
    Cleaner,
    Hvac,
    General,
    Other,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ServiceProviderContact {
    pub id: Uuid,
    pub household_id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateContactRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: Option<ProviderType>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateContactRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub provider_type: Option<ProviderType>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
}
