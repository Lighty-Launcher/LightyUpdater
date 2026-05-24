use crate::process::daemon;
use crate::errors::{CliError, CliResult};
use crate::state::paths;
use crate::state::registry::Registry;
use colored::Colorize;

pub fn execute(name: String, with_logs: bool) -> CliResult<()> {
    let mut registry = Registry::load()?;

    // Check if instance exists
    let _instance = registry.get_instance(&name)?;

    // Check if instance is running
    if let Some(pid) = daemon::read_pid(&name)? {
        return Err(CliError::Other(format!(
            "Instance '{}' is currently running (PID: {}). Stop it first with: lighty stop {}",
            name, pid, name
        )));
    }

    // Remove from registry
    registry.remove_instance(&name)?;

    // Remove PID file if it exists
    let pid_file = paths::pid_file(&name)?;
    if pid_file.exists() {
        std::fs::remove_file(&pid_file)?;
    }

    // Remove logs if requested
    if with_logs {
        let log_file = paths::log_file(&name)?;
        if log_file.exists() {
            std::fs::remove_file(&log_file)?;
            println!("{}", format!("Removed log file: {}", log_file.display()).cyan());
        }
    }

    println!();
    println!(
        "{}",
        format!("Instance '{}' removed successfully", name)
            .green()
            .bold()
    );
    println!();
    println!("  {}", "Note:".bold());
    println!(
        "    The instance directory and its contents (config.toml, updater/) were {}",
        "NOT deleted".yellow()
    );
    println!("    You can manually delete the directory if needed.");
    println!();

    Ok(())
}
