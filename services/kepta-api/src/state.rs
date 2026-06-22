use anyhow::Result;
use kepta_auth::jwt::JwtKeys;
use kepta_jobs::JobQueue;
use kepta_storage::DynStorage;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cfg: AppConfig,
    pub jwt: Arc<JwtKeys>,
    pub storage: DynStorage,
    pub jobs: Arc<JobQueue>,
}

impl AppState {
    pub async fn new(cfg: AppConfig, db: PgPool) -> Result<Self> {
        let jwt = Arc::new(JwtKeys::new(&cfg.jwt_secret));
        let storage = build_storage(&cfg).await?;
        let jobs = Arc::new(JobQueue::new(db.clone()));

        Ok(Self { db, cfg, jwt, storage, jobs })
    }
}

async fn build_storage(cfg: &AppConfig) -> Result<DynStorage> {
    use kepta_storage::s3::S3Storage;
    use object_store::aws::AmazonS3Builder;

    let mut builder = AmazonS3Builder::new()
        .with_bucket_name(&cfg.s3.bucket)
        .with_region(&cfg.s3.region)
        .with_access_key_id(&cfg.s3.access_key_id)
        .with_secret_access_key(&cfg.s3.secret_access_key);

    if let Some(endpoint) = &cfg.s3.endpoint {
        builder = builder.with_endpoint(endpoint).with_allow_http(true);
    }

    let s3 = builder.build()?;
    Ok(Arc::new(S3Storage::new(s3, cfg.s3.bucket.clone(), None)))
}

impl axum::extract::FromRef<AppState> for Arc<JwtKeys> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
