use lighty_models::VersionBuilder;

/// Resolves the actual file path from URL using O(1) HashMap lookup
/// For natives, tries multiple OS variants if direct lookup fails
pub fn resolve_file_path(
    version: &VersionBuilder,
    url_file_part: &str,
    base_url: &str,
    server_name: &str,
) -> Option<String> {
    let requested_url = format!("{}/{}/{}", base_url, server_name, url_file_part);

    // O(1) lookup using pre-built HashMap
    if let Some(path) = version.url_to_path_map.get(&requested_url) {
        return Some(path.clone());
    }

    // If direct lookup failed and it's a natives request, try OS-specific paths
    if url_file_part.starts_with("natives/") {
        // Extract the filename from natives/filename.jar
        let filename = url_file_part.strip_prefix("natives/").unwrap_or(url_file_part);

        // Try each OS variant
        for os in &["windows", "linux", "macos"] {
            let os_url = format!("{}/{}/natives/{}/{}", base_url, server_name, os, filename);
            if let Some(path) = version.url_to_path_map.get(&os_url) {
                return Some(path.clone());
            }
        }
    }

    None
}
