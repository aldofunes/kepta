use anyhow::Result;
use bytes::Bytes;
use object_store::{aws::AmazonS3, ObjectStoreExt, PutPayload, path::Path, signer::Signer};
use std::time::Duration;

use crate::Storage;

pub struct S3Storage {
    inner: AmazonS3,
    bucket: String,
    public_url: Option<String>,
}

impl S3Storage {
    pub fn new(inner: AmazonS3, bucket: String, public_url: Option<String>) -> Self {
        Self { inner, bucket, public_url }
    }
}

#[async_trait::async_trait]
impl Storage for S3Storage {
    async fn put(&self, key: &str, data: Bytes, _content_type: &str) -> Result<()> {
        self.inner.put(&Path::from(key), PutPayload::from(data)).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Bytes> {
        let result = self.inner.get(&Path::from(key)).await?;
        Ok(result.bytes().await?)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.inner.delete(&Path::from(key)).await?;
        Ok(())
    }

    async fn presign_get(&self, key: &str, expires_secs: u32) -> Result<String> {
        let url = self.inner
            .signed_url(
                http::Method::GET,
                &Path::from(key),
                Duration::from_secs(expires_secs as u64),
            )
            .await?;
        Ok(url.to_string())
    }
}
