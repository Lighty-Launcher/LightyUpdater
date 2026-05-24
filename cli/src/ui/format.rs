use clap::ValueEnum;
use colored::{ColoredString, Colorize};
use serde::Serialize;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormat {
    Json,
    Wide,
}

/// Status payload shared between commands that render an instance.
#[derive(Serialize)]
pub struct InstanceStatus {
    pub name: String,
    pub directory: String,
    pub config_path: String,
    pub updater_path: String,
    pub port: u16,
    pub status: String,
    pub pid: Option<u32>,
}

/// Lowercase "running"/"stopped" string for machine-readable output.
pub fn status_text(pid: Option<u32>) -> &'static str {
    if pid.is_some() { "running" } else { "stopped" }
}

/// Colored status for terminal output — green/red depending on liveness.
pub fn status_colored(pid: Option<u32>) -> ColoredString {
    let text = status_text(pid);
    if pid.is_some() { text.green() } else { text.red() }
}
