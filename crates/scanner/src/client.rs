use lighty_models::Client;
use lighty_utils::compute_sha1;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

pub async fn scan_client(path: &Path, server: &str, base_url: &str) -> Result<Option<Client>> {
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
        .context("Failed to get client jar filename")?
        .to_string_lossy()
        .to_string();

    let sha1 = compute_sha1(&client_path).await?;
    let metadata = fs::metadata(&client_path).await?;
    let size = metadata.len();

    Ok(Some(Client {
        name: "client".to_string(),
        url: format!("{}/{}/{}", base_url, server, file_name),
        path: file_name,
        sha1,
        size,
    }))
}
