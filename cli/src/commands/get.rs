use crate::errors::CliResult;
use crate::process::daemon;
use crate::state::config_file::resolve_updater_path;
use crate::state::registry::{Instance, Registry};
use crate::ui::format::{InstanceStatus, OutputFormat, status_colored, status_text};
use colored::Colorize;
use std::thread;
use std::time::Duration;
use sysinfo::System;

const WATCH_REFRESH: Duration = Duration::from_secs(2);
const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";

pub fn execute(output: Option<OutputFormat>, watch: bool) -> CliResult<()> {
    if !watch {
        return render_once(output);
    }

    loop {
        print!("{}", CLEAR_SCREEN);
        render_once(output)?;
        thread::sleep(WATCH_REFRESH);
    }
}

fn render_once(output: Option<OutputFormat>) -> CliResult<()> {
    let registry = Registry::load()?;
    let instances = registry.all_instances();

    if instances.is_empty() {
        print_empty();
        return Ok(());
    }

    let mut sys = System::new();
    match output {
        Some(OutputFormat::Json) => render_json(&instances, &mut sys),
        Some(OutputFormat::Wide) => render_wide(&instances, &mut sys),
        None => render_table(&instances, &mut sys),
    }
}

fn print_empty() {
    println!();
    println!("{}", "No instances found".yellow());
    println!();
    println!("  Create an instance: {}", "lighty create my-instance".cyan());
    println!();
}

fn render_table(instances: &[&Instance], sys: &mut System) -> CliResult<()> {
    println!();
    println!("{}", format!("{} instances", instances.len()).bold());
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
        let pid = daemon::read_pid_with_system(&instance.name, sys)?;
        let pid_cell = pid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "  {:<20} {:<10} {:<8} {}",
            instance.name.cyan(),
            status_colored(pid),
            pid_cell.cyan(),
            instance.directory.display().to_string().cyan()
        );
    }
    println!();
    Ok(())
}

fn render_wide(instances: &[&Instance], sys: &mut System) -> CliResult<()> {
    println!();
    println!("{}", format!("{} instances", instances.len()).bold());
    println!();
    println!(
        "  {:<20} {:<10} {:<8} {:<8} {}",
        "NAME".bold(),
        "STATUS".bold(),
        "PID".bold(),
        "PORT".bold(),
        "DIRECTORY".bold()
    );
    println!("  {}", "-".repeat(96));

    for instance in instances {
        let pid = daemon::read_pid_with_system(&instance.name, sys)?;
        let pid_cell = pid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "  {:<20} {:<10} {:<8} {:<8} {}",
            instance.name.cyan(),
            status_colored(pid),
            pid_cell.cyan(),
            instance.port.to_string().cyan(),
            instance.directory.display().to_string().cyan()
        );
    }
    println!();
    Ok(())
}

fn render_json(instances: &[&Instance], sys: &mut System) -> CliResult<()> {
    let mut payload = Vec::with_capacity(instances.len());
    for instance in instances {
        let pid = daemon::read_pid_with_system(&instance.name, sys)?;
        let updater_path = resolve_updater_path(&instance.config_path)?;
        payload.push(InstanceStatus {
            name: instance.name.clone(),
            directory: instance.directory.display().to_string(),
            config_path: instance.config_path.display().to_string(),
            updater_path: updater_path.display().to_string(),
            port: instance.port,
            status: status_text(pid).to_string(),
            pid,
        });
    }
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
