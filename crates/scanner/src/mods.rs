use super::utils::JarScanner;
use lighty_models::Mod;
use anyhow::Result;
use std::path::Path;

pub async fn scan_mods(path: &Path, server: &str, base_url: &str, batch_size: usize) -> Result<Vec<Mod>> {
    let mods_dir = path.join("mods");

    let scanner = JarScanner::new(
        mods_dir,
        server.to_string(),
        base_url.to_string(),
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
        })
        .await
}
