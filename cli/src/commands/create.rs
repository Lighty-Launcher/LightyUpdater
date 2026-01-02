use crate::errors::{CliError, CliResult};
use crate::registry::{Instance, Registry};
use chrono::Utc;
use colored::Colorize;
use std::path::PathBuf;

const CONFIG_TEMPLATE: &str = r#"# ===============================================================================
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
# HOT-RELOAD CONFIGURATION
# ===============================================================================
[hot_reload.config]
enabled = true                       # Enable automatic config.toml reload on changes
debounce_ms = 300                    # Delay after config.toml changes before reload (milliseconds)

[hot_reload.files]
enabled = true                       # Enable automatic server files rescan on changes
debounce_ms = 300                    # Delay after server files changes (client/mods/libs) before rescan (milliseconds)

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
# CDN CACHE PURGE (for storage files)
# ===============================================================================
[cdn]
enabled = false                      # Enable CDN cache purging for storage files
provider = "cloudflare"              # CDN provider: "cloudflare" or "cloudfront"
zone_id = ""                         # Cloudflare Zone ID (cloudflare only)
api_token = ""                       # Cloudflare API Token (requires Cache Purge permission)

# ===============================================================================
# CLOUDFLARE API CACHE PURGE (for API JSON responses)
# ===============================================================================
[cloudflare]
enabled = false                      # Enable Cloudflare cache purging for API JSON
zone_id = ""                         # Cloudflare Zone ID
api_token = ""                       # Cloudflare API Token (requires Cache Purge permission)
base_url = ""                        # API base URL (e.g., https://api.example.com)

# ===============================================================================
# SERVER DEFINITIONS
# ===============================================================================
# Expected folder structure: {base_path}/{name}/client/*.jar, libraries/*.jar,
# mods/*.jar, natives/*.dll|.so|.dylib, assets/*
# You can duplicate this [[servers]] section to add multiple servers

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

pub fn execute(name: String, dir: Option<String>) -> CliResult<()> {
    // Resolve target directory
    let target_dir = resolve_directory(dir)?;

    // Verify directory exists
    if !target_dir.exists() {
        return Err(CliError::DirectoryNotFound(
            target_dir.display().to_string(),
        ));
    }

    // Load registry
    let mut registry = Registry::load()?;

    // Check if instance name already exists
    if registry.instances.contains_key(&name) {
        return Err(CliError::InstanceAlreadyExists(name));
    }

    // Create config.toml
    let config_path = target_dir.join("config.toml");
    if config_path.exists() {
        println!(
            "{}",
            format!(
                "Warning: config.toml already exists in {}",
                target_dir.display()
            )
            .yellow()
        );
        println!("{}", "Skipping config.toml creation".yellow());
    } else {
        std::fs::write(&config_path, CONFIG_TEMPLATE)?;
    }

    // Create updater/ directory
    let updater_dir = target_dir.join("updater");
    std::fs::create_dir_all(&updater_dir)?;

    // Parse config to extract port
    let port = extract_port(&config_path)?;

    // Check for port conflicts
    for (existing_name, existing_instance) in &registry.instances {
        if existing_instance.port == port {
            println!(
                "{}",
                format!(
                    "Warning: Port {} is already in use by instance '{}'",
                    port, existing_name
                )
                .yellow()
            );
            println!(
                "{}",
                "Consider changing the port in config.toml".yellow()
            );
            break;
        }
    }

    // Create instance entry
    let instance = Instance {
        name: name.clone(),
        directory: target_dir.clone(),
        config_path: config_path.clone(),
        port,
        created_at: Utc::now(),
    };

    // Add to registry
    registry.add_instance(instance)?;

    // Success message
    println!();
    println!(
        "{}",
        format!("Instance '{}' created successfully", name)
            .green()
            .bold()
    );
    println!();
    println!("  {}", "Files created:".bold());
    println!(
        "    {} {}",
        "*".green(),
        config_path.display().to_string().cyan()
    );
    println!(
        "    {} {}",
        "*".green(),
        updater_dir.display().to_string().cyan()
    );
    println!();
    println!("  {}", "Working directory:".bold());
    println!("    {}", target_dir.display().to_string().cyan());
    println!();
    println!("  {}", "Next steps:".bold());
    println!("    1. Edit the config: {}", "nano config.toml".cyan());
    println!("    2. Start the instance: {}", "lighty start".cyan());
    println!();

    Ok(())
}

/// Resolve the target directory for the instance
fn resolve_directory(dir: Option<String>) -> CliResult<PathBuf> {
    match dir {
        Some(path) => {
            let path_buf = PathBuf::from(path);
            if path_buf.is_absolute() {
                Ok(path_buf)
            } else {
                // Relative path: resolve from current directory
                let current_dir = std::env::current_dir()?;
                Ok(current_dir.join(path_buf))
            }
        }
        None => {
            // No directory specified: use current directory
            std::env::current_dir().map_err(|e| e.into())
        }
    }
}

/// Extract port from config.toml
fn extract_port(config_path: &PathBuf) -> CliResult<u16> {
    let content = std::fs::read_to_string(config_path)?;
    let config: toml::Value = toml::from_str(&content)?;

    config
        .get("server")
        .and_then(|s| s.get("port"))
        .and_then(|p| p.as_integer())
        .map(|p| p as u16)
        .ok_or_else(|| {
            CliError::Config("Could not extract port from config.toml".to_string())
        })
}
