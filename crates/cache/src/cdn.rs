use super::errors::CacheError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

type Result<T> = std::result::Result<T, CacheError>;

pub struct CdnClient {
    provider: CdnProvider,
    zone_id: String,
    api_token: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub enum CdnProvider {
    Cloudflare,
    CloudFront,
}

#[derive(Serialize)]
struct PurgeRequest {
    files: Vec<String>,
}

#[derive(Deserialize)]
struct PurgeResponse {
    success: bool,
}

impl CdnClient {
    pub fn new(provider: &str, zone_id: String, api_token: String) -> Self {
        let provider = match provider.to_lowercase().as_str() {
            "cloudfront" => CdnProvider::CloudFront,
            _ => CdnProvider::Cloudflare, // Default to Cloudflare
        };

        Self {
            provider,
            zone_id,
            api_token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn purge_files(&self, file_urls: Vec<String>) -> Result<()> {
        if file_urls.is_empty() {
            return Ok(());
        }

        match self.provider {
            CdnProvider::Cloudflare => self.purge_cloudflare(file_urls).await,
            CdnProvider::CloudFront => {
                tracing::warn!("CloudFront CDN purge not implemented yet");
                Ok(())
            }
        }
    }

    async fn purge_cloudflare(&self, file_urls: Vec<String>) -> Result<()> {
        const MAX_RETRIES: usize = 3;
        const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
            self.zone_id
        );

        let body = PurgeRequest { files: file_urls.clone() };

        // Retry with exponential backoff
        for attempt in 0..MAX_RETRIES {
            match self.purge_cloudflare_internal(&url, &body).await {
                Ok(()) => {
                    tracing::info!("Cloudflare CDN cache purged for {} files", file_urls.len());
                    return Ok(());
                }
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    let backoff = INITIAL_BACKOFF * 2u32.pow(attempt as u32);
                    tracing::warn!(
                        "Cloudflare CDN purge attempt {} failed: {}. Retrying in {:?}...",
                        attempt + 1,
                        e,
                        backoff
                    );
                    tokio::time::sleep(backoff).await;
                }
                Err(e) => {
                    tracing::error!("Cloudflare CDN purge failed after {} attempts: {}", MAX_RETRIES, e);
                    return Err(e);
                }
            }
        }

        unreachable!()
    }

    async fn purge_cloudflare_internal(&self, url: &str, body: &PurgeRequest) -> Result<()> {
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
            Err(CacheError::CloudflareError("Cloudflare CDN purge failed".to_string()))
        }
    }
}
