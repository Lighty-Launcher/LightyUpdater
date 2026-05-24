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
        for change in self.added.iter().chain(self.modified.iter()) {
            if !change.url.is_empty() {
                let path = Self::extract_relative_path(&change.local_path);
                builder.add_url_mapping(change.url.clone(), path);
            }
        }

        for change in &self.removed {
            if !change.url.is_empty() {
                builder.remove_url_mapping(&change.url);
            }
        }
    }

    fn extract_relative_path(local_path: &str) -> String {
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

        for (path, new_lib) in &new_map {
            let path_str = path.as_ref().unwrap();
            let remote_key = format!("{}/libraries/{}", server_name, path_str);
            let local_path = format!("{}/libraries/{}", server_name, path_str);
            let url = new_lib.url.as_deref().unwrap_or_default().to_string();

            if let Some(old_lib) = old_map.get(path) {
                if old_lib.sha1 != new_lib.sha1 {
                    modified.push(FileChange {
                        file_type: FileType::Library,
                        remote_key,
                        local_path,
                        url,
                    });
                }
            } else {
                added.push(FileChange {
                    file_type: FileType::Library,
                    remote_key,
                    local_path,
                    url,
                });
            }
        }

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
                let old_map: HashMap<_, _> = old_natives.iter().map(|native| (&native.name, native)).collect();
                let new_map: HashMap<_, _> = new_natives.iter().map(|native| (&native.name, native)).collect();

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
        let old_map: HashMap<_, _> = old.assets.iter().map(|asset| (&asset.path, asset)).collect();
        let new_map: HashMap<_, _> = new.assets.iter().map(|asset| (&asset.path, asset)).collect();

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
        if let Some(client) = &new.client {
            added.push(FileChange {
                file_type: FileType::Client,
                remote_key: format!("{}/client.jar", server_name),
                local_path: format!("{}/client/client.jar", server_name),
                url: client.url.clone(),
            });
        }

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

        for mod_file in &new.mods {
            let url = mod_file.url.as_deref().unwrap_or_default().to_string();
            added.push(FileChange {
                file_type: FileType::Mod,
                remote_key: format!("{}/mods/{}", server_name, mod_file.name),
                local_path: format!("{}/mods/{}", server_name, mod_file.name),
                url,
            });
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use lighty_models::{
        Arguments, Asset, Client, JavaVersion, Library, MainClass, Mod, Native, VersionBuilder,
    };

    fn empty_builder() -> VersionBuilder {
        VersionBuilder {
            main_class: MainClass { main_class: String::new() },
            java_version: JavaVersion { major_version: 21 },
            arguments: Arguments { game: vec![], jvm: vec![] },
            libraries: vec![],
            mods: vec![],
            natives: None,
            client: None,
            assets: vec![],
            url_to_path_map: Default::default(),
        }
    }

    fn lib(name: &str, sha: &str) -> Library {
        Library {
            name: name.to_string(),
            url: Some(format!("https://cdn.example/{}", name)),
            path: Some(format!("net/example/{}.jar", name)),
            sha1: Some(sha.to_string()),
            size: Some(1024),
        }
    }

    fn mod_(name: &str, sha: &str) -> Mod {
        Mod {
            name: name.to_string(),
            url: Some(format!("https://cdn.example/mods/{}", name)),
            path: Some(name.to_string()),
            sha1: Some(sha.to_string()),
            size: Some(2048),
        }
    }

    fn client(sha: &str) -> Client {
        Client {
            name: "client.jar".to_string(),
            url: "https://cdn.example/client.jar".to_string(),
            path: "client.jar".to_string(),
            sha1: sha.to_string(),
            size: 4096,
        }
    }

    fn native(name: &str, sha: &str) -> Native {
        Native {
            name: name.to_string(),
            url: format!("https://cdn.example/natives/{}", name),
            path: name.to_string(),
            sha1: sha.to_string(),
            size: 512,
            os: "linux".to_string(),
        }
    }

    fn asset(path: &str, hash: &str) -> Asset {
        Asset {
            hash: hash.to_string(),
            size: 256,
            url: Some(format!("https://cdn.example/assets/{}", path)),
            path: Some(path.to_string()),
        }
    }

    #[test]
    fn first_scan_marks_everything_as_added() {
        let mut new = empty_builder();
        new.libraries.push(lib("a", "sha-a"));
        new.mods.push(mod_("m", "sha-m"));
        new.client = Some(client("sha-c"));

        let diff = FileDiff::compute("srv", None, &new);

        assert_eq!(diff.added.len(), 3);
        assert!(diff.modified.is_empty());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn identical_builders_produce_empty_diff() {
        let mut old_builder = empty_builder();
        old_builder.libraries.push(lib("a", "sha-a"));
        let new_builder = VersionBuilder { ..old_builder.clone() };

        let diff = FileDiff::compute("srv", Some(&old_builder), &new_builder);

        assert!(diff.added.is_empty());
        assert!(diff.modified.is_empty());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn detects_added_library() {
        let old = empty_builder();
        let mut new = empty_builder();
        new.libraries.push(lib("a", "sha-a"));

        let diff = FileDiff::compute("srv", Some(&old), &new);

        assert_eq!(diff.added.len(), 1);
        assert!(matches!(diff.added[0].file_type, FileType::Library));
    }

    #[test]
    fn detects_modified_library_via_sha1_change() {
        let mut old = empty_builder();
        old.libraries.push(lib("a", "old-sha"));
        let mut new = empty_builder();
        new.libraries.push(lib("a", "new-sha"));

        let diff = FileDiff::compute("srv", Some(&old), &new);

        assert!(diff.added.is_empty());
        assert_eq!(diff.modified.len(), 1);
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn detects_removed_library() {
        let mut old = empty_builder();
        old.libraries.push(lib("a", "sha-a"));
        let new = empty_builder();

        let diff = FileDiff::compute("srv", Some(&old), &new);

        assert!(diff.added.is_empty());
        assert!(diff.modified.is_empty());
        assert_eq!(diff.removed.len(), 1);
    }

    #[test]
    fn detects_added_modified_removed_mods_together() {
        let mut old = empty_builder();
        old.mods.push(mod_("keep", "same"));
        old.mods.push(mod_("change", "old"));
        old.mods.push(mod_("gone", "x"));
        let mut new = empty_builder();
        new.mods.push(mod_("keep", "same"));
        new.mods.push(mod_("change", "new"));
        new.mods.push(mod_("fresh", "z"));

        let diff = FileDiff::compute("srv", Some(&old), &new);

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.modified.len(), 1);
        assert_eq!(diff.removed.len(), 1);
    }

    #[test]
    fn client_diff_handles_all_three_transitions() {
        let mut with_old = empty_builder();
        with_old.client = Some(client("old"));
        let mut with_new = empty_builder();
        with_new.client = Some(client("new"));
        let empty = empty_builder();

        let modified = FileDiff::compute("srv", Some(&with_old), &with_new);
        let added = FileDiff::compute("srv", Some(&empty), &with_new);
        let removed = FileDiff::compute("srv", Some(&with_old), &empty);

        assert_eq!(modified.modified.len(), 1);
        assert_eq!(added.added.len(), 1);
        assert_eq!(removed.removed.len(), 1);
    }

    #[test]
    fn natives_diff_detects_per_entry_changes() {
        let mut old = empty_builder();
        old.natives = Some(vec![native("a", "old"), native("b", "x")]);
        let mut new = empty_builder();
        new.natives = Some(vec![native("a", "new"), native("c", "y")]);

        let diff = FileDiff::compute("srv", Some(&old), &new);

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.modified.len(), 1);
        assert_eq!(diff.removed.len(), 1);
    }

    #[test]
    fn assets_diff_uses_hash_not_sha1() {
        let mut old = empty_builder();
        old.assets.push(asset("icons/a.png", "h1"));
        let mut new = empty_builder();
        new.assets.push(asset("icons/a.png", "h2"));

        let diff = FileDiff::compute("srv", Some(&old), &new);

        assert_eq!(diff.modified.len(), 1);
        assert!(matches!(diff.modified[0].file_type, FileType::Asset));
    }

    #[test]
    fn apply_to_url_map_adds_and_removes_entries() {
        let mut builder = empty_builder();
        builder.add_url_mapping("https://cdn.example/old".into(), "libraries/old".into());

        let diff = FileDiff {
            added: vec![FileChange {
                file_type: FileType::Library,
                remote_key: "srv/libraries/new".into(),
                local_path: "srv/libraries/new".into(),
                url: "https://cdn.example/new".into(),
            }],
            modified: vec![],
            removed: vec![FileChange {
                file_type: FileType::Library,
                remote_key: "srv/libraries/old".into(),
                local_path: "srv/libraries/old".into(),
                url: "https://cdn.example/old".into(),
            }],
        };

        diff.apply_to_url_map(&mut builder);

        assert!(builder.url_to_path_map.contains_key("https://cdn.example/new"));
        assert!(!builder.url_to_path_map.contains_key("https://cdn.example/old"));
    }

    #[test]
    fn apply_to_url_map_ignores_empty_urls() {
        let mut builder = empty_builder();
        let diff = FileDiff {
            added: vec![FileChange {
                file_type: FileType::Library,
                remote_key: "srv/libraries/a".into(),
                local_path: "srv/libraries/a".into(),
                url: String::new(),
            }],
            modified: vec![],
            removed: vec![],
        };

        diff.apply_to_url_map(&mut builder);

        assert!(builder.url_to_path_map.is_empty());
    }
}
