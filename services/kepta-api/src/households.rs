use aide::axum::{routing::{get, post}, ApiRouter};
use axum::{extract::{Path, State}, Json};
use kepta_auth::AuthUser;
use kepta_types::households::{CreateHouseholdRequest, Household, Address};
use uuid::Uuid;

use crate::{error::{ApiError, ApiResult}, state::AppState};

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(list).post(create))
        .api_route("/{id}", get(get_one))
        .nest("/{id}/items", crate::items::router())
        .nest("/{id}/schedules", crate::schedules::router())
        .nest("/{id}/documents", crate::documents::router())
        .nest("/{id}/contacts", crate::contacts::router())
        .nest("/{id}/notifications", crate::notifications::router())
}

async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<Household>>> {
    let rows = kepta_db::households::find_by_user(&state.db, auth.user_id).await?;
    let households = rows.into_iter().map(row_to_household).collect();
    Ok(Json(households))
}

async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateHouseholdRequest>,
) -> ApiResult<Json<Household>> {
    if state.cfg.billing_enabled {
        let plan = kepta_db::subscriptions::get_plan_for_user(&state.db, auth.user_id).await?;
        let limits = kepta_domain::limits::PlanLimits {
            household_limit: plan.household_limit,
            member_limit: Some(kepta_domain::limits::FreeTierLimits::MEMBERS_PER_HOUSEHOLD),
            document_limit: plan.document_limit,
            storage_limit_mb: plan.storage_limit_mb,
        };
        let count = kepta_db::households::count_by_user(&state.db, auth.user_id).await?;
        if !limits.can_add_household(count) {
            return Err(ApiError::PaymentRequired);
        }
    }

    let address = serde_json::to_value(body.address.unwrap_or(Address {
        street: None, city: None, state: None, zip: None, country: None,
    }))?;

    let row = kepta_db::households::create(
        &state.db, auth.user_id, &body.name, address, body.year_built, body.sqft,
    ).await?;

    Ok(Json(row_to_household(row)))
}

async fn get_one(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Household>> {
    ensure_member(&state, id, auth.user_id).await?;
    let row = kepta_db::households::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("household {id}")))?;
    Ok(Json(row_to_household(row)))
}

pub async fn ensure_member(state: &AppState, household_id: Uuid, user_id: Uuid) -> ApiResult<()> {
    if !kepta_db::households::is_member(&state.db, household_id, user_id).await? {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

fn row_to_household(row: kepta_db::households::HouseholdRow) -> Household {
    Household {
        id: row.id,
        name: row.name,
        address: serde_json::from_value(row.address).unwrap_or(Address {
            street: None, city: None, state: None, zip: None, country: None,
        }),
        year_built: row.year_built,
        sqft: row.sqft,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
