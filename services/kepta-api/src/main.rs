use kepta_api::{config::AppConfig, router, state::AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,kepta_api=debug".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let cfg = AppConfig::load()?;
    let db = kepta_db::connect(&cfg.database_url).await?;

    tracing::info!("running migrations");
    kepta_db::migrate(&db).await?;

    let state = AppState::new(cfg, db).await?;
    let app = router::build(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
