use super::utils::JarScanner;
use super::errors::ScanError;
use lighty_models::Library;
use lighty_storage::StorageBackend;
use lighty_utils::path_to_maven_name;
use std::path::Path;
use std::sync::Arc;

type Result<T> = std::result::Result<T, ScanError>;

pub async fn scan_libraries(path: &Path, server: &str, storage: &Arc<dyn StorageBackend>, batch_size: usize, buffer_size: usize) -> Result<Vec<Library>> {
    let libraries_dir = path.join("libraries");

    let scanner = JarScanner::new(
        libraries_dir,
        server.to_string(),
        Arc::clone(storage),
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
        }, buffer_size)
        .await
}
