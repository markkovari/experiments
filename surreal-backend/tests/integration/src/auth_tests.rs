use surreal_core::{hash_password, verify_password, generate_token, verify_token, AuthUser, UserRole};
use surreal_db::{AuthRepository, Database, Repository};

#[tokio::test]
async fn test_password_hashing() {
    let password = "my_secure_password_123";
    let hash = hash_password(password).unwrap();

    assert_ne!(password, hash);
    assert!(verify_password(password, &hash).unwrap());
    assert!(!verify_password("wrong_password", &hash).unwrap());
}

#[tokio::test]
async fn test_jwt_token_generation_and_verification() {
    std::env::set_var("JWT_SECRET", "test-secret-key-for-testing");
    std::env::set_var("JWT_EXPIRATION", "3600");

    let password_hash = hash_password("password123").unwrap();
    let mut auth_user = AuthUser::new(
        "test@example.com".to_string(),
        password_hash,
        UserRole::User,
        "users:test123".to_string(),
    )
    .unwrap();
    auth_user.id = Some("auth_users:test1".to_string());

    // Generate token
    let token = generate_token(&auth_user).unwrap();
    assert!(!token.is_empty());

    // Verify token
    let claims = verify_token(&token).unwrap();
    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.role, "user");
    assert_eq!(claims.ref_id, "users:test123");
}

#[tokio::test]
async fn test_auth_user_registration() {
    let db = Database::new_in_memory().await.unwrap();
    let repo = AuthRepository::new(db);

    let password_hash = hash_password("password123").unwrap();
    let auth_user = AuthUser::new(
        "newuser@example.com".to_string(),
        password_hash,
        UserRole::User,
        "users:new123".to_string(),
    )
    .unwrap();

    let created = repo.create(&auth_user).await.unwrap();
    assert_eq!(created.email, "newuser@example.com");
    assert_eq!(created.role, UserRole::User);
    assert!(created.id.is_some());
}

#[tokio::test]
async fn test_duplicate_email_registration() {
    let db = Database::new_in_memory().await.unwrap();
    let repo = AuthRepository::new(db);

    let password_hash = hash_password("password123").unwrap();
    let auth_user1 = AuthUser::new(
        "duplicate@example.com".to_string(),
        password_hash.clone(),
        UserRole::User,
        "users:test1".to_string(),
    )
    .unwrap();

    repo.create(&auth_user1).await.unwrap();

    // Try to create another with same email
    let auth_user2 = AuthUser::new(
        "duplicate@example.com".to_string(),
        password_hash,
        UserRole::Doctor,
        "doctors:test2".to_string(),
    )
    .unwrap();

    let result = repo.create(&auth_user2).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_find_by_email() {
    let db = Database::new_in_memory().await.unwrap();
    let repo = AuthRepository::new(db);

    let password_hash = hash_password("password123").unwrap();
    let auth_user = AuthUser::new(
        "findme@example.com".to_string(),
        password_hash,
        UserRole::User,
        "users:findme".to_string(),
    )
    .unwrap();

    repo.create(&auth_user).await.unwrap();

    let found = repo.find_by_email("findme@example.com").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, "findme@example.com");

    let not_found = repo.find_by_email("notfound@example.com").await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_login_flow() {
    std::env::set_var("JWT_SECRET", "test-secret-key-for-testing-login-flow");
    std::env::set_var("JWT_EXPIRATION", "3600");

    let db = Database::new_in_memory().await.unwrap();
    let repo = AuthRepository::new(db);

    // Register user
    let password = "my_password_123";
    let password_hash = hash_password(password).unwrap();
    let auth_user = AuthUser::new(
        "login@example.com".to_string(),
        password_hash,
        UserRole::Doctor,
        "doctors:login123".to_string(),
    )
    .unwrap();

    let created = repo.create(&auth_user).await.unwrap();

    // Simulate login - find by email
    let found_user = repo.find_by_email("login@example.com").await.unwrap().unwrap();

    // Verify password
    assert!(verify_password(password, &found_user.password_hash).unwrap());

    // Generate token
    let token = generate_token(&found_user).unwrap();

    // Verify token
    let claims = verify_token(&token).unwrap();
    assert_eq!(claims.email, "login@example.com");
    assert_eq!(claims.role, "doctor");
    assert_eq!(claims.sub, created.id.unwrap());
}
