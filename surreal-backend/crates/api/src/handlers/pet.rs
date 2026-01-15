use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use surreal_core::Pet;
use surreal_db::{PetRepository, Repository};

use crate::dto::{CreatePetRequest, UpdatePetRequest};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn create_pet(
    State(state): State<AppState>,
    Json(req): Json<CreatePetRequest>,
) -> ApiResult<(StatusCode, Json<Pet>)> {
    let mut pet = Pet::new(req.owner_id, req.name, req.species)?;

    if let Some(breed) = req.breed {
        pet = pet.with_breed(breed);
    }

    if let Some(birth_date) = req.birth_date {
        pet = pet.with_birth_date(birth_date)?;
    }

    if let Some(weight) = req.weight_kg {
        pet = pet.with_weight(weight)?;
    }

    if let Some(notes) = req.medical_notes {
        pet = pet.with_medical_notes(notes);
    }

    let repo = PetRepository::new(state.db);
    let created = repo.create(&pet).await?;

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_pet(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Pet>> {
    let repo = PetRepository::new(state.db);
    let pet = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Pet {} not found", id)))?;

    Ok(Json(pet))
}

pub async fn list_pets(State(state): State<AppState>) -> ApiResult<Json<Vec<Pet>>> {
    let repo = PetRepository::new(state.db);
    let pets = repo.find_all().await?;

    Ok(Json(pets))
}

pub async fn list_pets_by_owner(
    State(state): State<AppState>,
    Path(owner_id): Path<String>,
) -> ApiResult<Json<Vec<Pet>>> {
    let repo = PetRepository::new(state.db);
    let pets = repo.find_by_owner(&owner_id).await?;

    Ok(Json(pets))
}

pub async fn update_pet(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePetRequest>,
) -> ApiResult<Json<Pet>> {
    let repo = PetRepository::new(state.db);
    let mut pet = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Pet {} not found", id)))?;

    if let Some(name) = req.name {
        pet.name = name;
    }

    if let Some(breed) = req.breed {
        pet.breed = Some(breed);
    }

    if let Some(weight) = req.weight_kg {
        pet.weight_kg = Some(weight);
    }

    if let Some(notes) = req.medical_notes {
        pet.medical_notes = Some(notes);
    }

    pet.updated_at = chrono::Utc::now();
    let updated = repo.update(&pet).await?;

    Ok(Json(updated))
}

pub async fn delete_pet(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let repo = PetRepository::new(state.db);
    let deleted = repo.delete(&id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(surreal_db::DbError::NotFound(format!("Pet {} not found", id)).into())
    }
}
