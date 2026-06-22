use kepta_auth::password;
use kepta_db::{auth::UserRow, households::HouseholdRow, Db};
use serde_json::json;
use uuid::Uuid;

/// Create a user with a known password. Email is unique per call.
pub async fn insert_user(db: &Db) -> (UserRow, &'static str) {
    let email = format!("user-{}@test.kepta.dev", Uuid::now_v7());
    const PASSWORD: &str = "test-password-123";
    let hash = password::hash(PASSWORD).unwrap();
    let user = kepta_db::auth::create(db, &email, Some(&hash), Some("Test User"))
        .await
        .unwrap();
    (user, PASSWORD)
}

/// Create a household owned by `user_id`.
pub async fn insert_household(db: &Db, user_id: Uuid) -> HouseholdRow {
    kepta_db::households::create(
        db,
        user_id,
        "Test Home",
        json!({"city": "Springfield", "country": "US"}),
        Some(1990),
        Some(1500),
    )
    .await
    .unwrap()
}

/// Create a second member (non-owner) in a household.
pub async fn invite_and_join(db: &Db, household_id: Uuid, user_id: Uuid) {
    sqlx::query!(
        "INSERT INTO household_members (household_id, user_id, role, joined_at)
         VALUES ($1, $2, 'member', NOW())",
        household_id,
        user_id
    )
    .execute(db)
    .await
    .unwrap();
}
