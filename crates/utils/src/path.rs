use std::path::Path;

pub fn path_to_maven_name(path: &Path) -> String {
    let components: Vec<_> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    if components.len() < 4 {
        return path.to_string_lossy().to_string();
    }

    let group = components[..components.len() - 3].join(".");
    let artifact = components[components.len() - 3];
    let version = components[components.len() - 2];

    format!("{}:{}:{}", group, artifact, version)
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
