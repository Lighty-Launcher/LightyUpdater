use crate::errors::{CliError, CliResult};
use std::path::PathBuf;

/// Get the lighty home directory (~/.lighty)
pub fn lighty_home() -> CliResult<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| CliError::Other("Could not find home directory".to_string()))?;
    Ok(home.join(".lighty"))
}

/// Get the instances registry file path (~/.lighty/instances.json)
pub fn instances_file() -> CliResult<PathBuf> {
    Ok(lighty_home()?.join("instances.json"))
}

/// Get the instances PID directory (~/.lighty/instances/)
pub fn instances_dir() -> CliResult<PathBuf> {
    Ok(lighty_home()?.join("instances"))
}

/// Get the PID file path for an instance
pub fn pid_file(name: &str) -> CliResult<PathBuf> {
    Ok(instances_dir()?.join(format!("{}.pid", name)))
}

/// Get the logs directory (~/.lighty/logs/)
pub fn logs_dir() -> CliResult<PathBuf> {
    Ok(lighty_home()?.join("logs"))
}

/// Get the log file path for an instance
pub fn log_file(name: &str) -> CliResult<PathBuf> {
    Ok(logs_dir()?.join(format!("{}.log", name)))
}

/// Ensure all required directories exist
pub fn ensure_directories() -> CliResult<()> {
    std::fs::create_dir_all(lighty_home()?)?;
    std::fs::create_dir_all(instances_dir()?)?;
    std::fs::create_dir_all(logs_dir()?)?;
    Ok(())
}
