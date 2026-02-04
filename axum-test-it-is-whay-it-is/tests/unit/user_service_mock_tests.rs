use axum_test_it_is_whay_it_is::modules::users::{
    domain::{CreateUserInput, LoginInput, User},
    repository::UserRepositoryTrait,
    service::UserService,
};
use axum_test_it_is_whay_it_is::shared::error::AppResult;
use mockall::mock;
use mockall::predicate::*;
use uuid::Uuid;

// Mock the UserRepositoryTrait
mock! {
    pub UserRepository {}

    #[async_trait::async_trait]
    impl UserRepositoryTrait for UserRepository {
        async fn create(&self, input: CreateUserInput) -> AppResult<User>;
        async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
        async fn find_all(&self) -> AppResult<Vec<User>>;
        async fn update(&self, id: Uuid, input: axum_test_it_is_whay_it_is::modules::users::domain::UpdateUserInput) -> AppResult<Option<User>>;
        async fn delete(&self, id: Uuid) -> AppResult<bool>;
    }

    impl Clone for UserRepository {
        fn clone(&self) -> Self;
    }
}

#[tokio::test]
async fn test_create_user_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    let user_id = Uuid::new_v4();
    let expected_user = User {
        id: user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Setup expectation: when create is called, return the expected_user
    mock_repo
        .expect_create()
        .times(1)
        .returning(move |_input| Ok(expected_user.clone()));

    let service = UserService::new(mock_repo, "test_secret".to_string());

    let input = CreateUserInput {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
    };

    let result = service.create(input).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, "Test User");
}

#[tokio::test]
async fn test_get_user_by_id_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    let user_id = Uuid::new_v4();
    let expected_user = User {
        id: user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Setup expectation
    mock_repo
        .expect_find_by_id()
        .with(eq(user_id))
        .times(1)
        .returning(move |_| Ok(Some(expected_user.clone())));

    let service = UserService::new(mock_repo, "test_secret".to_string());

    let result = service.get_by_id(user_id).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.id, user_id);
}

#[tokio::test]
async fn test_get_user_not_found_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    let user_id = Uuid::new_v4();

    // Setup expectation: return None (user not found)
    mock_repo
        .expect_find_by_id()
        .with(eq(user_id))
        .times(1)
        .returning(|_| Ok(None));

    let service = UserService::new(mock_repo, "test_secret".to_string());

    let result = service.get_by_id(user_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_login_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    let user_id = Uuid::new_v4();

    // Hash a password for testing
    let password_hash = axum_test_it_is_whay_it_is::auth::password::hash_password("password123")
        .expect("Failed to hash password");

    let expected_user = User {
        id: user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password_hash: password_hash.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Setup expectation: when finding by email, return the user
    mock_repo
        .expect_find_by_email()
        .with(eq("test@example.com"))
        .times(1)
        .returning(move |_| Ok(Some(expected_user.clone())));

    let service = UserService::new(mock_repo, "test_secret".to_string());

    let input = LoginInput {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = service.login(input).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.token.is_empty());
    assert_eq!(response.user.email, "test@example.com");
}

#[tokio::test]
async fn test_login_invalid_password_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    let user_id = Uuid::new_v4();

    // Hash a different password
    let password_hash = axum_test_it_is_whay_it_is::auth::password::hash_password("correct_password")
        .expect("Failed to hash password");

    let expected_user = User {
        id: user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password_hash,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Setup expectation
    mock_repo
        .expect_find_by_email()
        .with(eq("test@example.com"))
        .times(1)
        .returning(move |_| Ok(Some(expected_user.clone())));

    let service = UserService::new(mock_repo, "test_secret".to_string());

    let input = LoginInput {
        email: "test@example.com".to_string(),
        password: "wrong_password".to_string(),
    };

    // Try to login with wrong password
    let result = service.login(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_all_users_with_mock() {
    let mut mock_repo = MockUserRepository::new();

    let users = vec![
        User {
            id: Uuid::new_v4(),
            email: "user1@example.com".to_string(),
            name: "User 1".to_string(),
            password_hash: "hash1".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        User {
            id: Uuid::new_v4(),
            email: "user2@example.com".to_string(),
            name: "User 2".to_string(),
            password_hash: "hash2".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    mock_repo
        .expect_find_all()
        .times(1)
        .returning(move || Ok(users.clone()));

    let service = UserService::new(mock_repo, "test_secret".to_string());

    let result = service.get_all().await;
    assert!(result.is_ok());
    let fetched_users = result.unwrap();
    assert_eq!(fetched_users.len(), 2);
}
