use lighty_config::ServerConfig;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cache for fast server path lookups (O(1) instead of O(n))
/// Used by file watcher to quickly determine which server a file belongs to
pub struct ServerPathCache {
    /// Map: server directory PathBuf → server name String
    paths: Arc<RwLock<HashMap<PathBuf, String>>>,
}

impl ServerPathCache {
    pub fn new() -> Self {
        Self {
            paths: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Rebuild entire cache from server configs
    /// Called on initialization and config reload
    pub fn rebuild(&self, servers: &[Arc<ServerConfig>], base_path: &str) {
        let mut paths = self.paths.write();
        paths.clear();

        for server in servers {
            if server.enabled {
                let server_path = PathBuf::from(base_path).join(server.name.as_ref());
                paths.insert(server_path, server.name.to_string());
            }
        }

        tracing::debug!("Server path cache rebuilt with {} entries", paths.len());
    }

    /// Find which server a file path belongs to (O(1) amortized)
    /// Returns None if path doesn't belong to any tracked server
    pub fn find_server(&self, path: &Path) -> Option<String> {
        let paths = self.paths.read();

        // Check if path starts with any server directory
        for (server_path, server_name) in paths.iter() {
            if path.starts_with(server_path) {
                return Some(server_name.clone());
            }
        }

        None
    }

    /// Update a single server path (incremental update)
    pub fn update_server(&self, server_name: String, server_path: PathBuf) {
        let mut paths = self.paths.write();
        paths.insert(server_path, server_name);
    }

    /// Remove a server path
    pub fn remove_server(&self, server_path: &Path) {
        let mut paths = self.paths.write();
        paths.remove(server_path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_server() {
        let cache = ServerPathCache::new();

        let mut paths = cache.paths.write();
        paths.insert(PathBuf::from("/servers/survival"), "survival".to_string());
        paths.insert(PathBuf::from("/servers/creative"), "creative".to_string());
        drop(paths);

        // Test exact match
        assert_eq!(
            cache.find_server(&PathBuf::from("/servers/survival")),
            Some("survival".to_string())
        );

        // Test nested path
        assert_eq!(
            cache.find_server(&PathBuf::from("/servers/survival/mods/test.jar")),
            Some("survival".to_string())
        );

        // Test no match
        assert_eq!(
            cache.find_server(&PathBuf::from("/other/path")),
            None
        );
    }
}
