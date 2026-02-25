use crate::daemon;
use crate::errors::{CliError, CliResult};
use crate::instance_lookup::resolve_instance;
use crate::registry::Registry;
use colored::Colorize;

pub fn execute(name: Option<String>, force: bool) -> CliResult<()> {
    let registry = Registry::load()?;
    let instance = resolve_instance(&registry, name.as_deref())?;

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
