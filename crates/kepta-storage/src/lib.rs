use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;

pub mod memory;
pub mod s3;

/// Storage backend abstraction.
#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> Result<()>;
    async fn get(&self, key: &str) -> Result<Bytes>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn presign_get(&self, key: &str, expires_secs: u32) -> Result<String>;
}

pub type DynStorage = Arc<dyn Storage>;
