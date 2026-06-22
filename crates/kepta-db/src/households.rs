use crate::Db;
use anyhow::Result;
use kepta_types::households::{Household, MemberRole};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct HouseholdRow {
    pub id: Uuid,
    pub name: String,
    pub address: Value,
    pub year_built: Option<i32>,
    pub sqft: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_user(db: &Db, user_id: Uuid) -> Result<Vec<HouseholdRow>> {
    let rows = sqlx::query_as!(
        HouseholdRow,
        r#"
        SELECT h.id, h.name, h.address, h.year_built, h.sqft, h.created_at, h.updated_at
        FROM households h
        JOIN household_members hm ON hm.household_id = h.id
        WHERE hm.user_id = $1 AND hm.joined_at IS NOT NULL
        ORDER BY h.created_at DESC
        "#,
        user_id
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn find_by_id(db: &Db, id: Uuid) -> Result<Option<HouseholdRow>> {
    let row = sqlx::query_as!(
        HouseholdRow,
        "SELECT id, name, address, year_built, sqft, created_at, updated_at FROM households WHERE id = $1",
        id
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}

pub async fn create(
    db: &Db,
    user_id: Uuid,
    name: &str,
    address: Value,
    year_built: Option<i32>,
    sqft: Option<i32>,
) -> Result<HouseholdRow> {
    let mut tx = db.begin().await?;

    let row = sqlx::query_as!(
        HouseholdRow,
        r#"
        INSERT INTO households (name, address, year_built, sqft)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, address, year_built, sqft, created_at, updated_at
        "#,
        name, address, year_built, sqft
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO household_members (household_id, user_id, role, joined_at)
        VALUES ($1, $2, 'owner', NOW())
        "#,
        row.id, user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(row)
}

pub async fn count_by_user(db: &Db, user_id: Uuid) -> Result<i64> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM household_members WHERE user_id = $1 AND role = 'owner' AND joined_at IS NOT NULL",
        user_id
    )
    .fetch_one(db)
    .await?;
    Ok(count.unwrap_or(0))
}

pub async fn is_member(db: &Db, household_id: Uuid, user_id: Uuid) -> Result<bool> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM household_members WHERE household_id = $1 AND user_id = $2 AND joined_at IS NOT NULL)",
        household_id, user_id
    )
    .fetch_one(db)
    .await?;
    Ok(exists.unwrap_or(false))
}

pub async fn get_role(db: &Db, household_id: Uuid, user_id: Uuid) -> Result<Option<MemberRole>> {
    let role = sqlx::query_scalar!(
        "SELECT role FROM household_members WHERE household_id = $1 AND user_id = $2 AND joined_at IS NOT NULL",
        household_id, user_id
    )
    .fetch_optional(db)
    .await?;

    Ok(role.map(|r| match r.as_str() {
        "owner" => MemberRole::Owner,
        _ => MemberRole::Member,
    }))
}
