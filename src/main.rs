mod bootstrap;

use lighty_api::AppState;
use lighty_events::{AppEvent, EventBus};
use lighty_cache::CacheManager;
use lighty_watcher::ConfigWatcher;
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

    let cache_manager = Arc::new(CacheManager::new(Arc::clone(&config), Arc::clone(&events)).await);
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
            config_read.server.base_url.clone(),
            config_read.server.base_path.clone(),
            config_read.server.streaming_threshold_mb,
        );
        let app = router::build(&config_read, app_state);
        let addr = format!("{}:{}", config_read.server.host, config_read.server.port);
        (
            app,
            addr,
            config_read.server.tcp_nodelay,
            config_read.server.base_url.clone(),
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

async fn bind_server(addr: &str) -> Result<tokio::net::TcpListener> {
    tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            let port = addr.split(':').last().unwrap_or("unknown");
            tracing::error!("❌ Port {} is already in use", port);
            tracing::error!("   Another application is using this port");
            tracing::error!("   Solutions:");
            tracing::error!("   1. Stop the other application");
            tracing::error!("   2. Change the port in config.toml");
            #[cfg(target_os = "windows")]
            tracing::error!("   3. Find process: netstat -ano | findstr :{}", port);
            #[cfg(not(target_os = "windows"))]
            tracing::error!("   3. Find process: lsof -i :{}", port);
        } else {
            tracing::error!("❌ Failed to bind server on {}: {}", addr, e);
        }
        anyhow::anyhow!("Failed to bind server: {}", e)
    })
}
