use aide::axum::{routing::{get, post}, ApiRouter};
use axum::Json;
use kepta_auth::{password, AuthUser};
use kepta_domain::validation::{validate_email, validate_password};
use kepta_types::auth::{AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, UserProfile};

use crate::{error::{ApiError, ApiResult}, state::AppState};

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/register", post(register))
        .api_route("/login", post(login))
}

pub fn users_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/me", get(me).patch(update_me))
}

async fn register(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<Json<AuthResponse>> {
    validate_email(&body.email).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    validate_password(&body.password).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if kepta_db::auth::find_by_email(&state.db, &body.email).await?.is_some() {
        return Err(ApiError::BadRequest("email already registered".into()));
    }

    let hash = password::hash(&body.password).map_err(anyhow::Error::from)?;
    let user = kepta_db::auth::create(&state.db, &body.email, Some(&hash), body.display_name.as_deref()).await?;

    let token = state.jwt.sign(user.id, &user.email).map_err(anyhow::Error::from)?;

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: 900,
        user: UserProfile {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        },
    }))
}

async fn login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let user = kepta_db::auth::find_by_email(&state.db, &body.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized)?;

    let hash = user.password_hash.as_deref().ok_or(ApiError::Unauthorized)?;
    if !password::verify(&body.password, hash).map_err(anyhow::Error::from)? {
        return Err(ApiError::Unauthorized);
    }

    let token = state.jwt.sign(user.id, &user.email).map_err(anyhow::Error::from)?;

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: 900,
        user: UserProfile {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        },
    }))
}

async fn me(
    auth: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> ApiResult<Json<UserProfile>> {
    let user = kepta_db::auth::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("user not found".into()))?;

    Ok(Json(UserProfile {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
    }))
}

async fn update_me(
    auth: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(_body): Json<UpdateProfileRequest>,
) -> ApiResult<Json<UserProfile>> {
    let user = kepta_db::auth::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("user not found".into()))?;

    Ok(Json(UserProfile {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
    }))
}
