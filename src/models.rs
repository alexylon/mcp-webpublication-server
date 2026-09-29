use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Generic API response: any JSON value (object, array, ...).
#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ApiResponse {
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetRecentResourcesRequest {
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetResourceRequest {
    pub resource_gid: i64,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ToggleWishlistRequest {
    pub publication_gid: i64,
    pub wishlist_enabled: bool,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetImageRequest {
    pub rel_url: String,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListFoldersRequest {
    /// globalId of the parent folder. Omit it to list the folders at the root of the drive.
    #[serde(default)]
    pub parent_gid: Option<i64>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateFolderRequest {
    /// globalId of the parent folder. Omit it to create the folder at the root of the drive.
    #[serde(default)]
    pub parent_gid: Option<i64>,
    /// Name (label) of the new folder.
    pub name: String,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListResourcesRequest {
    /// globalId of the folder whose content is listed. Omit it for the root of the drive.
    #[serde(default)]
    pub folder_gid: Option<i64>,
    /// Zero-based page index (default 0).
    #[serde(default)]
    pub page_num: Option<u32>,
    /// Number of items per page (default 50).
    #[serde(default)]
    pub items_per_page: Option<u32>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreatePublicationFromFileRequest {
    /// Absolute path of the local file to upload (.epub, .pdf, .pptx, .docx, ...).
    pub file_path: String,
    /// globalId of the destination folder.
    pub parent_gid: i64,
    /// Optional label for the new publication. Defaults to the file name (the resource is renamed after creation).
    #[serde(default)]
    pub label: Option<String>,
    /// Wait for the generation to finish before returning (default true). Polls getPublicationProgress every 2s.
    #[serde(default)]
    pub wait: Option<bool>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PublicationRequest {
    /// globalId of the publication.
    pub publication_gid: i64,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RenameResourceRequest {
    /// globalId of the resource (publication, folder, component...).
    pub resource_gid: i64,
    /// New label of the resource.
    pub label: String,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MoveResourcesRequest {
    /// globalIds of the resources to move.
    pub resource_gids: Vec<i64>,
    /// globalId of the destination folder.
    pub new_parent_gid: i64,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreatePlaylistRequest {
    /// globalId of the folder where the playlist is created.
    pub folder_gid: i64,
    /// Optional label for the playlist (the resource is renamed after creation).
    #[serde(default)]
    pub label: Option<String>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct IncludeExtPagesRequest {
    /// globalId of the target publication/playlist that receives the pages.
    pub publication_gid: i64,
    /// globalId of the source publication the pages come from.
    pub src_publication_gid: i64,
    /// 1-based page numbers of the source publication to include (appended at the end of the target).
    pub pages: Vec<u32>,
    /// Trigger the processing of printable documents (default true).
    #[serde(default)]
    pub process_printables: Option<bool>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UploadComponentRequest {
    /// Absolute path of the local .zip (or .pdf/.epub) to upload as a COMPONENT resource.
    pub file_path: String,
    /// globalId of the destination folder.
    pub folder_gid: i64,
    /// Optional label for the component (the resource is renamed after creation).
    #[serde(default)]
    pub label: Option<String>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UploadWishlistFileRequest {
    /// globalId of the publication.
    pub publication_gid: i64,
    /// Absolute path of the local file to upload.
    pub file_path: String,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TrashResourcesRequest {
    /// globalIds of the resources to move to the trash.
    pub resource_gids: Vec<i64>,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetTemplateTxtFileRequest {
    /// globalId of the publication.
    pub publication_gid: i64,
    /// Path of the text file relative to the publication's templates folder (e.g. "common-ui.xml").
    pub rel_path: String,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SaveTemplateTxtFileRequest {
    /// globalId of the publication.
    pub publication_gid: i64,
    /// Path of the text file relative to the publication's templates folder (e.g. "common-ui.xml").
    pub rel_path: String,
    /// Full new content of the file.
    pub content: String,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetCustomAdminUrlRequest {
    /// globalId of the publication (usually the playlist used by the configurator).
    pub publication_gid: i64,
    /// URL of the custom admin (configurator component), e.g. https://fr.zone-secure.net/{clientId}/{componentGId}/
    pub url: String,
    /// Client (customer) id. Omit it to use the CLIENT_ID configured in the environment.
    #[serde(default)]
    pub client_id: Option<i64>,
}
