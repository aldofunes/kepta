use aide::axum::ApiRouter;
use axum::extract::Path;
use uuid::Uuid;

use crate::state::AppState;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
    // TODO: GET /preferences, PATCH /preferences, GET /webhooks, POST /webhooks
}
