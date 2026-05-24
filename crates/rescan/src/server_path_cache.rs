use lighty_config::ServerConfig;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cache for fast server path lookups
/// Paths are sorted by length (longest first) for quick prefix matching
/// Used by file watcher to quickly determine which server a file belongs to
pub struct ServerPathCache {
    paths: Arc<RwLock<Vec<(PathBuf, String)>>>,
}

impl ServerPathCache {
    pub fn new() -> Self {
        Self {
            paths: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn rebuild(&self, servers: &[Arc<ServerConfig>], base_path: &str) {
        let mut new_paths: Vec<(PathBuf, String)> = servers
            .iter()
            .filter(|server| server.enabled)
            .map(|server| {
                let server_path = PathBuf::from(base_path).join(server.name.as_ref());
                (server_path, server.name.to_string())
            })
            .collect();

        new_paths.sort_by(|a, b| b.0.as_os_str().len().cmp(&a.0.as_os_str().len()));

        *self.paths.write() = new_paths;

        tracing::debug!("Server path cache rebuilt with {} entries", self.paths.read().len());
    }

    pub fn find_server(&self, path: &Path) -> Option<String> {
        let paths = self.paths.read();

        paths
            .iter()
            .find(|(server_path, _)| path.starts_with(server_path))
            .map(|(_, server_name)| server_name.clone())
    }

    pub fn update_server(&self, server_name: String, server_path: PathBuf) {
        let mut paths = self.paths.write();
        paths.retain(|(_, name)| name != &server_name);
        paths.push((server_path, server_name));
        paths.sort_by(|a, b| b.0.as_os_str().len().cmp(&a.0.as_os_str().len()));
    }

    pub fn remove_server(&self, server_name: &str) {
        let mut paths = self.paths.write();
        paths.retain(|(_, name)| name != server_name);
    }

    pub fn len(&self) -> usize {
        self.paths.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.read().is_empty()
    }
}

impl Default for ServerPathCache {
    fn default() -> Self {
        Self::new()
    }
}
