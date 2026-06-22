use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub base_url: String,
    pub billing_enabled: bool,
    pub s3: S3Config,
    pub smtp: SmtpConfig,
    pub apns: Option<ApnsConfig>,
    pub google_oauth: Option<OAuthConfig>,
    pub apple_oauth: Option<OAuthConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub endpoint: Option<String>,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApnsConfig {
    pub team_id: String,
    pub key_id: String,
    pub private_key_path: String,
    pub bundle_id: String,
    pub sandbox: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let cfg = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()?;
        Ok(cfg)
    }
}
