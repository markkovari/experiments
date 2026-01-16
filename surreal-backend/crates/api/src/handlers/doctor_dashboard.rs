use axum::{
    extract::{Query, State},
    Extension, Json,
};

use surreal_core::{Claims, PaginatedResponse, PaginationParams, Pet, HealthCheck, User, UserRole};
use surreal_db::{CheckRepository, PetRepository, Repository, UserRepository};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Get all health checks (doctor view, paginated)
#[utoipa::path(
    get,
    path = "/doctor/checks",
    tag = "doctor",
    params(PaginationParams),
    responses(
        (status = 200, description = "All health checks", body = PaginatedResponse<HealthCheck>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Doctor only")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_checks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<HealthCheck>>> {
    // Verify user is a doctor
    if claims.role != UserRole::Doctor.as_str() {
        return Err(ApiError::Forbidden("Only doctors can access this endpoint".to_string()));
    }

    let check_repo = CheckRepository::new(state.db);
    let paginated_checks = check_repo.find_paginated(&params).await?;

    Ok(Json(paginated_checks))
}

/// Get all pets (doctor view, paginated)
#[utoipa::path(
    get,
    path = "/doctor/pets",
    tag = "doctor",
    params(PaginationParams),
    responses(
        (status = 200, description = "All pets", body = PaginatedResponse<Pet>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Doctor only")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_pets(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<Pet>>> {
    // Verify user is a doctor
    if claims.role != UserRole::Doctor.as_str() {
        return Err(ApiError::Forbidden("Only doctors can access this endpoint".to_string()));
    }

    let pet_repo = PetRepository::new(state.db);
    let paginated_pets = pet_repo.find_paginated(&params).await?;

    Ok(Json(paginated_pets))
}

/// Get all users (doctor view, paginated)
#[utoipa::path(
    get,
    path = "/doctor/users",
    tag = "doctor",
    params(PaginationParams),
    responses(
        (status = 200, description = "All users", body = PaginatedResponse<User>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Doctor only")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_users(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<User>>> {
    // Verify user is a doctor
    if claims.role != UserRole::Doctor.as_str() {
        return Err(ApiError::Forbidden("Only doctors can access this endpoint".to_string()));
    }

    let user_repo = UserRepository::new(state.db);
    let paginated_users = user_repo.find_paginated(&params).await?;

    Ok(Json(paginated_users))
}

/// Get doctor's own appointments (paginated)
#[utoipa::path(
    get,
    path = "/doctor/my-checks",
    tag = "doctor",
    params(PaginationParams),
    responses(
        (status = 200, description = "Doctor's appointments", body = PaginatedResponse<HealthCheck>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Doctor only")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_checks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<HealthCheck>>> {
    // Verify user is a doctor
    if claims.role != UserRole::Doctor.as_str() {
        return Err(ApiError::Forbidden("Only doctors can access this endpoint".to_string()));
    }

    let check_repo = CheckRepository::new(state.db);
    let paginated_checks = check_repo
        .find_by_doctor_paginated(&claims.ref_id, &params)
        .await?;

    Ok(Json(paginated_checks))
}
