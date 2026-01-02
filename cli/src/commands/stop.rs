use crate::daemon;
use crate::errors::{CliError, CliResult};
use crate::registry::Registry;
use colored::Colorize;
use std::env;

pub fn execute(name: Option<String>, force: bool) -> CliResult<()> {
    let registry = Registry::load()?;

    // Find the instance
    let instance = if let Some(name) = &name {
        // Stop by name
        registry.get_instance(name)?
    } else {
        // Stop from current directory
        let current_dir = env::current_dir()?;
        registry
            .find_by_directory(&current_dir)
            .ok_or(CliError::NoInstanceInCurrentDir)?
    };

    // Check if running
    let pid = daemon::read_pid(&instance.name)?
        .ok_or_else(|| CliError::InstanceNotRunning(instance.name.clone()))?;

    // Stop the daemon
    println!(
        "{}",
        format!("Stopping instance '{}' (PID: {})...", instance.name, pid).bold()
    );

    if force {
        println!("  {}", "Force kill enabled".yellow());
    }

    daemon::stop_daemon(&instance.name, force)?;

    println!();
    println!("{}", "Instance stopped successfully".green().bold());
    println!();

    Ok(())
}
