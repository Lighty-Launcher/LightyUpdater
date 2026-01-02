use super::models::FileSystem;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

impl FileSystem {
    pub async fn ensure_server_structure(
        base_path: &str,
        server_folder: &str,
    ) -> Result<PathBuf> {
        let full_path = PathBuf::from(base_path).join(server_folder);
        let abs_path = Self::get_absolute_path(&full_path)?;

        Self::create_directory(&abs_path, "Root directory").await?;
        Self::create_directory(&abs_path.join("client"), "Client directory").await?;
        Self::create_directory(&abs_path.join("libraries"), "Libraries directory").await?;
        Self::create_directory(&abs_path.join("mods"), "Mods directory").await?;

        let natives_path = abs_path.join("natives");
        Self::create_directory(&natives_path, "Natives directory").await?;
        Self::create_directory(&natives_path.join("windows"), "Natives/Windows").await?;
        Self::create_directory(&natives_path.join("linux"), "Natives/Linux").await?;
        Self::create_directory(&natives_path.join("macos"), "Natives/MacOS").await?;

        let assets_path = abs_path.join("assets");
        Self::create_directory(&assets_path, "Assets directory").await?;

        Ok(abs_path)
    }

    pub fn build_server_path(base_path: &str, server_folder: &str) -> PathBuf {
        PathBuf::from(base_path).join(server_folder)
    }

    async fn create_directory(path: &Path, description: &str) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path).await?;
            tracing::debug!("    Created: {} ({})", path.display(), description);
        } else {
            tracing::debug!("    Exists:  {} ({})", path.display(), description);
        }
        Ok(())
    }

    fn get_absolute_path(path: &Path) -> Result<PathBuf> {
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        Ok(abs_path)
    }

    pub fn get_absolute_path_string(path: &str) -> Result<String> {
        let path_buf = PathBuf::from(path);
        let abs = Self::get_absolute_path(&path_buf)?;
        Ok(abs.to_string_lossy().to_string())
    }
}
