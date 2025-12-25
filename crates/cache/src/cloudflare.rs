use super::errors::CacheError;
use serde::{Deserialize, Serialize};

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
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
            self.zone_id
        );

        // Purge metadata JSON
        let files = vec![format!("/{}.json", server_name)];

        let body = PurgeRequest { files };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body)
            .send()
            .await?;

        let result: PurgeResponse = response.json().await?;

        if result.success {
            tracing::info!("Cloudflare cache purged for {}", server_name);
            Ok(())
        } else {
            Err(CacheError::CloudflareError("Cloudflare purge failed".to_string()))
        }
    }
}
