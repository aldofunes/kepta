use aide::axum::{routing::get, ApiRouter};
use axum::{extract::{Path, State}, Json};
use kepta_auth::AuthUser;
use kepta_types::contacts::{ProviderType, ServiceProviderContact};
use uuid::Uuid;

use crate::{error::ApiResult, households::ensure_member, state::AppState};

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(list))
}

async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
) -> ApiResult<Json<Vec<ServiceProviderContact>>> {
    ensure_member(&state, household_id, auth.user_id).await?;
    let rows = kepta_db::contacts::list(&state.db, household_id).await?;
    let contacts = rows.into_iter().map(|r| ServiceProviderContact {
        id: r.id,
        household_id: r.household_id,
        name: r.name,
        provider_type: match r.provider_type.as_str() {
            "plumber" => ProviderType::Plumber,
            "electrician" => ProviderType::Electrician,
            "cleaner" => ProviderType::Cleaner,
            "hvac" => ProviderType::Hvac,
            "general" => ProviderType::General,
            _ => ProviderType::Other,
        },
        phone: r.phone,
        email: r.email,
        website: r.website,
        notes: r.notes,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }).collect();
    Ok(Json(contacts))
}
