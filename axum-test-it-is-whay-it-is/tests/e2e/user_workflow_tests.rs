use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum_test_it_is_whay_it_is::test_helpers::create_test_app;
use serde_json::json;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_complete_user_workflow() {
    let (_container, pool) = super::common::setup_test_db().await;
    let app = create_test_app(pool.clone()).await;

    // 1. Register a new user
    let register_body = json!({
        "email": "test@example.com",
        "name": "Test User",
        "password": "password123"
    });

    let request = Request::builder()
        .uri("/api/users/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_body).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Login
    let login_body = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let request = Request::builder()
        .uri("/api/users/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_body).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = body["data"]["token"].as_str().unwrap();

    // 3. Get profile with token
    let request = Request::builder()
        .uri("/api/users/profile")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["data"]["email"], "test@example.com");
    assert_eq!(body["data"]["name"], "Test User");

    pool.close().await;
}

#[tokio::test]
async fn test_unauthorized_access() {
    let (_container, pool) = super::common::setup_test_db().await;
    let app = create_test_app(pool.clone()).await;

    // Try to access protected route without token
    let request = Request::builder()
        .uri("/api/users/profile")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.close().await;
}

#[tokio::test]
async fn test_invalid_credentials() {
    let (_container, pool) = super::common::setup_test_db().await;
    let app = create_test_app(pool.clone()).await;

    // Register user
    let register_body = json!({
        "email": "test@example.com",
        "name": "Test User",
        "password": "password123"
    });

    let request = Request::builder()
        .uri("/api/users/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_body).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    // Try login with wrong password
    let login_body = json!({
        "email": "test@example.com",
        "password": "wrongpassword"
    });

    let request = Request::builder()
        .uri("/api/users/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.close().await;
}

#[tokio::test]
async fn test_health_endpoint() {
    let (_container, pool) = super::common::setup_test_db().await;
    let app = create_test_app(pool.clone()).await;

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["status"], "ok");

    pool.close().await;
}
