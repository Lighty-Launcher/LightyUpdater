use crate::errors::CliResult;
use crate::state::paths;
use super::platform::add_to_path_automatically;
use super::shell::is_in_path;
use crate::ui::install_hint::print_manual_path_add;
use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;

pub fn execute() -> CliResult<()> {
    let current_exe = env::current_exe()?;

    paths::ensure_directories()?;
    let bin_dir = paths::lighty_home()?.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let binary_name = if cfg!(windows) { "lighty.exe" } else { "lighty" };
    let target_path = bin_dir.join(binary_name);

    if target_path.exists() && target_path.canonicalize()? == current_exe.canonicalize()? {
        println!();
        println!("{}", "lighty is already installed!".green());
        println!("  Location: {}", target_path.display().to_string().cyan());
        println!();
        return Ok(());
    }

    println!();
    println!("{}", "Installing lighty...".yellow());
    println!("  From: {}", current_exe.display().to_string().cyan());
    println!("  To:   {}", target_path.display().to_string().cyan());

    fs::copy(&current_exe, &target_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)?;
        tracing::info!("✓ Set executable permissions (chmod +x)");
    }

    println!();
    println!("{}", "Installation successful!".green());
    println!();

    if !is_in_path(&bin_dir) {
        println!("{}", "Adding lighty to PATH...".yellow());
        println!();

        if add_to_path_automatically(&bin_dir) {
            println!("{}", "✓ Successfully added to PATH!".green());
            println!();
            println!("{}", "Please restart your terminal or run:".bold());

            #[cfg(target_os = "windows")]
            {
                println!("  {}", "Restart PowerShell/CMD".cyan());
            }
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            {
                let shell_rc = super::shell::detect_shell_rc();
                println!("  {}", format!("source {}", shell_rc).cyan());
            }
        } else {
            println!(
                "{}",
                "⚠ Could not automatically add to PATH. Please add manually:".yellow()
            );
            println!();
            print_manual_path_add(&bin_dir);
        }

        println!();
        println!("{}", "After restarting terminal, run:".yellow());
        println!("  {}", "lighty --help".cyan());
    } else {
        println!("{}", "✓ The lighty binary is already in your PATH!".green());
        println!("You can now run {} from anywhere.", "lighty".cyan());
    }

    println!();
    Ok(())
}

pub fn check_and_suggest_install() {
    let Ok(current_exe) = env::current_exe() else { return };
    let Ok(lighty_home) = paths::lighty_home() else { return };
    let bin_dir = lighty_home.join("bin");

    if current_exe.starts_with(&bin_dir) {
        return;
    }

    if let Some(parent) = current_exe.parent() {
        if is_in_path(parent) {
            return;
        }
    }

    suggest_install(&current_exe);
}

fn suggest_install(current_exe: &Path) {
    println!();
    println!("{}", "Note: lighty is not installed in your PATH.".yellow());
    println!(
        "Run {} to install it globally:",
        format!("{} install", current_exe.display()).cyan()
    );
    println!();
}
