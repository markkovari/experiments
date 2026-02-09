use crate::common;

#[tokio::test]
async fn register_and_login_flow() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Register
    let res = client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "testuser",
            "email": "testuser@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["username"], "testuser");

    let register_token = body["token"].as_str().unwrap().to_string();

    // Login with same credentials
    let res = client
        .post(format!("{url}/users/login"))
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["token"].is_string());

    // Use register token on protected route
    let res = client
        .get(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {register_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn missing_token_returns_401() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{url}/posts"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn invalid_token_returns_401() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{url}/posts"))
        .header("Authorization", "Bearer invalid-token")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Register first
    client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "wrongpass",
            "email": "wrongpass@test.com",
            "password": "correct"
        }))
        .send()
        .await
        .unwrap();

    // Login with wrong password
    let res = client
        .post(format!("{url}/users/login"))
        .json(&serde_json::json!({
            "username": "wrongpass",
            "password": "incorrect"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn register_with_empty_fields_returns_400() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "",
            "email": "",
            "password": ""
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("must not be empty"));
}

#[tokio::test]
async fn register_with_invalid_email_returns_400() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "testuser",
            "email": "not-an-email",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("email"));
}
