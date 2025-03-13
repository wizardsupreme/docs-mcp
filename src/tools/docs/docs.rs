use std::{future::Future, pin::Pin, sync::Arc};

use mcp_core::{
    handler::{PromptError, ResourceError},
    prompt::Prompt,
    protocol::ServerCapabilities,
    Content, Resource, Tool, ToolError,
};
use mcp_server::router::CapabilitiesBuilder;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::Mutex;

// Cache for documentation lookups to avoid repeated requests
#[derive(Clone)]
pub struct DocCache {
    cache: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl Default for DocCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DocCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get(key).cloned()
    }

    pub async fn set(&self, key: String, value: String) {
        let mut cache = self.cache.lock().await;
        cache.insert(key, value);
    }
}

#[derive(Clone)]
pub struct DocRouter {
    client: Client,
    cache: DocCache,
}

impl Default for DocRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl DocRouter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: DocCache::new(),
        }
    }

    // Fetch crate documentation from docs.rs
    async fn lookup_crate(&self, crate_name: String, version: Option<String>) -> Result<String, ToolError> {
        // Check cache first
        let cache_key = if let Some(ver) = &version {
            format!("{}:{}", crate_name, ver)
        } else {
            crate_name.clone()
        };

        if let Some(doc) = self.cache.get(&cache_key).await {
            return Ok(doc);
        }

        // Construct the docs.rs URL for the crate
        let url = if let Some(ver) = version {
            format!("https://docs.rs/crate/{}/{}/", crate_name, ver)
        } else {
            format!("https://docs.rs/crate/{}/", crate_name)
        };

        // Fetch the documentation page
        let response = self.client.get(&url).send().await.map_err(|e| {
            ToolError::ExecutionError(format!("Failed to fetch documentation: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(ToolError::ExecutionError(format!(
                "Failed to fetch documentation. Status: {}",
                response.status()
            )));
        }

        let body = response.text().await.map_err(|e| {
            ToolError::ExecutionError(format!("Failed to read response body: {}", e))
        })?;

        // Cache the result
        self.cache.set(cache_key, body.clone()).await;
        
        Ok(body)
    }

    // Search crates.io for crates matching a query
    async fn search_crates(&self, query: String, limit: Option<u32>) -> Result<String, ToolError> {
        let limit = limit.unwrap_or(10).min(100); // Cap at 100 results
        
        let url = format!("https://crates.io/api/v1/crates?q={}&per_page={}", query, limit);
        
        let response = self.client.get(&url).send().await.map_err(|e| {
            ToolError::ExecutionError(format!("Failed to search crates.io: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(ToolError::ExecutionError(format!(
                "Failed to search crates.io. Status: {}",
                response.status()
            )));
        }

        let body = response.text().await.map_err(|e| {
            ToolError::ExecutionError(format!("Failed to read response body: {}", e))
        })?;
        
        Ok(body)
    }

    // Get documentation for a specific item in a crate
    async fn lookup_item(&self, crate_name: String, item_path: String, version: Option<String>) -> Result<String, ToolError> {
        // Check cache first
        let cache_key = if let Some(ver) = &version {
            format!("{}:{}:{}", crate_name, ver, item_path)
        } else {
            format!("{}:{}", crate_name, item_path)
        };

        if let Some(doc) = self.cache.get(&cache_key).await {
            return Ok(doc);
        }

        // Construct the docs.rs URL for the specific item
        let url = if let Some(ver) = version {
            format!("https://docs.rs/{}/{}/{}/", crate_name, ver, item_path.replace("::", "/"))
        } else {
            format!("https://docs.rs/{}/latest/{}/", crate_name, item_path.replace("::", "/"))
        };

        // Fetch the documentation page
        let response = self.client.get(&url).send().await.map_err(|e| {
            ToolError::ExecutionError(format!("Failed to fetch item documentation: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(ToolError::ExecutionError(format!(
                "Failed to fetch item documentation. Status: {}",
                response.status()
            )));
        }

        let body = response.text().await.map_err(|e| {
            ToolError::ExecutionError(format!("Failed to read response body: {}", e))
        })?;

        // Cache the result
        self.cache.set(cache_key, body.clone()).await;
        
        Ok(body)
    }
}

impl mcp_server::Router for DocRouter {
    fn name(&self) -> String {
        "rust-docs".to_string()
    }

    fn instructions(&self) -> String {
        "This server provides tools for looking up Rust crate documentation. \
        You can search for crates, lookup documentation for specific crates or \
        items within crates. Use these tools to find information about Rust libraries \
        you are not familiar with.".to_string()
    }

    fn capabilities(&self) -> ServerCapabilities {
        CapabilitiesBuilder::new()
            .with_tools(true)
            .with_resources(false, false)
            .with_prompts(false)
            .build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "lookup_crate".to_string(),
                "Look up documentation for a Rust crate".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "crate_name": {
                            "type": "string",
                            "description": "The name of the crate to look up"
                        },
                        "version": {
                            "type": "string",
                            "description": "The version of the crate (optional, defaults to latest)"
                        }
                    },
                    "required": ["crate_name"]
                }),
            ),
            Tool::new(
                "search_crates".to_string(),
                "Search for Rust crates on crates.io".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (optional, defaults to 10, max 100)"
                        }
                    },
                    "required": ["query"]
                }),
            ),
            Tool::new(
                "lookup_item".to_string(),
                "Look up documentation for a specific item in a Rust crate".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "crate_name": {
                            "type": "string",
                            "description": "The name of the crate"
                        },
                        "item_path": {
                            "type": "string",
                            "description": "Path to the item (e.g., 'std::vec::Vec')"
                        },
                        "version": {
                            "type": "string",
                            "description": "The version of the crate (optional, defaults to latest)"
                        }
                    },
                    "required": ["crate_name", "item_path"]
                }),
            ),
        ]
    }

    fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Content>, ToolError>> + Send + 'static>> {
        let this = self.clone();
        let tool_name = tool_name.to_string();
        let arguments = arguments.clone();

        Box::pin(async move {
            match tool_name.as_str() {
                "lookup_crate" => {
                    let crate_name = arguments
                        .get("crate_name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ToolError::InvalidParameters("crate_name is required".to_string()))?
                        .to_string();
                    
                    let version = arguments
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    let doc = this.lookup_crate(crate_name, version).await?;
                    Ok(vec![Content::text(doc)])
                }
                "search_crates" => {
                    let query = arguments
                        .get("query")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ToolError::InvalidParameters("query is required".to_string()))?
                        .to_string();
                    
                    let limit = arguments
                        .get("limit")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);
                    
                    let results = this.search_crates(query, limit).await?;
                    Ok(vec![Content::text(results)])
                }
                "lookup_item" => {
                    let crate_name = arguments
                        .get("crate_name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ToolError::InvalidParameters("crate_name is required".to_string()))?
                        .to_string();
                    
                    let item_path = arguments
                        .get("item_path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ToolError::InvalidParameters("item_path is required".to_string()))?
                        .to_string();
                    
                    let version = arguments
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    let doc = this.lookup_item(crate_name, item_path, version).await?;
                    Ok(vec![Content::text(doc)])
                }
                _ => Err(ToolError::NotFound(format!("Tool {} not found", tool_name))),
            }
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![]
    }

    fn read_resource(
        &self,
        _uri: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, ResourceError>> + Send + 'static>> {
        Box::pin(async move {
            Err(ResourceError::NotFound("Resource not found".to_string()))
        })
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![]
    }

    fn get_prompt(
        &self,
        prompt_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, PromptError>> + Send + 'static>> {
        let prompt_name = prompt_name.to_string();
        Box::pin(async move {
            Err(PromptError::NotFound(format!(
                "Prompt {} not found",
                prompt_name
            )))
        })
    }
}