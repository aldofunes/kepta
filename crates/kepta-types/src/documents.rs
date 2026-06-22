use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Manual,
    Receipt,
    Warranty,
    Contract,
    WorkOrder,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OcrStatus {
    Pending,
    Processing,
    Done,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Document {
    pub id: Uuid,
    pub household_id: Uuid,
    pub item_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub doc_type: DocumentType,
    pub title: String,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub ocr_status: OcrStatus,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub doc_type: Option<DocumentType>,
    pub item_id: Option<Uuid>,
}

/// Presigned URL for downloading a document
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DocumentDownload {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}
