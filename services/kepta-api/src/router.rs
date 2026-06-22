use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use axum::{http::StatusCode, Extension};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};

use crate::state::AppState;

pub fn build(state: AppState) -> axum::Router {
    let mut openapi = OpenApi::default();

    let api = ApiRouter::new()
        .nest("/auth", crate::auth::router())
        .nest("/households", crate::households::router())
        .nest("/users", crate::auth::users_router())
        .nest("/subscriptions", crate::subscriptions::router())
        .with_state(state.clone())
        .finish_api_with(&mut openapi, |t| {
            t.title("Kepta API").version(env!("CARGO_PKG_VERSION"))
        });

    axum::Router::new()
        .route("/health", axum::routing::get(health))
        .nest("/api/v1", api)
        .route("/openapi.json", axum::routing::get(serve_openapi))
        .layer(Extension(openapi))
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(false))
                .on_response(DefaultOnResponse::new()),
        )
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn serve_openapi(Extension(api): Extension<OpenApi>) -> axum::Json<OpenApi> {
    axum::Json(api)
}
