use std::env;
use std::path::Path;

/// Resolve which shell-rc file an `export PATH=...` line should be appended to
/// when running on Unix-like systems. Looks at `$SHELL`, then falls back to
/// existing files in `$HOME`, then to `.bash_profile` (macOS) / `.bashrc`
/// (Linux) when nothing else is present.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn detect_shell_rc() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());

    let zshrc = format!("{}/.zshrc", home);
    if Path::new(&zshrc).exists() || env::var("SHELL").unwrap_or_default().contains("zsh") {
        return zshrc;
    }

    let bashrc = format!("{}/.bashrc", home);
    if Path::new(&bashrc).exists() {
        return bashrc;
    }

    #[cfg(target_os = "macos")]
    {
        format!("{}/.bash_profile", home)
    }

    #[cfg(target_os = "linux")]
    {
        bashrc
    }
}

/// Is `dir` (canonicalised) present in the current process's `$PATH`?
pub fn is_in_path(dir: &Path) -> bool {
    let path_env = match env::var("PATH") {
        Ok(value) => value,
        Err(_) => return false,
    };
    let canonical_dir = match dir.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => return false,
    };

    for path in env::split_paths(&path_env) {
        if let Ok(canonical) = path.canonicalize() {
            if canonical == canonical_dir {
                return true;
            }
        }
    }
    false
}
