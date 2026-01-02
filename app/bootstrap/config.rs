use lighty_events::{AppEvent, EventBus};
use lighty_config::Config;
use lighty_filesystem::FileSystem;
use anyhow::Result;
use std::sync::Arc;

pub async fn load(config_path: &str, events: &Arc<EventBus>) -> Result<Config> {
    let abs_config_path = FileSystem::get_absolute_path_string(config_path)?;

    events.emit(AppEvent::ConfigLoading {
        path: abs_config_path.clone(),
    });

    let config_exists = std::path::Path::new(config_path).exists();
    let config = Config::from_file_with_events(config_path, Some(events)).await?;

    if !config_exists {
        events.emit(AppEvent::ConfigCreated {
            path: abs_config_path,
        });
    }

    events.emit(AppEvent::ConfigLoaded {
        servers_count: config.servers.len(),
    });

    Ok(config)
}
