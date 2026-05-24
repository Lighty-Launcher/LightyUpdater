use crate::errors::CliResult;
use crate::state::instance_lookup::resolve_instance;
use crate::state::paths;
use crate::state::registry::Registry;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn execute(name: Option<String>, follow: bool, tail: usize) -> CliResult<()> {
    let registry = Registry::load()?;
    let instance = resolve_instance(&registry, name.as_deref())?;

    let log_path = paths::log_file(&instance.name)?;

    if !log_path.exists() {
        println!("No logs found for instance '{}'", instance.name);
        println!("Log file: {}", log_path.display());
        return Ok(());
    }

    if follow {
        // Follow mode: tail -f
        follow_logs(&log_path, tail)?;
    } else {
        // Print last N lines
        print_last_lines(&log_path, tail)?;
    }

    Ok(())
}

fn print_last_lines(log_path: &Path, tail: usize) -> CliResult<()> {
    if tail == 0 {
        return Ok(());
    }

    const CHUNK_SIZE: usize = 8 * 1024;

    let mut file = File::open(log_path)?;
    let mut pos = file.metadata()?.len();
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let mut newline_count = 0usize;

    while pos > 0 && newline_count <= tail {
        let read_size = std::cmp::min(CHUNK_SIZE as u64, pos) as usize;
        pos -= read_size as u64;

        file.seek(SeekFrom::Start(pos))?;

        let mut chunk = vec![0u8; read_size];
        file.read_exact(&mut chunk)?;

        newline_count += chunk.iter().filter(|&&b| b == b'\n').count();
        chunks.push(chunk);
    }

    let total_len: usize = chunks.iter().map(|chunk| chunk.len()).sum();
    let mut data = Vec::with_capacity(total_len);
    for chunk in chunks.iter().rev() {
        data.extend_from_slice(chunk);
    }

    let content = String::from_utf8_lossy(&data);
    let mut selected: Vec<&str> = content.lines().rev().take(tail).collect();
    selected.reverse();

    for line in selected {
        println!("{}", line);
    }

    Ok(())
}

fn follow_logs(log_path: &Path, initial_tail: usize) -> CliResult<()> {
    // Print last N lines first
    print_last_lines(log_path, initial_tail)?;

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
            Err(error) => {
                eprintln!("Error reading log file: {}", error);
                break;
            }
        }
    }

    Ok(())
}
