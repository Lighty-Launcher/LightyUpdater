use super::defaults::DEFAULT_CONFIG_TEMPLATE;
use super::migration::migrate_config_if_needed;
use super::models::Config;
use std::path::Path;
use std::sync::Arc;

impl Config {
    /// Loads configuration from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        Self::from_file_with_events(path, None).await
    }

    /// Loads configuration from a file with optional event bus for notifications
    pub async fn from_file_with_events<P: AsRef<Path>>(
        path: P,
        events: Option<&Arc<lighty_events::EventBus>>,
    ) -> anyhow::Result<Self> {
        let path = path.as_ref();

        // Create default config if it doesn't exist
        if !path.exists() {
            create_default_config(path).await?;
        }

        // Migrate config if needed
        migrate_config_if_needed(path, events).await?;

        // Read and parse config
        let content = tokio::fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }
}

/// Creates a default configuration file
async fn create_default_config<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    tokio::fs::write(path, DEFAULT_CONFIG_TEMPLATE).await?;
    Ok(())
}
