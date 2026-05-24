use crate::errors::CliResult;
use crate::process::daemon;
use crate::state::paths;
use crate::state::registry::Registry;
use crate::ui::install_hint::{print_manual_home_remove, print_manual_path_remove};
use colored::Colorize;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use sysinfo::System;

pub fn execute(force: bool) -> CliResult<()> {
    let registry = Registry::load()?;
    let running = running_instances(&registry)?;

    if !running.is_empty() && !force {
        warn_running_instances(&running);
        return Ok(());
    }
    if force && !running.is_empty() {
        stop_running_instances(&running);
    }

    let lighty_home = paths::lighty_home()?;
    if !lighty_home.exists() {
        println!();
        println!("{}", "No lighty data directory found".yellow());
        return Ok(());
    }

    match fs::remove_dir_all(&lighty_home) {
        Ok(_) => print_success(&lighty_home),
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            partial_cleanup(&lighty_home)?;
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn running_instances(registry: &Registry) -> CliResult<Vec<String>> {
    let mut sys = System::new();
    let mut running = Vec::new();
    for instance in registry.all_instances() {
        if let Ok(Some(_)) = daemon::read_pid_with_system(&instance.name, &mut sys) {
            running.push(instance.name.clone());
        }
    }
    Ok(running)
}

fn warn_running_instances(running: &[String]) {
    println!();
    println!("{}", "Warning: The following instances are still running:".yellow());
    for name in running {
        println!("  - {}", name.cyan());
    }
    println!();
    println!(
        "Stop them first with {} or use {} to force uninstall",
        "lighty stop".cyan(),
        "--force".cyan()
    );
    println!();
}

fn stop_running_instances(running: &[String]) {
    println!();
    println!("{}", "Stopping all running instances...".yellow());
    for name in running {
        match daemon::stop_daemon(name, true) {
            Ok(_) => println!("  Stopped {}", name.cyan()),
            Err(error) => println!(
                "  Failed to stop {}: {}",
                name.cyan(),
                error.to_string().red()
            ),
        }
    }
}

fn print_success(lighty_home: &Path) {
    println!();
    println!("{}", "Removing lighty data...".yellow());
    println!();
    println!(
        "{}",
        "Successfully removed all lighty data and installed binary".green()
    );
    println!();
    println!("{}", "Don't forget to remove from PATH:".yellow());
    println!();
    print_manual_path_remove(&lighty_home.join("bin"));
}

fn partial_cleanup(lighty_home: &Path) -> CliResult<()> {
    println!(
        "  {}",
        "Cannot remove running binary, cleaning up data files...".yellow()
    );

    let instances_file = paths::instances_file()?;
    let logs_dir = paths::logs_dir()?;
    let instances_dir = paths::instances_dir()?;

    if instances_file.exists() {
        let _ = fs::remove_file(&instances_file);
    }
    if logs_dir.exists() {
        let _ = fs::remove_dir_all(&logs_dir);
    }
    if instances_dir.exists() {
        let _ = fs::remove_dir_all(&instances_dir);
    }

    println!();
    println!("{}", "Successfully removed lighty data".green());
    println!();
    println!("{}", "To complete uninstall:".bold());
    println!();
    println!("{}", "1. Close ALL terminals running lighty".yellow());
    println!();
    println!("{}", "2. Then delete the remaining directory:".yellow());
    print_manual_home_remove(lighty_home);
    println!();
    println!("{}", "3. Remove from PATH:".yellow());
    print_manual_path_remove(&lighty_home.join("bin"));
    Ok(())
}
