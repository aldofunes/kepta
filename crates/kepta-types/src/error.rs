use schemars::JsonSchema;
use serde::Serialize;

/// RFC 9457 Problem Details
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub type_uri: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl ProblemDetails {
    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            type_uri: format!("https://kepta.app/errors/{status}"),
            title: title.into(),
            status,
            detail: detail.into(),
            instance: None,
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(404, "Not Found", detail)
    }

    pub fn unauthorized() -> Self {
        Self::new(401, "Unauthorized", "Authentication required")
    }

    pub fn forbidden() -> Self {
        Self::new(403, "Forbidden", "Insufficient permissions")
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(400, "Bad Request", detail)
    }

    pub fn payment_required() -> Self {
        Self::new(402, "Payment Required", "Upgrade your plan to continue")
    }

    pub fn internal() -> Self {
        Self::new(500, "Internal Server Error", "An unexpected error occurred")
    }
}
