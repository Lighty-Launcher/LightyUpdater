use super::models::{AppState, ApiError};
use crate::models::{ServerListResponse, ServerInfo};
use lighty_models::VersionBuilder;
use axum::{
    extract::{Path as AxumPath, State},
    response::Json,
};

pub async fn list_servers(State(state): State<AppState>) -> Result<Json<ServerListResponse>, ApiError> {
    let server_names = state.cache.get_all_servers().await;
    let mut servers = Vec::new();

    for name in server_names {
        if let Some(config) = state.cache.get_server_config(&name).await {
            let last_update = state.cache.get_last_update(&name)
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            servers.push(ServerInfo {
                name: name.clone(),
                loader: config.loader.clone(),
                minecraft_version: config.minecraft_version.clone(),
                url: format!("{}/{}.json", state.base_url, name),
                last_update,
            });
        }
    }

    Ok(Json(ServerListResponse { servers }))
}

pub async fn get_server_metadata(
    State(state): State<AppState>,
    AxumPath(server_name_with_ext): AxumPath<String>,
) -> Result<Json<VersionBuilder>, ApiError> {
    let server_name = server_name_with_ext
        .strip_suffix(".json")
        .unwrap_or(&server_name_with_ext)
        .to_string();

    // Check if server is enabled
    if let Some(server_config) = state.cache.get_server_config(&server_name).await {
        if !server_config.enabled {
            let available = state.cache.get_all_servers().await;
            return Err(ApiError::ServerNotFound {
                server: server_name,
                available,
            });
        }
    }

    match state.cache.get(&server_name).await {
        Some(builder) => Ok(Json((*builder).clone())),
        None => {
            let available = state.cache.get_all_servers().await;
            Err(ApiError::ServerNotFound {
                server: server_name,
                available,
            })
        }
    }
}
