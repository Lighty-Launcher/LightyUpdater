use crate::handlers::models::AppState;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Attempts to serve file from RAM cache
pub async fn try_serve_from_cache(
    state: &AppState,
    server_name: &str,
    actual_path: &str,
) -> Option<Response> {
    if let Some(file_cache) = state.cache.get_file(server_name, actual_path).await {
        tracing::debug!("serve_file: serving from RAM cache");

        // Zero-copy: file_cache.data is already Bytes which uses Arc internally
        // Cloning Bytes is cheap (just increments reference count)
        return Some(
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, file_cache.mime_type.clone())],
                file_cache.data,
            )
                .into_response(),
        );
    }

    None
}
