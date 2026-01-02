use crate::errors::{CliError, CliResult};
use crate::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub directory: PathBuf,
    pub config_path: PathBuf,
    pub port: u16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub instances: HashMap<String, Instance>,
}

impl Registry {
    /// Load the registry from disk, or create a new one if it doesn't exist
    pub fn load() -> CliResult<Self> {
        let registry_path = paths::instances_file()?;

        if !registry_path.exists() {
            return Ok(Registry {
                instances: HashMap::new(),
            });
        }

        let content = std::fs::read_to_string(&registry_path)?;
        let registry: Registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// Save the registry to disk
    pub fn save(&self) -> CliResult<()> {
        paths::ensure_directories()?;
        let registry_path = paths::instances_file()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&registry_path, content)?;
        Ok(())
    }

    /// Add a new instance to the registry
    pub fn add_instance(&mut self, instance: Instance) -> CliResult<()> {
        // Check if instance name already exists
        if self.instances.contains_key(&instance.name) {
            return Err(CliError::InstanceAlreadyExists(instance.name.clone()));
        }

        // Check for port conflicts
        for (name, existing) in &self.instances {
            if existing.port == instance.port {
                return Err(CliError::PortAlreadyInUse(instance.port, name.clone()));
            }
        }

        self.instances.insert(instance.name.clone(), instance);
        self.save()?;
        Ok(())
    }

    /// Remove an instance from the registry
    pub fn remove_instance(&mut self, name: &str) -> CliResult<()> {
        if self.instances.remove(name).is_none() {
            return Err(CliError::InstanceNotFound(name.to_string()));
        }
        self.save()?;
        Ok(())
    }

    /// Get an instance by name
    pub fn get_instance(&self, name: &str) -> CliResult<&Instance> {
        self.instances
            .get(name)
            .ok_or_else(|| CliError::InstanceNotFound(name.to_string()))
    }

    /// Find an instance by directory
    pub fn find_by_directory(&self, dir: &Path) -> Option<&Instance> {
        let canonical_dir = dir.canonicalize().ok()?;
        self.instances
            .values()
            .find(|instance| {
                instance
                    .directory
                    .canonicalize()
                    .ok()
                    .map(|d| d == canonical_dir)
                    .unwrap_or(false)
            })
    }

    /// Get all instances as a sorted vector
    pub fn all_instances(&self) -> Vec<&Instance> {
        let mut instances: Vec<&Instance> = self.instances.values().collect();
        instances.sort_by(|a, b| a.name.cmp(&b.name));
        instances
    }
}
