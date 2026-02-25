use crate::errors::{CliError, CliResult};
use serde::Deserialize;
use std::path::{Path, PathBuf};

const DEFAULT_UPDATER_PATH: &str = "updater";

#[derive(Debug, Default, Deserialize)]
struct CliConfig {
    #[serde(default)]
    server: CliServerConfig,
}

#[derive(Debug, Default, Deserialize)]
struct CliServerConfig {
    port: Option<u16>,
    base_path: Option<String>,
}

fn read_config(path: &Path) -> CliResult<CliConfig> {
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

pub fn read_server_port(path: &Path) -> CliResult<u16> {
    let config = read_config(path)?;
    config
        .server
        .port
        .ok_or_else(|| CliError::Config("Could not extract port from config.toml".to_string()))
}

pub fn resolve_updater_path(path: &Path) -> CliResult<PathBuf> {
    let config = read_config(path)?;
    let base_path = config
        .server
        .base_path
        .unwrap_or_else(|| DEFAULT_UPDATER_PATH.to_string());

    if Path::new(&base_path).is_absolute() {
        return Ok(PathBuf::from(base_path));
    }

    let parent = path.parent().ok_or_else(|| {
        CliError::Config(format!(
            "Could not resolve config directory for {}",
            path.display()
        ))
    })?;
    Ok(parent.join(base_path))
}
