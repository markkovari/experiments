use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};

use surreal_core::{Claims, HealthCheck, PaginatedResponse, PaginationParams, Pet};
use surreal_db::{CheckRepository, PetRepository, Repository};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Get current user's pets (paginated)
#[utoipa::path(
    get,
    path = "/me/pets",
    tag = "me",
    params(PaginationParams),
    responses(
        (status = 200, description = "User's pets", body = PaginatedResponse<Pet>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_pets(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<Pet>>> {
    let pet_repo = PetRepository::new(state.db);

    // Use reference_id to get user's pets
    let paginated_pets = pet_repo
        .find_by_owner_paginated(&claims.ref_id, &params)
        .await?;

    Ok(Json(paginated_pets))
}

/// Get a specific pet of the current user
#[utoipa::path(
    get,
    path = "/me/pets/{pet_id}",
    tag = "me",
    params(
        ("pet_id" = String, Path, description = "Pet ID")
    ),
    responses(
        (status = 200, description = "Pet found", body = Pet),
        (status = 404, description = "Pet not found"),
        (status = 403, description = "Not your pet")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_pet(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(pet_id): Path<String>,
) -> ApiResult<Json<Pet>> {
    let pet_repo = PetRepository::new(state.db);

    let pet = pet_repo
        .find_by_id(&pet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pet not found".to_string()))?;

    // Verify the pet belongs to the current user
    if pet.owner_id != claims.ref_id {
        return Err(ApiError::Forbidden(
            "This pet does not belong to you".to_string(),
        ));
    }

    Ok(Json(pet))
}

/// Get health checks for a specific pet of the current user (paginated)
#[utoipa::path(
    get,
    path = "/me/pets/{pet_id}/checks",
    tag = "me",
    params(
        ("pet_id" = String, Path, description = "Pet ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "Pet's health checks", body = PaginatedResponse<HealthCheck>),
        (status = 404, description = "Pet not found"),
        (status = 403, description = "Not your pet")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_pet_checks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(pet_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<HealthCheck>>> {
    let pet_repo = PetRepository::new(state.db.clone());

    // Verify pet exists and belongs to user
    let pet = pet_repo
        .find_by_id(&pet_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pet not found".to_string()))?;

    if pet.owner_id != claims.ref_id {
        return Err(ApiError::Forbidden(
            "This pet does not belong to you".to_string(),
        ));
    }

    // Get health checks for this pet
    let check_repo = CheckRepository::new(state.db);
    let paginated_checks = check_repo.find_by_pet_paginated(&pet_id, &params).await?;

    Ok(Json(paginated_checks))
}
