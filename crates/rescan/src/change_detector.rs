use crate::models::ChangeDetector;
use lighty_models::VersionBuilder;

impl ChangeDetector {
    /// Détecte si des changements existent et retourne les détails
    pub fn detect_changes(
        old: &VersionBuilder,
        new: &VersionBuilder,
    ) -> (bool, Vec<String>) {
        let mut changes = Vec::new();

        Self::check_client_changes(old, new, &mut changes);
        Self::check_libraries_changes(old, new, &mut changes);
        Self::check_mods_changes(old, new, &mut changes);
        Self::check_natives_changes(old, new, &mut changes);
        Self::check_assets_changes(old, new, &mut changes);

        (!changes.is_empty(), changes)
    }

    fn check_client_changes(
        old: &VersionBuilder,
        new: &VersionBuilder,
        changes: &mut Vec<String>,
    ) {
        if !Self::client_has_changed(old, new) {
            return;
        }

        let change_msg = match (&old.client, &new.client) {
            (None, Some(_)) => "client added",
            (Some(_), None) => "client removed",
            _ => "client updated",
        };
        changes.push(change_msg.to_string());
    }

    fn check_libraries_changes(
        old: &VersionBuilder,
        new: &VersionBuilder,
        changes: &mut Vec<String>,
    ) {
        if !Self::libraries_have_changed(old, new) {
            return;
        }

        let old_count = old.libraries.len();
        let new_count = new.libraries.len();

        if old_count != new_count {
            let change_msg = if new_count > old_count {
                format!("libraries added ({} -> {})", old_count, new_count)
            } else {
                format!("libraries removed ({} -> {})", old_count, new_count)
            };
            changes.push(change_msg);
        } else {
            changes.push("libraries updated".to_string());
        }
    }

    fn check_mods_changes(
        old: &VersionBuilder,
        new: &VersionBuilder,
        changes: &mut Vec<String>,
    ) {
        if !Self::mods_have_changed(old, new) {
            return;
        }

        let old_count = old.mods.len();
        let new_count = new.mods.len();

        if old_count != new_count {
            let change_msg = if new_count > old_count {
                format!("mods added ({} -> {})", old_count, new_count)
            } else {
                format!("mods removed ({} -> {})", old_count, new_count)
            };
            changes.push(change_msg);
        } else {
            changes.push("mods updated".to_string());
        }
    }

    fn check_natives_changes(
        old: &VersionBuilder,
        new: &VersionBuilder,
        changes: &mut Vec<String>,
    ) {
        if !Self::natives_have_changed(old, new) {
            return;
        }

        let change_msg = match (&old.natives, &new.natives) {
            (None, Some(_)) => "natives added",
            (Some(_), None) => "natives removed",
            _ => "natives updated",
        };
        changes.push(change_msg.to_string());
    }

    fn check_assets_changes(
        old: &VersionBuilder,
        new: &VersionBuilder,
        changes: &mut Vec<String>,
    ) {
        if !Self::assets_have_changed(old, new) {
            return;
        }

        let old_count = old.assets.len();
        let new_count = new.assets.len();

        if old_count != new_count {
            let change_msg = if new_count > old_count {
                format!("assets added ({} -> {})", old_count, new_count)
            } else {
                format!("assets removed ({} -> {})", old_count, new_count)
            };
            changes.push(change_msg);
        } else {
            changes.push("assets updated".to_string());
        }
    }

    fn client_has_changed(old: &VersionBuilder, new: &VersionBuilder) -> bool {
        match (&old.client, &new.client) {
            (Some(old_client), Some(new_client)) => {
                old_client.sha1 != new_client.sha1 || old_client.size != new_client.size
            }
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        }
    }

    fn libraries_have_changed(old: &VersionBuilder, new: &VersionBuilder) -> bool {
        if old.libraries.len() != new.libraries.len() {
            return true;
        }

        old.libraries
            .iter()
            .zip(new.libraries.iter())
            .any(|(old_lib, new_lib)| {
                old_lib.sha1 != new_lib.sha1 || old_lib.size != new_lib.size
            })
    }

    fn mods_have_changed(old: &VersionBuilder, new: &VersionBuilder) -> bool {
        if old.mods.len() != new.mods.len() {
            return true;
        }

        old.mods
            .iter()
            .zip(new.mods.iter())
            .any(|(old_mod, new_mod)| {
                old_mod.sha1 != new_mod.sha1 || old_mod.size != new_mod.size
            })
    }

    fn natives_have_changed(old: &VersionBuilder, new: &VersionBuilder) -> bool {
        match (&old.natives, &new.natives) {
            (Some(old_natives), Some(new_natives)) => {
                if old_natives.len() != new_natives.len() {
                    return true;
                }
                old_natives
                    .iter()
                    .zip(new_natives.iter())
                    .any(|(old_n, new_n)| {
                        old_n.sha1 != new_n.sha1 || old_n.size != new_n.size
                    })
            }
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        }
    }

    fn assets_have_changed(old: &VersionBuilder, new: &VersionBuilder) -> bool {
        if old.assets.len() != new.assets.len() {
            return true;
        }

        old.assets
            .iter()
            .zip(new.assets.iter())
            .any(|(old_asset, new_asset)| {
                old_asset.hash != new_asset.hash || old_asset.size != new_asset.size
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lighty_models::{Arguments, Client, JavaVersion, Library, MainClass};

    fn empty() -> VersionBuilder {
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

    fn lib(sha: &str, size: u64) -> Library {
        Library {
            name: "lib".into(),
            url: Some("u".into()),
            path: Some("p".into()),
            sha1: Some(sha.into()),
            size: Some(size),
        }
    }

    #[test]
    fn identical_builders_show_no_changes() {
        let a = empty();
        let b = empty();
        let (changed, list) = ChangeDetector::detect_changes(&a, &b);
        assert!(!changed);
        assert!(list.is_empty());
    }

    #[test]
    fn client_added_shows_in_changes() {
        let old = empty();
        let mut new = empty();
        new.client = Some(Client {
            name: "c".into(),
            url: "u".into(),
            path: "p".into(),
            sha1: "s".into(),
            size: 10,
        });
        let (changed, list) = ChangeDetector::detect_changes(&old, &new);
        assert!(changed);
        assert!(list.iter().any(|m| m == "client added"));
    }

    #[test]
    fn library_count_increase_is_described() {
        let mut old = empty();
        old.libraries.push(lib("a", 1));
        let mut new = empty();
        new.libraries.push(lib("a", 1));
        new.libraries.push(lib("b", 2));

        let (changed, list) = ChangeDetector::detect_changes(&old, &new);
        assert!(changed);
        assert!(list.iter().any(|m| m.contains("libraries added")));
    }

    #[test]
    fn library_size_change_is_detected() {
        let mut old = empty();
        old.libraries.push(lib("same-sha", 10));
        let mut new = empty();
        new.libraries.push(lib("same-sha", 99));

        let (changed, list) = ChangeDetector::detect_changes(&old, &new);
        assert!(changed);
        assert!(list.iter().any(|m| m == "libraries updated"));
    }
}

