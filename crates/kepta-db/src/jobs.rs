use crate::Db;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

pub struct JobRow {
    pub id: Uuid,
    pub job_type: String,
    pub payload: Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub error: Option<String>,
    pub run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn enqueue(db: &Db, job_type: &str, payload: Value) -> Result<Uuid> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO jobs (job_type, payload)
        VALUES ($1, $2)
        RETURNING id
        "#,
        job_type, payload
    )
    .fetch_one(db)
    .await?;
    Ok(id)
}

/// Claim the next pending job using FOR UPDATE SKIP LOCKED.
pub async fn claim_next(db: &Db) -> Result<Option<JobRow>> {
    let row = sqlx::query_as!(
        JobRow,
        r#"
        UPDATE jobs SET status = 'processing', attempts = attempts + 1, updated_at = NOW()
        WHERE id = (
            SELECT id FROM jobs
            WHERE status = 'pending' AND run_at <= NOW()
            ORDER BY run_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, job_type, payload, status, attempts, max_attempts, error, run_at, created_at, updated_at
        "#
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}

pub async fn complete(db: &Db, id: Uuid) -> Result<()> {
    sqlx::query!(
        "UPDATE jobs SET status = 'completed', updated_at = NOW() WHERE id = $1",
        id
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn fail(db: &Db, id: Uuid, error: &str) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = CASE WHEN attempts >= max_attempts THEN 'failed' ELSE 'pending' END,
            error = $2,
            run_at = NOW() + INTERVAL '5 minutes',
            updated_at = NOW()
        WHERE id = $1
        "#,
        id, error
    )
    .execute(db)
    .await?;
    Ok(())
}
