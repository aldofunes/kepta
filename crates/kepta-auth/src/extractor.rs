use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use std::sync::Arc;
use uuid::Uuid;

use crate::jwt::JwtKeys;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<JwtKeys>: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let keys = Arc::<JwtKeys>::from_ref(state);
        let claims = keys
            .verify(bearer.token())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let user_id = claims.sub.parse::<Uuid>().map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser { user_id, email: claims.email })
    }
}

impl aide::OperationInput for AuthUser {}
