use chrono::{DateTime, NaiveDate, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FrequencyUnit {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleSource {
    Manual,
    AiSuggestion,
    Template,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MaintenanceSchedule {
    pub id: Uuid,
    pub item_id: Uuid,
    pub household_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub frequency_unit: FrequencyUnit,
    pub frequency_value: i32,
    pub next_due_at: NaiveDate,
    pub last_completed_at: Option<NaiveDate>,
    pub source: ScheduleSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateScheduleRequest {
    pub item_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub frequency_unit: FrequencyUnit,
    pub frequency_value: i32,
    pub next_due_at: NaiveDate,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateScheduleRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub frequency_unit: Option<FrequencyUnit>,
    pub frequency_value: Option<i32>,
    pub next_due_at: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CompleteScheduleRequest {
    pub notes: Option<String>,
    pub cost_cents: Option<i32>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MaintenanceLog {
    pub id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub household_id: Uuid,
    pub item_id: Option<Uuid>,
    pub completed_by: Uuid,
    pub completed_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub cost_cents: Option<i32>,
    pub currency: String,
}
