use crate::errors::CliResult;
use crate::paths;
use colored::Colorize;
use std::path::Path;

pub fn execute() -> CliResult<()> {
    // Get the current executable path
    let current_exe = std::env::current_exe()?;

    // Create bin directory in lighty home
    paths::ensure_directories()?;
    let bin_dir = paths::lighty_home()?.join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    // Determine the binary name
    let binary_name = if cfg!(windows) {
        "lighty.exe"
    } else {
        "lighty"
    };

    let target_path = bin_dir.join(binary_name);

    // Check if already installed
    if target_path.exists() && target_path.canonicalize()? == current_exe.canonicalize()? {
        println!();
        println!("{}", "lighty is already installed!".green());
        println!("  Location: {}", target_path.display().to_string().cyan());
        println!();
        return Ok(());
    }

    // Copy the binary
    println!();
    println!("{}", "Installing lighty...".yellow());
    println!("  From: {}", current_exe.display().to_string().cyan());
    println!("  To:   {}", target_path.display().to_string().cyan());

    std::fs::copy(&current_exe, &target_path)?;

    // Make executable on Unix (automatic chmod)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target_path, perms)?;
        tracing::info!("✓ Set executable permissions (chmod +x)");
    }

    println!();
    println!("{}", "Installation successful!".green());
    println!();

    // Check if bin directory is in PATH
    if !is_in_path(&bin_dir) {
        println!("{}", "Adding lighty to PATH...".yellow());
        println!();

        let path_added = add_to_path_automatically(&bin_dir);

        if path_added {
            println!("{}", "✓ Successfully added to PATH!".green());
            println!();
            println!("{}", "Please restart your terminal or run:".bold());

            #[cfg(target_os = "windows")]
            {
                println!("  {}", "Restart PowerShell/CMD".cyan());
            }

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            {
                let shell_rc = detect_shell_rc();
                println!("  {}", format!("source {}", shell_rc).cyan());
            }
        } else {
            println!("{}", "⚠ Could not automatically add to PATH. Please add manually:".yellow());
            println!();

            #[cfg(target_os = "windows")]
            {
                println!("  {}", "PowerShell (Run as Administrator):".bold());
                println!("    {}", format!(
                    "[Environment]::SetEnvironmentVariable(\"Path\", $env:Path + \";{}\", \"Machine\")",
                    bin_dir.display()
                ).cyan());
                println!();
                println!("  {}", "Or for current user only:".bold());
                println!("    {}", format!(
                    "[Environment]::SetEnvironmentVariable(\"Path\", $env:Path + \";{}\", \"User\")",
                    bin_dir.display()
                ).cyan());
            }

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            {
                let shell_rc = detect_shell_rc();
                println!("  {}", format!("Add to {}:", shell_rc).bold());
                println!("    {}", format!("export PATH=\"{}:$PATH\"", bin_dir.display()).cyan());
                println!();
                println!("  Then run:");
                println!("    {}", format!("source {}", shell_rc).cyan());
            }
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

fn is_in_path(dir: &Path) -> bool {
    if let Ok(path_env) = std::env::var("PATH") {
        let canonical_dir = match dir.canonicalize() {
            Ok(d) => d,
            Err(_) => return false,
        };

        for path in std::env::split_paths(&path_env) {
            if let Ok(canonical_path) = path.canonicalize() {
                if canonical_path == canonical_dir {
                    return true;
                }
            }
        }
    }
    false
}

/// Automatically add directory to PATH
fn add_to_path_automatically(bin_dir: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        add_to_path_windows(bin_dir)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        add_to_path_unix(bin_dir)
    }
}

#[cfg(target_os = "windows")]
fn add_to_path_windows(bin_dir: &Path) -> bool {
    use std::process::Command;

    let bin_path = bin_dir.display().to_string();

    // Try to add to User PATH (doesn't require admin)
    let result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$currentPath = [Environment]::GetEnvironmentVariable('Path', 'User'); \
                 if ($currentPath -notlike '*{}*') {{ \
                     [Environment]::SetEnvironmentVariable('Path', $currentPath + ';{}', 'User'); \
                     exit 0 \
                 }} else {{ \
                     exit 1 \
                 }}",
                bin_path, bin_path
            ),
        ])
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn add_to_path_unix(bin_dir: &Path) -> bool {
    let shell_rc = detect_shell_rc();
    let export_line = format!("\n# Added by lighty installer\nexport PATH=\"{}:$PATH\"\n", bin_dir.display());

    // Read current content
    let current_content = std::fs::read_to_string(&shell_rc).unwrap_or_default();

    // Check if already added
    if current_content.contains(&bin_dir.display().to_string()) {
        return true;
    }

    // Append to shell RC file
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&shell_rc)
    {
        file.write_all(export_line.as_bytes()).is_ok()
    } else {
        false
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn detect_shell_rc() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    // Check for zsh (most common on macOS)
    let zshrc = format!("{}/.zshrc", home);
    if std::path::Path::new(&zshrc).exists() || std::env::var("SHELL").unwrap_or_default().contains("zsh") {
        return zshrc;
    }

    // Check for bash
    let bashrc = format!("{}/.bashrc", home);
    if std::path::Path::new(&bashrc).exists() {
        return bashrc;
    }

    // Default to bash_profile for macOS
    #[cfg(target_os = "macos")]
    {
        format!("{}/.bash_profile", home)
    }

    #[cfg(target_os = "linux")]
    {
        bashrc
    }
}

pub fn check_and_suggest_install() {
    // Only suggest if we're not already in a standard location
    if let Ok(current_exe) = std::env::current_exe() {
        if let Ok(lighty_home) = paths::lighty_home() {
            let bin_dir = lighty_home.join("bin");

            // Skip if already installed
            if current_exe.starts_with(&bin_dir) {
                return;
            }

            // Skip if in PATH
            if let Some(parent) = current_exe.parent() {
                if is_in_path(parent) {
                    return;
                }
            }

            // Suggest installation
            println!();
            println!("{}", "Note: lighty is not installed in your PATH.".yellow());
            println!("Run {} to install it globally:", format!("{} install", current_exe.display()).cyan());
            println!();
        }
    }
}
