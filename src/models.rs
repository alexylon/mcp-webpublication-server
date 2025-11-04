use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetResourceRequest {
    pub resource_gid: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}
