use crate::errors::CdnError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

type Result<T> = std::result::Result<T, CdnError>;

const DEFAULT_API_BASE: &str = "https://api.cloudflare.com";

pub struct CloudflareClient {
    api_base: String,
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
            api_base: DEFAULT_API_BASE.to_string(),
            zone_id,
            api_token,
            client: reqwest::Client::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_api_base(api_base: String, zone_id: String, api_token: String) -> Self {
        Self {
            api_base,
            zone_id,
            api_token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn purge_cache(&self, server_name: &str) -> Result<()> {
        const MAX_RETRIES: usize = 3;
        const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

        let url = format!(
            "{}/client/v4/zones/{}/purge_cache",
            self.api_base, self.zone_id
        );

        let files = vec![format!("/{}.json", server_name)];
        let body = PurgeRequest { files };

        for attempt in 0..MAX_RETRIES {
            match self.purge_cache_internal(&url, &body).await {
                Ok(()) => {
                    tracing::info!("Cloudflare cache purged for {}", server_name);
                    return Ok(());
                }
                Err(error) if attempt < MAX_RETRIES - 1 => {
                    let backoff = INITIAL_BACKOFF * 2u32.pow(attempt as u32);
                    tracing::warn!(
                        "Cloudflare purge attempt {} failed for {}: {}. Retrying in {:?}...",
                        attempt + 1,
                        server_name,
                        error,
                        backoff
                    );
                    tokio::time::sleep(backoff).await;
                }
                Err(error) => {
                    tracing::error!(
                        "Cloudflare purge failed for {} after {} attempts: {}",
                        server_name,
                        MAX_RETRIES,
                        error
                    );
                    return Err(error);
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
            Err(CdnError::Cloudflare("Cloudflare purge failed".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn purge_cache_succeeds_on_200_with_success_true() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/client/v4/zones/zone-1/purge_cache"))
            .and(header("Authorization", "Bearer token-x"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = CloudflareClient::with_api_base(
            server.uri(),
            "zone-1".to_string(),
            "token-x".to_string(),
        );

        client.purge_cache("survival").await.unwrap();
    }

    #[tokio::test]
    async fn purge_cache_retries_then_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/client/v4/zones/zone-1/purge_cache"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/client/v4/zones/zone-1/purge_cache"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true
            })))
            .mount(&server)
            .await;

        let client = CloudflareClient::with_api_base(
            server.uri(),
            "zone-1".to_string(),
            "token-x".to_string(),
        );

        client.purge_cache("survival").await.unwrap();
    }

    #[tokio::test]
    async fn purge_cache_returns_error_when_api_reports_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/client/v4/zones/zone-1/purge_cache"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false
            })))
            .mount(&server)
            .await;

        let client = CloudflareClient::with_api_base(
            server.uri(),
            "zone-1".to_string(),
            "token-x".to_string(),
        );

        let err = client.purge_cache("survival").await.unwrap_err();
        assert!(matches!(err, CdnError::Cloudflare(_)));
    }
}
