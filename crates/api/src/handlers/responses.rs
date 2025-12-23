use super::models::ApiError;
use crate::models::{ErrorResponse, ErrorDetail};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            ApiError::ServerNotFound { server, available } => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "SERVER_NOT_FOUND".to_string(),
                        message: format!("Server '{}' not found", server),
                        available_servers: Some(available),
                    },
                },
            ),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "NOT_FOUND".to_string(),
                        message: "Resource not found".to_string(),
                        available_servers: None,
                    },
                },
            ),
            ApiError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "INTERNAL_ERROR".to_string(),
                        message: msg,
                        available_servers: None,
                    },
                },
            ),
            ApiError::InvalidPath(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "INVALID_PATH".to_string(),
                        message: msg,
                        available_servers: None,
                    },
                },
            ),
        };

        (status, Json(error_response)).into_response()
    }
}
