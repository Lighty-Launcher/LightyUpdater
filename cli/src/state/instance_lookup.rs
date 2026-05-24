use crate::errors::{CliError, CliResult};
use crate::state::registry::{Instance, Registry};
use std::env;

pub fn resolve_instance<'a>(
    registry: &'a Registry,
    name: Option<&str>,
) -> CliResult<&'a Instance> {
    if let Some(name) = name {
        return registry.get_instance(name);
    }

    let current_dir = env::current_dir()?;
    registry
        .find_by_directory(&current_dir)
        .ok_or(CliError::NoInstanceInCurrentDir)
}
