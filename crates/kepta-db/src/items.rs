use crate::Db;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

pub struct ItemRow {
    pub id: Uuid,
    pub household_id: Uuid,
    pub room_id: Option<Uuid>,
    pub category: String,
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

pub async fn list(db: &Db, household_id: Uuid) -> Result<Vec<ItemRow>> {
    let rows = sqlx::query_as!(
        ItemRow,
        r#"
        SELECT id, household_id, room_id, category, name, brand, model_number,
               serial_number, purchase_date, install_date, warranty_expiry, notes,
               created_at, updated_at
        FROM items
        WHERE household_id = $1 AND deleted_at IS NULL
        ORDER BY name ASC
        "#,
        household_id
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn find_by_id(db: &Db, id: Uuid, household_id: Uuid) -> Result<Option<ItemRow>> {
    let row = sqlx::query_as!(
        ItemRow,
        r#"
        SELECT id, household_id, room_id, category, name, brand, model_number,
               serial_number, purchase_date, install_date, warranty_expiry, notes,
               created_at, updated_at
        FROM items
        WHERE id = $1 AND household_id = $2 AND deleted_at IS NULL
        "#,
        id, household_id
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}

pub async fn delete(db: &Db, id: Uuid, household_id: Uuid) -> Result<bool> {
    let result = sqlx::query!(
        "UPDATE items SET deleted_at = NOW() WHERE id = $1 AND household_id = $2 AND deleted_at IS NULL",
        id, household_id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
