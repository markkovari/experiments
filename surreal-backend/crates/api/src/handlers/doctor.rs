use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use surreal_core::Doctor;
use surreal_db::{DoctorRepository, Repository};

use crate::dto::{CreateDoctorRequest, UpdateDoctorRequest};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn create_doctor(
    State(state): State<AppState>,
    Json(req): Json<CreateDoctorRequest>,
) -> ApiResult<(StatusCode, Json<Doctor>)> {
    let doctor = Doctor::new(
        req.name,
        req.email,
        req.phone,
        req.specialization,
        req.license_number,
        req.years_experience,
    )?;

    let repo = DoctorRepository::new(state.db);
    let created = repo.create(&doctor).await?;

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_doctor(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Doctor>> {
    let repo = DoctorRepository::new(state.db);
    let doctor = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Doctor {} not found", id)))?;

    Ok(Json(doctor))
}

pub async fn list_doctors(State(state): State<AppState>) -> ApiResult<Json<Vec<Doctor>>> {
    let repo = DoctorRepository::new(state.db);
    let doctors = repo.find_all().await?;

    Ok(Json(doctors))
}

pub async fn list_available_doctors(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<Doctor>>> {
    let repo = DoctorRepository::new(state.db);
    let doctors = repo.find_available().await?;

    Ok(Json(doctors))
}

pub async fn update_doctor(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDoctorRequest>,
) -> ApiResult<Json<Doctor>> {
    let repo = DoctorRepository::new(state.db);
    let mut doctor = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Doctor {} not found", id)))?;

    if let Some(name) = req.name {
        doctor.name = name;
    }

    if let Some(phone) = req.phone {
        doctor.phone = phone;
    }

    if let Some(available) = req.is_available {
        doctor.set_availability(available);
    }

    doctor.updated_at = chrono::Utc::now();
    let updated = repo.update(&doctor).await?;

    Ok(Json(updated))
}

pub async fn delete_doctor(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let repo = DoctorRepository::new(state.db);
    let deleted = repo.delete(&id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(surreal_db::DbError::NotFound(format!("Doctor {} not found", id)).into())
    }
}
