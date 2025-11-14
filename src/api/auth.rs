use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize)]
struct VerifyEmailRequest {
    token: String,
}

#[derive(Debug, Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

#[derive(Debug, Deserialize, Validate)]
struct ResetPasswordRequest {
    token: String,
    #[validate(length(min = 8))]
    new_password: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

async fn register(
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

async fn login(
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

async fn verify_email(
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

async fn forgot_password(
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

async fn reset_password(
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


async fn refresh_token(
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

async fn logout(req: Request) -> AppResult<Json<MessageResponse>> {
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

async fn get_current_user(req: Request) -> AppResult<Json<UserResponse>> {
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
