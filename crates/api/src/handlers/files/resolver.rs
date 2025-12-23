use lighty_models::VersionBuilder;

/// Resolves the actual file path from URL using O(1) HashMap lookup
pub fn resolve_file_path(
    version: &VersionBuilder,
    url_file_part: &str,
    base_url: &str,
    server_name: &str,
) -> Option<String> {
    let requested_url = format!("{}/{}/{}", base_url, server_name, url_file_part);

    // O(1) lookup using pre-built HashMap
    version.url_to_path_map.get(&requested_url).cloned()
}
