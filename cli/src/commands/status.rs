use crate::daemon;
use crate::errors::CliResult;
use crate::registry::Registry;
use colored::Colorize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct InstanceStatus {
    name: String,
    directory: String,
    config_path: String,
    updater_path: String,
    port: u16,
    status: String,
    pid: Option<u32>,
}

fn get_updater_path(config_path: &PathBuf) -> CliResult<PathBuf> {
    // Read config.toml to get base_path
    let config_content = std::fs::read_to_string(config_path)?;
    let config: toml::Value = toml::from_str(&config_content)?;

    let base_path = config
        .get("server")
        .and_then(|s| s.get("base_path"))
        .and_then(|b| b.as_str())
        .unwrap_or("updater");

    // Resolve path (absolute or relative to config directory)
    let updater_path = if std::path::Path::new(base_path).is_absolute() {
        PathBuf::from(base_path)
    } else {
        config_path.parent().unwrap().join(base_path)
    };

    Ok(updater_path)
}

pub fn execute(name: Option<String>, json: bool) -> CliResult<()> {
    let registry = Registry::load()?;

    if let Some(name) = name {
        // Show status for specific instance
        let instance = registry.get_instance(&name)?;
        let pid = daemon::read_pid(&instance.name)?;
        let status = if pid.is_some() { "running" } else { "stopped" };
        let updater_path = get_updater_path(&instance.config_path)?;

        if json {
            let status_obj = InstanceStatus {
                name: instance.name.clone(),
                directory: instance.directory.display().to_string(),
                config_path: instance.config_path.display().to_string(),
                updater_path: updater_path.display().to_string(),
                port: instance.port,
                status: status.to_string(),
                pid,
            };
            println!("{}", serde_json::to_string_pretty(&status_obj)?);
        } else {
            println!();
            println!("  {}: {}", "Instance".bold(), instance.name.cyan());
            println!("  {}: {}", "Directory".bold(), instance.directory.display().to_string().cyan());
            println!("  {}: {}", "Config".bold(), instance.config_path.display().to_string().cyan());
            println!("  {}: {}", "Updater Path".bold(), updater_path.display().to_string().cyan());
            println!("  {}: {}", "Port".bold(), instance.port.to_string().cyan());
            println!(
                "  {}: {}",
                "Status".bold(),
                if pid.is_some() {
                    status.green()
                } else {
                    status.red()
                }
            );
            if let Some(pid) = pid {
                println!("  {}: {}", "PID".bold(), pid.to_string().cyan());
            }
            println!();
        }
    } else {
        // Show status for all instances
        let instances = registry.all_instances();

        if instances.is_empty() {
            println!();
            println!("{}", "No instances found".yellow());
            println!();
            println!("  Create an instance: {}", "lighty create -n my-instance".cyan());
            println!();
            return Ok(());
        }

        // Reuse System instance to avoid creating N instances for N servers (performance fix)
        let mut sys = sysinfo::System::new();

        if json {
            let mut statuses = Vec::new();
            for instance in instances {
                let pid = crate::daemon::read_pid_with_system(&instance.name, &mut sys)?;
                let status = if pid.is_some() { "running" } else { "stopped" };
                let updater_path = get_updater_path(&instance.config_path)?;
                statuses.push(InstanceStatus {
                    name: instance.name.clone(),
                    directory: instance.directory.display().to_string(),
                    config_path: instance.config_path.display().to_string(),
                    updater_path: updater_path.display().to_string(),
                    port: instance.port,
                    status: status.to_string(),
                    pid,
                });
            }
            println!("{}", serde_json::to_string_pretty(&statuses)?);
        } else {
            println!();
            println!("{}", format!("{} instances:", instances.len()).bold());
            println!();
            println!(
                "  {:<20} {:<10} {:<8} {}",
                "NAME".bold(),
                "STATUS".bold(),
                "PID".bold(),
                "DIRECTORY".bold()
            );
            println!("  {}", "-".repeat(80));

            for instance in instances {
                let pid = crate::daemon::read_pid_with_system(&instance.name, &mut sys)?;
                let status = if pid.is_some() { "running" } else { "stopped" };

                println!(
                    "  {:<20} {:<10} {:<8} {}",
                    instance.name.cyan(),
                    if pid.is_some() {
                        status.green()
                    } else {
                        status.red()
                    },
                    pid.map(|p| p.to_string())
                        .unwrap_or_else(|| "-".to_string())
                        .cyan(),
                    instance.directory.display().to_string().cyan()
                );
            }
            println!();
        }
    }

    Ok(())
}
