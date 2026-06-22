use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Household {
    pub id: Uuid,
    pub name: String,
    pub address: Address,
    pub year_built: Option<i32>,
    pub sqft: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateHouseholdRequest {
    pub name: String,
    pub address: Option<Address>,
    pub year_built: Option<i32>,
    pub sqft: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateHouseholdRequest {
    pub name: Option<String>,
    pub address: Option<Address>,
    pub year_built: Option<i32>,
    pub sqft: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Member,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HouseholdMember {
    pub user_id: Uuid,
    pub household_id: Uuid,
    pub role: MemberRole,
    pub email: String,
    pub display_name: Option<String>,
    pub invited_at: DateTime<Utc>,
    pub joined_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: Option<MemberRole>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Room {
    pub id: Uuid,
    pub household_id: Uuid,
    pub name: String,
    pub floor: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateRoomRequest {
    pub name: String,
    pub floor: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateRoomRequest {
    pub name: Option<String>,
    pub floor: Option<i32>,
}
