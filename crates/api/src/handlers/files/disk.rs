use crate::errors::ApiError;
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

/// Serves a file from disk, either by streaming or loading into memory
/// The threshold is configurable via server.streaming_threshold_mb
pub async fn serve_from_disk(full_path: PathBuf, streaming_threshold_bytes: u64) -> Result<Response, ApiError> {
    if !full_path.exists() {
        tracing::warn!("serve_file: File does not exist: '{}'", full_path.display());
        return Err(ApiError::NotFound);
    }

    let mime_type = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    // Get file metadata to check size
    let metadata = tokio::fs::metadata(&full_path).await.map_err(|e| {
        tracing::error!(
            "serve_file: Failed to get metadata for '{}': {}",
            full_path.display(),
            e
        );
        ApiError::NotFound
    })?;

    let file_size = metadata.len();

    if file_size > streaming_threshold_bytes {
        stream_large_file(full_path, mime_type, file_size).await
    } else {
        load_small_file(full_path, mime_type).await
    }
}

/// Streams a large file
async fn stream_large_file(
    full_path: PathBuf,
    mime_type: String,
    file_size: u64,
) -> Result<Response, ApiError> {
    tracing::debug!(
        "serve_file: streaming large file ({:.2} MB)",
        file_size as f64 / 1024.0 / 1024.0
    );

    let file = tokio::fs::File::open(&full_path).await.map_err(|e| {
        tracing::error!(
            "serve_file: Failed to open file '{}': {}",
            full_path.display(),
            e
        );
        ApiError::NotFound
    })?;

    // Convert file to stream
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime_type)],
        body,
    )
        .into_response())
}

/// Loads a small file into memory for better performance
async fn load_small_file(full_path: PathBuf, mime_type: String) -> Result<Response, ApiError> {
    let content = tokio::fs::read(&full_path).await.map_err(|e| {
        tracing::error!(
            "serve_file: Failed to read file '{}': {}",
            full_path.display(),
            e
        );
        ApiError::NotFound
    })?;

    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime_type)],
        content,
    )
        .into_response())
}
