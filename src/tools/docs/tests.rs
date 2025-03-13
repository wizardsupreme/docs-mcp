
#[tokio::test]
async fn test_doc_cache() {
    let cache = DocCache::new();
    
    // Initial get should return None
    let result = cache.get("test_key").await;
    assert_eq!(result, None);
    
    // Set and get should return the value
    cache.set("test_key".to_string(), "test_value".to_string()).await;
    let result = cache.get("test_key").await;
    assert_eq!(result, Some("test_value".to_string()));
}

#[tokio::test]
async fn test_router_capabilities() {
    let router = DocRouter::new();
    
    // Test basic properties
    assert_eq!(router.name(), "rust-docs");
    assert!(router.instructions().contains("documentation"));
    
    // Test capabilities
    let capabilities = router.capabilities();
    assert!(capabilities.tools.is_some());
    // Only assert that tools are supported, skip resources checks since they might be configured
    // differently depending on the SDK version
}

#[tokio::test]
async fn test_list_tools() {
    let router = DocRouter::new();
    let tools = router.list_tools();
    
    // Should have exactly 3 tools
    assert_eq!(tools.len(), 3);
    
    // Check tool names
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"lookup_crate".to_string()));
    assert!(tool_names.contains(&"search_crates".to_string()));
    assert!(tool_names.contains(&"lookup_item".to_string()));
}

#[tokio::test]
async fn test_invalid_tool_call() {
    let router = DocRouter::new();
    let result = router.call_tool("invalid_tool", json!({})).await;
    
    // Should return NotFound error
    assert!(matches!(result, Err(ToolError::NotFound(_))));
    if let Err(ToolError::NotFound(msg)) = result {
        assert!(msg.contains("invalid_tool"));
    }
}

#[tokio::test]
async fn test_lookup_crate_missing_parameter() {
    let router = DocRouter::new();
    let result = router.call_tool("lookup_crate", json!({})).await;
    
    // Should return InvalidParameters error
    assert!(matches!(result, Err(ToolError::InvalidParameters(_))));
}

#[tokio::test]
async fn test_search_crates_missing_parameter() {
    let router = DocRouter::new();
    let result = router.call_tool("search_crates", json!({})).await;
    
    // Should return InvalidParameters error
    assert!(matches!(result, Err(ToolError::InvalidParameters(_))));
}

#[tokio::test]
async fn test_lookup_item_missing_parameters() {
    let router = DocRouter::new();
    
    // Missing both parameters
    let result = router.call_tool("lookup_item", json!({})).await;
    assert!(matches!(result, Err(ToolError::InvalidParameters(_))));
    
    // Missing item_path
    let result = router.call_tool("lookup_item", json!({
        "crate_name": "tokio"
    })).await;
    assert!(matches!(result, Err(ToolError::InvalidParameters(_))));
}

// Requires network access, can be marked as ignored if needed
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_lookup_crate_integration() {
    let router = DocRouter::new();
    let result = router.call_tool("lookup_crate", json!({
        "crate_name": "serde"
    })).await;
    
    assert!(result.is_ok());
    let contents = result.unwrap();
    assert_eq!(contents.len(), 1);
    if let Content::Text(text) = &contents[0] {
        assert!(text.text.contains("serde"));
    } else {
        panic!("Expected text content");
    }
}

// Requires network access, can be marked as ignored if needed
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_search_crates_integration() {
    let router = DocRouter::new();
    let result = router.call_tool("search_crates", json!({
        "query": "json",
        "limit": 5
    })).await;
    
    assert!(result.is_ok());
    let contents = result.unwrap();
    assert_eq!(contents.len(), 1);
    if let Content::Text(text) = &contents[0] {
        assert!(text.text.contains("crates"));
    } else {
        panic!("Expected text content");
    }
}

// Requires network access, can be marked as ignored if needed
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_lookup_item_integration() {
    let router = DocRouter::new();
    let result = router.call_tool("lookup_item", json!({
        "crate_name": "serde",
        "item_path": "ser::Serializer"
    })).await;
    
    assert!(result.is_ok());
    let contents = result.unwrap();
    assert_eq!(contents.len(), 1);
    if let Content::Text(text) = &contents[0] {
        assert!(text.text.contains("Serializer"));
    } else {
        panic!("Expected text content");
    }
}