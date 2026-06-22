use crate::helpers::{register_user, spawn_app};
use serde_json::json;
use uuid::Uuid;

// ── POST /api/v1/households ───────────────────────────────────────────────────

#[tokio::test]
async fn create_household_returns_200_with_household() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    let res = app
        .post_household(&json!({ "name": "My Home" }), &user.token)
        .await;

    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "My Home");
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn create_household_rejects_unauthenticated() {
    let app = spawn_app().await;
    let res = app
        .post_household(&json!({ "name": "My Home" }), "bad-token")
        .await;
    assert_eq!(res.status().as_u16(), 401);
}

#[tokio::test]
async fn create_household_persists_optional_fields() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    let res = app
        .post_household(
            &json!({
                "name": "Big House",
                "year_built": 1985,
                "sqft": 2400,
                "address": { "city": "Portland", "state": "OR" }
            }),
            &user.token,
        )
        .await;

    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Big House");
    assert_eq!(body["year_built"], 1985);
    assert_eq!(body["sqft"], 2400);
}

// ── GET /api/v1/households ────────────────────────────────────────────────────

#[tokio::test]
async fn list_households_returns_empty_for_new_user() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    let res = app.get_households(&user.token).await;

    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_households_returns_all_user_households() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    app.post_household(&json!({ "name": "Home One" }), &user.token).await;
    app.post_household(&json!({ "name": "Home Two" }), &user.token).await;

    let res = app.get_households(&user.token).await;
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_households_does_not_return_other_users_households() {
    let app = spawn_app().await;
    let alice = register_user(&app).await;
    let bob = register_user(&app).await;

    app.post_household(&json!({ "name": "Alice's Home" }), &alice.token).await;

    let res = app.get_households(&bob.token).await;
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body.as_array().unwrap().len(), 0);
}

// ── GET /api/v1/households/:id ────────────────────────────────────────────────

#[tokio::test]
async fn get_household_returns_200_for_owner() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    let create_res = app
        .post_household(&json!({ "name": "My Home" }), &user.token)
        .await;
    let id = Uuid::parse_str(
        create_res.json::<serde_json::Value>().await.unwrap()["id"]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    let res = app.get_household(id, &user.token).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "My Home");
}

#[tokio::test]
async fn get_household_returns_403_for_non_member() {
    let app = spawn_app().await;
    let owner = register_user(&app).await;
    let intruder = register_user(&app).await;

    let create_res = app
        .post_household(&json!({ "name": "Owner's Home" }), &owner.token)
        .await;
    let id = Uuid::parse_str(
        create_res.json::<serde_json::Value>().await.unwrap()["id"]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    let res = app.get_household(id, &intruder.token).await;
    assert_eq!(res.status().as_u16(), 403);
}

#[tokio::test]
async fn get_household_returns_403_for_nonexistent_id() {
    let app = spawn_app().await;
    let user = register_user(&app).await;

    // is_member check fires first and returns 403 for any id the user doesn't own
    let res = app.get_household(Uuid::now_v7(), &user.token).await;
    assert_eq!(res.status().as_u16(), 403);
}
