use axum::{extract::State, http::StatusCode, Json};

use surreal_core::{
    generate_token, hash_password, token_expiration_seconds, verify_password, AuthResponse,
    AuthToken, AuthUser, Doctor, LoginCredentials, RegisterDoctorRequest, RegisterUserRequest,
    Specialization, User, UserInfo, UserRole,
};
use surreal_db::{AuthRepository, DoctorRepository, Repository, UserRepository};

use crate::error::ApiResult;
use crate::state::AppState;

/// Register a new user (pet owner)
#[utoipa::path(
    post,
    path = "/auth/register/user",
    tag = "auth",
    request_body = RegisterUserRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Email already exists")
    )
)]
pub async fn register_user(
    State(state): State<AppState>,
    Json(req): Json<RegisterUserRequest>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    // Create user in users table
    let user = User::new(req.email.clone(), req.name)?
        .with_phone(req.phone.unwrap_or_default())?
        .with_address(req.address.unwrap_or_default());

    let user_repo = UserRepository::new(state.db.clone());
    let created_user = user_repo.create(&user).await?;
    let user_id = created_user.id.clone().unwrap();

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Create auth entry
    let auth_user = AuthUser::new(req.email, password_hash, UserRole::User, user_id.clone())?;

    let auth_repo = AuthRepository::new(state.db);
    let created_auth = auth_repo.create(&auth_user).await?;

    // Generate JWT
    let token = generate_token(&created_auth)?;

    let response = AuthResponse {
        token: AuthToken {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: token_expiration_seconds(),
        },
        user: UserInfo::from(created_auth),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Register a new doctor
#[utoipa::path(
    post,
    path = "/auth/register/doctor",
    tag = "auth",
    request_body = RegisterDoctorRequest,
    responses(
        (status = 201, description = "Doctor registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Email already exists")
    )
)]
pub async fn register_doctor(
    State(state): State<AppState>,
    Json(req): Json<RegisterDoctorRequest>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    // Parse specialization
    let specialization: Specialization =
        serde_json::from_str(&format!("\"{}\"", req.specialization))?;

    // Create doctor in doctors table
    let doctor = Doctor::new(
        req.name,
        req.email.clone(),
        req.phone,
        specialization,
        req.license_number,
        req.years_experience,
    )?;

    let doctor_repo = DoctorRepository::new(state.db.clone());
    let created_doctor = doctor_repo.create(&doctor).await?;
    let doctor_id = created_doctor.id.clone().unwrap();

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Create auth entry
    let auth_user = AuthUser::new(req.email, password_hash, UserRole::Doctor, doctor_id)?;

    let auth_repo = AuthRepository::new(state.db);
    let created_auth = auth_repo.create(&auth_user).await?;

    // Generate JWT
    let token = generate_token(&created_auth)?;

    let response = AuthResponse {
        token: AuthToken {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: token_expiration_seconds(),
        },
        user: UserInfo::from(created_auth),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Login
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginCredentials,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(credentials): Json<LoginCredentials>,
) -> ApiResult<Json<AuthResponse>> {
    let auth_repo = AuthRepository::new(state.db);

    // Find user by email
    let auth_user: AuthUser = auth_repo
        .find_by_email(&credentials.email)
        .await?
        .ok_or_else(|| surreal_core::CoreError::AuthError("Invalid credentials".to_string()))?;

    // Verify password
    if !verify_password(&credentials.password, &auth_user.password_hash)? {
        return Err(surreal_core::CoreError::AuthError("Invalid credentials".to_string()).into());
    }

    // Generate JWT
    let token = generate_token(&auth_user)?;

    let response = AuthResponse {
        token: AuthToken {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: token_expiration_seconds(),
        },
        user: UserInfo::from(auth_user),
    };

    Ok(Json(response))
}

/// Get current user info
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user info", body = UserInfo),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn me(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<surreal_core::Claims>,
) -> ApiResult<Json<UserInfo>> {
    let auth_repo = AuthRepository::new(state.db);
    let auth_user = auth_repo
        .find_by_id(&claims.sub)
        .await?
        .ok_or_else(|| surreal_core::CoreError::AuthError("User not found".to_string()))?;

    Ok(Json(UserInfo::from(auth_user)))
}
