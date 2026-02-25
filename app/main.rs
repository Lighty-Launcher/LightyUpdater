#[tokio::main]
async fn main() -> lighty_runtime::Result<()> {
    lighty_runtime::run_server_with_default_config(env!("CARGO_PKG_VERSION")).await
}
