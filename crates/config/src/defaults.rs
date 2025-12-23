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

pub fn enable_compression() -> bool {
    true  // Enable HTTP compression (gzip/brotli) by default
}

pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"# ===============================================================================
# LightyUpdater Configuration
# ===============================================================================

[server]
# Network
host = "0.0.0.0"
port = 8080
base_url = "http://localhost:8080"
base_path = "updater"

# Performance
tcp_nodelay = true                   # Disable Nagle's algorithm (lower latency)
timeout_secs = 60                    # Request timeout in seconds
max_concurrent_requests = 1000       # Max simultaneous connections
max_body_size_mb = 100               # Max request body size
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
config_watch_debounce_ms = 500       # Config file change debounce
file_watcher_debounce_ms = 500       # Server files change debounce

# Performance
checksum_buffer_size = 8192          # SHA1 calculation buffer (bytes)

# Batch processing
[cache.batch]
client = 100
libraries = 100
mods = 100
natives = 100
assets = 100

# ===============================================================================
# SERVER DEFINITIONS
# ===============================================================================
# Expected folder structure: {base_path}/{name}/client/*.jar, libraries/*.jar,
# mods/*.jar, natives/*.dll|.so|.dylib, assets/*

#[[servers]]
#name = "example"                    # Server ID (used in URLs and folder name)
#enabled = true
#loader = "vanilla"                  # vanilla | forge | fabric | quilt
#loader_version = ""                 # Empty for vanilla
#minecraft_version = "1.21"
#main_class = "net.minecraft.client.main.Main"
#java_version = 21
#enable_client = true
#enable_libraries = true
#enable_mods = true
#enable_natives = true
#enable_assets = true
#game_args = []
#jvm_args = []
"#;
