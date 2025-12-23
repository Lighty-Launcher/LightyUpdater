use std::path::PathBuf;

/// Main server scanner struct
pub struct ServerScanner;

/// Common scanner for JAR files with parallel processing
pub struct JarScanner {
    pub base_dir: PathBuf,
    pub server: String,
    pub base_url: String,
    pub batch_size: usize,
}

/// Information about a scanned JAR file
pub struct JarFileInfo {
    pub file_name: String,
    pub relative_path: PathBuf,
    pub url: String,
    pub url_path: String,
    pub sha1: String,
    pub size: u64,
}

/// Information about a scanned file
pub struct FileInfo {
    pub file_name: String,
    pub relative_path: PathBuf,
    pub url: String,
    pub url_path: String,
    pub sha1: String,
    pub size: u64,
}
