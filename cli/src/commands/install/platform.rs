use std::path::Path;

/// Try to inject `bin_dir` into the OS PATH automatically. Returns `true` when
/// the change was committed (or already present), `false` if the operation
/// failed and the user has to run the commands manually.
pub fn add_to_path_automatically(bin_dir: &Path) -> bool {
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
    use super::shell::detect_shell_rc;
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    let shell_rc = detect_shell_rc();
    let bin_display = bin_dir.display().to_string();
    let export_line = format!("\n# Added by lighty installer\nexport PATH=\"{}:$PATH\"\n", bin_display);

    let current_content = fs::read_to_string(&shell_rc).unwrap_or_default();
    if current_content.contains(&bin_display) {
        return true;
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&shell_rc) {
        file.write_all(export_line.as_bytes()).is_ok()
    } else {
        false
    }
}
