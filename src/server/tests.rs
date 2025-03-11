use mcp_server::router::RouterService;
use mcp_server::{Server};
use crate::DocRouter;
use mcp_server::Router;

#[tokio::test]
async fn test_server_initialization() {
    // Basic test to ensure the server initializes properly
    let doc_router = DocRouter::new();
    let router_name = doc_router.name();
    let router = RouterService(doc_router);
    let _server = Server::new(router);
    
    // Server should be created successfully without panics
    assert!(router_name.contains("rust-docs"));
}