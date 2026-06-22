pub mod households;
pub mod items;
pub mod schedules;
pub mod documents;
pub mod contacts;
pub mod auth;
pub mod subscriptions;
pub mod jobs;

use sqlx::PgPool;

pub type Db = PgPool;

pub async fn connect(url: &str) -> anyhow::Result<Db> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(url)
        .await?;
    Ok(pool)
}

pub async fn migrate(pool: &Db) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
