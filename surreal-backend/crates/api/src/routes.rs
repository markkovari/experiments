use axum::{
    routing::{get, patch, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{check, doctor, health, pet, user};
use crate::openapi::ApiDoc;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    info!("Creating API router");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_router = Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // User routes
        .route("/users", post(user::create_user).get(user::list_users))
        .route(
            "/users/:id",
            get(user::get_user)
                .put(user::update_user)
                .delete(user::delete_user),
        )
        // Pet routes
        .route("/pets", post(pet::create_pet).get(pet::list_pets))
        .route(
            "/pets/:id",
            get(pet::get_pet)
                .put(pet::update_pet)
                .delete(pet::delete_pet),
        )
        .route("/users/:owner_id/pets", get(pet::list_pets_by_owner))
        // Doctor routes
        .route(
            "/doctors",
            post(doctor::create_doctor).get(doctor::list_doctors),
        )
        .route(
            "/doctors/:id",
            get(doctor::get_doctor)
                .put(doctor::update_doctor)
                .delete(doctor::delete_doctor),
        )
        .route("/doctors/available", get(doctor::list_available_doctors))
        // Health check routes
        .route(
            "/checks",
            post(check::create_check).get(check::list_checks),
        )
        .route(
            "/checks/:id",
            get(check::get_check)
                .put(check::update_check)
                .delete(check::delete_check),
        )
        .route("/checks/:id/start", patch(check::start_check))
        .route("/checks/:id/complete", patch(check::complete_check))
        .route("/checks/:id/cancel", patch(check::cancel_check))
        .route("/pets/:pet_id/checks", get(check::list_checks_by_pet))
        .route(
            "/doctors/:doctor_id/checks",
            get(check::list_checks_by_doctor),
        )
        .with_state(state);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/", api_router)
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surreal_db::Database;

    #[tokio::test]
    async fn test_create_router() {
        let db = Database::new_in_memory().await.unwrap();
        let state = AppState::new(db);
        let _router = create_router(state);
        // Router creation should succeed
    }
}
