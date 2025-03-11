use anyhow::Result;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use cratedocs_mcp::server::http_sse_server::App;

const BIND_ADDRESS: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse socket address
    let addr: SocketAddr = BIND_ADDRESS.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!("Rust Documentation Server listening on {}", listener.local_addr()?);
    tracing::info!("Access the Rust Documentation Server at http://{}/sse", BIND_ADDRESS);
    
    // Create app and run server
    let app = App::new();
    axum::serve(listener, app.router()).await?;
    
    Ok(())
}