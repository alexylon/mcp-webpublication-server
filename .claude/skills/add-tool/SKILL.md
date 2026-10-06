---
name: add-tool
description: Add a new MCP tool to the WebPublication server, or wrap another upstream API call. Covers the request struct, the tool handler, README and instructions updates, then a compile and tools/list check. Use when asked to add, expose or wrap a WebPublication endpoint as a tool.
---

# Add a WebPublication tool

## 1. Get the captured request first
Ask the user for the request exactly as the manager UI sends it (DevTools → Network → Copy as cURL), plus a sample response if they have one. Do not write code against guessed names. From it, note:
- the service and method (e.g. `workspaceManagerWs/cloneResource`) and the HTTP verb
- the payload format: query string, JSON, urlencoded form or multipart
- the exact field names and casing (`resourceGId`, `globalId`, `publiGId`…), and which fields are lists
- where the new resource's id sits in the response

## 2. Request struct in `src/models.rs`

```rust
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DoThingRequest {
    /// globalId of the publication.
    pub publication_gid: i64,
    /// Optional label for the result (default: ...).
    #[serde(default)]
    pub label: Option<String>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}
```

- Field names are snake_case; the upstream camelCase names only appear in the handler.
- The `///` docs become the schema descriptions the LLM reads.
- `client_id` always comes last, with exactly that doc line.

## 3. Handler in `src/service.rs`
- Add the struct to the `use crate::models::{…}` list.
- Add an `async fn` inside the `#[tool_router] impl WebPublication` block.
  - The fn name is the tool name, and registration is automatic.
  - Shared helpers that aren't tools go in a plain `impl WebPublication` block.
- Start with `let client_id = self.client_id(request.client_id);`.
- Use the helper that matches the captured request:

| Captured request | Helper |
|---|---|
| GET, query string | `make_get_request(endpoint, "method", &[("clientId", client_id.as_str()), …])` |
| PUT, JSON body (plus query params) | `make_put_request(endpoint, "method", &[("clientId", client_id.as_str())], json!({…}))` |
| POST `application/x-www-form-urlencoded` | `make_post_urlencoded_request(endpoint, "method", &[("key", value_string), …])`. For a list, repeat the key once per value. |
| POST `multipart/form-data` | `make_post_form_request(endpoint, "method", vec![…, ("filename", file_name)], Some(FilePart { field: "file", path }))`, with `file_name` from `Self::file_name(&path)?` |

- If the request doesn't fit the table (a raw-text response, or a POST with only query params), write it inline. Follow `get_template_txt` and `duplicate_resource`.
- A service missing from `ApiEndpoint` gets a new variant. Several variants are already defined but unused.
- Return `Self::text_result(&data)`.
- Errors:
  - `McpError::internal_error(msg, None)` for upstream failures.
  - `McpError::invalid_params(msg, None)` for bad input.
- Existing helpers to reuse:
  - `self.root_drive_gid(&client_id)` when the folder is omitted.
  - `Self::extract_global_id(&data)` reads a created resource's id (`globalId`, then `resourceGId`).
  - `self.apply_label(&client_id, gid, request.label.as_ref(), &mut result)` does an optional rename after creation.
- Multi-step tools:
  - Return `Err` only when the first create or clone call fails.
  - After that, record failures in the result, e.g. `result["moveError"] = json!(e.message)`, so the caller keeps the globalId of what already exists.
- Write `#[tool(description = …)]` for the LLM. Say what the tool does, where its inputs come from ("use list_folders first…") and how to check the result.

## 4. Docs
- README.md: add a bullet under `## Features`. Also add a `### tool_name` section under `## Tools` with **Input**, **Output** and **API** (the exact upstream method and fields, as captured).
- If the tool belongs to a workflow (drive navigation, catalogue configurator setup), add it to the `instructions` string in `get_info`.

## 5. Check
1. `cargo check`.
2. `/smoke-test` to confirm the tool and its input schema appear in `tools/list`.
   - Call the new tool only if it is read-only.
   - Otherwise ask the user first, giving the exact arguments.
3. Remind the user to reconnect the server in their MCP client so it loads the new binary.
