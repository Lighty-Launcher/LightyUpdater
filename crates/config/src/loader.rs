use super::defaults::DEFAULT_CONFIG_TEMPLATE;
use super::migration::migrate_config_if_needed;
use super::models::Config;
use super::errors::ConfigError;
use std::path::Path;
use std::sync::Arc;

type Result<T> = std::result::Result<T, ConfigError>;

impl Config {
    /// Loads configuration from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with_events(path, None).await
    }

    /// Loads configuration from a file with optional event bus for notifications
    /// This version includes migration and should ONLY be used at startup
    pub async fn from_file_with_events<P: AsRef<Path>>(
        path: P,
        events: Option<&Arc<lighty_events::EventBus>>,
    ) -> Result<Self> {
        let path = path.as_ref();

        // Create default config if it doesn't exist
        if !path.exists() {
            create_default_config(path).await?;
        }

        // Migrate config if needed (ONLY at startup)
        migrate_config_if_needed(path, events).await?;

        // Read and parse config
        let content = tokio::fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }

    /// Loads configuration from a file WITHOUT migration
    /// This should be used for hot-reload to avoid re-migrating on every change
    pub async fn from_file_no_migration<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Read and parse config directly (no migration)
        let content = tokio::fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }
}

/// Creates a default configuration file
async fn create_default_config<P: AsRef<Path>>(path: P) -> Result<()> {
    tokio::fs::write(path, DEFAULT_CONFIG_TEMPLATE).await?;
    Ok(())
}
