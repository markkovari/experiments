use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::json;
use tower::ServiceExt;

use surreal_api::{create_router, AppState};
use surreal_core::{PetSpecies, Specialization};
use surreal_db::Database;
use surreal_migrations::MigrationRunner;

async fn setup_test_app() -> axum::Router {
    std::env::set_var("JWT_SECRET", "test-secret-key-for-testing-only");
    std::env::set_var("JWT_EXPIRATION", "3600");

    let db = Database::new_in_memory().await.unwrap();

    let runner = MigrationRunner::new(db.clone());
    runner.run().await.unwrap();

    let state = AppState::new(db);
    create_router(state)
}

#[allow(dead_code)]
async fn register_and_login_user(app: &axum::Router) -> String {
    // Register user
    let register_body = json!({
        "email": "testuser@example.com",
        "password": "password123",
        "name": "Test User",
        "phone": "+1234567890"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register/user")
                .header("content-type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "User registration failed"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    auth_response["token"]["access_token"]
        .as_str()
        .expect("Failed to get access_token from registration response")
        .to_string()
}

async fn register_and_login_doctor(app: &axum::Router) -> String {
    // Register doctor
    let register_body = json!({
        "email": "doctor@example.com",
        "password": "password123",
        "name": "Dr. Smith",
        "phone": "+1234567890",
        "specialization": "GeneralPractice",
        "license_number": "LIC123456",
        "years_experience": 10
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register/doctor")
                .header("content-type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Doctor registration failed"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    auth_response["token"]["access_token"]
        .as_str()
        .expect("Failed to get access_token from registration response")
        .to_string()
}

#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_user_crud() {
    let app = setup_test_app().await;
    let token = register_and_login_doctor(&app).await; // Use doctor token for CRUD operations

    // Create user
    let create_body = json!({
        "email": "newuser@example.com",
        "name": "Test User",
        "phone": "+1234567890"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user_id = created_user["id"].as_str().unwrap();

    // Get user
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/users/{}", user_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // List users
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/users")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Update user
    let update_body = json!({
        "name": "Updated User"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/users/{}", user_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Delete user
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/users/{}", user_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_pet_crud() {
    let app = setup_test_app().await;
    let token = register_and_login_doctor(&app).await;

    // First create a user (owner)
    let create_user_body = json!({
        "email": "owner@example.com",
        "name": "Pet Owner"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_user_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let owner_id = created_user["id"].as_str().unwrap();

    // Create pet
    let create_pet_body = json!({
        "owner_id": owner_id,
        "name": "Buddy",
        "species": PetSpecies::Dog,
        "breed": "Golden Retriever",
        "weight_kg": 30.5
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/pets")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_pet_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_pet: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let pet_id = created_pet["id"].as_str().unwrap();

    // Get pet
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/pets/{}", pet_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Get pets by owner
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/users/{}/pets", owner_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Update pet
    let update_body = json!({
        "weight_kg": 32.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/pets/{}", pet_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_doctor_crud() {
    let app = setup_test_app().await;
    let token = register_and_login_doctor(&app).await;

    // Create doctor
    let create_body = json!({
        "name": "Dr. Smith",
        "email": "smith@clinic.com",
        "phone": "+1234567890",
        "specialization": Specialization::GeneralPractice,
        "license_number": "LIC-12345",
        "years_experience": 10
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/doctors")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_doctor: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let doctor_id = created_doctor["id"].as_str().unwrap();

    // Get available doctors
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/doctors/available")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Update doctor availability
    let update_body = json!({
        "is_available": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/doctors/{}", doctor_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_check_workflow() {
    let app = setup_test_app().await;
    let token = register_and_login_doctor(&app).await;

    // Create user
    let user_body = json!({
        "email": "owner@test.com",
        "name": "Owner"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(user_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let owner_id = user["id"].as_str().unwrap();

    // Create pet
    let pet_body = json!({
        "owner_id": owner_id,
        "name": "Max",
        "species": PetSpecies::Dog
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/pets")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(pet_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let pet: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let pet_id = pet["id"].as_str().unwrap();

    // Create doctor
    let doctor_body = json!({
        "name": "Dr. Vet",
        "email": "vet@clinic.com",
        "phone": "+1111111111",
        "specialization": Specialization::GeneralPractice,
        "license_number": "LIC-001",
        "years_experience": 5
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/doctors")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(doctor_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let doctor: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let doctor_id = doctor["id"].as_str().unwrap();

    // Create health check
    let scheduled = Utc::now() + Duration::hours(2);
    let check_body = json!({
        "pet_id": pet_id,
        "doctor_id": doctor_id,
        "scheduled_at": scheduled.to_rfc3339()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/checks")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(check_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let check: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let check_id = check["id"].as_str().unwrap();

    // Start check
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/checks/{}/start", check_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Complete check
    let complete_body = json!({
        "diagnosis": "Healthy",
        "treatment": "Vaccination",
        "cost": 100.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/checks/{}/complete", check_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(complete_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_not_found() {
    let app = setup_test_app().await;
    let token = register_and_login_doctor(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/users/00000000-0000-0000-0000-000000000000")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_health_check_details() {
    let app = setup_test_app().await;
    let token = register_and_login_doctor(&app).await;

    // Create user
    let user_body = json!({
        "email": "updateowner@test.com",
        "name": "Update Owner"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(user_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let owner_id = user["id"].as_str().unwrap();

    // Create pet
    let pet_body = json!({
        "owner_id": owner_id,
        "name": "UpdatePet",
        "species": PetSpecies::Cat
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/pets")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(pet_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let pet: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let pet_id = pet["id"].as_str().unwrap();

    // Create doctor
    let doctor_body = json!({
        "name": "Dr. Update",
        "email": "update@clinic.com",
        "phone": "+3333333333",
        "specialization": Specialization::Surgery,
        "license_number": "LIC-UPDATE",
        "years_experience": 7
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/doctors")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(doctor_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let doctor: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let doctor_id = doctor["id"].as_str().unwrap();

    // Create health check
    let scheduled = Utc::now() + Duration::days(1);
    let check_body = json!({
        "pet_id": pet_id,
        "doctor_id": doctor_id,
        "scheduled_at": scheduled.to_rfc3339()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/checks")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(check_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let check: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let check_id = check["id"].as_str().unwrap();

    // Test 1: Update scheduled_at while status is Scheduled
    let new_scheduled = Utc::now() + Duration::days(2);
    let update_body = json!({
        "scheduled_at": new_scheduled.to_rfc3339()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/checks/{}", check_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test 2: Start the check
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/checks/{}/start", check_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test 3: Update notes while in progress
    let update_notes_body = json!({
        "notes": "Patient seems anxious"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/checks/{}", check_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_notes_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test 4: Complete the check
    let complete_body = json!({
        "diagnosis": "Tooth decay",
        "treatment": "Dental cleaning",
        "cost": 200.0
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/checks/{}/complete", check_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(complete_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test 5: Update diagnosis and cost after completion
    let update_completed_body = json!({
        "diagnosis": "Severe tooth decay",
        "cost": 250.0
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/checks/{}", check_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_completed_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated_check: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        updated_check["diagnosis"].as_str().unwrap(),
        "Severe tooth decay"
    );
    assert_eq!(updated_check["cost"].as_f64().unwrap(), 250.0);

    // Test 6: Try to reschedule a completed check (should fail)
    let reschedule_body = json!({
        "scheduled_at": (Utc::now() + Duration::days(3)).to_rfc3339()
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/checks/{}", check_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(reschedule_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return an error because we can't reschedule a completed check
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
