use crate::errors::{CliError, CliResult};
use crate::paths;
use crate::registry::Registry;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::thread;
use std::time::Duration;

pub fn execute(name: Option<String>, follow: bool, lines: usize) -> CliResult<()> {
    let registry = Registry::load()?;

    // Find the instance
    let instance = if let Some(name) = &name {
        // Logs by name
        registry.get_instance(name)?
    } else {
        // Logs from current directory
        let current_dir = env::current_dir()?;
        registry
            .find_by_directory(&current_dir)
            .ok_or(CliError::NoInstanceInCurrentDir)?
    };

    let log_path = paths::log_file(&instance.name)?;

    if !log_path.exists() {
        println!("No logs found for instance '{}'", instance.name);
        println!("Log file: {}", log_path.display());
        return Ok(());
    }

    if follow {
        // Follow mode: tail -f
        follow_logs(&log_path, lines)?;
    } else {
        // Print last N lines
        print_last_lines(&log_path, lines)?;
    }

    Ok(())
}

fn print_last_lines(log_path: &std::path::Path, lines: usize) -> CliResult<()> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);

    let all_lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()?;

    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    for line in &all_lines[start..] {
        println!("{}", line);
    }

    Ok(())
}

fn follow_logs(log_path: &std::path::Path, initial_lines: usize) -> CliResult<()> {
    // Print last N lines first
    print_last_lines(log_path, initial_lines)?;

    // Follow new lines
    let mut file = File::open(log_path)?;
    file.seek(SeekFrom::End(0))?;

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    println!("--- Following logs (Ctrl+C to stop) ---");

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // No new data, wait a bit
                thread::sleep(Duration::from_millis(100));
            }
            Ok(_) => {
                print!("{}", line);
            }
            Err(e) => {
                eprintln!("Error reading log file: {}", e);
                break;
            }
        }
    }

    Ok(())
}
