use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::api::middleware::auth::{auth_middleware, AuthUser};
use crate::api::middleware::rbac::require_school_admin;
use crate::models::user::User;
use crate::repositories::user_repo::UserRepository;
use crate::services::user_service::UserService;
use crate::utils::error::{AppError, AppResult};
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/me", get(get_current_user_full).put(update_current_user))
        .route("/me/change-password", post(change_password))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Admin-only routes
    let admin_routes = Router::new()
        .route("/", post(create_user))
        .route("/:id", delete(delete_user))
        .route_layer(middleware::from_fn(require_school_admin))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    protected_routes
}

#[derive(Debug, Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
    
    #[validate(length(min = 3))]
    full_name: String,
    
    phone: Option<String>,
    nik: Option<String>,
    
    #[validate(custom = "validate_role")]
    role: String,
    
    school_id: Option<i32>,
}

fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    match role {
        "super_admin" | "school_admin" | "parent" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateUserRequest {
    #[validate(length(min = 3))]
    full_name: Option<String>,
    
    phone: Option<String>,
    nik: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct ChangePasswordRequest {
    old_password: String,
    
    #[validate(length(min = 8))]
    new_password: String,
}

#[derive(Debug, Deserialize)]
struct ListUsersQuery {
    #[serde(default = "default_page")]
    page: i64,
    
    #[serde(default = "default_page_size")]
    page_size: i64,
    
    search: Option<String>,
    role: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: i32,
    email: String,
    full_name: String,
    phone: Option<String>,
    nik: Option<String>,
    role: String,
    school_id: Option<i32>,
    email_verified: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            phone: user.phone,
            nik: user.nik,
            role: user.role,
            school_id: user.school_id,
            email_verified: user.email_verified,
        }
    }
}

#[derive(Debug, Serialize)]
struct ListUsersResponse {
    users: Vec<UserResponse>,
    total: i64,
    page: i64,
    page_size: i64,
    total_pages: i64,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

async fn create_user(
    State(state): State<AppState>,
    req: Request,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Determine school_id based on role
    let school_id = if auth_user.role == "super_admin" {
        payload.school_id
    } else {
        auth_user.school_id
    };

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Create user
    let user = user_service
        .create_user(
            school_id,
            payload.email,
            payload.password,
            payload.full_name,
            payload.phone,
            payload.nik,
            payload.role,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

async fn list_users(
    State(state): State<AppState>,
    req: Request,
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Json<ListUsersResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Determine school_id filter based on role
    let school_id_filter = if auth_user.role == "super_admin" {
        None
    } else {
        auth_user.school_id
    };

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // List users
    let (users, total) = user_service
        .list_users(school_id_filter, query.page, query.page_size, query.search, query.role)
        .await?;

    let total_pages = (total as f64 / query.page_size as f64).ceil() as i64;

    Ok(Json(ListUsersResponse {
        users: users.into_iter().map(|u| u.into()).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    }))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserResponse>> {
    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Get user
    let user = user_service.get_user(id).await?;

    Ok(Json(user.into()))
}

async fn get_current_user_full(
    State(state): State<AppState>,
    req: Request,
) -> AppResult<Json<UserResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Get user
    let user = user_service.get_user(auth_user.id).await?;

    Ok(Json(user.into()))
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Update user
    let user = user_service
        .update_user(id, payload.full_name, payload.phone, payload.nik)
        .await?;

    Ok(Json(user.into()))
}

async fn update_current_user(
    State(state): State<AppState>,
    req: Request,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Update user
    let user = user_service
        .update_user(auth_user.id, payload.full_name, payload.phone, payload.nik)
        .await?;

    Ok(Json(user.into()))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<MessageResponse>> {
    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Delete user
    user_service.delete_user(id).await?;

    Ok(Json(MessageResponse {
        message: "User deleted successfully".to_string(),
    }))
}

async fn change_password(
    State(state): State<AppState>,
    req: Request,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Change password
    user_service
        .change_password(auth_user.id, payload.old_password, payload.new_password)
        .await?;

    Ok(Json(MessageResponse {
        message: "Password changed successfully".to_string(),
    }))
}
