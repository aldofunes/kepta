use aide::axum::{routing::get, ApiRouter};
use axum::{extract::{Path, State}, Json};
use kepta_auth::AuthUser;
use kepta_types::schedules::{FrequencyUnit, MaintenanceSchedule, ScheduleSource};
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
) -> ApiResult<Json<Vec<MaintenanceSchedule>>> {
    ensure_member(&state, household_id, auth.user_id).await?;
    let rows = kepta_db::schedules::list(&state.db, household_id).await?;
    let schedules = rows.into_iter().map(|r| MaintenanceSchedule {
        id: r.id,
        item_id: r.item_id,
        household_id: r.household_id,
        title: r.title,
        description: r.description,
        frequency_unit: match r.frequency_unit.as_str() {
            "week" => FrequencyUnit::Week,
            "month" => FrequencyUnit::Month,
            "year" => FrequencyUnit::Year,
            _ => FrequencyUnit::Day,
        },
        frequency_value: r.frequency_value,
        next_due_at: r.next_due_at,
        last_completed_at: r.last_completed_at,
        source: match r.source.as_str() {
            "ai_suggestion" => ScheduleSource::AiSuggestion,
            "template" => ScheduleSource::Template,
            _ => ScheduleSource::Manual,
        },
        created_at: r.created_at,
        updated_at: r.updated_at,
    }).collect();
    Ok(Json(schedules))
}
