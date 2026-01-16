use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateCheckRequest {
    pub pet_id: String,
    pub doctor_id: String,
    pub scheduled_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateCheckRequest {
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub cost: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateCheckDetailsRequest {
    pub scheduled_at: Option<DateTime<Utc>>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub cost: Option<f32>,
}
