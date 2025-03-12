use anyhow::Result;
use clap::{Parser, Subcommand};
use cratedocs_mcp::DocRouter;
use mcp_server::router::RouterService;
use mcp_server::{ByteTransport, Server};
use std::net::SocketAddr;
use tokio::io::{stdin, stdout};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{self, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the server in stdin/stdout mode
    Stdio {
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
    /// Run the server with HTTP/SSE interface
    Http {
        /// Address to bind the HTTP server to
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        address: String,
        
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stdio { debug } => run_stdio_server(debug).await,
        Commands::Http { address, debug } => run_http_server(address, debug).await,
    }
}

async fn run_stdio_server(debug: bool) -> Result<()> {
    // Set up file appender for logging
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "stdio-server.log");

    // Initialize the tracing subscriber with file logging
    let level = if debug { tracing::Level::DEBUG } else { tracing::Level::INFO };
    
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(level.into()))
        .with_writer(file_appender)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting MCP documentation server in STDIN/STDOUT mode");

    // Create an instance of our documentation router
    let router = RouterService(DocRouter::new());

    // Create and run the server
    let server = Server::new(router);
    let transport = ByteTransport::new(stdin(), stdout());

    tracing::info!("Documentation server initialized and ready to handle requests");
    Ok(server.run(transport).await?)
}

async fn run_http_server(address: String, debug: bool) -> Result<()> {
    // Setup tracing
    let level = if debug { "debug" } else { "info" };
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{},{}", level, env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse socket address
    let addr: SocketAddr = address.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!("Rust Documentation Server listening on {}", listener.local_addr()?);
    tracing::info!("Access the Rust Documentation Server at http://{}/sse", addr);
    
    // Create app and run server
    let app = cratedocs_mcp::server::http_sse_server::App::new();
    axum::serve(listener, app.router()).await?;
    
    Ok(())
}