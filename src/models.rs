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

/// Generic API response: any JSON value (object, array, ...).
#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ApiResponse {
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetImageRequest {
    pub rel_url: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListFoldersRequest {
    /// globalId of the parent folder. Omit it to list the folders at the root of the drive.
    #[serde(default)]
    pub parent_gid: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateFolderRequest {
    /// globalId of the parent folder. Omit it to create the folder at the root of the drive.
    #[serde(default)]
    pub parent_gid: Option<i64>,
    /// Name (label) of the new folder.
    pub name: String,
}
