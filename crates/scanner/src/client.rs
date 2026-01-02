use super::errors::ScanError;
use lighty_models::Client;
use lighty_storage::StorageBackend;
use lighty_utils::compute_sha1_with_size;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

type Result<T> = std::result::Result<T, ScanError>;

pub async fn scan_client(path: &Path, server: &str, storage: &Arc<dyn StorageBackend>, buffer_size: usize) -> Result<Option<Client>> {
    let client_dir = path.join("client");

    if !client_dir.exists() {
        return Ok(None);
    }

    // Find the first .jar file in the client directory
    let mut entries = fs::read_dir(&client_dir).await?;
    let mut jar_file = None;

    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        if entry_path.is_file() && entry_path.extension().map_or(false, |ext| ext == "jar") {
            jar_file = Some(entry_path);
            break;
        }
    }

    let client_path = match jar_file {
        Some(path) => path,
        None => return Ok(None),
    };

    let file_name = client_path
        .file_name()
        .ok_or_else(|| ScanError::InvalidMetadata("Failed to get client jar filename".to_string()))?
        .to_string_lossy()
        .to_string();

    let (sha1, size) = compute_sha1_with_size(&client_path, buffer_size).await?;

    let remote_key = format!("{}/{}", server, file_name);
    let url = storage.get_url(&remote_key);

    Ok(Some(Client {
        name: "client".to_string(),
        url,
        path: file_name,
        sha1,
        size,
    }))
}
