use aide::axum::{routing::get, ApiRouter};
use axum::{extract::State, Json};
use kepta_auth::AuthUser;
use kepta_types::subscriptions::Plan;
use uuid::Uuid;

use crate::{error::ApiResult, state::AppState};

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/plans", get(list_plans))
}

async fn list_plans(State(state): State<AppState>) -> ApiResult<Json<Vec<Plan>>> {
    let rows = kepta_db::subscriptions::list_plans(&state.db).await?;
    let plans = rows.into_iter().map(|r| Plan {
        id: r.id,
        slug: r.slug,
        name: r.name,
        price_cents: r.price_cents,
        household_limit: r.household_limit,
        storage_limit_mb: r.storage_limit_mb,
        document_limit: r.document_limit,
    }).collect();
    Ok(Json(plans))
}
