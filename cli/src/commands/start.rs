use crate::daemon;
use crate::errors::{CliError, CliResult};
use crate::registry::Registry;
use colored::Colorize;
use std::env;

pub fn execute(name: Option<String>) -> CliResult<()> {
    let registry = Registry::load()?;

    // Find the instance
    let instance = if let Some(name) = &name {
        // Start by name
        registry.get_instance(name)?
    } else {
        // Start from current directory
        let current_dir = env::current_dir()?;
        registry
            .find_by_directory(&current_dir)
            .ok_or(CliError::NoInstanceInCurrentDir)?
    };

    // Check if already running
    if let Some(pid) = daemon::read_pid(&instance.name)? {
        return Err(CliError::InstanceAlreadyRunning(
            instance.name.clone(),
            pid,
        ));
    }

    // Start the daemon
    println!("{}", format!("Starting instance '{}'...", instance.name).bold());
    println!("  Working directory: {}", instance.directory.display().to_string().cyan());
    println!("  Config: {}", instance.config_path.display().to_string().cyan());
    println!("  Port: {}", instance.port.to_string().cyan());
    println!();

    let pid = daemon::start_daemon(
        &instance.name,
        &instance.directory,
        &instance.config_path,
    )?;

    let log_path = crate::paths::log_file(&instance.name)?;

    println!("{}", "Instance started successfully".green().bold());
    println!("  PID: {}", pid.to_string().cyan());
    println!("  Logs: {}", log_path.display().to_string().cyan());
    println!();
    println!("  {}", "View logs:".bold());
    println!("    {}", format!("lighty logs -n {}", instance.name).cyan());
    println!("    {}", format!("lighty logs -n {} -f", instance.name).cyan());
    println!();

    Ok(())
}
