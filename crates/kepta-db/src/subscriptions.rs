use crate::Db;
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct PlanRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub price_cents: i32,
    pub household_limit: Option<i32>,
    pub storage_limit_mb: Option<i32>,
    pub document_limit: Option<i32>,
}

pub struct SubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub external_id: Option<String>,
}

pub async fn get_plan_for_user(db: &Db, user_id: Uuid) -> Result<PlanRow> {
    let row = sqlx::query_as!(
        PlanRow,
        r#"
        SELECT p.id, p.slug, p.name, p.price_cents, p.household_limit, p.storage_limit_mb, p.document_limit
        FROM subscription_plans p
        JOIN subscriptions s ON s.plan_id = p.id
        WHERE s.user_id = $1 AND s.status = 'active'
        ORDER BY s.created_at DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(db)
    .await?;

    if let Some(row) = row {
        return Ok(row);
    }

    // Default to free plan
    let free = sqlx::query_as!(
        PlanRow,
        "SELECT id, slug, name, price_cents, household_limit, storage_limit_mb, document_limit FROM subscription_plans WHERE slug = 'free'"
    )
    .fetch_one(db)
    .await?;

    Ok(free)
}

pub async fn list_plans(db: &Db) -> Result<Vec<PlanRow>> {
    let rows = sqlx::query_as!(
        PlanRow,
        "SELECT id, slug, name, price_cents, household_limit, storage_limit_mb, document_limit FROM subscription_plans ORDER BY price_cents ASC"
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}
