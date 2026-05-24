use colored::Colorize;
use std::path::Path;

/// Commands the user should run to add `bin_dir` to PATH manually. Used when
/// the automatic PATH-injection in `install` fails.
pub fn print_manual_path_add(bin_dir: &Path) {
    #[cfg(target_os = "windows")]
    {
        println!("  {}", "PowerShell (Run as Administrator):".bold());
        println!(
            "    {}",
            format!(
                "[Environment]::SetEnvironmentVariable(\"Path\", $env:Path + \";{}\", \"Machine\")",
                bin_dir.display()
            )
            .cyan()
        );
        println!();
        println!("  {}", "Or for current user only:".bold());
        println!(
            "    {}",
            format!(
                "[Environment]::SetEnvironmentVariable(\"Path\", $env:Path + \";{}\", \"User\")",
                bin_dir.display()
            )
            .cyan()
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        println!("  {}", "Add to ~/.bashrc or ~/.zshrc:".bold());
        println!(
            "    {}",
            format!("export PATH=\"{}:$PATH\"", bin_dir.display()).cyan()
        );
        println!();
        println!("  Then run:");
        println!("    {}", "source ~/.bashrc  # or ~/.zshrc".cyan());
    }
}

/// Commands the user should run to remove `bin_dir` from PATH. Used by
/// `uninstall` after wiping the lighty home directory.
pub fn print_manual_path_remove(bin_dir: &Path) {
    #[cfg(target_os = "windows")]
    {
        println!("   {}", "PowerShell:".bold());
        println!(
            "     {}",
            "$path = [Environment]::GetEnvironmentVariable(\"Path\", \"User\")".cyan()
        );
        println!(
            "     {}",
            format!("$newPath = $path -replace \";?{}[;]?\", \"\"", bin_dir.display()).cyan()
        );
        println!(
            "     {}",
            "[Environment]::SetEnvironmentVariable(\"Path\", $newPath, \"User\")".cyan()
        );
        println!();
        println!(
            "   {}",
            "Or manually remove from System > Environment Variables".yellow()
        );
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("   {}", "Remove this line from ~/.bashrc or ~/.zshrc:".bold());
        println!(
            "     {}",
            format!("export PATH=\"{}:$PATH\"", bin_dir.display()).cyan()
        );
    }
}

/// Commands the user must run themselves when the lighty home directory cannot
/// be deleted (typically because the running binary still holds it open on
/// Windows). Used by `uninstall`'s partial-cleanup branch.
pub fn print_manual_home_remove(lighty_home: &Path) {
    #[cfg(target_os = "windows")]
    {
        println!("   {}", "PowerShell:".bold());
        println!(
            "     {}",
            format!("Remove-Item -Recurse -Force \"{}\"", lighty_home.display()).cyan()
        );
        println!();
        println!("   {}", "Command Prompt:".bold());
        println!(
            "     {}",
            format!("rmdir /s /q \"{}\"", lighty_home.display()).cyan()
        );
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!(
            "   {}",
            format!("rm -rf \"{}\"", lighty_home.display()).cyan()
        );
    }
}
