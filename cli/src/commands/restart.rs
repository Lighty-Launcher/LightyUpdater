use crate::commands::{start, stop};
use crate::errors::CliResult;

pub fn execute(name: Option<String>, force: bool) -> CliResult<()> {
    // Stop the instance (ignore error if not running)
    let _ = stop::execute(name.clone(), force);

    // Wait a moment before restarting
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Start the instance
    start::execute(name)
}
