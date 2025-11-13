use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetResourceRequest {
    pub resource_gid: i64,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ToggleWishlistRequest {
    pub publication_gid: i64,
    pub wishlist_enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetImageRequest {
    pub rel_url: String,
}
