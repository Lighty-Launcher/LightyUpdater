use crate::errors::{CliError, CliResult};
use crate::paths;
use std::process::{Command, Stdio};
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Check if a process with the given PID is running (optimized: reuses System instance)
pub fn is_running_with_system(sys: &mut System, pid: u32) -> bool {
    let pid_obj = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid_obj]), true);
    sys.process(pid_obj).is_some()
}

/// Read PID from file (optimized: reuses System instance)
pub fn read_pid_with_system(name: &str, sys: &mut System) -> CliResult<Option<u32>> {
    let pid_path = paths::pid_file(name)?;

    if !pid_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&pid_path)?;
    let pid: u32 = content
        .trim()
        .parse()
        .map_err(|_| CliError::Other(format!("Invalid PID in {}", pid_path.display())))?;

    // Check if process is actually running
    if is_running_with_system(sys, pid) {
        Ok(Some(pid))
    } else {
        // Clean up stale PID file
        let _ = std::fs::remove_file(&pid_path);
        Ok(None)
    }
}

/// Read PID from file (convenience wrapper, creates System)
pub fn read_pid(name: &str) -> CliResult<Option<u32>> {
    let mut sys = System::new();
    read_pid_with_system(name, &mut sys)
}

/// Write PID to file
pub fn write_pid(name: &str, pid: u32) -> CliResult<()> {
    paths::ensure_directories()?;
    let pid_path = paths::pid_file(name)?;
    std::fs::write(&pid_path, pid.to_string())?;
    Ok(())
}

/// Remove PID file
pub fn remove_pid(name: &str) -> CliResult<()> {
    let pid_path = paths::pid_file(name)?;
    if pid_path.exists() {
        std::fs::remove_file(&pid_path)?;
    }
    Ok(())
}

/// Start lighty as a daemon in server mode
pub fn start_daemon(instance_name: &str, working_dir: &std::path::Path, config_path: &std::path::Path) -> CliResult<u32> {
    // Get the current lighty binary (itself)
    let lighty_binary = std::env::current_exe()?;

    // Prepare log file
    paths::ensure_directories()?;
    let log_path = paths::log_file(instance_name)?;
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Spawn the server process using the CLI in serve mode
    // Use platform-specific flags to detach from parent process group
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;

        let child = Command::new(lighty_binary)
            .arg("serve")
            .arg("--config")
            .arg(config_path)
            .current_dir(working_dir)
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            .stdin(Stdio::null())
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .spawn()?;

        let pid = child.id();
        write_pid(instance_name, pid)?;
        Ok(pid)
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let child = Command::new(lighty_binary)
            .arg("serve")
            .arg("--config")
            .arg(config_path)
            .current_dir(working_dir)
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            .stdin(Stdio::null())
            .process_group(0)  // Create new process group
            .spawn()?;

        let pid = child.id();
        write_pid(instance_name, pid)?;
        Ok(pid)
    }
}

/// Stop a running daemon
pub fn stop_daemon(name: &str, force: bool) -> CliResult<()> {
    let pid = read_pid(name)?
        .ok_or_else(|| CliError::InstanceNotRunning(name.to_string()))?;

    let mut sys = System::new();
    let pid_obj = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid_obj]), true);

    let process = sys.process(pid_obj)
        .ok_or_else(|| CliError::InstanceNotRunning(name.to_string()))?;

    if force {
        // Force kill
        process.kill();
        remove_pid(name)?;
        return Ok(());
    }

    // Graceful shutdown
    if let Some(false) = process.kill_with(sysinfo::Signal::Term) {
        return Err(CliError::StopFailed(format!(
            "Failed to send SIGTERM to process {}",
            pid
        )));
    }

    // Wait up to 5 seconds for graceful shutdown
    // Reuse sys instance to avoid creating 50 System instances
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if !is_running_with_system(&mut sys, pid) {
            remove_pid(name)?;
            return Ok(());
        }
    }

    // If still running, suggest force
    Err(CliError::StopFailed(format!(
        "Process {} did not stop after 5 seconds. Use --force to kill it",
        pid
    )))
}
