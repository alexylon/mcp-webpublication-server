use anyhow::Result;
use reqwest::Client;
use rmcp::{
    handler::server::{wrapper::Parameters, ServerHandler, tool::ToolRouter},
    model::{CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use std::sync::Arc;

use crate::models::{ApiResponse, GetResourceRequest};

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub client_id: String,
    pub wp_token: String,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let base_url = std::env::var("API_URL")
            .map_err(|_| anyhow::anyhow!("API_URL not found in environment"))?;
        let client_id = std::env::var("CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("CLIENT_ID not found in environment"))?;
        let wp_token = std::env::var("WP_TOKEN")
            .map_err(|_| anyhow::anyhow!("WP_TOKEN not found in environment"))?;

        Ok(Self {
            base_url,
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

#[derive(Clone)]
pub struct WebPublication {
    client: Arc<Client>,
    config: ApiConfig,
    tool_router: ToolRouter<Self>,
}

impl WebPublication {
    pub fn new() -> Result<Self> {
        let config = ApiConfig::from_env()?;
        let client = Client::builder()
            .cookie_store(true)
            .build()?;

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
        let url = format!("{}{}/{}", self.config.base_url, endpoint.path(), method);

        tracing::info!("Making request to: {}", url);

        let mut request = self.client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("Cookie", format!("WP_token={}", self.config.wp_token));

        for (key, value) in params {
            request = request.query(&[(key, value)]);
        }

        let response = request.send().await.map_err(|e| {
            McpError::internal_error(format!("Request failed: {}", e), None)
        })?;

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
                generation, customization, and other Webpublication platform features."
                    .to_string(),
            ),
        }
    }
}

#[tool_router]
impl WebPublication {
    #[tool(description = "Get a resource/publication from the Webpublication API. Provide the resource/publication GID (e.g., 2473843) to fetch resource information.")]
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

    #[tool(description = "Get the publication settings from the Webpublication API. Provide the resource/publication GID (e.g., 2473843) to fetch resource information.")]
    async fn get_publication_settings(
        &self,
        Parameters(request): Parameters<GetResourceRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Getting publication settings with GID: {}", request.resource_gid);

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
}
