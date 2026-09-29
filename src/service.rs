#![allow(unused)]

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router, ErrorData as McpError,
};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::models::{
    ApiResponse, CreateFolderRequest, CreatePlaylistRequest, CreatePublicationFromFileRequest,
    GetImageRequest, GetRecentResourcesRequest, GetResourceRequest, IncludeExtPagesRequest,
    ListFoldersRequest, ListResourcesRequest, MoveResourcesRequest, PublicationRequest,
    RenameResourceRequest, ToggleWishlistRequest, TrashResourcesRequest, UploadComponentRequest,
    UploadWishlistFileRequest,
};

/// Delay between two polls of getPublicationProgress.
const PROGRESS_POLL_INTERVAL: Duration = Duration::from_secs(2);
/// Maximum time to wait for a publication generation before giving up.
const PROGRESS_TIMEOUT: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub api_url: String,
    pub drive_url: String,
    pub client_id: String,
    pub wp_token: String,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self> {
        // Load .env from the current directory, then fall back to the project directory
        // (useful when the binary is launched by an MCP client from another cwd).
        dotenv::dotenv().ok();
        dotenv::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env")).ok();

        let api_url = std::env::var("API_URL")
            .map_err(|_| anyhow::anyhow!("API_URL not found in environment"))?;
        let drive_url = std::env::var("DRIVE_URL")
            .map_err(|_| anyhow::anyhow!("DRIVE_URL not found in environment"))?;
        let client_id = std::env::var("CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("CLIENT_ID not found in environment"))?;
        let wp_token = std::env::var("WP_TOKEN")
            .map_err(|_| anyhow::anyhow!("WP_TOKEN not found in environment"))?;

        Ok(Self {
            api_url,
            drive_url,
            client_id,
            wp_token,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ApiEndpoint {
    LoginWs,
    WorkspaceManagerWs,
    GenerationWs,
    CustomizationWs,
    EnrichmentWs,
    MembershipWs,
    LicenceWs,
    GalleryManagerWs,
    PageManagerWs,
    DriveSecurityWs,
    ImageWs,
}

impl ApiEndpoint {
    pub fn path(&self) -> &str {
        match self {
            ApiEndpoint::LoginWs => "loginWs",
            ApiEndpoint::WorkspaceManagerWs => "workspaceManagerWs",
            ApiEndpoint::GenerationWs => "generationWs",
            ApiEndpoint::CustomizationWs => "customizationWs",
            ApiEndpoint::EnrichmentWs => "enrichmentWs",
            ApiEndpoint::MembershipWs => "membershipWs",
            ApiEndpoint::LicenceWs => "licenceWs",
            ApiEndpoint::GalleryManagerWs => "galleryManagerWs",
            ApiEndpoint::PageManagerWs => "pageManagerWs",
            ApiEndpoint::DriveSecurityWs => "driveSecurityWs",
            ApiEndpoint::ImageWs => "imageWs",
        }
    }
}

/// A local file to attach to a multipart request.
struct FilePart {
    /// Name of the multipart field carrying the file bytes (usually "file").
    field: &'static str,
    /// Absolute path of the local file.
    path: String,
}

#[derive(Clone)]
pub struct WebPublication {
    client: Arc<Client>,
    config: ApiConfig,
    tool_router: ToolRouter<Self>,
}

impl WebPublication {
    pub fn new() -> Result<Self> {
        let config = ApiConfig::from_env()?;
        let client = Client::builder().cookie_store(true).build()?;

        Ok(Self {
            client: Arc::new(client),
            config,
            tool_router: Self::tool_router(),
        })
    }

    /// Resolves the client id to use: the per-call override, or the configured default.
    fn client_id(&self, override_id: Option<i64>) -> String {
        match override_id {
            Some(id) => id.to_string(),
            None => self.config.client_id.clone(),
        }
    }

    fn cookie_header(&self) -> String {
        format!("WP_token={}", self.config.wp_token)
    }

    fn format_json(value: &serde_json::Value) -> Result<String, McpError> {
        serde_json::to_string_pretty(value).map_err(|e| {
            McpError::internal_error(format!("Failed to format response: {}", e), None)
        })
    }

    fn parse_body(status: reqwest::StatusCode, text: String) -> Result<ApiResponse, McpError> {
        if !status.is_success() {
            return Err(McpError::internal_error(
                format!("Request failed with status: {} - {}", status, text),
                None,
            ));
        }

        if text.trim().is_empty() {
            return Ok(ApiResponse {
                data: serde_json::json!({ "status": status.as_u16() }),
            });
        }

        serde_json::from_str::<ApiResponse>(&text).map_err(|e| {
            McpError::internal_error(
                format!("Failed to parse response: {} - body: {}", e, text),
                None,
            )
        })
    }

    async fn make_get_request(
        &self,
        endpoint: ApiEndpoint,
        method: &str,
        params: &[(&str, &str)],
    ) -> Result<ApiResponse, McpError> {
        let url = format!("{}{}/{}", self.config.api_url, endpoint.path(), method);

        tracing::info!("Making request to: {}", url);

        let mut request = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("Cookie", self.cookie_header());

        for (key, value) in params {
            request = request.query(&[(key, value)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Request failed: {}", e), None))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Self::parse_body(status, text)
    }

    async fn make_put_request(
        &self,
        endpoint: ApiEndpoint,
        method: &str,
        params: &[(&str, &str)],
        body: serde_json::Value,
    ) -> Result<ApiResponse, McpError> {
        let url = format!("{}{}/{}", self.config.api_url, endpoint.path(), method);

        tracing::info!("Making PUT request to: {}", url);

        let mut request = self
            .client
            .put(&url)
            .header("Content-Type", "application/json")
            .header("Cookie", self.cookie_header())
            .json(&body);

        for (key, value) in params {
            request = request.query(&[(key, value)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Request failed: {}", e), None))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Self::parse_body(status, text)
    }

    /// POST with an application/x-www-form-urlencoded body. Repeated keys are allowed
    /// (used for `pages` / `resourcesGIds` list parameters).
    async fn make_post_urlencoded_request(
        &self,
        endpoint: ApiEndpoint,
        method: &str,
        fields: &[(&str, String)],
    ) -> Result<ApiResponse, McpError> {
        let url = format!("{}{}/{}", self.config.api_url, endpoint.path(), method);

        tracing::info!("Making POST (urlencoded) request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Cookie", self.cookie_header())
            .form(fields)
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Request failed: {}", e), None))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Self::parse_body(status, text)
    }

    /// POST with a multipart/form-data body, optionally attaching a local file.
    async fn make_post_form_request(
        &self,
        endpoint: ApiEndpoint,
        method: &str,
        fields: Vec<(&str, String)>,
        file: Option<FilePart>,
    ) -> Result<ApiResponse, McpError> {
        let url = format!("{}{}/{}", self.config.api_url, endpoint.path(), method);

        tracing::info!("Making POST (multipart) request to: {}", url);

        let mut form = reqwest::multipart::Form::new();
        for (key, value) in fields {
            form = form.text(key.to_string(), value);
        }

        if let Some(file) = file {
            let file_name = Self::file_name(&file.path)?;
            let bytes = tokio::fs::read(&file.path).await.map_err(|e| {
                McpError::invalid_params(format!("Cannot read file '{}': {}", file.path, e), None)
            })?;
            let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
            form = form.part(file.field, part);
        }

        let response = self
            .client
            .post(&url)
            .header("Cookie", self.cookie_header())
            .multipart(form)
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Request failed: {}", e), None))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Self::parse_body(status, text)
    }

    /// Returns the file name (with extension) of a local path.
    fn file_name(path: &str) -> Result<String, McpError> {
        Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string())
            .ok_or_else(|| {
                McpError::invalid_params(format!("Invalid file path: '{}'", path), None)
            })
    }

    /// Returns the globalId of the root drive of a client.
    async fn root_drive_gid(&self, client_id: &str) -> Result<i64, McpError> {
        let params = [("clientId", client_id)];
        let response = self
            .make_get_request(ApiEndpoint::WorkspaceManagerWs, "getCustomerContext", &params)
            .await?;

        response.data["driveHierarchy"]["rootGlobalId"]
            .as_i64()
            .ok_or_else(|| {
                McpError::internal_error("rootGlobalId not found in getCustomerContext response", None)
            })
    }

    /// Renames a resource (publication, folder, component...).
    async fn rename(&self, client_id: &str, resource_gid: i64, label: &str) -> Result<ApiResponse, McpError> {
        let fields = [
            ("clientId", client_id.to_string()),
            ("resourceGId", resource_gid.to_string()),
            ("resourceLabel", label.to_string()),
        ];
        self.make_post_urlencoded_request(ApiEndpoint::WorkspaceManagerWs, "updateResourceName", &fields)
            .await
    }

    /// Extracts the globalId of a created resource from an API response.
    fn extract_global_id(data: &serde_json::Value) -> Option<i64> {
        data["globalId"]
            .as_i64()
            .or_else(|| data["resourceGId"].as_i64())
    }

    async fn publication_progress(&self, client_id: &str, publication_gid: i64) -> Result<ApiResponse, McpError> {
        let gid = publication_gid.to_string();
        let params = [("clientId", client_id), ("publicationGId", gid.as_str())];
        self.make_get_request(ApiEndpoint::GenerationWs, "getPublicationProgress", &params)
            .await
    }

    /// Polls getPublicationProgress until the publication is LIVE or in ERROR.
    async fn wait_for_publication(&self, client_id: &str, publication_gid: i64) -> Result<serde_json::Value, McpError> {
        let start = std::time::Instant::now();
        loop {
            let progress = self.publication_progress(client_id, publication_gid).await?;
            let status = progress.data["status"].as_str().unwrap_or("").to_string();
            tracing::info!(
                "Publication {} progress: {} ({}%)",
                publication_gid,
                status,
                progress.data["percentage"]
            );
            match status.as_str() {
                "LIVE" => return Ok(progress.data),
                "ERROR" => {
                    return Err(McpError::internal_error(
                        format!(
                            "Publication generation failed: {}",
                            Self::format_json(&progress.data)?
                        ),
                        None,
                    ))
                }
                _ => {}
            }
            if start.elapsed() > PROGRESS_TIMEOUT {
                return Err(McpError::internal_error(
                    format!(
                        "Timed out waiting for publication {} (last status: {}). Use get_publication_progress to keep polling.",
                        publication_gid, status
                    ),
                    None,
                ));
            }
            tokio::time::sleep(PROGRESS_POLL_INTERVAL).await;
        }
    }

    async fn make_get_file_request(
        &self,
        client_id: &str,
        rel_url: &str,
        params: &[(&str, &str)],
    ) -> Result<Vec<u8>, McpError> {
        let url = format!("{}{}/{}", self.config.drive_url, client_id, rel_url);

        tracing::info!("Making request to: {}", &self.config.drive_url);

        let mut request = self.client.get(&url);

        for (key, value) in params {
            request = request.query(&[(key, value)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("Request failed: {}", e), None))?;

        if !response.status().is_success() {
            return Err(McpError::internal_error(
                format!("Request failed with status: {}", response.status()),
                None,
            ));
        }

        let bytes = response.bytes().await.map_err(|e| {
            McpError::internal_error(format!("Failed to read response bytes: {}", e), None)
        })?;

        Ok(bytes.to_vec())
    }

    fn text_result(data: &serde_json::Value) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(Self::format_json(data)?)]))
    }
}

#[tool_handler]
impl ServerHandler for WebPublication {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "mcp-webpublication-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some(
                "A Webpublication API service that provides access to various workspace management, \
                generation, customization, and other Webpublication platform features.\n\n\
                **IMPORTANT WORKFLOW**:\n\
                - Every tool accepts an optional client_id (customer id). Omit it to use the default client \
                configured in the environment; pass it when working on another customer account.\n\
                - If resourceGId parameter is not provided for get_resource, OR if publicationGId parameter \
                is not provided for get_publication_settings, you MUST first call get_recent_resources to \
                retrieve the globalId of the desired publication.\n\
                - When the user provides a publication name, it corresponds to the 'label' field in the \
                get_recent_resources response. Match the user-provided name to the label field.\n\
                - Use the globalId from get_recent_resources as the resource_gid parameter for both \
                get_resource and get_publication_settings tools. \
                When a publication is found by name/label, always mention its globalId in your first sentence. \
                The cover image of a publication is retrieved by get_cover_image and the parameter is retrieved by get_publication_settings as coverImage.relUrl \
                The returned month value is zero-based. Add 1 to it to get the calendar month. For example, 'month': 5 represents June (5 + 1 = 6).\n\
                - Drive navigation: list_folders lists sub-folders, list_resources lists the content of a folder (publications, playlists, components...), \
                create_folder / rename_resource / move_resources organise it.\n\
                - Catalogue configurator setup: create_publication_from_file uploads a local ePub/PDF as a new publication and waits for it to be LIVE; \
                create_playlist creates an empty playlist; include_ext_pages adds pages of a source publication into it; \
                toggle_wishlist enables the wishlist; upload_component uploads a zip as a COMPONENT resource served at DRIVE_URL/{clientId}/{componentGlobalId}/; \
                upload_wishlist_products / upload_wishlist_images attach the products Excel and the product images zip to a wishlist publication."
                    .to_string(),
            ),
        }
    }
}

#[tool_router]
impl WebPublication {
    #[tool(
        description = "Get the 20 most recent publications from the Webpublication API. \
    Use their globalId as the resource_gid or publicationGId parameter for get_resource or get_publication_settings to get more info about the publication. \
    The name of the publication is its label.\
    When a publication is found by name/label, always mention its globalId in your first sentence."
    )]
    async fn get_recent_resources(
        &self,
        Parameters(request): Parameters<GetRecentResourcesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let params = [
            ("clientId", client_id.as_str()),
            ("include", "PUBLICATION"),
            ("itemsPerPage", "20"),
            ("pageNum", "0"),
        ];

        let response = self
            .make_get_request(ApiEndpoint::WorkspaceManagerWs, "getRecentResources", &params)
            .await?;

        Self::text_result(&response.data)
    }

    #[tool(
        description = "Get a resource/publication from the Webpublication API. \
    Provide the globalId from get_recent_resources, if not supplied by the user, as the resource_gid parameter (e.g., 2473843) \
    to fetch detailed resource information.\
    The returned month value is zero-based. Add 1 to it to get the calendar month. For example, 'month': 5 represents June (5 + 1 = 6)."
    )]
    async fn get_resource(
        &self,
        Parameters(request): Parameters<GetResourceRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Getting resource with GID: {}", request.resource_gid);

        let client_id = self.client_id(request.client_id);
        let resource_gid_str = request.resource_gid.to_string();
        let params = [
            ("clientId", client_id.as_str()),
            ("resourceGId", resource_gid_str.as_str()),
        ];

        let response = self
            .make_get_request(ApiEndpoint::WorkspaceManagerWs, "getResource", &params)
            .await?;

        Self::text_result(&response.data)
    }

    #[tool(
        description = "Get the publication settings from the Webpublication API. \
    Provide the globalId from get_recent_resources, if not supplied by the user, \
    as the resource_gid parameter (e.g., 2473843) to fetch detailed resource settings"
    )]
    async fn get_publication_settings(
        &self,
        Parameters(request): Parameters<GetResourceRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Getting publication settings with GID: {}", request.resource_gid);

        let client_id = self.client_id(request.client_id);
        let resource_gid_str = request.resource_gid.to_string();
        let params = [
            ("clientId", client_id.as_str()),
            ("publicationGId", resource_gid_str.as_str()),
        ];

        let response = self
            .make_get_request(ApiEndpoint::GenerationWs, "getPublicationSettings", &params)
            .await?;

        Self::text_result(&response.data)
    }

    #[tool(
        description = "Toggle wishlist status for a publication. \
    Provide the globalId from get_recent_resources, if not supplied by the user, \
    as the publication_gid parameter (e.g., 2473843), and specify whether to enable or disable \
    the wishlist using wishlist_enabled (true/false). The current wishlist status can be obtained \
    from get_publication_settings -> wishlistEnabled."
    )]
    async fn toggle_wishlist(
        &self,
        Parameters(request): Parameters<ToggleWishlistRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!(
            "Toggling wishlist for publication GID: {}, wishlist_enabled: {}",
            request.publication_gid,
            request.wishlist_enabled
        );

        let client_id = self.client_id(request.client_id);
        let params = [("clientId", client_id.as_str())];

        let body = serde_json::json!({
            "clientId": client_id,
            "globalId": request.publication_gid,
            "wishlistEnabled": request.wishlist_enabled
        });

        let response = self
            .make_put_request(ApiEndpoint::GenerationWs, "updatePublicationSettings", &params, body)
            .await?;

        Self::text_result(&response.data)
    }

    #[tool(
        description = "Get the cover image of the publication. \
    Provide the relUrl as a parameter from get_publication_settings in the response field coverImage.relUrl"
    )]
    async fn get_cover_image(
        &self,
        Parameters(request): Parameters<GetImageRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Getting image with relUrl: {}", request.rel_url);

        let client_id = self.client_id(request.client_id);
        let refresh_response = self
            .make_get_request(ApiEndpoint::LoginWs, "refresh", &[])
            .await?;
        let token = refresh_response.data["token"]
            .as_str()
            .ok_or_else(|| McpError::internal_error("Token not found in refresh response", None))?;

        let params = [("token", token)];

        let image_bytes = self
            .make_get_file_request(&client_id, &request.rel_url, &params)
            .await?;

        // Decide MIME type by file extension (lowercased)
        let mime_type = match request.rel_url.to_lowercase().as_str() {
            p if p.ends_with(".png") => "image/png",
            p if p.ends_with(".jpg") || p.ends_with(".jpeg") => "image/jpeg",
            p if p.ends_with(".gif") => "image/gif",
            p if p.ends_with(".webp") => "image/webp",
            _ => "image/jpeg",
        };

        let base64_image = general_purpose::STANDARD.encode(&image_bytes);

        Ok(CallToolResult::success(vec![Content::image(
            base64_image,
            mime_type.to_string(),
        )]))
    }

    #[tool(
        description = "List the sub-folders (drives) of a folder in the Webpublication drive. \
    Omit parent_gid to list the folders at the root of the drive. \
    Each folder has a globalId and a label (name)."
    )]
    async fn list_folders(
        &self,
        Parameters(request): Parameters<ListFoldersRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let parent_gid = match request.parent_gid {
            Some(gid) => gid,
            None => self.root_drive_gid(&client_id).await?,
        };
        tracing::info!("Listing folders under parent GID: {}", parent_gid);

        let parent_gid_str = parent_gid.to_string();
        let params = [
            ("clientId", client_id.as_str()),
            ("parentGId", parent_gid_str.as_str()),
        ];

        let response = self
            .make_get_request(ApiEndpoint::WorkspaceManagerWs, "getDrives", &params)
            .await?;

        Self::text_result(&response.data)
    }

    #[tool(
        description = "List the resources (publications, playlists, components, files, sub-folders...) contained in a folder, \
    with pagination (paginator.totalPage tells how many pages exist). Omit folder_gid for the root of the drive. \
    Each resource has a globalId, a label and a type (PUBLICATION, EPUB_PLAYLIST, COMPONENT, DRIVE...)."
    )]
    async fn list_resources(
        &self,
        Parameters(request): Parameters<ListResourcesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let folder_gid = match request.folder_gid {
            Some(gid) => gid,
            None => self.root_drive_gid(&client_id).await?,
        };
        let folder_gid_str = folder_gid.to_string();
        let page_num = request.page_num.unwrap_or(0).to_string();
        let items_per_page = request.items_per_page.unwrap_or(50).to_string();
        let params = [
            ("clientId", client_id.as_str()),
            ("driveGId", folder_gid_str.as_str()),
            ("pageNum", page_num.as_str()),
            ("itemsPerPage", items_per_page.as_str()),
            ("sortOn", "POSITION"),
            ("ascending", "true"),
        ];

        let response = self
            .make_get_request(ApiEndpoint::WorkspaceManagerWs, "getPaginatedResources", &params)
            .await?;

        Self::text_result(&response.data)
    }

    #[tool(
        description = "Create a new folder (drive) in the Webpublication drive. \
    Provide the folder name and optionally the parent_gid (globalId of the parent folder, obtained from list_folders). \
    Omit parent_gid to create the folder at the root of the drive. \
    Returns the updated list of drives including the new folder."
    )]
    async fn create_folder(
        &self,
        Parameters(request): Parameters<CreateFolderRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let parent_gid = match request.parent_gid {
            Some(gid) => gid,
            None => self.root_drive_gid(&client_id).await?,
        };
        tracing::info!("Creating folder '{}' under parent GID: {}", request.name, parent_gid);

        let fields = vec![
            ("clientId", client_id.clone()),
            ("parentGId", parent_gid.to_string()),
            ("driveLabel", request.name.clone()),
            ("image", String::new()),
            ("imageFilename", String::new()),
        ];

        let response = self
            .make_post_form_request(ApiEndpoint::WorkspaceManagerWs, "createDrive", fields, None)
            .await?;

        Self::text_result(&response.data)
    }

    #[tool(
        description = "Rename a resource (publication, playlist, folder, component...). \
    Provide the resource_gid (globalId) and the new label."
    )]
    async fn rename_resource(
        &self,
        Parameters(request): Parameters<RenameResourceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        tracing::info!("Renaming resource {} to '{}'", request.resource_gid, request.label);
        let response = self.rename(&client_id, request.resource_gid, &request.label).await?;
        Self::text_result(&response.data)
    }

    #[tool(
        description = "Move one or several resources (publications, folders, components...) into another folder. \
    Provide the resource_gids (globalIds) and the new_parent_gid (globalId of the destination folder)."
    )]
    async fn move_resources(
        &self,
        Parameters(request): Parameters<MoveResourcesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        tracing::info!("Moving resources {:?} to folder {}", request.resource_gids, request.new_parent_gid);

        let mut fields: Vec<(&str, String)> = vec![("clientId", client_id.clone())];
        for gid in &request.resource_gids {
            fields.push(("resourcesGIds", gid.to_string()));
        }
        fields.push(("newParentGId", request.new_parent_gid.to_string()));

        let response = self
            .make_post_urlencoded_request(ApiEndpoint::WorkspaceManagerWs, "moveResources", &fields)
            .await?;
        Self::text_result(&response.data)
    }


    #[tool(
        description = "Move one or several resources (publications, folders, components...) to the trash. \
    This is reversible from the manager's trash (unTrashResources), unlike a permanent delete. \
    Provide the resource_gids (globalIds)."
    )]
    async fn trash_resources(
        &self,
        Parameters(request): Parameters<TrashResourcesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        tracing::info!("Trashing resources {:?}", request.resource_gids);

        let mut fields: Vec<(&str, String)> = vec![("clientId", client_id.clone())];
        for gid in &request.resource_gids {
            fields.push(("resourcesGIds", gid.to_string()));
        }

        let response = self
            .make_post_urlencoded_request(ApiEndpoint::WorkspaceManagerWs, "trashResources", &fields)
            .await?;
        Self::text_result(&response.data)
    }

    #[tool(
        description = "Create a new publication from a local file (.epub exported from InDesign, .pdf, .pptx, .docx...). \
    Provide the absolute file_path and the parent_gid (globalId of the destination folder). \
    By default the tool waits until the publication is LIVE (polling every 2s, up to 15 minutes) and returns the final progress; \
    set wait=false to return right after the upload and poll with get_publication_progress yourself. \
    Provide label to rename the publication after creation (default: the file name). \
    Returns the created resource (its globalId is the publication gid) and, when waited, the final progress."
    )]
    async fn create_publication_from_file(
        &self,
        Parameters(request): Parameters<CreatePublicationFromFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let file_name = Self::file_name(&request.file_path)?;
        tracing::info!(
            "Creating publication from '{}' in folder {}",
            request.file_path,
            request.parent_gid
        );

        let fields = vec![
            ("parentDriveGId", request.parent_gid.to_string()),
            ("clientId", client_id.clone()),
            ("filename", file_name),
            ("creationSource", "DESKTOP".to_string()),
            ("tmpId", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis().to_string())
                .unwrap_or_default()),
        ];
        let file = FilePart { field: "file", path: request.file_path.clone() };

        let created = self
            .make_post_form_request(ApiEndpoint::GenerationWs, "createPublication", fields, Some(file))
            .await?;

        let publication_gid = Self::extract_global_id(&created.data).ok_or_else(|| {
            McpError::internal_error(
                format!("globalId not found in createPublication response: {}", created.data),
                None,
            )
        })?;

        let mut result = serde_json::json!({
            "publicationGId": publication_gid,
            "resource": created.data,
        });

        if request.wait.unwrap_or(true) {
            let progress = self.wait_for_publication(&client_id, publication_gid).await?;
            result["progress"] = progress;
        }

        if let Some(label) = &request.label {
            let renamed = self.rename(&client_id, publication_gid, label).await?;
            result["renamed"] = serde_json::json!({ "label": label, "response": renamed.data });
        }

        Self::text_result(&result)
    }

    #[tool(
        description = "Get the generation progress of a publication (status UPLOADING/CREATED/PROCESSING/COMPLETED/LIVE/ERROR and percentage). \
    Use it after create_publication_from_file with wait=false, or when a wait timed out."
    )]
    async fn get_publication_progress(
        &self,
        Parameters(request): Parameters<PublicationRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let response = self.publication_progress(&client_id, request.publication_gid).await?;
        Self::text_result(&response.data)
    }

    #[tool(
        description = "Create a new empty playlist (EPUB_PLAYLIST) in a folder. \
    Provide the folder_gid (globalId of the destination folder) and optionally a label. \
    Returns the globalId of the playlist; then add pages with include_ext_pages."
    )]
    async fn create_playlist(
        &self,
        Parameters(request): Parameters<CreatePlaylistRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        tracing::info!("Creating playlist in folder {}", request.folder_gid);

        let fields = [
            ("clientId", client_id.clone()),
            ("folderGId", request.folder_gid.to_string()),
        ];
        let created = self
            .make_post_urlencoded_request(ApiEndpoint::GenerationWs, "createNewPlaylist", &fields)
            .await?;

        let playlist_gid = Self::extract_global_id(&created.data).ok_or_else(|| {
            McpError::internal_error(
                format!("globalId not found in createNewPlaylist response: {}", created.data),
                None,
            )
        })?;

        let mut result = serde_json::json!({
            "playlistGId": playlist_gid,
            "resource": created.data,
        });

        if let Some(label) = &request.label {
            let renamed = self.rename(&client_id, playlist_gid, label).await?;
            result["renamed"] = serde_json::json!({ "label": label, "response": renamed.data });
        }

        Self::text_result(&result)
    }

    #[tool(
        description = "Include pages of a source publication into a target publication or playlist, as references \
    (the pages stay managed in the source publication). Pages are appended at the end of the target. \
    Provide publication_gid (target), src_publication_gid (source) and pages (1-based page numbers)."
    )]
    async fn include_ext_pages(
        &self,
        Parameters(request): Parameters<IncludeExtPagesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        tracing::info!(
            "Including pages {:?} of {} into {}",
            request.pages,
            request.src_publication_gid,
            request.publication_gid
        );

        let mut fields: Vec<(&str, String)> = vec![
            ("clientId", client_id.clone()),
            ("publiGId", request.publication_gid.to_string()),
            ("srcPubliGId", request.src_publication_gid.to_string()),
        ];
        for page in &request.pages {
            fields.push(("pages", page.to_string()));
        }
        fields.push((
            "processPrintables",
            request.process_printables.unwrap_or(true).to_string(),
        ));

        let response = self
            .make_post_urlencoded_request(ApiEndpoint::GenerationWs, "includeExtPages", &fields)
            .await?;
        Self::text_result(&response.data)
    }

    #[tool(
        description = "Upload a local .zip (static web app, e.g. the built catalogue configurator) as a COMPONENT resource in a folder. \
    The component is then served at DRIVE_URL/{clientId}/{componentGlobalId}/. \
    Provide the absolute file_path, the folder_gid (destination folder) and optionally a label."
    )]
    async fn upload_component(
        &self,
        Parameters(request): Parameters<UploadComponentRequest>,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let file_name = Self::file_name(&request.file_path)?;
        tracing::info!("Uploading component '{}' to folder {}", request.file_path, request.folder_gid);

        let fields = vec![
            ("clientId", client_id.clone()),
            ("folderGId", request.folder_gid.to_string()),
            ("filename", file_name),
        ];
        let file = FilePart { field: "file", path: request.file_path.clone() };

        let created = self
            .make_post_form_request(ApiEndpoint::PageManagerWs, "createPage", fields, Some(file))
            .await?;

        let component_gid = Self::extract_global_id(&created.data);
        let mut result = serde_json::json!({
            "componentGId": component_gid,
            "url": component_gid.map(|gid| format!("{}{}/{}/", self.config.drive_url, client_id, gid)),
            "resource": created.data,
        });

        if let (Some(label), Some(gid)) = (&request.label, component_gid) {
            let renamed = self.rename(&client_id, gid, label).await?;
            result["renamed"] = serde_json::json!({ "label": label, "response": renamed.data });
        }

        Self::text_result(&result)
    }

    #[tool(
        description = "Upload the wishlist products Excel file (.xls/.xlsx) of a publication. \
    Provide the publication_gid and the absolute file_path. The wishlist must be enabled on the publication first (toggle_wishlist)."
    )]
    async fn upload_wishlist_products(
        &self,
        Parameters(request): Parameters<UploadWishlistFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.upload_wishlist_file("uploadWishlistProducts", request).await
    }

    #[tool(
        description = "Upload the wishlist product images of a publication: a .zip of images or a single image file. \
    Provide the publication_gid and the absolute file_path. The wishlist must be enabled on the publication first (toggle_wishlist)."
    )]
    async fn upload_wishlist_images(
        &self,
        Parameters(request): Parameters<UploadWishlistFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.upload_wishlist_file("uploadWishlistImages", request).await
    }
}

impl WebPublication {
    /// Shared implementation of the generationWs/uploadWishlist* multipart uploads.
    async fn upload_wishlist_file(
        &self,
        method: &str,
        request: UploadWishlistFileRequest,
    ) -> Result<CallToolResult, McpError> {
        let client_id = self.client_id(request.client_id);
        let file_name = Self::file_name(&request.file_path)?;
        tracing::info!(
            "{} '{}' for publication {}",
            method,
            request.file_path,
            request.publication_gid
        );

        let fields = vec![
            ("clientId", client_id),
            ("publicationGId", request.publication_gid.to_string()),
            ("filename", file_name),
        ];
        let file = FilePart { field: "file", path: request.file_path.clone() };

        let response = self
            .make_post_form_request(ApiEndpoint::GenerationWs, method, fields, Some(file))
            .await?;
        Self::text_result(&response.data)
    }
}
