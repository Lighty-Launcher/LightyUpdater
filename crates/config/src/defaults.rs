/// Default values for configuration fields

pub fn tcp_nodelay() -> bool {
    true
}

pub fn timeout_secs() -> u64 {
    60
}

pub fn max_body_size() -> usize {
    100
}

pub fn max_concurrent_requests() -> usize {
    1000
}

pub fn config_watch_debounce_ms() -> u64 {
    500
}

pub fn max_memory_cache_gb() -> u64 {
    0  // 0 = unlimited
}

pub fn batch_size_default() -> usize {
    100
}

pub fn batch_config() -> super::models::BatchConfig {
    super::models::BatchConfig {
        client: batch_size_default(),
        libraries: batch_size_default(),
        mods: batch_size_default(),
        natives: batch_size_default(),
        assets: batch_size_default(),
    }
}

pub fn allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

pub fn server_enabled() -> bool {
    true
}

pub fn streaming_threshold_mb() -> u64 {
    100  // Files larger than 100MB will be streamed instead of loaded into memory
}

pub fn file_watcher_debounce_ms() -> u64 {
    500  // Wait 500ms after last file change before rescanning
}

pub fn checksum_buffer_size() -> usize {
    8192  // 8KB buffer for SHA1 calculation
}

pub fn hash_concurrency() -> usize {
    100  // Max concurrent hash computations (controls I/O load)
}

pub fn config_reload_channel_size() -> usize {
    100  // Config reload event channel buffer size
}

pub fn enable_compression() -> bool {
    true  // Enable HTTP compression (gzip/brotli) by default
}

// Storage defaults
pub fn storage_backend() -> super::models::StorageBackend {
    super::models::StorageBackend::Local
}

pub fn keep_local_backup() -> bool {
    true
}

pub fn auto_upload() -> bool {
    true
}

pub fn storage_settings() -> super::models::StorageSettings {
    super::models::StorageSettings {
        backend: storage_backend(),
        keep_local_backup: keep_local_backup(),
        auto_upload: auto_upload(),
        s3: s3_settings(),
    }
}

pub fn s3_region() -> String {
    "auto".to_string()
}

pub fn s3_region_arc() -> std::sync::Arc<str> {
    std::sync::Arc::from("auto")
}

pub fn s3_bucket_name() -> String {
    "lighty-updater".to_string()
}

pub fn s3_bucket_name_arc() -> std::sync::Arc<str> {
    std::sync::Arc::from("lighty-updater")
}

pub fn s3_settings() -> super::models::S3Settings {
    super::models::S3Settings {
        enabled: false,
        endpoint_url: std::sync::Arc::from(""),
        region: s3_region_arc(),
        access_key_id: String::new(),
        secret_access_key: String::new(),
        bucket_name: s3_bucket_name_arc(),
        public_url: std::sync::Arc::from(""),
        bucket_prefix: std::sync::Arc::from(""),
    }
}

// Cloudflare defaults
pub fn purge_on_update() -> bool {
    true
}

pub fn cloudflare_settings() -> super::models::CloudflareSettings {
    super::models::CloudflareSettings {
        enabled: false,
        zone_id: String::new(),
        api_token: String::new(),
        purge_on_update: purge_on_update(),
    }
}

pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"# ===============================================================================
# LightyUpdater Configuration
# ===============================================================================

[server]
# Network
host = "0.0.0.0"                     # Server bind address (0.0.0.0 = all interfaces)
port = 8080                          # Server port
base_url = "http://localhost:8080"   # Public base URL for file downloads
base_path = "updater"                # Base directory for server files (relative to executable if not absolute)

# Performance
tcp_nodelay = true                   # Disable Nagle's algorithm (lower latency)
timeout_secs = 60                    # Request timeout in seconds
max_concurrent_requests = 1000       # Max simultaneous connections
max_body_size_mb = 100               # Max request body size in MB
streaming_threshold_mb = 100         # Files >100MB streamed, <100MB cached in RAM
enable_compression = true            # HTTP compression (gzip/brotli/deflate)

# CORS
allowed_origins = ["*"]              # "*" = all origins | ["https://example.com"] for production

[cache]
# Core settings
enabled = true                       # Enable in-memory file caching
auto_scan = true                     # Scan servers on startup
rescan_interval = 30                 # Rescan interval in seconds (0 = file watcher only)
max_memory_cache_gb = 0              # Max RAM for cache in GB (0 = unlimited)

# Hot-reload
config_watch_debounce_ms = 500       # Config file change debounce (milliseconds)
file_watcher_debounce_ms = 500       # Server files change debounce (milliseconds)

# Performance
checksum_buffer_size = 8192          # SHA1 calculation buffer (bytes)
hash_concurrency = 100               # Max concurrent hash computations
config_reload_channel_size = 100     # Config reload event channel buffer size

# Batch processing
[cache.batch]
client = 100                         # Client JAR scan batch size
libraries = 100                      # Libraries scan batch size
mods = 100                           # Mods scan batch size
natives = 100                        # Natives scan batch size
assets = 100                         # Assets scan batch size

# ===============================================================================
# STORAGE BACKEND
# ===============================================================================
[storage]
backend = "local"                    # Storage backend: "local" or "s3"
keep_local_backup = true             # Keep local files when using S3
auto_upload = true                   # Auto-upload to S3 on file changes

# S3 Configuration (only used if backend = "s3")
[storage.s3]
enabled = false                      # Enable S3 storage backend
endpoint_url = ""                    # S3 endpoint (e.g., https://s3.amazonaws.com)
region = "auto"                      # S3 region (e.g., us-east-1 or "auto")
access_key_id = ""                   # AWS Access Key ID
secret_access_key = ""               # AWS Secret Access Key
bucket_name = "lighty-updater"       # S3 bucket name
public_url = ""                      # Public URL for file downloads (optional)
bucket_prefix = ""                   # Prefix for all S3 keys (optional)

# ===============================================================================
# CLOUDFLARE CACHE PURGE
# ===============================================================================
[cloudflare]
enabled = false                      # Enable Cloudflare cache purging
zone_id = ""                         # Cloudflare Zone ID
api_token = ""                       # Cloudflare API Token (requires Cache Purge permission)
purge_on_update = true               # Auto-purge cache on file updates

# ===============================================================================
# SERVER DEFINITIONS
# ===============================================================================
# Expected folder structure: {base_path}/{name}/client/*.jar, libraries/*.jar,
# mods/*.jar, natives/*.dll|.so|.dylib, assets/*

#[[servers]]
#name = "example"                    # Server ID (used in URLs and folder name)
#enabled = true                      # Enable this server
#loader = "vanilla"                  # Loader type: vanilla | forge | fabric | quilt
#loader_version = ""                 # Loader version (empty for vanilla)
#minecraft_version = "1.21"          # Minecraft version
#main_class = "net.minecraft.client.main.Main"  # Main class to launch
#java_version = 21                   # Required Java version
#enable_client = true                # Include client JAR
#enable_libraries = true             # Include libraries
#enable_mods = true                  # Include mods
#enable_natives = true               # Include native libraries
#enable_assets = true                # Include assets
#game_args = []                      # Additional game arguments
#jvm_args = []                       # Additional JVM arguments
"#;
