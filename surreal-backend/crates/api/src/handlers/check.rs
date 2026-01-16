use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use surreal_core::HealthCheck;
use surreal_db::{CheckRepository, Repository};

use crate::dto::{CreateCheckRequest, UpdateCheckDetailsRequest, UpdateCheckRequest};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn create_check(
    State(state): State<AppState>,
    Json(req): Json<CreateCheckRequest>,
) -> ApiResult<(StatusCode, Json<HealthCheck>)> {
    let check = HealthCheck::new(req.pet_id, req.doctor_id, req.scheduled_at)?;

    let repo = CheckRepository::new(state.db);
    let created = repo.create(&check).await?;

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<HealthCheck>> {
    let repo = CheckRepository::new(state.db);
    let check = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Health check {} not found", id)))?;

    Ok(Json(check))
}

pub async fn list_checks(State(state): State<AppState>) -> ApiResult<Json<Vec<HealthCheck>>> {
    let repo = CheckRepository::new(state.db);
    let checks = repo.find_all().await?;

    Ok(Json(checks))
}

pub async fn list_checks_by_pet(
    State(state): State<AppState>,
    Path(pet_id): Path<String>,
) -> ApiResult<Json<Vec<HealthCheck>>> {
    let repo = CheckRepository::new(state.db);
    let checks = repo.find_by_pet(&pet_id).await?;

    Ok(Json(checks))
}

pub async fn list_checks_by_doctor(
    State(state): State<AppState>,
    Path(doctor_id): Path<String>,
) -> ApiResult<Json<Vec<HealthCheck>>> {
    let repo = CheckRepository::new(state.db);
    let checks = repo.find_by_doctor(&doctor_id).await?;

    Ok(Json(checks))
}

pub async fn update_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCheckDetailsRequest>,
) -> ApiResult<Json<HealthCheck>> {
    let repo = CheckRepository::new(state.db);
    let mut check = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Health check {} not found", id)))?;

    // Update scheduled_at only if status is Scheduled
    if let Some(scheduled_at) = req.scheduled_at {
        if check.status != surreal_core::CheckStatus::Scheduled {
            return Err(surreal_core::CoreError::ValidationError(
                "Can only reschedule a scheduled check".to_string(),
            )
            .into());
        }
        if scheduled_at < chrono::Utc::now() {
            return Err(surreal_core::CoreError::InvalidDate(
                "Cannot schedule check in the past".to_string(),
            )
            .into());
        }
        check.scheduled_at = scheduled_at;
    }

    // Update diagnosis, treatment, notes, cost for any status
    if let Some(diagnosis) = req.diagnosis {
        check.diagnosis = Some(diagnosis);
    }

    if let Some(treatment) = req.treatment {
        check.treatment = Some(treatment);
    }

    if let Some(notes) = req.notes {
        check.notes = Some(notes);
    }

    if let Some(cost) = req.cost {
        if cost < 0.0 {
            return Err(surreal_core::CoreError::ValidationError(
                "Cost cannot be negative".to_string(),
            )
            .into());
        }
        check.cost = Some(cost);
    }

    check.updated_at = chrono::Utc::now();
    let updated = repo.update(&check).await?;

    Ok(Json(updated))
}

pub async fn start_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<HealthCheck>> {
    let repo = CheckRepository::new(state.db);
    let mut check = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Health check {} not found", id)))?;

    check.start()?;
    let updated = repo.update(&check).await?;

    Ok(Json(updated))
}

pub async fn complete_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCheckRequest>,
) -> ApiResult<Json<HealthCheck>> {
    let repo = CheckRepository::new(state.db);
    let mut check = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Health check {} not found", id)))?;

    let diagnosis = req.diagnosis.ok_or_else(|| {
        surreal_core::CoreError::ValidationError("Diagnosis is required".to_string())
    })?;

    check.complete(diagnosis, req.treatment, req.cost)?;

    if let Some(notes) = req.notes {
        check.add_notes(notes);
    }

    let updated = repo.update(&check).await?;

    Ok(Json(updated))
}

pub async fn cancel_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<HealthCheck>> {
    let repo = CheckRepository::new(state.db);
    let mut check = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("Health check {} not found", id)))?;

    check.cancel()?;
    let updated = repo.update(&check).await?;

    Ok(Json(updated))
}

pub async fn delete_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let repo = CheckRepository::new(state.db);
    let deleted = repo.delete(&id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(surreal_db::DbError::NotFound(format!("Health check {} not found", id)).into())
    }
}
