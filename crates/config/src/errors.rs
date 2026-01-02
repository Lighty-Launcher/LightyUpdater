use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("TOML edit error: {0}")]
    TomlEditError(#[from] toml_edit::TomlError),

    #[error("Config file not found: {0}")]
    ConfigNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Migration failed: {0}")]
    MigrationError(String),
}
