use crate::Db;
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct DocumentRow {
    pub id: Uuid,
    pub household_id: Uuid,
    pub item_id: Option<Uuid>,
    pub doc_type: String,
    pub title: String,
    pub file_key: String,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub ocr_status: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list(db: &Db, household_id: Uuid) -> Result<Vec<DocumentRow>> {
    let rows = sqlx::query_as!(
        DocumentRow,
        r#"
        SELECT id, household_id, item_id, type AS doc_type, title, file_key,
               file_size_bytes, mime_type, ocr_status, uploaded_by, created_at, updated_at
        FROM documents
        WHERE household_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
        household_id
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn count_by_household(db: &Db, household_id: Uuid) -> Result<i64> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM documents WHERE household_id = $1 AND deleted_at IS NULL",
        household_id
    )
    .fetch_one(db)
    .await?;
    Ok(count.unwrap_or(0))
}

pub async fn total_size_bytes(db: &Db, household_id: Uuid) -> Result<i64> {
    let total = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(file_size_bytes), 0)::BIGINT FROM documents WHERE household_id = $1 AND deleted_at IS NULL",
        household_id
    )
    .fetch_one(db)
    .await?;
    Ok(total.unwrap_or(0))
}
