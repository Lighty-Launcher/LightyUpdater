use crate::models::RescanOrchestrator;
use lighty_config::ServerConfig;
use lighty_events::AppEvent;
use lighty_models::VersionBuilder;
use std::sync::Arc;

impl RescanOrchestrator {
    pub(crate) async fn update_cache_if_changed(
        &self,
        server_config: &ServerConfig,
        new_builder: VersionBuilder,
    ) {
        let old_builder = self.cache.get(&server_config.name);

        let diff = lighty_file_diff::FileDiff::compute(
            &server_config.name,
            old_builder.as_ref().map(|arc| arc.as_ref()),
            &new_builder,
        );

        let has_changes = !diff.added.is_empty() || !diff.modified.is_empty() || !diff.removed.is_empty();

        if has_changes {
            self.refresh_file_cache_for_diff(&server_config.name, &diff).await;

            if let Some(storage) = &self.storage {
                if storage.is_remote() {
                    if let Err(e) = self.sync_cloud_storage(&server_config.name, &diff).await {
                        tracing::error!(
                            "Failed to sync cloud storage for server {}: {}",
                            server_config.name,
                            e
                        );
                    }

                    let file_urls: Vec<String> = diff
                        .added
                        .iter()
                        .chain(diff.modified.iter())
                        .chain(diff.removed.iter())
                        .map(|change| change.url.clone())
                        .filter(|url| !url.is_empty())
                        .collect();

                    if !file_urls.is_empty() {
                        self.events.emit(AppEvent::CdnPurgeRequested {
                            server: server_config.name.to_string(),
                            urls: file_urls,
                        });
                    }
                }
            }

            let is_new = old_builder.is_none();
            let mut new_builder_mut = new_builder;
            if is_new {
                new_builder_mut.build_url_map();
            } else {
                diff.apply_to_url_map(&mut new_builder_mut);
            }

            self.cache
                .insert(server_config.name.to_string(), Arc::new(new_builder_mut));
            self.last_updated
                .insert(server_config.name.to_string(), Self::current_timestamp());

            self.events.emit(AppEvent::CloudflarePurgeRequested {
                server: server_config.name.to_string(),
            });

            if is_new {
                self.events.emit(AppEvent::CacheNew {
                    server: server_config.name.to_string(),
                });
            } else {
                let change_summary = format!(
                    "{} added, {} modified, {} removed",
                    diff.added.len(),
                    diff.modified.len(),
                    diff.removed.len()
                );
                self.events.emit(AppEvent::CacheUpdated {
                    server: server_config.name.to_string(),
                    changes: vec![change_summary],
                });
            }
        } else {
            self.events.emit(AppEvent::CacheUnchanged {
                server: server_config.name.to_string(),
            });
        }
    }

    async fn refresh_file_cache_for_diff(
        &self,
        server_name: &str,
        diff: &lighty_file_diff::FileDiff,
    ) {
        for change in &diff.removed {
            if let Some(path) = relative_cache_path(server_name, &change.local_path) {
                self.file_cache_manager.invalidate_file(server_name, path).await;
            }
        }

        for change in diff.added.iter().chain(diff.modified.iter()) {
            if let Some(path) = relative_cache_path(server_name, &change.local_path) {
                let full_path = self.base_path.join(path);
                if let Err(error) = self
                    .file_cache_manager
                    .refresh_file_from_disk(server_name, path, &full_path)
                    .await
                {
                    tracing::warn!(
                        "Failed to refresh RAM cache entry for {} ({}): {}",
                        server_name,
                        path,
                        error
                    );
                }
            }
        }
    }
}

fn relative_cache_path<'a>(server_name: &str, local_path: &'a str) -> Option<&'a str> {
    local_path
        .strip_prefix(server_name)
        .and_then(|path| path.strip_prefix('/').or_else(|| path.strip_prefix('\\')))
}
