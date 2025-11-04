# MCP Webpublication Server

MCP server for Webpublication API - provides access to workspace management, generation, customization, and publication features.

## Features

- **get_resource**: Get resource/publication information
- **get_publication_settings**: Get publication settings and configuration
- Cookie-based authentication with WP_token
- Support for multiple API endpoints (workspaceManagerWs, generationWs, customizationWs, etc.)

## Quick Start

1. Copy `.env.example` to `.env` and add your credentials:
```env
API_URL=your_api_url
CLIENT_ID=your_client_id
WP_TOKEN=your_wp_token
```

2. Build and test:
```bash
cargo build --release
npx @modelcontextprotocol/inspector ./target/release/mcp-webpublication-server-poc
```

Open `http://127.0.0.1:6274` and test with publication GID like 2473843.

## Usage

### Testing with MCP Inspector

```bash
npx @modelcontextprotocol/inspector cargo run
# or
npx @modelcontextprotocol/inspector ./target/release/mcp-webpublication-server-poc
```

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "webpublication": {
      "command": "/path/to/mcp-webpublication-server/target/release/mcp-webpublication-server",
      "env": {
        "API_URL": "your_api_url",
        "CLIENT_ID": "your_client_id",
        "WP_TOKEN": "your_wp_token"
      }
    }
  }
}
```

## Tools

### get_resource
- **Input**: `resource_gid` (number, e.g., 2473843)
- **Output**: Resource/publication information with metadata

### get_publication_settings
- **Input**: `resource_gid` (number, e.g., 2473843)
- **Output**: Publication settings and configuration details

## Development

```bash
# Run with logging
RUST_LOG=debug cargo run

# Build release
cargo build --release
```

## Resources

- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)


[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
