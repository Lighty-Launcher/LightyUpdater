use crate::daemon;
use crate::errors::CliResult;
use crate::paths;
use crate::registry::Registry;
use colored::Colorize;

pub fn execute(force: bool) -> CliResult<()> {
    let registry = Registry::load()?;
    let instances = registry.all_instances();

    // Check if any instances are running
    let mut running_instances = Vec::new();
    let mut sys = sysinfo::System::new();
    for instance in &instances {
        if let Ok(Some(_)) = daemon::read_pid_with_system(&instance.name, &mut sys) {
            running_instances.push(instance.name.clone());
        }
    }

    if !running_instances.is_empty() && !force {
        println!();
        println!("{}", "Warning: The following instances are still running:".yellow());
        for name in &running_instances {
            println!("  - {}", name.cyan());
        }
        println!();
        println!("Stop them first with {} or use {} to force uninstall",
            "lighty stop".cyan(),
            "--force".cyan()
        );
        println!();
        return Ok(());
    }

    // Stop all running instances if force flag is set
    if force && !running_instances.is_empty() {
        println!();
        println!("{}", "Stopping all running instances...".yellow());
        for name in &running_instances {
            if let Err(e) = daemon::stop_daemon(name, true) {
                println!("  Failed to stop {}: {}", name.cyan(), e.to_string().red());
            } else {
                println!("  Stopped {}", name.cyan());
            }
        }
    }

    // Remove the .lighty directory (including the installed binary)
    let lighty_home = paths::lighty_home()?;

    if lighty_home.exists() {
        println!();
        println!("{}", "Removing lighty data...".yellow());

        // Try to remove everything
        let remove_result = std::fs::remove_dir_all(&lighty_home);

        match remove_result {
            Ok(_) => {
                println!();
                println!("{}", "Successfully removed all lighty data and installed binary".green());
                println!();
                println!("{}", "Don't forget to remove from PATH:".yellow());

                #[cfg(target_os = "windows")]
                {
                    println!();
                    println!("   {}", "PowerShell:".bold());
                    println!("     {}", "$path = [Environment]::GetEnvironmentVariable(\"Path\", \"User\")".cyan());
                    println!("     {}", format!("$newPath = $path -replace \";?{}[;]?\", \"\"", lighty_home.join("bin").display()).cyan());
                    println!("     {}", "[Environment]::SetEnvironmentVariable(\"Path\", $newPath, \"User\")".cyan());
                    println!();
                    println!("   {}", "Or manually remove from System > Environment Variables".yellow());
                }

                #[cfg(not(target_os = "windows"))]
                {
                    println!("   {}", "Remove this line from ~/.bashrc or ~/.zshrc:".bold());
                    println!("     {}", format!("export PATH=\"{}:$PATH\"", lighty_home.join("bin").display()).cyan());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                // Cannot delete because binary is running
                // Remove what we can: instances.json, logs/, instances/
                println!("  {}", "Cannot remove running binary, cleaning up data files...".yellow());

                let instances_file = paths::instances_file()?;
                let logs_dir = paths::logs_dir()?;
                let instances_dir = paths::instances_dir()?;

                // Remove instances.json
                if instances_file.exists() {
                    let _ = std::fs::remove_file(&instances_file);
                }

                // Remove logs directory
                if logs_dir.exists() {
                    let _ = std::fs::remove_dir_all(&logs_dir);
                }

                // Remove instances directory
                if instances_dir.exists() {
                    let _ = std::fs::remove_dir_all(&instances_dir);
                }

                println!();
                println!("{}", "Successfully removed lighty data".green());
                println!();
                println!("{}", "To complete uninstall:".bold());
                println!();
                println!("{}", "1. Close ALL terminals running lighty".yellow());
                println!();
                println!("{}", "2. Then delete the remaining directory:".yellow());

                #[cfg(target_os = "windows")]
                {
                    println!("   {}", "PowerShell:".bold());
                    println!("     {}", format!("Remove-Item -Recurse -Force \"{}\"", lighty_home.display()).cyan());
                    println!();
                    println!("   {}", "Command Prompt:".bold());
                    println!("     {}", format!("rmdir /s /q \"{}\"", lighty_home.display()).cyan());
                }

                #[cfg(not(target_os = "windows"))]
                {
                    println!("   {}", format!("rm -rf \"{}\"", lighty_home.display()).cyan());
                }

                println!();
                println!("{}", "3. Remove from PATH:".yellow());

                #[cfg(target_os = "windows")]
                {
                    println!("   {}", "PowerShell:".bold());
                    println!("     {}", "$path = [Environment]::GetEnvironmentVariable(\"Path\", \"User\")".cyan());
                    println!("     {}", format!("$newPath = $path -replace \";?{}[;]?\", \"\"", lighty_home.join("bin").display()).cyan());
                    println!("     {}", "[Environment]::SetEnvironmentVariable(\"Path\", $newPath, \"User\")".cyan());
                    println!();
                    println!("   {}", "Or manually remove from System > Environment Variables".yellow());
                }

                #[cfg(not(target_os = "windows"))]
                {
                    println!("   {}", "Remove this line from ~/.bashrc or ~/.zshrc:".bold());
                    println!("     {}", format!("export PATH=\"{}:$PATH\"", lighty_home.join("bin").display()).cyan());
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    } else {
        println!();
        println!("{}", "No lighty data directory found".yellow());
    }


    Ok(())
}
