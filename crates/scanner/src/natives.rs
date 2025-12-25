use super::utils::scan_files_parallel;
use super::errors::ScanError;
use lighty_models::Native;
use lighty_storage::StorageBackend;
use std::path::Path;
use std::sync::Arc;

type Result<T> = std::result::Result<T, ScanError>;

const NATIVE_OS_TYPES: &[&str] = &["windows", "linux", "macos"];

pub async fn scan_natives(path: &Path, server: &str, storage: &Arc<dyn StorageBackend>, concurrency: usize, buffer_size: usize) -> Result<Vec<Native>> {
    let natives_dir = path.join("natives");

    if !natives_dir.exists() {
        return Ok(vec![]);
    }

    // Scan each OS directory separately and collect OS type
    let mut all_natives = Vec::new();

    for os in NATIVE_OS_TYPES {
        let os_dir = natives_dir.join(os);
        if !os_dir.exists() {
            continue;
        }

        let os_str = os.to_string();
        let natives = scan_files_parallel(
            os_dir,
            server.to_string(),
            Arc::clone(storage),
            |path| path.is_file(), // Accept all files
            move |info| {
                Ok(Native {
                    name: format!("natives:{}:{}", os_str, info.file_name),
                    url: info.url,
                    path: info.url_path,
                    sha1: info.sha1,
                    size: info.size,
                    os: os_str.clone(),
                })
            },
            concurrency,
            buffer_size,
        )
        .await?;

        all_natives.extend(natives);
    }

    Ok(all_natives)
}
