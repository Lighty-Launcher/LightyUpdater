use lighty_models::VersionBuilder;
use std::collections::HashMap;

/// Changements détectés entre deux versions
#[derive(Debug)]
pub struct FileDiff {
    pub added: Vec<FileChange>,
    pub modified: Vec<FileChange>,
    pub removed: Vec<FileChange>,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub file_type: FileType,
    pub remote_key: String,
    pub local_path: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum FileType {
    Client,
    Library,
    Mod,
    Native,
    Asset,
}

impl FileDiff {
    /// Détecte les changements granulaires entre deux VersionBuilder
    pub fn compute(
        server_name: &str,
        old: Option<&VersionBuilder>,
        new: &VersionBuilder,
    ) -> Self {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();

        if let Some(old) = old {
            Self::diff_client(server_name, old, new, &mut added, &mut modified, &mut removed);
            Self::diff_libraries(server_name, old, new, &mut added, &mut modified, &mut removed);
            Self::diff_mods(server_name, old, new, &mut added, &mut modified, &mut removed);
            Self::diff_natives(server_name, old, new, &mut added, &mut modified, &mut removed);
            Self::diff_assets(server_name, old, new, &mut added, &mut modified, &mut removed);
        } else {
            // First scan: all files are "added"
            Self::add_all_files(server_name, new, &mut added);
        }

        Self {
            added,
            modified,
            removed,
        }
    }

    /// Updates the URL map incrementally based on this diff (avoids full rebuild)
    pub fn apply_to_url_map(&self, builder: &mut VersionBuilder) {
        // Add new files and update modified files
        for change in self.added.iter().chain(self.modified.iter()) {
            if !change.url.is_empty() {
                let path = Self::extract_relative_path(&change.local_path);
                builder.add_url_mapping(change.url.clone(), path);
            }
        }

        // Remove deleted files
        for change in &self.removed {
            if !change.url.is_empty() {
                builder.remove_url_mapping(&change.url);
            }
        }
    }

    /// Extracts the relative path from a local path (removes server name prefix)
    fn extract_relative_path(local_path: &str) -> String {
        // local_path format: "server_name/subfolder/file.jar"
        // We want: "subfolder/file.jar"
        if let Some(idx) = local_path.find('/') {
            local_path[idx + 1..].to_string()
        } else {
            local_path.to_string()
        }
    }

    fn diff_client(
        server_name: &str,
        old: &VersionBuilder,
        new: &VersionBuilder,
        added: &mut Vec<FileChange>,
        modified: &mut Vec<FileChange>,
        removed: &mut Vec<FileChange>,
    ) {
        match (&old.client, &new.client) {
            (None, Some(client)) => {
                added.push(FileChange {
                    file_type: FileType::Client,
                    remote_key: format!("{}/client.jar", server_name),
                    local_path: format!("{}/client/client.jar", server_name),
                    url: client.url.clone(),
                });
            }
            (Some(old_client), None) => {
                removed.push(FileChange {
                    file_type: FileType::Client,
                    remote_key: format!("{}/client.jar", server_name),
                    local_path: format!("{}/client/client.jar", server_name),
                    url: old_client.url.clone(),
                });
            }
            (Some(old_client), Some(new_client)) => {
                if old_client.sha1 != new_client.sha1 {
                    modified.push(FileChange {
                        file_type: FileType::Client,
                        remote_key: format!("{}/client.jar", server_name),
                        local_path: format!("{}/client/client.jar", server_name),
                        url: new_client.url.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    fn diff_libraries(
        server_name: &str,
        old: &VersionBuilder,
        new: &VersionBuilder,
        added: &mut Vec<FileChange>,
        modified: &mut Vec<FileChange>,
        removed: &mut Vec<FileChange>,
    ) {
        // Create maps for O(1) lookup
        let old_map: HashMap<_, _> = old
            .libraries
            .iter()
            .map(|lib| (&lib.path, lib))
            .collect();
        let new_map: HashMap<_, _> = new
            .libraries
            .iter()
            .map(|lib| (&lib.path, lib))
            .collect();

        // Find added and modified
        for (path, new_lib) in &new_map {
            let path_str = path.as_ref().unwrap();
            let remote_key = format!("{}/libraries/{}", server_name, path_str);
            let local_path = format!("{}/libraries/{}", server_name, path_str);
            let url = new_lib.url.as_deref().unwrap_or_default().to_string();

            if let Some(old_lib) = old_map.get(path) {
                // Exists in both: check if modified
                if old_lib.sha1 != new_lib.sha1 {
                    modified.push(FileChange {
                        file_type: FileType::Library,
                        remote_key,
                        local_path,
                        url,
                    });
                }
            } else {
                // Only in new: added
                added.push(FileChange {
                    file_type: FileType::Library,
                    remote_key,
                    local_path,
                    url,
                });
            }
        }

        // Find removed
        for (path, old_lib) in &old_map {
            if !new_map.contains_key(path) {
                let path_str = path.as_ref().unwrap();
                let url = old_lib.url.as_deref().unwrap_or_default().to_string();
                removed.push(FileChange {
                    file_type: FileType::Library,
                    remote_key: format!("{}/libraries/{}", server_name, path_str),
                    local_path: format!("{}/libraries/{}", server_name, path_str),
                    url,
                });
            }
        }
    }

    fn diff_mods(
        server_name: &str,
        old: &VersionBuilder,
        new: &VersionBuilder,
        added: &mut Vec<FileChange>,
        modified: &mut Vec<FileChange>,
        removed: &mut Vec<FileChange>,
    ) {
        let old_map: HashMap<_, _> = old.mods.iter().map(|m| (&m.name, m)).collect();
        let new_map: HashMap<_, _> = new.mods.iter().map(|m| (&m.name, m)).collect();

        for (name, new_mod) in &new_map {
            let remote_key = format!("{}/mods/{}", server_name, name);
            let local_path = format!("{}/mods/{}", server_name, name);
            let url = new_mod.url.as_deref().unwrap_or_default().to_string();

            if let Some(old_mod) = old_map.get(name) {
                if old_mod.sha1 != new_mod.sha1 {
                    modified.push(FileChange {
                        file_type: FileType::Mod,
                        remote_key,
                        local_path,
                        url,
                    });
                }
            } else {
                added.push(FileChange {
                    file_type: FileType::Mod,
                    remote_key,
                    local_path,
                    url,
                });
            }
        }

        for (name, old_mod) in &old_map {
            if !new_map.contains_key(name) {
                let url = old_mod.url.as_deref().unwrap_or_default().to_string();
                removed.push(FileChange {
                    file_type: FileType::Mod,
                    remote_key: format!("{}/mods/{}", server_name, name),
                    local_path: format!("{}/mods/{}", server_name, name),
                    url,
                });
            }
        }
    }

    fn diff_natives(
        server_name: &str,
        old: &VersionBuilder,
        new: &VersionBuilder,
        added: &mut Vec<FileChange>,
        modified: &mut Vec<FileChange>,
        removed: &mut Vec<FileChange>,
    ) {
        match (&old.natives, &new.natives) {
            (None, Some(new_natives)) => {
                for native in new_natives {
                    added.push(FileChange {
                        file_type: FileType::Native,
                        remote_key: format!("{}/natives/{}", server_name, native.name),
                        local_path: format!("{}/natives/{}", server_name, native.name),
                        url: native.url.clone(),
                    });
                }
            }
            (Some(old_natives), None) => {
                for native in old_natives {
                    removed.push(FileChange {
                        file_type: FileType::Native,
                        remote_key: format!("{}/natives/{}", server_name, native.name),
                        local_path: format!("{}/natives/{}", server_name, native.name),
                        url: native.url.clone(),
                    });
                }
            }
            (Some(old_natives), Some(new_natives)) => {
                let old_map: HashMap<_, _> = old_natives.iter().map(|n| (&n.name, n)).collect();
                let new_map: HashMap<_, _> = new_natives.iter().map(|n| (&n.name, n)).collect();

                for (name, new_native) in &new_map {
                    let remote_key = format!("{}/natives/{}", server_name, name);
                    let local_path = format!("{}/natives/{}", server_name, name);
                    let url = new_native.url.clone();

                    if let Some(old_native) = old_map.get(name) {
                        if old_native.sha1 != new_native.sha1 {
                            modified.push(FileChange {
                                file_type: FileType::Native,
                                remote_key,
                                local_path,
                                url,
                            });
                        }
                    } else {
                        added.push(FileChange {
                            file_type: FileType::Native,
                            remote_key,
                            local_path,
                            url,
                        });
                    }
                }

                for (name, old_native) in &old_map {
                    if !new_map.contains_key(name) {
                        removed.push(FileChange {
                            file_type: FileType::Native,
                            remote_key: format!("{}/natives/{}", server_name, name),
                            local_path: format!("{}/natives/{}", server_name, name),
                            url: old_native.url.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn diff_assets(
        server_name: &str,
        old: &VersionBuilder,
        new: &VersionBuilder,
        added: &mut Vec<FileChange>,
        modified: &mut Vec<FileChange>,
        removed: &mut Vec<FileChange>,
    ) {
        let old_map: HashMap<_, _> = old.assets.iter().map(|a| (&a.path, a)).collect();
        let new_map: HashMap<_, _> = new.assets.iter().map(|a| (&a.path, a)).collect();

        for (path, new_asset) in &new_map {
            let path_str = path.as_ref().unwrap();
            let remote_key = format!("{}/assets/{}", server_name, path_str);
            let local_path = format!("{}/assets/{}", server_name, path_str);
            let url = new_asset.url.as_deref().unwrap_or_default().to_string();

            if let Some(old_asset) = old_map.get(path) {
                if old_asset.hash != new_asset.hash {
                    modified.push(FileChange {
                        file_type: FileType::Asset,
                        remote_key,
                        local_path,
                        url,
                    });
                }
            } else {
                added.push(FileChange {
                    file_type: FileType::Asset,
                    remote_key,
                    local_path,
                    url,
                });
            }
        }

        for (path, old_asset) in &old_map {
            if !new_map.contains_key(path) {
                let path_str = path.as_ref().unwrap();
                let url = old_asset.url.as_deref().unwrap_or_default().to_string();
                removed.push(FileChange {
                    file_type: FileType::Asset,
                    remote_key: format!("{}/assets/{}", server_name, path_str),
                    local_path: format!("{}/assets/{}", server_name, path_str),
                    url,
                });
            }
        }
    }

    fn add_all_files(server_name: &str, new: &VersionBuilder, added: &mut Vec<FileChange>) {
        // Client
        if let Some(client) = &new.client {
            added.push(FileChange {
                file_type: FileType::Client,
                remote_key: format!("{}/client.jar", server_name),
                local_path: format!("{}/client/client.jar", server_name),
                url: client.url.clone(),
            });
        }

        // Libraries
        for lib in &new.libraries {
            if let Some(path) = &lib.path {
                let url = lib.url.as_deref().unwrap_or_default().to_string();
                added.push(FileChange {
                    file_type: FileType::Library,
                    remote_key: format!("{}/libraries/{}", server_name, path),
                    local_path: format!("{}/libraries/{}", server_name, path),
                    url,
                });
            }
        }

        // Mods
        for mod_file in &new.mods {
            let url = mod_file.url.as_deref().unwrap_or_default().to_string();
            added.push(FileChange {
                file_type: FileType::Mod,
                remote_key: format!("{}/mods/{}", server_name, mod_file.name),
                local_path: format!("{}/mods/{}", server_name, mod_file.name),
                url,
            });
        }

        // Natives
        if let Some(natives) = &new.natives {
            for native in natives {
                added.push(FileChange {
                    file_type: FileType::Native,
                    remote_key: format!("{}/natives/{}", server_name, native.name),
                    local_path: format!("{}/natives/{}", server_name, native.name),
                    url: native.url.clone(),
                });
            }
        }

        // Assets
        for asset in &new.assets {
            if let Some(path) = &asset.path {
                let url = asset.url.as_deref().unwrap_or_default().to_string();
                added.push(FileChange {
                    file_type: FileType::Asset,
                    remote_key: format!("{}/assets/{}", server_name, path),
                    local_path: format!("{}/assets/{}", server_name, path),
                    url,
                });
            }
        }
    }
}
