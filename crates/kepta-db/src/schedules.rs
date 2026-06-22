use crate::Db;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

pub struct ScheduleRow {
    pub id: Uuid,
    pub item_id: Uuid,
    pub household_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub frequency_unit: String,
    pub frequency_value: i32,
    pub next_due_at: NaiveDate,
    pub last_completed_at: Option<NaiveDate>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list(db: &Db, household_id: Uuid) -> Result<Vec<ScheduleRow>> {
    let rows = sqlx::query_as!(
        ScheduleRow,
        r#"
        SELECT id, item_id, household_id, title, description, frequency_unit, frequency_value,
               next_due_at, last_completed_at, source, created_at, updated_at
        FROM maintenance_schedules
        WHERE household_id = $1 AND deleted_at IS NULL
        ORDER BY next_due_at ASC
        "#,
        household_id
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// Fetch all schedules due on or before `due_by` across all households (for the worker).
pub async fn list_due(db: &Db, due_by: NaiveDate) -> Result<Vec<ScheduleRow>> {
    let rows = sqlx::query_as!(
        ScheduleRow,
        r#"
        SELECT id, item_id, household_id, title, description, frequency_unit, frequency_value,
               next_due_at, last_completed_at, source, created_at, updated_at
        FROM maintenance_schedules
        WHERE next_due_at <= $1 AND deleted_at IS NULL
        ORDER BY next_due_at ASC
        "#,
        due_by
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}
