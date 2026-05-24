use crate::errors::RescanError;
use crate::models::RescanOrchestrator;
use futures::stream::{self, StreamExt};
use std::sync::Arc;

type Result<T> = std::result::Result<T, RescanError>;

impl RescanOrchestrator {
    pub(crate) async fn sync_cloud_storage(
        &self,
        server_name: &str,
        diff: &lighty_file_diff::FileDiff,
    ) -> Result<()> {
        let storage = self.storage.as_ref().ok_or_else(|| {
            RescanError::InvalidConfig("Storage backend not initialized".to_string())
        })?;
        let base_path = self.base_path.clone();
        let concurrency = {
            let config = self.config.read().await;
            config.cache.hash_concurrency
        };
        if concurrency == 0 {
            return Err(RescanError::InvalidConfig(
                "cache.hash_concurrency must be greater than 0".to_string(),
            ));
        }

        tracing::info!(
            "Syncing cloud storage for {}: {} added, {} modified, {} removed",
            server_name,
            diff.added.len(),
            diff.modified.len(),
            diff.removed.len()
        );

        let upload_entries: Vec<_> = diff
            .added
            .iter()
            .chain(diff.modified.iter())
            .map(|change| (change.local_path.clone(), change.remote_key.clone()))
            .collect();

        let upload_results: Vec<_> = stream::iter(upload_entries)
            .map(|(local_path, remote_key)| {
                let storage = Arc::clone(storage);
                let local_path = base_path.join(local_path);
                async move {
                    tracing::debug!("Uploading: {}", remote_key);
                    storage.upload_file(&local_path, &remote_key).await
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        for result in upload_results {
            result?;
        }

        let delete_entries: Vec<_> = diff
            .removed
            .iter()
            .map(|change| change.remote_key.clone())
            .collect();

        let delete_results: Vec<_> = stream::iter(delete_entries)
            .map(|remote_key| {
                let storage = Arc::clone(storage);
                async move {
                    tracing::debug!("Deleting: {}", remote_key);
                    storage.delete_file(&remote_key).await
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        for result in delete_results {
            result?;
        }

        tracing::info!("Cloud storage sync complete for {}", server_name);
        Ok(())
    }
}
