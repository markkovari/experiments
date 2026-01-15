use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use surreal_core::PetSpecies;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePetRequest {
    pub owner_id: String,
    pub name: String,
    pub species: PetSpecies,
    pub breed: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub weight_kg: Option<f32>,
    pub medical_notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePetRequest {
    pub name: Option<String>,
    pub breed: Option<String>,
    pub weight_kg: Option<f32>,
    pub medical_notes: Option<String>,
}
