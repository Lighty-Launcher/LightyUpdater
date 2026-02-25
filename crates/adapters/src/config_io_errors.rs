use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigIoError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("TOML edit error: {0}")]
    TomlEditError(#[from] toml_edit::TomlError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
