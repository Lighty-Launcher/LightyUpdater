use crate::errors::CdnError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

type Result<T> = std::result::Result<T, CdnError>;

const DEFAULT_API_BASE: &str = "https://api.cloudflare.com";

pub struct CdnClient {
    provider: CdnProvider,
    api_base: String,
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
            _ => CdnProvider::Cloudflare,
        };

        Self {
            provider,
            api_base: DEFAULT_API_BASE.to_string(),
            zone_id,
            api_token,
            client: reqwest::Client::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_api_base(
        provider: &str,
        api_base: String,
        zone_id: String,
        api_token: String,
    ) -> Self {
        let mut c = Self::new(provider, zone_id, api_token);
        c.api_base = api_base;
        c
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
            "{}/client/v4/zones/{}/purge_cache",
            self.api_base, self.zone_id
        );

        let body = PurgeRequest { files: file_urls.clone() };

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
            Err(CdnError::Cloudflare("Cloudflare CDN purge failed".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn empty_urls_skip_network_call() {
        let client = CdnClient::new("cloudflare", "zone".into(), "token".into());
        client.purge_files(vec![]).await.unwrap();
    }

    #[tokio::test]
    async fn cloudfront_provider_returns_ok_without_calling_anything() {
        let client = CdnClient::new("cloudfront", "zone".into(), "token".into());
        client
            .purge_files(vec!["https://x.example/a".into()])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn cloudflare_purge_succeeds_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/client/v4/zones/zone-1/purge_cache"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = CdnClient::with_api_base(
            "cloudflare",
            server.uri(),
            "zone-1".into(),
            "token-x".into(),
        );

        client
            .purge_files(vec!["https://cdn/a.jar".into()])
            .await
            .unwrap();
    }
}
