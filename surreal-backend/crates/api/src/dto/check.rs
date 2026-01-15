use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCheckRequest {
    pub pet_id: String,
    pub doctor_id: String,
    pub scheduled_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateCheckRequest {
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub cost: Option<f32>,
}
