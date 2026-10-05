# MCP Webpublication Server

MCP server for Webpublication API - provides access to workspace management, generation, customization, and publication features.

## Features

- **get_recent_resources**: Get the last 20 publications
- **get_resource**: Get resource/publication information
- **get_publication_settings**: Get publication settings and configuration
- **toggle_wishlist**: Enable/disable Wishlist
- **get_cover_image**: Get the publication's cover image as bytes and encode it to base64 so the AI can see it
- **list_folders**: List the sub-folders of a folder (root of the drive by default)
- **list_resources**: List the content of a folder (publications, playlists, components...) with pagination
- **create_folder**: Create a new folder in the drive (root by default)
- **rename_resource** / **move_resources** / **trash_resources** / **duplicate_resource**: Organise the drive
- **create_publication_from_file**: Upload a local ePub/PDF as a new publication and wait until it is LIVE
- **get_publication_progress**: Poll the generation status of a publication
- **create_playlist** + **include_ext_pages**: Create an empty playlist and add pages of another publication into it
- **upload_component**: Upload a zip as a COMPONENT served at `DRIVE_URL/{clientId}/{componentGId}/`
- **upload_wishlist_products** / **upload_wishlist_images**: Attach the products Excel and the images zip to a wishlist publication
- **get_template_txt_file** / **save_template_txt_file**: Read/overwrite a text file of a publication's templates folder (e.g. `common-ui.xml`)
- **set_custom_admin_url**: Insert/replace `<custom_admin url="..."/>` in `common-ui.xml` > `<configs>` (configurator URL shown in the manager)
- Every tool accepts an optional `client_id` to work on another customer than the configured `CLIENT_ID`
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

The binary loads `.env` from the project directory first, then falls back to the current directory,
so an MCP client can launch it from any working directory without picking up another project's `.env`.

2. Build release:
```bash
cargo build --release
```

The resulting binary executable can be found at `/path/to/mcp-webpublication-server/target/release/mcp-webpublication-server`

## Usage

### Testing with MCP Inspector
Run at the project's root

```bash
npx @modelcontextprotocol/inspector cargo run
# or
npx @modelcontextprotocol/inspector ./target/release/mcp-webpublication-server
```

Open `http://127.0.0.1:6274` and test tools.


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

### list_folders
- **Input**: `parent_gid` (number, optional) - globalId of the parent folder. Omit it for the root of the drive (resolved via `getCustomerContext -> driveHierarchy.rootGlobalId`)
- **Output**: List of sub-folders with their globalId and label
- **API**: `GET workspaceManagerWs/getDrives`

### create_folder
- **Input**:
  - `name` (string) - label of the new folder
  - `parent_gid` (number, optional) - globalId of the parent folder, omit it for the root of the drive
- **Output**: The created folder (`globalId`, `parentId`) and the updated drive tree (`driveDto.drives`)
- **API**: `POST workspaceManagerWs/createDrive` (multipart form: clientId, parentGId, driveLabel, image, imageFilename)

### list_resources
- **Input**: `folder_gid` (number, optional, root by default), `page_num` (0-based, default 0), `items_per_page` (default 50)
- **Output**: `paginator.list` of resources (`globalId`, `label`, `type`: PUBLICATION, COMPONENT, DIRECTORY...) and `paginator.totalPage`
- **API**: `GET workspaceManagerWs/getPaginatedResources`

### rename_resource
- **Input**: `resource_gid` (number), `label` (string)
- **API**: `POST workspaceManagerWs/updateResourceName` (urlencoded: clientId, resourceGId, resourceLabel)

### move_resources
- **Input**: `resource_gids` (number[]), `new_parent_gid` (number)
- **API**: `POST workspaceManagerWs/moveResources` (urlencoded: clientId, resourcesGIds (repeated), newParentGId)

### duplicate_resource
- **Input**: `resource_gid`, `label` (optional, renames the copy), `new_parent_gid` (optional, moves the copy)
- **Output**: `copyGId` and the cloned resource. The copy keeps the original's settings (wishlist, `common-ui.xml`...)
- **API**: `POST workspaceManagerWs/cloneResource?clientId&globalId` (no body), then `updateResourceName` / `moveResources`

### trash_resources
- **Input**: `resource_gids` (number[])
- **Note**: Moves resources to the trash (reversible from the manager). No permanent delete is exposed.
- **API**: `POST workspaceManagerWs/trashResources`

### create_publication_from_file
- **Input**: `file_path` (absolute local path: .epub, .pdf, .pptx...), `parent_gid` (folder), `label` (optional, renames after creation), `wait` (default true)
- **Output**: `publicationGId`, the created resource, the final `progress` (when waited) and the rename response
- **API**: `POST generationWs/createPublication` (multipart: file, filename, parentDriveGId, clientId, creationSource=DESKTOP, tmpId), then `GET generationWs/getPublicationProgress` every 2s until `status` is `LIVE` or `ERROR` (15 min timeout)

### get_publication_progress
- **Input**: `publication_gid` (number)
- **Output**: `status` (UPLOADING, CREATED, PROCESSING, COMPLETED, LIVE, ERROR), `percentage`, `currentPage`, `totalpages`

### create_playlist
- **Input**: `folder_gid` (number), `label` (optional)
- **Output**: `playlistGId` and the created EPUB_PLAYLIST resource
- **API**: `POST generationWs/createNewPlaylist` (urlencoded: clientId, folderGId)

### include_ext_pages
- **Input**: `publication_gid` (target), `src_publication_gid` (source), `pages` (1-based numbers), `process_printables` (default true)
- **Output**: the target's pages (`ComboPageXml` objects)
- **API**: `POST generationWs/includeExtPages`

### upload_component
- **Input**: `file_path` (absolute local .zip), `folder_gid`, `label` (optional)
- **Output**: `componentGId`, `url` (`DRIVE_URL/{clientId}/{componentGId}/`) and the created COMPONENT resource
- **API**: `POST pageManagerWs/createPage` (multipart: file, filename, clientId, folderGId)

### upload_wishlist_products / upload_wishlist_images
- **Input**: `publication_gid`, `file_path` (.xlsx for products; .zip of images or a single image for images)
- **Output**: the parsed products (products) or the uploaded items (images)
- **API**: `POST generationWs/uploadWishlistProducts` / `uploadWishlistImages` (multipart: file, filename, publicationGId, clientId)
- **Note**: the products template is available at `DRIVE_URL/wishlist-products.xlsx`

### get_template_txt_file / save_template_txt_file
- **Input**: `publication_gid`, `rel_path` (relative to the publication's templates folder, e.g. `common-ui.xml`), plus `content` (full file) for save
- **API**: `GET customizationWs/getTemplateTxtFile?clientId&globalId&relPath` / `POST customizationWs/saveTemplateTxtFile` (urlencoded: clientId, globalId, relPath, content; returns 204)

### set_custom_admin_url
- **Input**: `publication_gid`, `url` (typically `upload_component -> url`)
- **Output**: `customAdminUrl` as re-read from `getPublicationSettings` (updated immediately)
- **Note**: idempotent; replaces an existing `<custom_admin>` node or inserts one before `</configs>`

## Resources

- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)


[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
