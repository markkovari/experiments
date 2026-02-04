use axum_test_it_is_whay_it_is::modules::users::{
    domain::{CreateUserInput, UpdateUserInput},
    repository::{UserRepository, UserRepositoryTrait},
};

#[tokio::test]
async fn test_create_user() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let input = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
    };

    let user = repo.create(input).await.expect("Failed to create user");

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, "Test User");
    assert!(!user.password_hash.is_empty());

    pool.close().await;
}

#[tokio::test]
async fn test_find_user_by_id() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let input = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
    };

    let created_user = repo.create(input).await.unwrap();
    let found_user = repo.find_by_id(created_user.id).await.unwrap();

    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.email, created_user.email);

    pool.close().await;
}

#[tokio::test]
async fn test_find_user_by_email() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let input = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
    };

    let created_user = repo.create(input).await.unwrap();
    let found_user = repo.find_by_email("test@example.com").await.unwrap();

    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, created_user.id);

    pool.close().await;
}

#[tokio::test]
async fn test_update_user() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let input = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
    };

    let created_user = repo.create(input).await.unwrap();

    let update_input = UpdateUserInput {
        email: None,
        name: Some("Updated Name".to_string()),
    };

    let updated_user = repo.update(created_user.id, update_input).await.unwrap();

    assert!(updated_user.is_some());
    let updated_user = updated_user.unwrap();
    assert_eq!(updated_user.name, "Updated Name");
    assert_eq!(updated_user.email, "test@example.com");

    pool.close().await;
}

#[tokio::test]
async fn test_delete_user() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let input = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
    };

    let created_user = repo.create(input).await.unwrap();
    let deleted = repo.delete(created_user.id).await.unwrap();

    assert!(deleted);

    let found = repo.find_by_id(created_user.id).await.unwrap();
    assert!(found.is_none());

    pool.close().await;
}

#[tokio::test]
async fn test_duplicate_email() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let input1 = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "User 1".to_string(),
        password: "password123".to_string(),
    };

    repo.create(input1).await.unwrap();

    let input2 = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "User 2".to_string(),
        password: "password456".to_string(),
    };

    let result = repo.create(input2).await;
    assert!(result.is_err());

    pool.close().await;
}

#[tokio::test]
async fn test_find_all_users() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    // Create multiple users
    for i in 1..=3 {
        let input = CreateUserInput {
            email: format!("user{}@example.com", i),
            name: format!("User {}", i),
            password: "password123".to_string(),
        };
        repo.create(input).await.unwrap();
    }

    let users = repo.find_all().await.unwrap();
    assert_eq!(users.len(), 3);

    pool.close().await;
}
