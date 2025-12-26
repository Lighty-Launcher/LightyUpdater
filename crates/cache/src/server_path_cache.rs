use lighty_config::ServerConfig;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cache for fast server path lookups
/// Paths are sorted by length (longest first) for quick prefix matching
/// Used by file watcher to quickly determine which server a file belongs to
pub struct ServerPathCache {
    /// Vec of (server_path, server_name) sorted by path length (descending)
    /// Sorted order ensures most specific paths are checked first
    paths: Arc<RwLock<Vec<(PathBuf, String)>>>,
}

impl ServerPathCache {
    pub fn new() -> Self {
        Self {
            paths: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Rebuild entire cache from server configs
    /// Called on initialization and config reload
    /// Paths are sorted by length (longest first) for efficient matching
    pub fn rebuild(&self, servers: &[Arc<ServerConfig>], base_path: &str) {
        let mut new_paths: Vec<(PathBuf, String)> = servers
            .iter()
            .filter(|server| server.enabled)
            .map(|server| {
                let server_path = PathBuf::from(base_path).join(server.name.as_ref());
                (server_path, server.name.to_string())
            })
            .collect();

        // Sort by path length (descending) - longest paths first
        // This ensures most specific paths match first
        new_paths.sort_by(|a, b| {
            b.0.as_os_str().len().cmp(&a.0.as_os_str().len())
        });

        *self.paths.write() = new_paths;

        tracing::debug!("Server path cache rebuilt with {} entries", self.paths.read().len());
    }

    /// Find which server a file path belongs to
    /// O(k) where k = number of servers, but optimized with sorted paths
    /// Returns None if path doesn't belong to any tracked server
    pub fn find_server(&self, path: &Path) -> Option<String> {
        let paths = self.paths.read();

        // First matching path is most specific (due to sort order)
        paths.iter()
            .find(|(server_path, _)| path.starts_with(server_path))
            .map(|(_, server_name)| server_name.clone())
    }

    /// Update a single server path (incremental update)
    pub fn update_server(&self, server_name: String, server_path: PathBuf) {
        let mut paths = self.paths.write();

        // Remove existing entry for this server if exists
        paths.retain(|(_, name)| name != &server_name);

        // Add new entry
        paths.push((server_path, server_name));

        // Re-sort by length
        paths.sort_by(|a, b| {
            b.0.as_os_str().len().cmp(&a.0.as_os_str().len())
        });
    }

    /// Remove a server path
    pub fn remove_server(&self, server_path: &Path) {
        let mut paths = self.paths.write();
        paths.retain(|(path, _)| path != server_path);
    }

    /// Get number of tracked servers
    pub fn len(&self) -> usize {
        self.paths.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.paths.read().is_empty()
    }
}

impl Default for ServerPathCache {
    fn default() -> Self {
        Self::new()
    }
}
