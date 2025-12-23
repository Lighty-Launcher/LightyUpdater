use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    // Application lifecycle
    Starting,
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

    // Errors
    Error { context: String, error: String },
}

pub struct EventBus {
    #[allow(dead_code)]
    pub(super) silent_mode: bool,
}
