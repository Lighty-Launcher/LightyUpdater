use thiserror::Error;

#[derive(Error, Debug)]
pub enum CdnError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Cloudflare API error: {0}")]
    Cloudflare(String),
}

impl From<reqwest::Error> for CdnError {
    fn from(err: reqwest::Error) -> Self {
        CdnError::Http(err.to_string())
    }
}
