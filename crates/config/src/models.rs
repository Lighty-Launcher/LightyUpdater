use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerSettings,
    pub cache: CacheSettings,
    #[serde(default = "super::defaults::hot_reload_settings")]
    pub hot_reload: HotReloadSettings,
    #[serde(default = "super::defaults::storage_settings")]
    pub storage: StorageSettings,
    #[serde(default = "super::defaults::cloudflare_settings")]
    pub cloudflare: CloudflareSettings,
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

// Custom deserializer for Arc<str>
fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(Arc::from(s.as_str()))
}

// Custom deserializer for Arc<str> with default support
fn deserialize_arc_str_default<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(Arc::from(s.unwrap_or_default().as_str()))
}

// Custom serializer for Arc<str>
fn serialize_arc_str<S>(value: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(value.as_ref())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    #[serde(deserialize_with = "deserialize_arc_str")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub base_url: Arc<str>,
    #[serde(deserialize_with = "deserialize_arc_str")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub base_path: Arc<str>,
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
    #[serde(default = "super::defaults::max_memory_cache_gb")]
    pub max_memory_cache_gb: u64,
    #[serde(default = "super::defaults::batch_config")]
    pub batch: BatchConfig,
    #[serde(default = "super::defaults::checksum_buffer_size")]
    pub checksum_buffer_size: usize,
    #[serde(default = "super::defaults::hash_concurrency")]
    pub hash_concurrency: usize,
    #[serde(default = "super::defaults::config_reload_channel_size")]
    pub config_reload_channel_size: usize,
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
    #[serde(deserialize_with = "deserialize_arc_str")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub name: Arc<str>,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageSettings {
    #[serde(default = "super::defaults::storage_backend")]
    pub backend: StorageBackend,
    #[serde(default = "super::defaults::keep_local_backup")]
    pub keep_local_backup: bool,
    #[serde(default = "super::defaults::auto_upload")]
    pub auto_upload: bool,
    #[serde(default = "super::defaults::s3_settings")]
    pub s3: S3Settings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Local,
    S3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Settings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_arc_str_default")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub endpoint_url: Arc<str>,
    #[serde(default = "super::defaults::s3_region_arc")]
    #[serde(deserialize_with = "deserialize_arc_str_default")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub region: Arc<str>,
    #[serde(default)]
    pub access_key_id: String,
    #[serde(default)]
    pub secret_access_key: String,
    #[serde(default = "super::defaults::s3_bucket_name_arc")]
    #[serde(deserialize_with = "deserialize_arc_str_default")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub bucket_name: Arc<str>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_arc_str_default")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub public_url: Arc<str>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_arc_str_default")]
    #[serde(serialize_with = "serialize_arc_str")]
    pub bucket_prefix: Arc<str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudflareSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub zone_id: String,
    #[serde(default)]
    pub api_token: String,
    #[serde(default = "super::defaults::purge_on_update")]
    pub purge_on_update: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotReloadSettings {
    #[serde(default = "super::defaults::hot_reload_config_settings")]
    pub config: HotReloadConfigSettings,
    #[serde(default = "super::defaults::hot_reload_files_settings")]
    pub files: HotReloadFilesSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotReloadConfigSettings {
    #[serde(default = "super::defaults::hot_reload_config_enabled")]
    pub enabled: bool,
    #[serde(default = "super::defaults::config_watch_debounce_ms")]
    pub debounce_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotReloadFilesSettings {
    #[serde(default = "super::defaults::hot_reload_files_enabled")]
    pub enabled: bool,
    #[serde(default = "super::defaults::file_watcher_debounce_ms")]
    pub debounce_ms: u64,
}
