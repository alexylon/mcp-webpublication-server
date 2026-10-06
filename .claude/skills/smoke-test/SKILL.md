---
name: smoke-test
description: Build the release binary and drive the MCP server through the MCP Inspector CLI. Lists tools and their input schemas, then calls read-only tools against the live account. Use to verify that a change to the server works, or when asked to run, launch or try the server or a tool.
---

# Smoke-test the server

Run everything from the project root. The binary loads the project `.env` by itself, so no env flags are needed.

1. Build what MCP clients run:
   ```bash
   cargo build --release
   ```

2. List tools and check the new or changed tool's `name`, `description` and `inputSchema`. `required` should list only the mandatory fields; `client_id` is never required.
   ```bash
   npx -y @modelcontextprotocol/inspector@2 --cli ./target/release/mcp-webpublication-server --method tools/list \
     | jq '.tools[] | select(.name == "TOOL_NAME")'
   ```
   `tools/list` makes no upstream calls, so it works even with an expired token.

3. Call **read-only tools only**: `get_recent_resources`, `get_resource`, `get_publication_settings`, `get_cover_image`, `get_publication_progress`, `get_template_txt_file`, `list_folders`, `list_resources`.
   ```bash
   npx -y @modelcontextprotocol/inspector@2 --cli ./target/release/mcp-webpublication-server \
     --method tools/call --tool-name list_resources --tool-arg items_per_page=5
   ```
   - Pass one `--tool-arg key=value` per argument. Numbers, booleans and JSON arrays (`resource_gids=[1,2]`) are sent as JSON values. A value that doesn't fit the schema type arrives as `null` (`invalid type: null, expected i64`).
   - Take gids from an earlier read call, e.g. `get_recent_resources` or `list_folders`.
   - On success, stdout is a `CallToolResult`; its `content[0].text` is the pretty-printed upstream JSON.
   - On failure, the CLI exits 1 and prints `{"error": …}` to stderr. Server logs also go to stderr.
   - A 401 `SESSION_EXPIRED` means `WP_TOKEN` in `.env` has expired. Stop and ask the user for a fresh token.

4. Any other tool (create, upload, save, set, toggle, rename, move, trash, duplicate, include) changes the live account. Before calling one, ask the user, giving the exact arguments.

5. Report what was checked. Remind the user to reconnect the server in their MCP client (e.g. `/mcp` in Claude Code, or restart Claude Desktop) to load the new binary.
