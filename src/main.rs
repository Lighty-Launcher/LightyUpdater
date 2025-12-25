mod bootstrap;

use lighty_api::AppState;
use lighty_events::{AppEvent, EventBus};
use lighty_cache::CacheManager;
use lighty_watcher::ConfigWatcher;
use lighty_storage::{LocalBackend, StorageBackend};
#[cfg(feature = "s3")]
use lighty_storage::S3Backend;
use lighty_config::StorageBackend as StorageBackendType;
use crate::bootstrap::{config, logging, router, server};
use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    logging::initialize();

    let events = EventBus::new(true);
    events.emit(AppEvent::Starting);

    let config_path = std::env::var("LIGHTY_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
    let config = config::load(&config_path, &events).await?;

    server::initialize_folders(&config, &events).await?;

    let config = Arc::new(tokio::sync::RwLock::new(config));

    // Initialize storage backend
    let storage = initialize_storage(&config).await?;

    // Initialize Cloudflare client if configured
    let cloudflare = initialize_cloudflare(&config).await;

    let cache_manager = Arc::new(
        CacheManager::new(
            Arc::clone(&config),
            Arc::clone(&events),
            Some(storage),
            cloudflare,
        )
        .await
    );
    cache_manager.initialize().await?;

    let config_watcher = Arc::new(ConfigWatcher::new(
        Arc::clone(&config),
        config_path,
        cache_manager.clone(),
    ));
    let config_watcher_handle = config_watcher.clone().start_watching().await?;

    cache_manager.start_auto_rescan().await;

    let (app, addr, tcp_nodelay, base_url) = {
        let config_read = config.read().await;
        let app_state = AppState::new(
            Arc::clone(&cache_manager),
            config_read.server.base_url.to_string(),
            config_read.server.base_path.to_string(),
            config_read.server.streaming_threshold_mb,
        );
        let app = router::build(&config_read, app_state);
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

async fn initialize_storage(config: &Arc<tokio::sync::RwLock<lighty_config::Config>>) -> Result<Arc<dyn StorageBackend>> {
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
            ).await?;

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

async fn initialize_cloudflare(
    config: &Arc<tokio::sync::RwLock<lighty_config::Config>>
) -> Option<Arc<lighty_cache::CloudflareClient>> {
    let config_read = config.read().await;

    if config_read.cloudflare.enabled {
        let client = lighty_cache::CloudflareClient::new(
            config_read.cloudflare.zone_id.clone(),
            config_read.cloudflare.api_token.clone(),
        );
        tracing::info!("Initialized Cloudflare cache purge client");
        Some(Arc::new(client))
    } else {
        None
    }
}

async fn bind_server(addr: &str) -> Result<tokio::net::TcpListener> {
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
