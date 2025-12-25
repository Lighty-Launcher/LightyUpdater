use crate::errors::ApiError;
use super::models::ParsedRequest;
use super::validator::validate_path_component;

/// Parses and validates the request path
pub fn parse_request_path(requested_path: &str) -> Result<ParsedRequest, ApiError> {
    if requested_path.is_empty() {
        tracing::warn!("serve_file: requested_path is empty");
        return Err(ApiError::NotFound);
    }

    let parts: Vec<&str> = requested_path.splitn(2, '/').collect();

    if parts.len() != 2 {
        tracing::warn!("serve_file: parts.len() = {}, expected 2", parts.len());
        return Err(ApiError::NotFound);
    }

    let server_name = parts[0];
    let url_file_part = parts[1];

    // Validate path components to prevent path traversal
    validate_path_component(server_name)?;
    validate_path_component(url_file_part)?;

    tracing::debug!(
        "serve_file: server_name = '{}', url_file_part = '{}'",
        server_name,
        url_file_part
    );

    Ok(ParsedRequest {
        server_name: server_name.to_string(),
        url_file_part: url_file_part.to_string(),
    })
}
