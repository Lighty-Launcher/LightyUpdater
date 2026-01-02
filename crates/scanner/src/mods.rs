use super::utils::JarScanner;
use super::errors::ScanError;
use lighty_models::Mod;
use lighty_storage::StorageBackend;
use std::path::Path;
use std::sync::Arc;

type Result<T> = std::result::Result<T, ScanError>;

pub async fn scan_mods(path: &Path, server: &str, storage: &Arc<dyn StorageBackend>, batch_size: usize, buffer_size: usize) -> Result<Vec<Mod>> {
    let mods_dir = path.join("mods");

    let scanner = JarScanner::new(
        mods_dir,
        server.to_string(),
        Arc::clone(storage),
        batch_size,
    );

    scanner
        .scan(|info| {
            Ok(Mod {
                name: info.file_name,
                url: Some(info.url),
                path: Some(info.url_path),
                sha1: Some(info.sha1),
                size: Some(info.size),
            })
        }, buffer_size)
        .await
}
