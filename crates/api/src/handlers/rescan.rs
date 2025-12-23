use super::models::{AppState, ApiError};
use axum::{
    extract::{Path as AxumPath, State},
    response::Json,
};

pub async fn force_rescan(
    State(state): State<AppState>,
    AxumPath(server_name): AxumPath<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.cache
        .force_rescan(&server_name)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Server {} rescanned successfully", server_name)
    })))
}
