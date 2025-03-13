use anyhow::Result;
use clap::{Parser, Subcommand};
use cratedocs_mcp::tools::DocRouter;
use mcp_core::Content;
use mcp_server::router::RouterService;
use mcp_server::{ByteTransport, Router, Server};
use serde_json::json;
use std::net::SocketAddr;
use tokio::io::{stdin, stdout};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{self, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(author, version = "0.1.0", about, long_about = None)]
#[command(propagate_version = true)]
#[command(disable_version_flag = true)]
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
    /// Test tools directly from the CLI
    Test {
        /// The tool to test (lookup_crate, search_crates, lookup_item)
        #[arg(long, default_value = "lookup_crate")]
        tool: String,
        
        /// Crate name for lookup_crate and lookup_item
        #[arg(long)]
        crate_name: Option<String>,
        
        /// Item path for lookup_item (e.g., std::vec::Vec)
        #[arg(long)]
        item_path: Option<String>,
        
        /// Search query for search_crates
        #[arg(long)]
        query: Option<String>,
        
        /// Crate version (optional)
        #[arg(long)]
        version: Option<String>,
        
        /// Result limit for search_crates
        #[arg(long)]
        limit: Option<u32>,
        
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
        Commands::Test { 
            tool, 
            crate_name, 
            item_path, 
            query, 
            version, 
            limit, 
            debug 
        } => run_test_tool(tool, crate_name, item_path, query, version, limit, debug).await,
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
    let app = cratedocs_mcp::transport::http_sse_server::App::new();
    axum::serve(listener, app.router()).await?;
    
    Ok(())
}

/// Run a direct test of a documentation tool from the CLI
async fn run_test_tool(
    tool: String,
    crate_name: Option<String>,
    item_path: Option<String>,
    query: Option<String>,
    version: Option<String>,
    limit: Option<u32>,
    debug: bool,
) -> Result<()> {
    // Print help information if the tool is "help"
    if tool == "help" {
        println!("CrateDocs CLI Tool Tester\n");
        println!("Usage examples:");
        println!("  cargo run --bin cratedocs -- test --tool lookup_crate --crate-name serde");
        println!("  cargo run --bin cratedocs -- test --tool lookup_crate --crate-name tokio --version 1.35.0");
        println!("  cargo run --bin cratedocs -- test --tool lookup_item --crate-name tokio --item-path sync::mpsc::Sender");
        println!("  cargo run --bin cratedocs -- test --tool lookup_item --crate-name serde --item-path Serialize --version 1.0.147");
        println!("  cargo run --bin cratedocs -- test --tool search_crates --query logger\n");
        println!("Available tools:");
        println!("  lookup_crate   - Look up documentation for a Rust crate");
        println!("  lookup_item    - Look up documentation for a specific item in a crate");
        println!("                   Format: 'module::path::ItemName' (e.g., 'sync::mpsc::Sender')");
        println!("                   The tool will try to detect if it's a struct, enum, trait, fn, or macro");
        println!("  search_crates  - Search for crates on crates.io");
        println!("  help           - Show this help information\n");
        return Ok(());
    }
    // Set up console logging
    let level = if debug { tracing::Level::DEBUG } else { tracing::Level::INFO };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .without_time()
        .with_target(false)
        .init();

    // Create router instance
    let router = DocRouter::new();
    
    tracing::info!("Testing tool: {}", tool);
    
    // Prepare arguments based on the tool being tested
    let arguments = match tool.as_str() {
        "lookup_crate" => {
            let crate_name = crate_name.ok_or_else(|| 
                anyhow::anyhow!("--crate-name is required for lookup_crate tool"))?;
            
            json!({
                "crate_name": crate_name,
                "version": version,
            })
        },
        "lookup_item" => {
            let crate_name = crate_name.ok_or_else(|| 
                anyhow::anyhow!("--crate-name is required for lookup_item tool"))?;
            let item_path = item_path.ok_or_else(|| 
                anyhow::anyhow!("--item-path is required for lookup_item tool"))?;
            
            json!({
                "crate_name": crate_name,
                "item_path": item_path,
                "version": version,
            })
        },
        "search_crates" => {
            let query = query.ok_or_else(|| 
                anyhow::anyhow!("--query is required for search_crates tool"))?;
            
            json!({
                "query": query,
                "limit": limit,
            })
        },
        _ => return Err(anyhow::anyhow!("Unknown tool: {}", tool)),
    };
    
    // Call the tool and get results
    tracing::debug!("Calling {} with arguments: {}", tool, arguments);
    println!("Executing {} tool...", tool);
    
    let result = match router.call_tool(&tool, arguments).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("\nERROR: {}", e);
            eprintln!("\nTip: Try these suggestions:");
            eprintln!("  - For crate docs: cargo run --bin cratedocs -- test --tool lookup_crate --crate-name tokio");
            eprintln!("  - For item lookup: cargo run --bin cratedocs -- test --tool lookup_item --crate-name tokio --item-path sync::mpsc::Sender");
            eprintln!("  - For item lookup with version: cargo run --bin cratedocs -- test --tool lookup_item --crate-name serde --item-path Serialize --version 1.0.147");
            eprintln!("  - For crate search: cargo run --bin cratedocs -- test --tool search_crates --query logger --limit 5");
            eprintln!("  - For help: cargo run --bin cratedocs -- test --tool help");
            return Ok(());
        }
    };
    
    // Print results
    if !result.is_empty() {
        for content in result {
            match content {
                Content::Text(text) => {
                    println!("\n--- TOOL RESULT ---\n");
                    // Access the raw string from TextContent.text field
                    println!("{}", text.text);
                    println!("\n--- END RESULT ---");
                },
                _ => println!("Received non-text content"),
            }
        }
    } else {
        println!("Tool returned no results");
    }
    
    Ok(())
}