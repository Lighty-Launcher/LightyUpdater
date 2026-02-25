use crate::errors::CliResult;

pub async fn run_server(config_path: std::path::PathBuf) -> CliResult<()> {
    lighty_runtime::run_server(
        config_path.to_string_lossy().to_string(),
        env!("CARGO_PKG_VERSION"),
    )
    .await?;
    Ok(())
}
