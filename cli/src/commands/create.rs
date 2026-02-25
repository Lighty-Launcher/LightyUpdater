use crate::errors::{CliError, CliResult};
use crate::config_file::read_server_port;
use crate::registry::{Instance, Registry};
use chrono::Utc;
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(name: String, dir: Option<String>) -> CliResult<()> {
    // Resolve target directory
    let target_dir = resolve_directory(dir)?;

    // Verify directory exists
    if !target_dir.exists() {
        return Err(CliError::DirectoryNotFound(
            target_dir.display().to_string(),
        ));
    }

    // Load registry
    let mut registry = Registry::load()?;

    // Check if instance name already exists
    if registry.instances.contains_key(&name) {
        return Err(CliError::InstanceAlreadyExists(name));
    }

    // Create config.toml
    let config_path = target_dir.join("config.toml");
    if config_path.exists() {
        println!(
            "{}",
            format!(
                "Warning: config.toml already exists in {}",
                target_dir.display()
            )
            .yellow()
        );
        println!("{}", "Skipping config.toml creation".yellow());
    } else {
        std::fs::write(&config_path, lighty_config::DEFAULT_CONFIG_TEMPLATE)?;
    }

    // Create updater/ directory
    let updater_dir = target_dir.join("updater");
    std::fs::create_dir_all(&updater_dir)?;

    // Parse config to extract port
    let port = read_server_port(&config_path)?;

    // Check for port conflicts
    for (existing_name, existing_instance) in &registry.instances {
        if existing_instance.port == port {
            println!(
                "{}",
                format!(
                    "Warning: Port {} is already in use by instance '{}'",
                    port, existing_name
                )
                .yellow()
            );
            println!(
                "{}",
                "Consider changing the port in config.toml".yellow()
            );
            break;
        }
    }

    // Create instance entry
    let instance = Instance {
        name: name.clone(),
        directory: target_dir.clone(),
        config_path: config_path.clone(),
        port,
        created_at: Utc::now(),
    };

    // Add to registry
    registry.add_instance(instance)?;

    // Success message
    println!();
    println!(
        "{}",
        format!("Instance '{}' created successfully", name)
            .green()
            .bold()
    );
    println!();
    println!("  {}", "Files created:".bold());
    println!(
        "    {} {}",
        "*".green(),
        config_path.display().to_string().cyan()
    );
    println!(
        "    {} {}",
        "*".green(),
        updater_dir.display().to_string().cyan()
    );
    println!();
    println!("  {}", "Working directory:".bold());
    println!("    {}", target_dir.display().to_string().cyan());
    println!();
    println!("  {}", "Next steps:".bold());
    println!("    1. Edit the config: {}", "nano config.toml".cyan());
    println!("    2. Start the instance: {}", "lighty start".cyan());
    println!();

    Ok(())
}

/// Resolve the target directory for the instance
fn resolve_directory(dir: Option<String>) -> CliResult<PathBuf> {
    match dir {
        Some(path) => {
            let path_buf = PathBuf::from(path);
            if path_buf.is_absolute() {
                Ok(path_buf)
            } else {
                // Relative path: resolve from current directory
                let current_dir = std::env::current_dir()?;
                Ok(current_dir.join(path_buf))
            }
        }
        None => {
            // No directory specified: use current directory
            Ok(std::env::current_dir()?)
        }
    }
}
