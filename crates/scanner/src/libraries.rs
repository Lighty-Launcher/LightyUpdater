use super::utils::JarScanner;
use lighty_models::Library;
use lighty_utils::path_to_maven_name;
use anyhow::Result;
use std::path::Path;

pub async fn scan_libraries(path: &Path, server: &str, base_url: &str, batch_size: usize) -> Result<Vec<Library>> {
    let libraries_dir = path.join("libraries");

    let scanner = JarScanner::new(
        libraries_dir,
        server.to_string(),
        base_url.to_string(),
        batch_size,
    );

    scanner
        .scan(|info| {
            let maven_name = path_to_maven_name(&info.relative_path);

            Ok(Library {
                name: maven_name,
                url: Some(info.url),
                path: Some(info.url_path),
                sha1: Some(info.sha1),
                size: Some(info.size),
            })
        })
        .await
}
