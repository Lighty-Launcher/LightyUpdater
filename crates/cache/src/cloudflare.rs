use super::errors::CacheError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

type Result<T> = std::result::Result<T, CacheError>;

pub struct CloudflareClient {
    zone_id: String,
    api_token: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct PurgeRequest {
    files: Vec<String>,
}

#[derive(Deserialize)]
struct PurgeResponse {
    success: bool,
}

impl CloudflareClient {
    pub fn new(zone_id: String, api_token: String) -> Self {
        Self {
            zone_id,
            api_token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn purge_cache(&self, server_name: &str) -> Result<()> {
        const MAX_RETRIES: usize = 3;
        const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
            self.zone_id
        );

        // Purge metadata JSON
        let files = vec![format!("/{}.json", server_name)];
        let body = PurgeRequest { files };

        // Retry with exponential backoff
        for attempt in 0..MAX_RETRIES {
            match self.purge_cache_internal(&url, &body).await {
                Ok(()) => {
                    tracing::info!("Cloudflare cache purged for {}", server_name);
                    return Ok(());
                }
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    let backoff = INITIAL_BACKOFF * 2u32.pow(attempt as u32);
                    tracing::warn!(
                        "Cloudflare purge attempt {} failed for {}: {}. Retrying in {:?}...",
                        attempt + 1,
                        server_name,
                        e,
                        backoff
                    );
                    tokio::time::sleep(backoff).await;
                }
                Err(e) => {
                    tracing::error!("Cloudflare purge failed for {} after {} attempts: {}", server_name, MAX_RETRIES, e);
                    return Err(e);
                }
            }
        }

        unreachable!()
    }

    async fn purge_cache_internal(&self, url: &str, body: &PurgeRequest) -> Result<()> {
        const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

        let response = self
            .client
            .post(url)
            .timeout(REQUEST_TIMEOUT)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(body)
            .send()
            .await?;

        let result: PurgeResponse = response.json().await?;

        if result.success {
            Ok(())
        } else {
            Err(CacheError::CloudflareError("Cloudflare purge failed".to_string()))
        }
    }
}
