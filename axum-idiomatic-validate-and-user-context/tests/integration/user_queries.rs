use axum_idiomatic_validate_and_user_context::models::user::{create_user, find_by_username};

use crate::common;

#[tokio::test]
async fn create_and_find_user() {
    let pool = common::test_pool().await;

    let user = create_user(&pool, "alice", "alice@test.com", "secret123")
        .await
        .unwrap();

    assert_eq!(user.username, "alice");
    assert_eq!(user.email, "alice@test.com");

    let found = find_by_username(&pool, "alice").await.unwrap().unwrap();
    assert_eq!(found.id, user.id);
}

#[tokio::test]
async fn find_nonexistent_user_returns_none() {
    let pool = common::test_pool().await;
    let found = find_by_username(&pool, "ghost").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn duplicate_username_returns_conflict() {
    let pool = common::test_pool().await;

    create_user(&pool, "bob", "bob@test.com", "pass1")
        .await
        .unwrap();

    let result = create_user(&pool, "bob", "bob2@test.com", "pass2").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn duplicate_email_returns_conflict() {
    let pool = common::test_pool().await;

    create_user(&pool, "carol", "carol@test.com", "pass1")
        .await
        .unwrap();

    let result = create_user(&pool, "carol2", "carol@test.com", "pass2").await;
    assert!(result.is_err());
}
