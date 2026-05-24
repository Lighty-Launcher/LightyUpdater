use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    // Application lifecycle
    Starting { version: String },
    Ready { addr: String, base_url: String },
    Shutdown,

    // Configuration
    ConfigLoading { path: String },
    ConfigLoaded { servers_count: usize },
    ConfigCreated { path: String },
    ConfigMigrated { added_fields: Vec<String> },
    ConfigReloaded,
    ConfigError { error: String },

    // Server initialization
    ServerFolderInit { name: String, path: String },
    ServerFolderCreated { name: String },
    AllServersInitialized,

    // Scanning
    ScanStarted { server: String },
    ScanCompleted { server: String, duration: Duration },
    InitialScanStarted,

    // Cache events
    CacheNew { server: String },
    CacheUpdated { server: String, changes: Vec<String> },
    CacheUnchanged { server: String },

    // Server discovery
    NewServerDetected { name: String },
    ServerRemoved { name: String },

    // Auto-scan
    AutoScanEnabled { interval: u64 },
    ContinuousScanEnabled,

    // CDN / edge cache
    CdnPurgeRequested { server: String, urls: Vec<String> },
    CdnPurgeCompleted { server: String, ok: bool, error: Option<String> },
    CloudflarePurgeRequested { server: String },
    CloudflarePurgeCompleted { server: String, ok: bool, error: Option<String> },

    // Errors
    Error { context: String, error: String },
}
