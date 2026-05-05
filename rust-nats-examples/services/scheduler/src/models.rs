use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub org_id: ObjectId,
    pub action_type: String,
    pub payload: serde_json::Value,
    pub trigger_type: TriggerType,
    pub cron_expression: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TriggerType {
    Manual,
    Cron,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Execution {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub action_id: ObjectId,
    pub org_id: ObjectId,
    pub user_id: ObjectId,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    pub org_id: String,
    pub action_type: String,
    pub payload: serde_json::Value,
    pub trigger_type: TriggerType,
    pub cron_expression: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateActionRequest {
    pub action_type: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub trigger_type: Option<TriggerType>,
    pub cron_expression: Option<String>,
}
