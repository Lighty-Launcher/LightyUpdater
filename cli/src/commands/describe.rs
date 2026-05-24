use crate::errors::CliResult;
use crate::process::daemon;
use crate::state::config_file::resolve_updater_path;
use crate::state::instance_lookup::resolve_instance;
use crate::state::registry::{Instance, Registry};
use crate::ui::format::{InstanceStatus, OutputFormat, status_colored, status_text};
use colored::Colorize;

pub fn execute(name: Option<String>, output: Option<OutputFormat>) -> CliResult<()> {
    let registry = Registry::load()?;
    let instance = resolve_instance(&registry, name.as_deref())?;
    let pid = daemon::read_pid(&instance.name)?;
    let payload = build_status(instance, pid)?;

    match output {
        Some(OutputFormat::Json) => {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        Some(OutputFormat::Wide) | None => {
            print_block(&payload, pid);
        }
    }
    Ok(())
}

fn build_status(instance: &Instance, pid: Option<u32>) -> CliResult<InstanceStatus> {
    let updater_path = resolve_updater_path(&instance.config_path)?;
    Ok(InstanceStatus {
        name: instance.name.clone(),
        directory: instance.directory.display().to_string(),
        config_path: instance.config_path.display().to_string(),
        updater_path: updater_path.display().to_string(),
        port: instance.port,
        status: status_text(pid).to_string(),
        pid,
    })
}

fn print_block(payload: &InstanceStatus, pid: Option<u32>) {
    println!();
    println!("  {}: {}", "Instance".bold(), payload.name.cyan());
    println!("  {}: {}", "Directory".bold(), payload.directory.cyan());
    println!("  {}: {}", "Config".bold(), payload.config_path.cyan());
    println!("  {}: {}", "Updater Path".bold(), payload.updater_path.cyan());
    println!("  {}: {}", "Port".bold(), payload.port.to_string().cyan());
    println!("  {}: {}", "Status".bold(), status_colored(pid));
    if let Some(pid) = pid {
        println!("  {}: {}", "PID".bold(), pid.to_string().cyan());
    }
    println!();
}
