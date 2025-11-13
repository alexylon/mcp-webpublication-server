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
use std::sync::Arc;

use crate::models::{ApiResponse, GetImageRequest, GetResourceRequest, ToggleWishlistRequest};

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub api_url: String,
    pub drive_url: String,
    pub client_id: String,
    pub wp_token: String,
    pub drive_token: String,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let api_url = std::env::var("API_URL")
            .map_err(|_| anyhow::anyhow!("API_URL not found in environment"))?;
        let drive_url = std::env::var("DRIVE_URL")
            .map_err(|_| anyhow::anyhow!("DRIVE_URL not found in environment"))?;
        let client_id = std::env::var("CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("CLIENT_ID not found in environment"))?;
        let wp_token = std::env::var("WP_TOKEN")
            .map_err(|_| anyhow::anyhow!("WP_TOKEN not found in environment"))?;
        let drive_token = std::env::var("DRIVE_TOKEN")
            .map_err(|_| anyhow::anyhow!("DRIVE_TOKEN not found in environment"))?;

        Ok(Self {
            api_url,
            drive_url,
            client_id,
            wp_token,
            drive_token,
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
            .header("Cookie", format!("WP_token={}", self.config.wp_token));

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

        let data = response.json::<ApiResponse>().await.map_err(|e| {
            McpError::internal_error(format!("Failed to parse response: {}", e), None)
        })?;

        Ok(data)
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
            .header("Cookie", format!("WP_token={}", self.config.wp_token))
            .json(&body);

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

        let data = response.json::<ApiResponse>().await.map_err(|e| {
            McpError::internal_error(format!("Failed to parse response: {}", e), None)
        })?;

        Ok(data)
    }

    async fn make_get_image_request(
        &self,
        rel_url: &str,
        params: &[(&str, &str)],
    ) -> Result<Vec<u8>, McpError> {
        let url = format!("{}{}/{}", self.config.drive_url, self.config.client_id, rel_url);

        tracing::info!("Making request to: {}", &self.config.drive_url);

        let mut request = self
            .client
            .get(&url);

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
                - If resourceGId parameter is not provided for get_resource, OR if publicationGId parameter \
                is not provided for get_publication_settings, you MUST first call get_recent_resources to \
                retrieve the globalId of the desired publication.\n\
                - When the user provides a publication name, it corresponds to the 'label' field in the \
                get_recent_resources response. Match the user-provided name to the label field.\n\
                - Use the globalId from get_recent_resources as the resource_gid parameter for both \
                get_resource and get_publication_settings tools. \
                When a publication is found by name/label, always mention its globalId in your first sentence. \
                The cover image of a publication is retrieved by get_cover_image and the parameter is retrieved by get_publication_settings as coverImage.relUrl"
                    .to_string(),
            ),
        }
    }
}

#[tool_router]
impl WebPublication {
    #[tool(
        description = "Get a resource/publication from the Webpublication API. \
    Provide the globalId from get_recent_resources, if not supplied by the user, as the resource_gid parameter (e.g., 2473843) \
    to fetch detailed resource information."
    )]
    async fn get_resource(
        &self,
        Parameters(request): Parameters<GetResourceRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Getting resource with GID: {}", request.resource_gid);

        let resource_gid_str = request.resource_gid.to_string();
        let params = [
            ("clientId", self.config.client_id.as_str()),
            ("resourceGId", resource_gid_str.as_str()),
        ];

        let response = self
            .make_get_request(ApiEndpoint::WorkspaceManagerWs, "getResource", &params)
            .await?;

        let formatted = serde_json::to_string_pretty(&response.data).map_err(|e| {
            McpError::internal_error(format!("Failed to format response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
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
        tracing::info!(
            "Getting publication settings with GID: {}",
            request.resource_gid
        );

        let resource_gid_str = request.resource_gid.to_string();
        let params = [
            ("clientId", self.config.client_id.as_str()),
            ("publicationGId", resource_gid_str.as_str()),
        ];

        let response = self
            .make_get_request(ApiEndpoint::GenerationWs, "getPublicationSettings", &params)
            .await?;

        let formatted = serde_json::to_string_pretty(&response.data).map_err(|e| {
            McpError::internal_error(format!("Failed to format response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    #[tool(
        description = "Get the 20 most recent publications from the Webpublication API. \
    Use their globalId as the resource_gid or publicationGId parameter for get_resource or get_publication_settings to get more info about the publication. \
    The name of the publication is its label.\
    When a publication is found by name/label, always mention its globalId in your first sentence."
    )]
    async fn get_recent_resources(&self) -> Result<CallToolResult, McpError> {
        let params = [
            ("clientId", self.config.client_id.as_str()),
            ("include", "PUBLICATION"),
            ("itemsPerPage", "20"),
            ("pageNum", "0"),
        ];

        let response = self
            .make_get_request(
                ApiEndpoint::WorkspaceManagerWs,
                "getRecentResources",
                &params,
            )
            .await?;

        let formatted = serde_json::to_string_pretty(&response.data).map_err(|e| {
            McpError::internal_error(format!("Failed to format response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
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

        let publication_gid_str = request.publication_gid.to_string();
        let params = [("clientId", self.config.client_id.as_str())];

        let body = serde_json::json!({
            "clientId": self.config.client_id,
            "globalId": request.publication_gid,
            "wishlistEnabled": request.wishlist_enabled
        });

        let response = self
            .make_put_request(
                ApiEndpoint::GenerationWs,
                "updatePublicationSettings",
                &params,
                body,
            )
            .await?;

        let formatted = serde_json::to_string_pretty(&response.data).map_err(|e| {
            McpError::internal_error(format!("Failed to format response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    #[tool(
        description = "Get the cover image of the publication. \
    Provide the relUrl as a parameter from get_publication_settings in the response field coverImage.relUrl"
    )]
    async fn get_cover_image(
        &self,
        Parameters(request): Parameters<GetImageRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!(
            "Getting image with relUrl: {}",
            request.rel_url
        );

        let params = [
            ("token", self.config.drive_token.as_str()),
        ];

        let image_bytes = self
            .make_get_image_request(&request.rel_url, &params)
            .await?;

        // Encode image bytes as base64
        let base64_image = general_purpose::STANDARD.encode(&image_bytes);

        // Determine MIME type from file extension
        let mime_type = if request.rel_url.ends_with(".png") {
            "image/png"
        } else if request.rel_url.ends_with(".jpg") || request.rel_url.ends_with(".jpeg") {
            "image/jpeg"
        } else if request.rel_url.ends_with(".gif") {
            "image/gif"
        } else if request.rel_url.ends_with(".webp") {
            "image/webp"
        } else {
            "image/jpeg" // default to JPEG
        };

        Ok(CallToolResult::success(vec![Content::image(
            base64_image,
            mime_type.to_string(),
        )]))
    }
}
