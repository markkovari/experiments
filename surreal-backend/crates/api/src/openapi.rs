use utoipa::OpenApi;

use crate::dto::{
    check::{CreateCheckRequest, UpdateCheckDetailsRequest, UpdateCheckRequest},
    doctor::{CreateDoctorRequest, UpdateDoctorRequest},
    pet::{CreatePetRequest, UpdatePetRequest},
    user::{CreateUserRequest, UpdateUserRequest},
};
use surreal_core::{CheckStatus, Doctor, HealthCheck, Pet, PetSpecies, Specialization, User};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Veterinary Clinic API",
        version = "1.0.0",
        description = "REST API for managing a veterinary clinic with SurrealDB backend",
        contact(
            name = "Mark Kovari",
            email = "mark@example.com"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server")
    ),
    paths(
        crate::handlers::health::health_check,
        crate::handlers::user::create_user,
        crate::handlers::user::list_users,
        crate::handlers::user::get_user,
        crate::handlers::user::update_user,
        crate::handlers::user::delete_user,
        crate::handlers::pet::create_pet,
    ),
    components(
        schemas(
            User,
            Pet,
            PetSpecies,
            Doctor,
            Specialization,
            HealthCheck,
            CheckStatus,
            CreateUserRequest,
            UpdateUserRequest,
            CreatePetRequest,
            UpdatePetRequest,
            CreateDoctorRequest,
            UpdateDoctorRequest,
            CreateCheckRequest,
            UpdateCheckDetailsRequest,
            UpdateCheckRequest,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "pets", description = "Pet management endpoints"),
        (name = "doctors", description = "Doctor management endpoints"),
        (name = "checks", description = "Health check management endpoints")
    )
)]
pub struct ApiDoc;
