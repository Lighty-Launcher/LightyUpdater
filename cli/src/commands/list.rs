use crate::daemon;
use crate::errors::CliResult;
use crate::registry::Registry;
use colored::Colorize;

pub fn execute() -> CliResult<()> {
    let registry = Registry::load()?;
    let instances = registry.all_instances();

    if instances.is_empty() {
        println!();
        println!("{}", "No instances found".yellow());
        println!();
        println!("  Create an instance: {}", "lighty create -n my-instance".cyan());
        println!();
        return Ok(());
    }

    println!();
    println!("{}", format!("{} instances found:", instances.len()).bold());
    println!();

    for instance in instances {
        let pid = daemon::read_pid(&instance.name)?;
        let status = if pid.is_some() {
            "running".green()
        } else {
            "stopped".red()
        };

        println!(
            "  {} {:<20} {}",
            "*".cyan(),
            instance.name.bold(),
            instance.directory.display().to_string().cyan()
        );
        println!(
            "    Port: {:<10} Status: {}",
            instance.port.to_string().cyan(),
            status
        );
    }

    println!();

    Ok(())
}
