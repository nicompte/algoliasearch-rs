use chrono::{DateTime, Utc};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexOperationResponse {
    pub operation: String,
    pub destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Vec<String>>
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexOperationResult {
    pub deleted_at: DateTime<Utc>,
    #[serde(rename = "taskID")]
    pub task_id: u64,
}
