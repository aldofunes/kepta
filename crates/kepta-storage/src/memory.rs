use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::RwLock;

use crate::Storage;

/// In-memory storage backend — for tests and local dev without S3.
pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<String, Bytes>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self { data: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn put(&self, key: &str, data: Bytes, _content_type: &str) -> Result<()> {
        self.data.write().await.insert(key.to_string(), data);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Bytes> {
        self.data
            .read()
            .await
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("key not found: {key}"))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.data.write().await.remove(key);
        Ok(())
    }

    async fn presign_get(&self, key: &str, _expires_secs: u32) -> Result<String> {
        Ok(format!("memory://{key}"))
    }
}
