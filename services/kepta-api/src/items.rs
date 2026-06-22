use aide::axum::{routing::{delete, get, patch, post}, ApiRouter};
use axum::{extract::{Path, State}, Json};
use kepta_auth::AuthUser;
use kepta_types::items::{Item, ItemCategory, CreateItemRequest};
use uuid::Uuid;

use crate::{error::{ApiError, ApiResult}, households::ensure_member, state::AppState};

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(list).post(create))
        .api_route("/{item_id}", get(get_one).delete(remove))
}

async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Item>>> {
    ensure_member(&state, household_id, auth.user_id).await?;
    let rows = kepta_db::items::list(&state.db, household_id).await?;
    let items = rows.into_iter().map(row_to_item).collect();
    Ok(Json(items))
}

async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
    Json(body): Json<CreateItemRequest>,
) -> ApiResult<Json<Item>> {
    ensure_member(&state, household_id, auth.user_id).await?;
    // TODO: insert item
    Err(ApiError::Internal(anyhow::anyhow!("not implemented")))
}

async fn get_one(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Item>> {
    ensure_member(&state, household_id, auth.user_id).await?;
    let row = kepta_db::items::find_by_id(&state.db, item_id, household_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("item {item_id}")))?;
    Ok(Json(row_to_item(row)))
}

async fn remove(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<axum::http::StatusCode> {
    ensure_member(&state, household_id, auth.user_id).await?;
    let deleted = kepta_db::items::delete(&state.db, item_id, household_id).await?;
    if deleted {
        Ok(axum::http::StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(format!("item {item_id}")))
    }
}

fn row_to_item(row: kepta_db::items::ItemRow) -> Item {
    Item {
        id: row.id,
        household_id: row.household_id,
        room_id: row.room_id,
        category: match row.category.as_str() {
            "appliance" => ItemCategory::Appliance,
            "system" => ItemCategory::System,
            "fixture" => ItemCategory::Fixture,
            _ => ItemCategory::Other,
        },
        name: row.name,
        brand: row.brand,
        model_number: row.model_number,
        serial_number: row.serial_number,
        purchase_date: row.purchase_date,
        install_date: row.install_date,
        warranty_expiry: row.warranty_expiry,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
