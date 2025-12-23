use super::{assets, client, libraries, mods, natives};
use super::models::ServerScanner;
use lighty_config::{ServerConfig, BatchConfig};
use lighty_models::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

impl ServerScanner {
    pub async fn scan_server(
        config: &ServerConfig,
        base_url: &str,
        base_path: &str,
        batch_config: &BatchConfig,
    ) -> Result<VersionBuilder> {
        let start = std::time::Instant::now();

        let server_path = PathBuf::from(base_path).join(&config.name);
        Self::validate_server_path(&server_path, &config.name)?;

        let builder = Self::build_version_metadata(config, &server_path, base_url, batch_config).await?;

        let duration = start.elapsed();
        tracing::debug!(
            "Scanned '{}' in {:.2}s",
            config.name,
            duration.as_secs_f64()
        );

        Ok(builder)
    }

    pub async fn scan_server_silent(
        config: &ServerConfig,
        base_url: &str,
        base_path: &str,
        batch_config: &BatchConfig,
    ) -> Result<VersionBuilder> {
        let server_path = PathBuf::from(base_path).join(&config.name);
        Self::validate_server_path(&server_path, &config.name)?;
        Self::build_version_metadata(config, &server_path, base_url, batch_config).await
    }

    fn validate_server_path(path: &PathBuf, folder: &str) -> Result<()> {
        if !path.exists() {
            anyhow::bail!("Server folder does not exist: {}", folder);
        }
        Ok(())
    }

    async fn build_version_metadata(
        config: &ServerConfig,
        server_path: &PathBuf,
        base_url: &str,
        batch_config: &BatchConfig,
    ) -> Result<VersionBuilder> {
        let mut builder = VersionBuilder {
            main_class: MainClass {
                main_class: config.main_class.clone(),
            },
            java_version: JavaVersion {
                major_version: config.java_version,
            },
            arguments: Arguments {
                game: config.game_args.clone(),
                jvm: config.jvm_args.clone(),
            },
            libraries: if config.enable_libraries {
                libraries::scan_libraries(server_path, &config.name, base_url, batch_config.libraries).await?
            } else {
                vec![]
            },
            mods: if config.enable_mods {
                mods::scan_mods(server_path, &config.name, base_url, batch_config.mods).await?
            } else {
                vec![]
            },
            natives: if config.enable_natives {
                Some(natives::scan_natives(server_path, &config.name, base_url).await?)
            } else {
                None
            },
            client: if config.enable_client {
                client::scan_client(server_path, &config.name, base_url).await?
            } else {
                None
            },
            assets: if config.enable_assets {
                assets::scan_assets(server_path, &config.name, base_url).await?
            } else {
                vec![]
            },
            url_to_path_map: HashMap::new(),
        };

        // Build URL→path lookup map for O(1) file resolution
        builder.build_url_map();

        Ok(builder)
    }
}
