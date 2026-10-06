# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Rust MCP server for the WebPublication API, built on the official `rmcp` 0.8 SDK (stdio transport) with `schemars` 1.x for tool schemas. There are no tests and no CI.

## Build and verify
- MCP clients run the compiled `target/release/mcp-webpublication-server`. After a change, run `cargo build --release` and tell the user to reconnect the server in their client, otherwise they keep running the old binary.
- `/smoke-test` exercises the built server through the MCP Inspector CLI.
- Code must stay `cargo fmt` clean and pass `cargo clippy -- -D warnings`. Hooks in `.claude/settings.json` format each edited `.rs` file and block finishing until clippy passes.
- If `Cargo.toml` or `Cargo.lock` changed, also run `cargo audit`.

## Live API: read-only by default
- Every tool call hits the real WebPublication account from `.env`; there is no mock or staging mode.
- When verifying, call only read-only tools (`get_*`, `list_*`). Ask before calling anything that creates, uploads, overwrites (`save_template_txt_file` replaces the whole file), moves, trashes or duplicates. Never pass a `client_id` other than the default unless told to.

## Runtime and config
- stdout is the JSON-RPC channel: never `println!`/`print!`. Log with `tracing` (goes to stderr; `RUST_LOG` overrides the `mcp_webpublication_server=info` default).
- Required env: `API_URL`, `DRIVE_URL`, `CLIENT_ID`, `WP_TOKEN` (startup fails without them). `API_URL` and `DRIVE_URL` must end with `/`: URLs are built as `{API_URL}{endpoint}/{method}`.
- `.env` is loaded from the project dir baked in at compile time (`CARGO_MANIFEST_DIR`), then from the cwd. dotenv never overrides, so the MCP client's `env` block beats the project `.env`, which beats the cwd one.
- API calls authenticate with a `Cookie: WP_token=…` header. Drive file downloads instead use a short-lived `?token=` from `loginWs/refresh` (`make_get_file_request`).
- `WP_TOKEN` expires: a 401 `{"message":"SESSION_EXPIRED"}` means the user must supply a fresh token. It is not a code bug.

## Tools
- Never guess upstream method or field names. Ask the user for the request captured in the manager UI (DevTools → Copy as cURL) and copy the names exactly. `/add-tool` has the full recipe.
- Upstream mixes payload formats; use the helper matching the captured request: `make_get_request` (query string), `make_put_request` (JSON), `make_post_urlencoded_request` (form; a list is the same key repeated), `make_post_form_request` (multipart; file in field `file` plus a `filename` text field).
- IDs are i64 globalIds, but the parameter name changes per endpoint (`resourceGId`, `publicationGId`, `globalId`, `parentGId`, `resourcesGIds`…). Upstream calls folders "drives"; an omitted folder means the root drive (`root_drive_gid`).
- Every request struct ends with the optional `client_id: Option<i64>` field and its standard doc comment, resolved with `self.client_id(request.client_id)`.
- `#[tool(description = …)]` text and `///` field docs are the schema the LLM reads: make them precise usage guidance (purpose, defaults, where a value comes from).
- Multi-step tools return `Err` only if the first create/clone call fails. Once the resource exists, record later failures in the result JSON (`renameError`, `moveError`, `waitError`) so the caller keeps the globalId.
- When adding or changing a tool, update README.md (the Features bullet and the `### tool_name` section with Input/Output/API) and, for workflow tools, the `instructions` string in `get_info`.
- No permanent delete is exposed, by design: `trash_resources` is reversible from the manager.

## Hard rules
- No `unwrap()`/`expect()` in normal code paths. Tool arguments and API responses are untrusted: validate them and fail closed. If an expected field is missing, return an `McpError` that names it instead of guessing a default.
- Never leak `WP_TOKEN` or drive tokens through logs, errors or tool results. `reqwest::Error` text includes the request URL and its query string, so call `.without_url()` on errors from requests that carry a token in the URL.
- Write `//` comments only when they say something the code doesn't, and keep them short.
- Add unit tests for pure logic you add or change (response parsing, XML edits like `upsert_custom_admin`), plus a regression test when fixing a bug there. Keep that logic out of the HTTP code so it stays testable. Tests must never call the live API.
- Prefer fixing an issue directly over only documenting it.
- Before finishing, review the change once more for bugs, edge cases and regressions.
- Use plain international English in replies, commit messages, comments, docs and error messages: short, clear, no regional idioms or needless jargon.
- Never stage or commit unless explicitly asked. A request to fix, change or review code is not a request to commit. After a finished change, suggest a one-line commit message.

## Repo
- The GitHub repo is public: never commit `.env`, tokens or customer data.
- When asked to commit, commit straight to `main`. Subjects are imperative and name the tool and upstream method, e.g. "Add duplicate_resource tool (cloneResource with optional rename and move)".
