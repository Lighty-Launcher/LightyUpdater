use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Directory does not exist: {0}")]
    DirectoryNotFound(String),

    #[error("Instance '{0}' already exists")]
    InstanceAlreadyExists(String),

    #[error("Instance '{0}' not found")]
    InstanceNotFound(String),

    #[error("No instance found in current directory")]
    NoInstanceInCurrentDir,

    #[error("Instance '{0}' is already running (PID: {1})")]
    InstanceAlreadyRunning(String, u32),

    #[error("Instance '{0}' is not running")]
    InstanceNotRunning(String),

    #[error("Port {0} is already in use by instance '{1}'")]
    PortAlreadyInUse(u16, String),

    #[error("Failed to stop instance: {0}")]
    StopFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("{0}")]
    Other(String),
}

pub type CliResult<T> = Result<T, CliError>;
