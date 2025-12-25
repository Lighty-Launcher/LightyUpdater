use lighty_events::{AppEvent, EventBus};
use lighty_config::Config;
use lighty_filesystem::FileSystem;
use anyhow::Result;
use std::sync::Arc;

pub async fn initialize_folders(config: &Config, events: &Arc<EventBus>) -> Result<()> {
    if config.servers.is_empty() {
        return Ok(());
    }

    for server_config in &config.servers {
        // Skip disabled servers
        if !server_config.enabled {
            continue;
        }

        let path = format!("{}/{}", config.server.base_path, &server_config.name);
        events.emit(AppEvent::ServerFolderInit {
            name: server_config.name.to_string(),
            path: path.clone(),
        });

        FileSystem::ensure_server_structure(config.server.base_path.as_ref(), server_config.name.as_ref()).await?;

        events.emit(AppEvent::ServerFolderCreated {
            name: server_config.name.to_string(),
        });
    }

    events.emit(AppEvent::AllServersInitialized);

    Ok(())
}
