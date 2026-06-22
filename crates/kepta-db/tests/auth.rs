mod common;

use kepta_auth::password;
use kepta_db::Db;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn create_user_and_find_by_email(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    let found = kepta_db::auth::find_by_email(&db, &user.email).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, user.id);
    assert_eq!(found.email, user.email);
}

#[sqlx::test(migrations = "./migrations")]
async fn find_by_email_returns_none_for_unknown(db: PgPool) {
    let result = kepta_db::auth::find_by_email(&db, "nobody@test.kepta.dev")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_email_fails(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    let result = kepta_db::auth::create(&db, &user.email, None, None).await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "./migrations")]
async fn find_by_id_returns_user(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    let found = kepta_db::auth::find_by_id(&db, user.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, user.email);
}

#[sqlx::test(migrations = "./migrations")]
async fn find_by_id_unknown_returns_none(db: PgPool) {
    let result = kepta_db::auth::find_by_id(&db, uuid::Uuid::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn password_hash_is_stored_and_verifiable(db: PgPool) {
    let (user, password) = common::insert_user(&db).await;
    let found = kepta_db::auth::find_by_email(&db, &user.email)
        .await
        .unwrap()
        .unwrap();
    let hash = found.password_hash.unwrap();
    assert!(password::verify(password, &hash).unwrap());
    assert!(!password::verify("wrong-password", &hash).unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn oauth_upsert_and_find(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    kepta_db::auth::upsert_oauth(&db, user.id, "google", "google-sub-123")
        .await
        .unwrap();

    let found = kepta_db::auth::find_by_oauth(&db, "google", "google-sub-123")
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user.id);
}

#[sqlx::test(migrations = "./migrations")]
async fn oauth_upsert_is_idempotent(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    // Insert twice — should not error
    kepta_db::auth::upsert_oauth(&db, user.id, "google", "sub-abc").await.unwrap();
    kepta_db::auth::upsert_oauth(&db, user.id, "google", "sub-abc").await.unwrap();

    let found = kepta_db::auth::find_by_oauth(&db, "google", "sub-abc")
        .await
        .unwrap();
    assert!(found.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn unknown_oauth_provider_returns_none(db: PgPool) {
    let result = kepta_db::auth::find_by_oauth(&db, "github", "no-such-sub")
        .await
        .unwrap();
    assert!(result.is_none());
}
