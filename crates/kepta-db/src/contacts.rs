use crate::Db;
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct ContactRow {
    pub id: Uuid,
    pub household_id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list(db: &Db, household_id: Uuid) -> Result<Vec<ContactRow>> {
    let rows = sqlx::query_as!(
        ContactRow,
        r#"
        SELECT id, household_id, name, type AS provider_type, phone, email,
               website, notes, created_at, updated_at
        FROM service_provider_contacts
        WHERE household_id = $1 AND deleted_at IS NULL
        ORDER BY name ASC
        "#,
        household_id
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}
