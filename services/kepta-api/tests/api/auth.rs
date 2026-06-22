use crate::helpers::{register_user, spawn_app};
use serde_json::json;

// ── POST /api/v1/auth/register ────────────────────────────────────────────────

#[tokio::test]
async fn register_returns_200_with_token_and_profile() {
    let app = spawn_app().await;

    let res = app
        .register(&json!({
            "email": "new@example.com",
            "password": "strong-password-123!"
        }))
        .await;

    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["access_token"].as_str().is_some(), "token must be present");
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["user"]["email"], "new@example.com");
}

#[tokio::test]
async fn register_rejects_invalid_email() {
    let app = spawn_app().await;
    let res = app
        .register(&json!({ "email": "not-an-email", "password": "strong-password-123!" }))
        .await;
    assert_eq!(res.status().as_u16(), 400);
}

#[tokio::test]
async fn register_rejects_short_password() {
    let app = spawn_app().await;
    let res = app
        .register(&json!({ "email": "valid@example.com", "password": "short" }))
        .await;
    assert_eq!(res.status().as_u16(), 400);
}

#[tokio::test]
async fn register_rejects_duplicate_email() {
    let app = spawn_app().await;
    let body = json!({ "email": "dup@example.com", "password": "strong-password-123!" });

    let first = app.register(&body).await;
    assert_eq!(first.status().as_u16(), 200);

    let second = app.register(&body).await;
    assert_eq!(second.status().as_u16(), 400);
}

// ── POST /api/v1/auth/login ───────────────────────────────────────────────────

#[tokio::test]
async fn login_returns_token_for_valid_credentials() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    let res = app
        .login(&json!({ "email": user.email, "password": user.password }))
        .await;

    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["access_token"].as_str().is_some());
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    let res = app
        .login(&json!({ "email": user.email, "password": "wrong-password" }))
        .await;

    assert_eq!(res.status().as_u16(), 401);
}

#[tokio::test]
async fn login_rejects_unknown_email() {
    let app = spawn_app().await;
    let res = app
        .login(&json!({ "email": "nobody@example.com", "password": "any-password" }))
        .await;
    assert_eq!(res.status().as_u16(), 401);
}

// ── GET /api/v1/users/me ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_me_returns_profile_for_authenticated_user() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    let res = app.get_me(&user.token).await;

    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["email"], user.email);
}

#[tokio::test]
async fn get_me_rejects_missing_token() {
    let app = spawn_app().await;
    let res = app.get_unauthenticated("/api/v1/users/me").await;
    assert_eq!(res.status().as_u16(), 401);
}

#[tokio::test]
async fn get_me_rejects_invalid_token() {
    let app = spawn_app().await;
    let res = app.get_me("garbage.token.value").await;
    assert_eq!(res.status().as_u16(), 401);
}
