use lighty_api::{get_server_metadata, list_servers, serve_file, AppState};
use lighty_config::Config;
use axum::{http::StatusCode, routing::get, Router};
use std::time::Duration;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
};

pub fn build(config: &Config, app_state: AppState) -> Router {
    let max_body_size = config.server.max_body_size_mb * 1024 * 1024;
    let timeout = Duration::from_secs(config.server.timeout_secs);
    let max_concurrent_requests = config.server.max_concurrent_requests;

    let mut router = Router::new()
        .route("/", get(list_servers))
        .route("/:server_name.json", get(get_server_metadata))
        .fallback(serve_file)
        .layer(ConcurrencyLimitLayer::new(max_concurrent_requests))
        .layer(RequestBodyLimitLayer::new(max_body_size))
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, timeout));

    // Optionally enable compression based on config
    if config.server.enable_compression {
        router = router.layer(CompressionLayer::new());
    }

    router
        .layer(build_cors_layer(&config.server.allowed_origins))
        .with_state(app_state)
}

fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    if allowed_origins.iter().any(|o| o == "*") {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    }
}
