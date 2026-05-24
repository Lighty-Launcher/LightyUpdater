use crate::errors::RescanError;
use crate::models::RescanOrchestrator;
use futures::stream::{self, StreamExt};
use lighty_events::AppEvent;
use lighty_models::VersionBuilder;
use lighty_scanner::ServerScanner;
use std::sync::Arc;

type Result<T> = std::result::Result<T, RescanError>;

impl RescanOrchestrator {
    pub async fn scan_all_servers(&self) -> Result<()> {
        let storage = self.storage.as_ref().ok_or_else(|| {
            RescanError::InvalidConfig("Storage backend not initialized".to_string())
        })?;

        let (servers, base_path, batch_config, buffer_size, server_parallelism) = {
            let config = self.config.read().await;
            (
                config.servers.clone(),
                config.server.base_path.clone(),
                config.cache.batch.clone(),
                config.cache.checksum_buffer_size,
                config.cache.hash_concurrency,
            )
        };

        if server_parallelism == 0 {
            return Err(RescanError::InvalidConfig(
                "cache.hash_concurrency must be greater than 0".to_string(),
            ));
        }
        let concurrency = server_parallelism;
        let enabled_servers: Vec<_> = servers
            .into_iter()
            .filter(|server_config| server_config.enabled)
            .collect();

        let results: Vec<_> = stream::iter(enabled_servers)
            .map(|server_config| {
                let storage = Arc::clone(storage);
                let base_path = base_path.clone();
                let batch_config = batch_config.clone();
                async move {
                    let result = ServerScanner::scan_server(
                        &server_config,
                        &storage,
                        base_path.as_ref(),
                        &batch_config,
                        buffer_size,
                    )
                    .await;
                    (server_config.name.clone(), result)
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        for (server_name, result) in results {
            match result {
                Ok(mut builder) => {
                    builder.build_url_map();
                    self.cache.insert(server_name.to_string(), Arc::new(builder));
                    self.last_updated
                        .insert(server_name.to_string(), Self::current_timestamp());
                    self.events.emit(AppEvent::CacheNew {
                        server: server_name.to_string(),
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        "Server {} initial scan failed (probably empty), adding empty version to cache: {}",
                        server_name,
                        e
                    );

                    let (server_config, _) = {
                        let config = self.config.read().await;
                        let server_config = config
                            .servers
                            .iter()
                            .find(|s| s.name.as_ref() == server_name.as_ref())
                            .cloned();
                        (server_config, config.server.base_path.clone())
                    };

                    if let Some(config) = server_config {
                        let mut empty_builder = VersionBuilder {
                            main_class: lighty_models::MainClass {
                                main_class: config.main_class.clone(),
                            },
                            java_version: lighty_models::JavaVersion {
                                major_version: config.java_version,
                            },
                            arguments: lighty_models::Arguments {
                                game: config.game_args.clone(),
                                jvm: config.jvm_args.clone(),
                            },
                            libraries: Vec::new(),
                            mods: Vec::new(),
                            natives: None,
                            client: None,
                            assets: Vec::new(),
                            url_to_path_map: std::collections::HashMap::new(),
                        };
                        empty_builder.build_url_map();
                        self.cache
                            .insert(server_name.to_string(), Arc::new(empty_builder));
                        self.last_updated
                            .insert(server_name.to_string(), Self::current_timestamp());
                        self.events.emit(AppEvent::CacheNew {
                            server: server_name.to_string(),
                        });
                    } else {
                        self.events.emit(AppEvent::Error {
                            context: format!("Failed to scan server {}", server_name),
                            error: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn force_rescan_server(&self, server_name: &str) -> Result<()> {
        let storage = self.storage.as_ref().ok_or_else(|| {
            RescanError::InvalidConfig("Storage backend not initialized".to_string())
        })?;

        let (server_config, base_path, batch_config, buffer_size) = {
            let config = self.config.read().await;
            let server_config = config
                .servers
                .iter()
                .find(|s| s.name.as_ref() == server_name)
                .ok_or_else(|| RescanError::ServerNotFound(server_name.to_string()))?
                .clone();
            (
                server_config,
                config.server.base_path.clone(),
                config.cache.batch.clone(),
                config.cache.checksum_buffer_size,
            )
        };
        let had_existing_cache = self.cache.contains(server_name);

        match ServerScanner::scan_server(
            &server_config,
            storage,
            base_path.as_ref(),
            &batch_config,
            buffer_size,
        )
        .await
        {
            Ok(builder) => {
                self.update_cache_if_changed(&server_config, builder).await;
                tracing::info!("Successfully rescanned server: {}", server_name);
            }
            Err(e) => {
                tracing::warn!("Server {} scan failed: {}", server_name, e);
                if !had_existing_cache {
                    tracing::warn!(
                        "No previous cache for {}; inserting empty version as fallback",
                        server_name
                    );
                    let mut empty_builder = VersionBuilder {
                        main_class: lighty_models::MainClass {
                            main_class: server_config.main_class.clone(),
                        },
                        java_version: lighty_models::JavaVersion {
                            major_version: server_config.java_version,
                        },
                        arguments: lighty_models::Arguments {
                            game: server_config.game_args.clone(),
                            jvm: server_config.jvm_args.clone(),
                        },
                        libraries: Vec::new(),
                        mods: Vec::new(),
                        natives: None,
                        client: None,
                        assets: Vec::new(),
                        url_to_path_map: std::collections::HashMap::new(),
                    };
                    empty_builder.build_url_map();
                    self.cache.insert(server_name.to_string(), Arc::new(empty_builder));
                    self.last_updated
                        .insert(server_name.to_string(), Self::current_timestamp());
                } else {
                    tracing::warn!(
                        "Keeping previous cache for {} after failed rescan",
                        server_name
                    );
                }
            }
        }

        Ok(())
    }
}
