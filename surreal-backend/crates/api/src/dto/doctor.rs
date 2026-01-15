use serde::{Deserialize, Serialize};
use surreal_core::Specialization;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateDoctorRequest {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub specialization: Specialization,
    pub license_number: String,
    pub years_experience: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateDoctorRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub is_available: Option<bool>,
}
