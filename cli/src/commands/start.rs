use crate::process::daemon;
use crate::errors::{CliError, CliResult};
use crate::state::instance_lookup::resolve_instance;
use crate::state::registry::Registry;
use colored::Colorize;

pub fn execute(name: Option<String>) -> CliResult<()> {
    let registry = Registry::load()?;
    let instance = resolve_instance(&registry, name.as_deref())?;

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

    let log_path = crate::state::paths::log_file(&instance.name)?;

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
