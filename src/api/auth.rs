use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::api::middleware::auth::{auth_middleware, AuthUser};
use crate::dto::auth_dto::{AuthResponse, LoginRequest, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest, UserResponse};
use crate::repositories::user_repo::UserRepository;
use crate::services::auth_service::AuthService;
use crate::utils::error::{AppError, AppResult};
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    let public_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/verify-email", post(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password));

    let protected_routes = Router::new()
        .route("/me", get(get_current_user))
        .route("/logout", post(logout))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    public_routes.merge(protected_routes)
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"token": "abc123def456"}))]
pub struct VerifyEmailRequest {
    #[schema(example = "abc123def456")]
    pub token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com"}))]
pub struct ForgotPasswordRequest {
    #[schema(example = "user@example.com", format = "email")]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"token": "reset_token_123", "new_password": "newpassword123"}))]
pub struct ResetPasswordRequest {
    #[schema(example = "reset_token_123")]
    pub token: String,
    
    #[validate(length(min = 8))]
    #[schema(example = "newpassword123", min_length = 8)]
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({"message": "Operation completed successfully"}))]
pub struct MessageResponse {
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

/// Register new user
/// 
/// Creates a new user account and sends email verification.
/// 
/// # Authentication
/// No authentication required (public endpoint).
/// 
/// # Returns
/// - `201 Created`: User registered successfully, verification email sent
/// - `400 Bad Request`: Invalid input or email already exists
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 400, description = "Email already exists", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::Validation(format!("Validation error: {}", e)))?;

    // Create auth service
    let user_repo = UserRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, state.config.clone());

    // Register user
    let user = auth_service.register(payload).await?;

    Ok((StatusCode::CREATED, Json(user)))
}

/// User login
/// 
/// Authenticates user with email and password, returns JWT tokens.
/// 
/// # Authentication
/// No authentication required (public endpoint).
/// 
/// # Returns
/// - `200 OK`: Login successful, returns access token and refresh token
/// - `401 Unauthorized`: Invalid email or password
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::Validation(format!("Validation error: {}", e)))?;

    // Create auth service
    let user_repo = UserRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, state.config.clone());

    // Login user
    let response = auth_service.login(payload).await?;

    Ok(Json(response))
}

/// Verify email address
/// 
/// Verifies user email using the token sent via email.
/// 
/// # Authentication
/// No authentication required (public endpoint).
/// 
/// # Returns
/// - `200 OK`: Email verified successfully
/// - `400 Bad Request`: Invalid or expired token
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-email",
    tag = "Authentication",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = UserResponse),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse)
    )
)]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> AppResult<Json<UserResponse>> {
    // Create auth service
    let user_repo = UserRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, state.config.clone());

    // Verify email
    let user = auth_service.verify_email(&payload.token).await?;

    Ok(Json(user))
}

/// Request password reset
/// 
/// Sends password reset email to the user.
/// 
/// # Authentication
/// No authentication required (public endpoint).
/// 
/// # Returns
/// - `200 OK`: Password reset email sent
/// - `404 Not Found`: Email not found
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "Authentication",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = MessageResponse),
        (status = 404, description = "Email not found", body = ErrorResponse)
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    // Create auth service
    let user_repo = UserRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, state.config.clone());

    // Send reset password email
    auth_service.forgot_password(&payload.email).await?;

    Ok(Json(MessageResponse {
        message: "Password reset email sent. Please check your email.".to_string(),
    }))
}

/// Reset password
/// 
/// Resets user password using the token from email.
/// 
/// # Authentication
/// No authentication required (public endpoint).
/// 
/// # Returns
/// - `200 OK`: Password reset successfully
/// - `400 Bad Request`: Invalid or expired token
/// - `422 Unprocessable Entity`: Validation error (password too short)
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    tag = "Authentication",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = MessageResponse),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::Validation(format!("Validation error: {}", e)))?;

    // Create auth service
    let user_repo = UserRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, state.config.clone());

    // Reset password
    auth_service
        .reset_password(&payload.token, &payload.new_password)
        .await?;

    Ok(Json(MessageResponse {
        message: "Password reset successfully. You can now login with your new password."
            .to_string(),
    }))
}


/// Refresh access token
/// 
/// Generates a new access token using refresh token.
/// 
/// # Authentication
/// No authentication required (uses refresh token).
/// 
/// # Returns
/// - `200 OK`: New access token generated
/// - `401 Unauthorized`: Invalid or expired refresh token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "Authentication",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshTokenResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse)
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    // Create auth service
    let user_repo = UserRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, state.config.clone());

    // Refresh token
    let response = auth_service.refresh_token(&payload.refresh_token).await?;

    Ok(Json(response))
}

/// Logout user
/// 
/// Logs out the current user. Client should delete the token.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Returns
/// - `200 OK`: Logged out successfully
/// - `401 Unauthorized`: Missing or invalid token
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Authentication",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Logged out successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn logout(req: Request) -> AppResult<Json<MessageResponse>> {
    // Extract authenticated user from extensions
    let _auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // In a production system, you would:
    // 1. Invalidate the token in Redis/database
    // 2. Add token to blacklist
    // For now, we just return success and client should delete the token

    Ok(Json(MessageResponse {
        message: "Logged out successfully".to_string(),
    }))
}

/// Get current user
/// 
/// Returns information about the currently authenticated user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Returns
/// - `200 OK`: User information
/// - `401 Unauthorized`: Missing or invalid token
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "Authentication",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user information", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn get_current_user(req: Request) -> AppResult<Json<UserResponse>> {
    // Extract authenticated user from extensions
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    Ok(Json(UserResponse {
        id: auth_user.id,
        email: auth_user.email.clone(),
        full_name: "".to_string(), // We'll need to fetch from DB if needed
        role: auth_user.role.clone(),
        school_id: auth_user.school_id,
    }))
}
