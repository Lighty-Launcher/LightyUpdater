pub mod bootstrap;

use crate::errors::CliResult;
use lighty_api::AppState;
use lighty_cache::CacheManager;
use lighty_config::StorageBackend as StorageBackendType;
use lighty_events::{AppEvent, EventBus};
use lighty_storage::{LocalBackend, StorageBackend};
#[cfg(feature = "s3")]
use lighty_storage::S3Backend;
use lighty_watcher::ConfigWatcher;
use std::sync::Arc;

pub async fn run_server(config_path: std::path::PathBuf) -> CliResult<()> {
    bootstrap::logging::initialize();

    let events = EventBus::new(true);
    events.emit(AppEvent::Starting {
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    let config = bootstrap::config::load(&config_path.to_string_lossy(), &events).await?;

    bootstrap::server::initialize_folders(&config, &events).await?;

    let config = Arc::new(tokio::sync::RwLock::new(config));

    // Initialize storage backend
    let storage = initialize_storage(&config).await?;

    // Initialize CDN client if configured
    let cdn = initialize_cdn(&config).await;

    // Initialize Cloudflare API client if configured
    let cloudflare = initialize_cloudflare(&config).await;

    let cache_manager = Arc::new(
        CacheManager::new(
            Arc::clone(&config),
            Arc::clone(&events),
            Some(storage),
            cdn,
            cloudflare,
        )
        .await,
    );
    cache_manager.initialize().await.map_err(|e| anyhow::anyhow!(e))?;

    let config_watcher = Arc::new(ConfigWatcher::new(
        Arc::clone(&config),
        config_path.to_string_lossy().to_string(),
        cache_manager.clone(),
    ));
    let config_watcher_handle = config_watcher.clone().start_watching().await.map_err(|e| anyhow::anyhow!(e))?;

    cache_manager.start_auto_rescan().await;

    let (app, addr, tcp_nodelay, base_url) = {
        let config_read = config.read().await;
        let app_state = AppState::new(
            Arc::clone(&cache_manager),
            Arc::clone(&config),
        );
        let app = bootstrap::router::build(&config_read, app_state);
        let addr = format!("{}:{}", config_read.server.host, config_read.server.port);
        (
            app,
            addr,
            config_read.server.tcp_nodelay,
            config_read.server.base_url.to_string(),
        )
    };

    let listener = bind_server(&addr).await?;

    events.emit(AppEvent::Ready {
        addr: addr.to_string(),
        base_url,
    });

    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        tracing::info!("Shutdown signal received, initiating graceful shutdown...");
    };

    axum::serve(listener, app.into_make_service())
        .tcp_nodelay(tcp_nodelay)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    // Graceful shutdown: wait for config watcher to stop
    config_watcher_handle.abort();
    let _ = config_watcher_handle.await;

    cache_manager.shutdown().await;
    events.emit(AppEvent::Shutdown);
    Ok(())
}

async fn initialize_storage(
    config: &Arc<tokio::sync::RwLock<lighty_config::Config>>,
) -> anyhow::Result<Arc<dyn StorageBackend>> {
    let config_read = config.read().await;

    match config_read.storage.backend {
        StorageBackendType::Local => {
            let backend = LocalBackend::new(
                config_read.server.base_url.to_string(),
                std::path::PathBuf::from(config_read.server.base_path.as_ref()),
            );
            Ok(Arc::new(backend) as Arc<dyn StorageBackend>)
        }
        #[cfg(feature = "s3")]
        StorageBackendType::S3 => {
            if !config_read.storage.s3.enabled {
                anyhow::bail!("S3 backend selected but not enabled in configuration");
            }

            let backend = S3Backend::new(
                config_read.storage.s3.endpoint_url.to_string(),
                config_read.storage.s3.region.to_string(),
                config_read.storage.s3.access_key_id.clone(),
                config_read.storage.s3.secret_access_key.clone(),
                config_read.storage.s3.bucket_name.to_string(),
                config_read.storage.s3.public_url.to_string(),
                config_read.storage.s3.bucket_prefix.to_string(),
            )
            .await?;

            tracing::info!(
                "Initialized S3 storage backend: bucket={}, endpoint={}",
                config_read.storage.s3.bucket_name,
                config_read.storage.s3.endpoint_url
            );

            Ok(Arc::new(backend) as Arc<dyn StorageBackend>)
        }
        #[cfg(not(feature = "s3"))]
        StorageBackendType::S3 => {
            anyhow::bail!(
                "S3 backend selected but not compiled. Rebuild with --features s3 to enable S3 support.\n\
                Note: S3 support requires cmake to be installed on your system."
            )
        }
    }
}

async fn initialize_cdn(
    config: &Arc<tokio::sync::RwLock<lighty_config::Config>>,
) -> Option<Arc<lighty_cache::CdnClient>> {
    let config_read = config.read().await;

    if config_read.cdn.enabled {
        let client = lighty_cache::CdnClient::new(
            &config_read.cdn.provider,
            config_read.cdn.zone_id.clone(),
            config_read.cdn.api_token.clone(),
        );
        tracing::info!(
            "Initialized CDN cache purge client (provider: {})",
            config_read.cdn.provider
        );
        Some(Arc::new(client))
    } else {
        None
    }
}

async fn initialize_cloudflare(
    config: &Arc<tokio::sync::RwLock<lighty_config::Config>>,
) -> Option<Arc<lighty_cache::CloudflareClient>> {
    let config_read = config.read().await;

    if config_read.cloudflare.enabled {
        let client = lighty_cache::CloudflareClient::new(
            config_read.cloudflare.zone_id.clone(),
            config_read.cloudflare.api_token.clone(),
        );
        tracing::info!("Initialized Cloudflare API cache purge client");
        Some(Arc::new(client))
    } else {
        None
    }
}

async fn bind_server(addr: &str) -> anyhow::Result<tokio::net::TcpListener> {
    tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            let port = addr.split(':').last().unwrap_or("unknown");
            tracing::error!("❌ Port {} is already in use", port);
            tracing::error!("Another application is using this port");
            tracing::error!("Solutions:");
            tracing::error!("1. Stop the other application");
            tracing::error!("2. Change the port in config.toml");
            #[cfg(target_os = "windows")]
            tracing::error!("3. Find process: netstat -ano | findstr :{}", port);
            #[cfg(not(target_os = "windows"))]
            tracing::error!("3. Find process: lsof -i :{}", port);
        } else {
            tracing::error!("❌ Failed to bind server on {}: {}", addr, e);
        }
        anyhow::anyhow!("Failed to bind server: {}", e)
    })
}
