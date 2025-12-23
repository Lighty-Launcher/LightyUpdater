use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerSettings,
    pub cache: CacheSettings,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_arc_servers")]
    #[serde(serialize_with = "serialize_arc_servers")]
    pub servers: Vec<Arc<ServerConfig>>,
}

// Custom deserializer to wrap ServerConfig in Arc
fn deserialize_arc_servers<'de, D>(deserializer: D) -> Result<Vec<Arc<ServerConfig>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let servers: Vec<ServerConfig> = Vec::deserialize(deserializer)?;
    Ok(servers.into_iter().map(Arc::new).collect())
}

// Custom serializer to unwrap Arc<ServerConfig>
fn serialize_arc_servers<S>(servers: &Vec<Arc<ServerConfig>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(servers.len()))?;
    for server in servers {
        seq.serialize_element(server.as_ref())?;
    }
    seq.end()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub base_path: String,
    #[serde(default = "super::defaults::tcp_nodelay")]
    pub tcp_nodelay: bool,
    #[serde(default = "super::defaults::timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "super::defaults::max_body_size")]
    pub max_body_size_mb: usize,
    #[serde(default = "super::defaults::allowed_origins")]
    pub allowed_origins: Vec<String>,
    #[serde(default = "super::defaults::max_concurrent_requests")]
    pub max_concurrent_requests: usize,
    #[serde(default = "super::defaults::streaming_threshold_mb")]
    pub streaming_threshold_mb: u64,
    #[serde(default = "super::defaults::enable_compression")]
    pub enable_compression: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheSettings {
    pub enabled: bool,
    pub auto_scan: bool,
    pub rescan_interval: u64,
    #[serde(default = "super::defaults::config_watch_debounce_ms")]
    pub config_watch_debounce_ms: u64,
    #[serde(default = "super::defaults::max_memory_cache_gb")]
    pub max_memory_cache_gb: u64,
    #[serde(default = "super::defaults::batch_config")]
    pub batch: BatchConfig,
    #[serde(default = "super::defaults::file_watcher_debounce_ms")]
    pub file_watcher_debounce_ms: u64,
    #[serde(default = "super::defaults::checksum_buffer_size")]
    pub checksum_buffer_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchConfig {
    #[serde(default = "super::defaults::batch_size_default")]
    pub client: usize,
    #[serde(default = "super::defaults::batch_size_default")]
    pub libraries: usize,
    #[serde(default = "super::defaults::batch_size_default")]
    pub mods: usize,
    #[serde(default = "super::defaults::batch_size_default")]
    pub natives: usize,
    #[serde(default = "super::defaults::batch_size_default")]
    pub assets: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name: String,
    #[serde(default = "super::defaults::server_enabled")]
    pub enabled: bool,
    pub loader: String,
    pub loader_version: String,
    pub minecraft_version: String,
    pub main_class: String,
    pub java_version: u8,
    #[serde(default)]
    pub enable_client: bool,
    #[serde(default)]
    pub enable_libraries: bool,
    #[serde(default)]
    pub enable_mods: bool,
    #[serde(default)]
    pub enable_natives: bool,
    #[serde(default)]
    pub enable_assets: bool,
    #[serde(default)]
    pub game_args: Vec<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
}
