use crate::Db;
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub password_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_email(db: &Db, email: &str) -> Result<Option<UserRow>> {
    let row = sqlx::query_as!(
        UserRow,
        "SELECT id, email, display_name, password_hash, avatar_url, created_at, updated_at FROM users WHERE email = $1",
        email
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}

pub async fn find_by_id(db: &Db, id: Uuid) -> Result<Option<UserRow>> {
    let row = sqlx::query_as!(
        UserRow,
        "SELECT id, email, display_name, password_hash, avatar_url, created_at, updated_at FROM users WHERE id = $1",
        id
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}

pub async fn create(
    db: &Db,
    email: &str,
    password_hash: Option<&str>,
    display_name: Option<&str>,
) -> Result<UserRow> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, display_name, password_hash, avatar_url, created_at, updated_at
        "#,
        email, password_hash, display_name
    )
    .fetch_one(db)
    .await?;
    Ok(row)
}

pub async fn upsert_oauth(
    db: &Db,
    user_id: Uuid,
    provider: &str,
    provider_sub: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO oauth_accounts (user_id, provider, provider_sub)
        VALUES ($1, $2, $3)
        ON CONFLICT (provider, provider_sub) DO NOTHING
        "#,
        user_id, provider, provider_sub
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn find_by_oauth(db: &Db, provider: &str, provider_sub: &str) -> Result<Option<UserRow>> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT u.id, u.email, u.display_name, u.password_hash, u.avatar_url, u.created_at, u.updated_at
        FROM users u
        JOIN oauth_accounts oa ON oa.user_id = u.id
        WHERE oa.provider = $1 AND oa.provider_sub = $2
        "#,
        provider, provider_sub
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}
