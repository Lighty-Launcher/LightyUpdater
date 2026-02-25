use crate::config_io_errors::ConfigIoError;
use crate::config_io_validation::validate_config;
use lighty_config::Config;
use std::path::Path;
use std::sync::Arc;

type Result<T> = std::result::Result<T, ConfigIoError>;

pub async fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    load_config_with_events(path, None).await
}

pub async fn load_config_with_events<P: AsRef<Path>>(
    path: P,
    events: Option<&Arc<lighty_events::EventBus>>,
) -> Result<Config> {
    let path = path.as_ref();

    if !path.exists() {
        create_default_config(path).await?;
    }

    crate::config_io_migration::migrate_config_if_needed(path, events).await?;

    let content = tokio::fs::read_to_string(path).await?;
    let config: Config = toml::from_str(&content)?;
    validate_config(&config)?;

    Ok(config)
}

pub async fn load_config_no_migration<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = tokio::fs::read_to_string(path.as_ref()).await?;
    let config: Config = toml::from_str(&content)?;
    validate_config(&config)?;
    Ok(config)
}

async fn create_default_config<P: AsRef<Path>>(path: P) -> Result<()> {
    tokio::fs::write(path, lighty_config::DEFAULT_CONFIG_TEMPLATE).await?;
    Ok(())
}
