# MCP Webpublication Server

MCP server for Webpublication API - provides access to workspace management, generation, customization, and publication features.

## Features

- **get_recent_resources**: Get the last 20 publications
- **get_resource**: Get resource/publication information
- **get_publication_settings**: Get publication settings and configuration
- **toggle_wishlist**: Enable/disable Wishlist
- **get_cover_image**: Get the publication's cover image as bytes and encode it to base64 so the AI can see it
- Cookie-based authentication with WP_token
- Support for multiple API endpoints (workspaceManagerWs, generationWs, customizationWs, etc.)

## Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/)

## Quick Start

1. Copy `.env.example` to `.env` and add your credentials (environment variables needed for testing with the MCP Inspector):
```env
API_URL=your_api_url
DRIVE_URL=your_drive_url
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

### Using Claude

Configure the MCP Web Publication server for either Claude Desktop or the Claude CLI.

#### Claude Desktop

Add the snippet below to your Claude Desktop config file:

- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

#### Claude CLI

At the root of your project, add the same snippet to `.mcp.json`.

```json
{
  "mcpServers": {
    "webpublication": {
      "command": "/path/to/mcp-webpublication-server/target/release/mcp-webpublication-server",
      "env": {
        "API_URL": "your_api_url",
        "DRIVE_URL": "your_drive_url",
        "CLIENT_ID": "your_client_id",
        "WP_TOKEN": "your_wp_token"
      }
    }
  }
}
```

## Tools

### get_recent_resources
- **Input**: None
- **Output**: Returns the 20 most recent publications with their globalId and label (name)
- **Usage**: Use this first to find a publication's globalId when not provided by the user

### get_resource
- **Input**: `resource_gid` (number, e.g., 2473843)
- **Output**: Detailed resource/publication information with metadata
- **Note**: Month values are zero-based. Add 1 to get the calendar month (e.g., 5 = June)

### get_publication_settings
- **Input**: `resource_gid` (number, e.g., 2473843)
- **Output**: Publication settings and configuration details including wishlistEnabled and coverImage.relUrl

### toggle_wishlist
- **Input**:
  - `publication_gid` (number, e.g., 2473843)
  - `wishlist_enabled` (boolean: true/false)
- **Output**: Updated publication settings with new wishlist status
- **Note**: Check current status via `get_publication_settings -> wishlistEnabled`

### get_cover_image
- **Input**: `rel_url` (string) - obtained from `get_publication_settings -> coverImage.relUrl`
- **Output**: Cover image as base64-encoded image data

## Development

```bash
# Run with logging
RUST_LOG=debug cargo run

# Build release
cargo build --release
```

The resulting executable can be found at `/path/to/mcp-webpublication-server/target/release/mcp-webpublication-server`

## Resources

- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)


[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
