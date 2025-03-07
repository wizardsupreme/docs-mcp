# CrateDocs MCP

This is an MCP (Model Context Protocol) server that provides tools for Rust crate documentation lookup. It allows LLMs to look up documentation for Rust crates they are unfamiliar with.

## Features

- Lookup crate documentation: Get general documentation for a Rust crate
- Search crates: Search for crates on crates.io based on keywords
- Lookup item documentation: Get documentation for a specific item (e.g., struct, function, trait) within a crate

## Installation

```bash
git clone https://github.com/yourusername/cratedocs-mcp.git
cd cratedocs-mcp
cargo build --release
```

## Running the Server

There are two ways to run the documentation server:

### STDIN/STDOUT Mode

This mode is useful for integrating with LLM clients that communicate via standard input/output:

```bash
cargo run --bin doc-server
```

### HTTP/SSE Mode

This mode exposes an HTTP endpoint that uses Server-Sent Events (SSE) for communication:

```bash
cargo run --bin axum-docs
```

By default, the server will listen on `http://127.0.0.1:8080/sse`.

## Available Tools

The server provides the following tools:

### 1. `lookup_crate`

Retrieves documentation for a specified Rust crate.

Parameters:
- `crate_name` (required): The name of the crate to look up
- `version` (optional): The version of the crate (defaults to latest)

Example:
```json
{
  "name": "lookup_crate",
  "arguments": {
    "crate_name": "tokio",
    "version": "1.28.0"
  }
}
```

### 2. `search_crates`

Searches for Rust crates on crates.io.

Parameters:
- `query` (required): The search query
- `limit` (optional): Maximum number of results to return (defaults to 10, max 100)

Example:
```json
{
  "name": "search_crates",
  "arguments": {
    "query": "async runtime",
    "limit": 5
  }
}
```

### 3. `lookup_item`

Retrieves documentation for a specific item in a crate.

Parameters:
- `crate_name` (required): The name of the crate
- `item_path` (required): Path to the item (e.g., 'std::vec::Vec')
- `version` (optional): The version of the crate (defaults to latest)

Example:
```json
{
  "name": "lookup_item",
  "arguments": {
    "crate_name": "serde",
    "item_path": "serde::Deserialize",
    "version": "1.0.160"
  }
}
```

## Implementation Notes

- The server includes a caching mechanism to prevent redundant API calls for the same documentation
- It interfaces with docs.rs for crate documentation and crates.io for search functionality
- Results are returned as plain text/HTML content that can be parsed and presented by the client

## MCP Protocol Integration

This server implements the Model Context Protocol (MCP) which allows it to be easily integrated with LLM clients that support the protocol. For more information about MCP, visit [the MCP repository](https://github.com/modelcontextprotocol/mcp).

## License

MIT License