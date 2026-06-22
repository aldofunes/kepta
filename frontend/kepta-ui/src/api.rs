/// Thin wrappers around fetch for calling the Kepta REST API.
/// The base URL is read from the `KEPTA_API_URL` env var baked in at build time,
/// or defaults to `/api/v1` (same-origin for the hosted web app).

#[cfg(not(feature = "tauri"))]
pub fn base_url() -> &'static str {
    option_env!("KEPTA_API_URL").unwrap_or("/api/v1")
}
