// Re-export all public APIs from the workspace crates

pub use lighty_models::*;
pub use lighty_events::*;
pub use lighty_utils::*;
pub use lighty_filesystem::*;
pub use lighty_config::*;
pub use lighty_scanner::*;
pub use lighty_cache::*;
pub use lighty_watcher::*;
pub use lighty_api::*;

/// Prelude module for convenient imports
pub mod prelude {
    // Core models
    pub use lighty_models::{VersionBuilder, Library, Mod, Native, Client, Asset};

    // Events
    pub use lighty_events::{AppEvent, EventBus};

    // Cache management
    pub use lighty_cache::{CacheManager, FileCache};

    // Configuration
    pub use lighty_config::Config;

    // Scanner
    pub use lighty_scanner::ServerScanner;

    // Watcher
    pub use lighty_watcher::ConfigWatcher;

    // Filesystem
    pub use lighty_filesystem::FileSystem;
}
