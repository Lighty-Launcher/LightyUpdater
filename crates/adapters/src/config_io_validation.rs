use crate::config_io_errors::ConfigIoError;
use lighty_config::Config;

type Result<T> = std::result::Result<T, ConfigIoError>;

pub(crate) fn validate_config(config: &Config) -> Result<()> {
    ensure_non_zero("cache.hash_concurrency", config.cache.hash_concurrency)?;
    ensure_non_zero(
        "cache.config_reload_channel_size",
        config.cache.config_reload_channel_size,
    )?;
    ensure_non_zero("cache.checksum_buffer_size", config.cache.checksum_buffer_size)?;
    ensure_non_zero("cache.batch.client", config.cache.batch.client)?;
    ensure_non_zero("cache.batch.libraries", config.cache.batch.libraries)?;
    ensure_non_zero("cache.batch.mods", config.cache.batch.mods)?;
    ensure_non_zero("cache.batch.natives", config.cache.batch.natives)?;
    ensure_non_zero("cache.batch.assets", config.cache.batch.assets)?;
    Ok(())
}

fn ensure_non_zero(path: &str, value: usize) -> Result<()> {
    if value == 0 {
        return Err(ConfigIoError::InvalidConfig(format!(
            "{} must be greater than 0",
            path
        )));
    }
    Ok(())
}
