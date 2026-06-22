use chrono::{DateTime, NaiveDate, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemCategory {
    Appliance,
    System,
    Fixture,
    Other,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Item {
    pub id: Uuid,
    pub household_id: Uuid,
    pub room_id: Option<Uuid>,
    pub category: ItemCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model_number: Option<String>,
    pub serial_number: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub install_date: Option<NaiveDate>,
    pub warranty_expiry: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateItemRequest {
    pub category: ItemCategory,
    pub name: String,
    pub room_id: Option<Uuid>,
    pub brand: Option<String>,
    pub model_number: Option<String>,
    pub serial_number: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub install_date: Option<NaiveDate>,
    pub warranty_expiry: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateItemRequest {
    pub category: Option<ItemCategory>,
    pub name: Option<String>,
    pub room_id: Option<Uuid>,
    pub brand: Option<String>,
    pub model_number: Option<String>,
    pub serial_number: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub install_date: Option<NaiveDate>,
    pub warranty_expiry: Option<NaiveDate>,
    pub notes: Option<String>,
}
