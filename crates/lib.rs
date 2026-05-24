// Re-export all public APIs from the workspace crates

pub use lighty_models::*;
pub use lighty_events::*;
pub use lighty_utils::{
    compute_sha1,
    compute_sha1_sync,
    compute_sha1_with_size,
    compute_sha1_with_size_sync,
    normalize_path,
    path_to_maven_name,
    UtilsError,
};
pub use lighty_utils::checksum as utils_checksum;
pub use lighty_utils::errors as utils_errors;
pub use lighty_utils::path as utils_path;
pub use lighty_file_system::*;
pub use lighty_config::*;
pub use lighty_scanner::*;
pub use lighty_cache::*;
pub use lighty_watcher::*;
pub use lighty_api::{
    get_server_metadata,
    list_servers,
    serve_file,
    ApiError,
    AppState,
    ErrorDetail,
    ErrorResponse,
    ServerInfo,
    ServerListResponse,
};
pub use lighty_api::errors as api_errors;
pub use lighty_api::handlers as api_handlers;
pub use lighty_api::models as api_models;

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
    pub use lighty_file_system::FileSystem;
}
