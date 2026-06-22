use aide::axum::{routing::get, ApiRouter};
use axum::{extract::{Path, State}, Json};
use kepta_auth::AuthUser;
use kepta_types::documents::{Document, DocumentType, OcrStatus};
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
) -> ApiResult<Json<Vec<Document>>> {
    ensure_member(&state, household_id, auth.user_id).await?;
    let rows = kepta_db::documents::list(&state.db, household_id).await?;
    let docs = rows.into_iter().map(|r| Document {
        id: r.id,
        household_id: r.household_id,
        item_id: r.item_id,
        doc_type: match r.doc_type.as_str() {
            "receipt" => DocumentType::Receipt,
            "warranty" => DocumentType::Warranty,
            "contract" => DocumentType::Contract,
            "work_order" => DocumentType::WorkOrder,
            "manual" => DocumentType::Manual,
            _ => DocumentType::Other,
        },
        title: r.title,
        file_size_bytes: r.file_size_bytes,
        mime_type: r.mime_type,
        ocr_status: match r.ocr_status.as_str() {
            "processing" => OcrStatus::Processing,
            "done" => OcrStatus::Done,
            "failed" => OcrStatus::Failed,
            "skipped" => OcrStatus::Skipped,
            _ => OcrStatus::Pending,
        },
        uploaded_by: r.uploaded_by,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }).collect();
    Ok(Json(docs))
}
