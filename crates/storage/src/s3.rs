use crate::backend::StorageBackend;
use crate::StorageError;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{primitives::ByteStream, Client};
use std::path::Path;

/// S3-compatible storage backend
/// Compatible with: Cloudflare R2, AWS S3, MinIO, DigitalOcean Spaces, etc.
pub struct S3Backend {
    client: Client,
    bucket_name: String,
    public_url: String,
    bucket_prefix: String,
}

impl S3Backend {
    pub async fn new(
        endpoint_url: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
        bucket_name: String,
        public_url: String,
        bucket_prefix: String,
    ) -> Result<Self, StorageError> {
        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "lighty-s3",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new(region))
            .endpoint_url(endpoint_url)
            .load()
            .await;

        let client = Client::new(&config);

        Ok(Self {
            client,
            bucket_name,
            public_url,
            bucket_prefix,
        })
    }

    fn build_key(&self, remote_key: &str) -> String {
        if self.bucket_prefix.is_empty() {
            remote_key.to_string()
        } else {
            format!("{}/{}", self.bucket_prefix, remote_key)
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for S3Backend {
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<String, StorageError> {
        let key = self.build_key(remote_key);

        tracing::info!("Uploading {} to S3 bucket {}", key, self.bucket_name);

        let file_data = tokio::fs::read(local_path).await?;
        let byte_stream = ByteStream::from(file_data);

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(byte_stream)
            .send()
            .await
            .map_err(|e| StorageError::UploadError(key.clone(), e.to_string()))?;

        let url = self.get_url(remote_key);
        tracing::info!("Upload complete: {}", url);

        Ok(url)
    }

    async fn delete_file(&self, remote_key: &str) -> Result<(), StorageError> {
        let key = self.build_key(remote_key);

        tracing::info!("Deleting {} from S3 bucket {}", key, self.bucket_name);

        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| StorageError::DeleteError(key.clone(), e.to_string()))?;

        tracing::info!("Delete complete: {}", key);
        Ok(())
    }

    fn get_url(&self, remote_key: &str) -> String {
        let key = self.build_key(remote_key);
        format!("{}/{}", self.public_url, key)
    }

    fn is_remote(&self) -> bool {
        true
    }
}
