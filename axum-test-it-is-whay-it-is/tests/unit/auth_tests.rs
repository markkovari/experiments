use axum_test_it_is_whay_it_is::auth::{jwt, password};
use uuid::Uuid;

#[test]
fn test_password_hashing() {
    let password = "secure_password_123";
    let hash = password::hash_password(password).expect("Failed to hash password");

    // Verify correct password
    assert!(password::verify_password(password, &hash).unwrap());

    // Verify incorrect password
    assert!(!password::verify_password("wrong_password", &hash).unwrap());
}

#[test]
fn test_jwt_generation_and_verification() {
    let secret = "test_secret_key_at_least_32_characters_long";
    let user_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let name = "Test User".to_string();

    // Generate token
    let token = jwt::generate_token(user_id, email.clone(), name.clone(), secret)
        .expect("Failed to generate token");

    // Verify token
    let claims = jwt::verify_token(&token, secret).expect("Failed to verify token");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, email);
    assert_eq!(claims.name, name);
}

#[test]
fn test_jwt_invalid_token() {
    let secret = "test_secret_key_at_least_32_characters_long";
    let result = jwt::verify_token("invalid.token.here", secret);

    assert!(result.is_err());
}

#[test]
fn test_jwt_wrong_secret() {
    let secret1 = "secret_key_1_at_least_32_characters_long_here";
    let secret2 = "secret_key_2_at_least_32_characters_long_here";
    let user_id = Uuid::new_v4();

    let token = jwt::generate_token(
        user_id,
        "test@example.com".to_string(),
        "Test".to_string(),
        secret1,
    )
    .unwrap();

    let result = jwt::verify_token(&token, secret2);
    assert!(result.is_err());
}
