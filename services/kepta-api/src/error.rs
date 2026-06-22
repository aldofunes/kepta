use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use kepta_types::error::ProblemDetails;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("payment required")]
    PaymentRequired,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::Internal(anyhow::Error::from(e))
    }
}

impl aide::OperationOutput for ApiError {
    type Inner = ();
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, problem) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, ProblemDetails::not_found(msg)),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, ProblemDetails::unauthorized()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, ProblemDetails::forbidden()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ProblemDetails::bad_request(msg)),
            ApiError::PaymentRequired => (StatusCode::PAYMENT_REQUIRED, ProblemDetails::payment_required()),
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, ProblemDetails::internal())
            }
        };
        (status, Json(problem)).into_response()
    }
}
