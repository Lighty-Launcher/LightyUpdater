use super::{cache, disk, parser, resolver};
use crate::handlers::models::AppState;
use crate::errors::ApiError;
use lighty_filesystem::FileSystem;
use axum::{
    extract::State,
    response::Response,
};

pub async fn serve_file(
    State(state): State<AppState>,
    uri: axum::http::Uri,
) -> Result<Response, ApiError> {
    let requested_path = uri.path().trim_start_matches('/');
    tracing::debug!("serve_file: requested_path = '{}'", requested_path);

    // Parse and validate request path
    let parsed = parser::parse_request_path(requested_path)?;

    // Get server metadata from cache
    let version_data = match state.cache.get_version(&parsed.server_name).await {
        Some(v) => v,
        None => {
            return Err(ApiError::ServerNotFound {
                server: parsed.server_name,
                available: state.cache.get_all_servers().await,
            });
        }
    };

    let server_config = state
        .cache
        .get_server_config(&parsed.server_name)
        .await
        .ok_or(ApiError::NotFound)?;

    // Resolve actual file path from URL
    let actual_path = resolver::resolve_file_path(
        &version_data,
        &parsed.url_file_part,
        &state.base_url,
        &parsed.server_name,
    )
    .ok_or_else(|| {
        tracing::warn!("serve_file: Could not resolve path for '{}'", parsed.url_file_part);
        ApiError::NotFound
    })?;

    tracing::debug!("serve_file: resolved actual_path = '{}'", actual_path);

    // Try to serve from RAM cache first
    if let Some(response) = cache::try_serve_from_cache(&state, &parsed.server_name, &actual_path).await {
        return Ok(response);
    }

    // Fallback to disk if not in cache
    tracing::debug!("serve_file: file not in cache, falling back to disk");

    let full_path = FileSystem::build_server_path(&state.base_path, &server_config.name)
        .join(&actual_path);

    disk::serve_from_disk(full_path, state.streaming_threshold_bytes).await
}
