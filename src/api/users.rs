use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

use crate::api::middleware::auth::{auth_middleware, AuthUser};
use crate::api::middleware::rbac::require_school_admin;
use crate::models::user::User;
use crate::repositories::user_repo::UserRepository;
use crate::services::user_service::UserService;
use crate::utils::error::{AppError, AppResult};
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    // Public routes (with auth)
    let public_routes = Router::new()
        .route("/", get(list_users))
        .route("/:id", get(get_user).put(update_user))
        .route("/me", get(get_current_user_full).put(update_current_user))
        .route("/me/change-password", post(change_password))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Admin-only routes
    let admin_routes = Router::new()
        .route("/", post(create_user))
        .route("/:id", delete(delete_user))
        .route_layer(middleware::from_fn(require_school_admin))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    public_routes.merge(admin_routes)
}

/// Create user request (Admin only)
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "teacher@example.com",
    "password": "password123",
    "full_name": "Ahmad Rizki",
    "phone": "+628123456789",
    "nik": "3201234567890123",
    "role": "school_admin",
    "school_id": 1
}))]
pub struct CreateUserRequest {
    /// User email address
    #[schema(example = "teacher@example.com", format = "email")]
    email: String,
    
    /// User password (minimum 8 characters)
    #[schema(example = "password123", min_length = 8)]
    password: String,
    
    /// User full name
    #[schema(example = "Ahmad Rizki")]
    full_name: String,
    
    /// Phone number (optional)
    #[schema(example = "+628123456789")]
    phone: Option<String>,
    
    /// NIK - Nomor Induk Kependudukan (optional)
    #[schema(example = "3201234567890123", min_length = 16, max_length = 16)]
    nik: Option<String>,
    
    /// User role (super_admin, school_admin, parent)
    #[schema(example = "school_admin")]
    role: String,
    
    /// School ID (required for school_admin and parent, null for super_admin)
    #[schema(example = 1)]
    school_id: Option<i32>,
}

/// Update user request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "full_name": "Ahmad Rizki Pratama",
    "phone": "+628123456789",
    "nik": "3201234567890123"
}))]
pub struct UpdateUserRequest {
    /// User full name
    #[schema(example = "Ahmad Rizki Pratama")]
    full_name: Option<String>,
    
    /// Phone number
    #[schema(example = "+628123456789")]
    phone: Option<String>,
    
    /// NIK - Nomor Induk Kependudukan
    #[schema(example = "3201234567890123", min_length = 16, max_length = 16)]
    nik: Option<String>,
}

/// Change password request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "old_password": "oldpassword123",
    "new_password": "newpassword123"
}))]
pub struct ChangePasswordRequest {
    /// Current password
    #[schema(example = "oldpassword123")]
    old_password: String,
    
    /// New password (minimum 8 characters)
    #[schema(example = "newpassword123", min_length = 8)]
    new_password: String,
}

/// List users query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListUsersQuery {
    /// Page number (starts from 1)
    #[serde(default = "default_page")]
    #[param(example = 1, minimum = 1)]
    page: i64,
    
    /// Items per page
    #[serde(default = "default_page_size")]
    #[param(example = 10, minimum = 1, maximum = 100)]
    page_size: i64,
    
    /// Search query (searches in name, email)
    #[param(example = "Ahmad")]
    search: Option<String>,
    
    /// Filter by role (super_admin, school_admin, parent)
    #[param(example = "school_admin")]
    role: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

/// User response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": 1,
    "email": "user@example.com",
    "full_name": "Ahmad Rizki",
    "phone": "+628123456789",
    "nik": "3201234567890123",
    "role": "school_admin",
    "school_id": 1,
    "email_verified": true
}))]
pub struct UserResponse {
    #[schema(example = 1)]
    pub id: i32,
    
    #[schema(example = "user@example.com")]
    pub email: String,
    
    #[schema(example = "Ahmad Rizki")]
    pub full_name: String,
    
    #[schema(example = "+628123456789")]
    pub phone: Option<String>,
    
    #[schema(example = "3201234567890123")]
    pub nik: Option<String>,
    
    #[schema(example = "school_admin")]
    pub role: String,
    
    #[schema(example = 1)]
    pub school_id: Option<i32>,
    
    #[schema(example = true)]
    pub email_verified: bool,
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

/// List users response with pagination
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "users": [{
        "id": 1,
        "email": "user@example.com",
        "full_name": "Ahmad Rizki",
        "phone": "+628123456789",
        "nik": "3201234567890123",
        "role": "school_admin",
        "school_id": 1,
        "email_verified": true
    }],
    "total": 50,
    "page": 1,
    "page_size": 10,
    "total_pages": 5
}))]
pub struct ListUsersResponse {
    pub users: Vec<UserResponse>,
    
    #[schema(example = 50)]
    pub total: i64,
    
    #[schema(example = 1)]
    pub page: i64,
    
    #[schema(example = 10)]
    pub page_size: i64,
    
    #[schema(example = 5)]
    pub total_pages: i64,
}

/// Success message response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({"message": "Operation completed successfully"}))]
pub struct MessageResponse {
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

/// Create new user
/// 
/// Creates a new user account. Only SchoolAdmin and SuperAdmin can create users.
/// 
/// # Authentication
/// Requires JWT Bearer token with SchoolAdmin or SuperAdmin role.
/// 
/// # Returns
/// - `201 Created`: User created successfully
/// - `400 Bad Request`: Email already exists or invalid role
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not admin
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Email already exists", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    // Validate role
    if !["super_admin", "school_admin", "parent"].contains(&payload.role.as_str()) {
        return Err(AppError::Validation("Invalid role".to_string()));
    }

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

/// List users
/// 
/// Returns a paginated list of users. SuperAdmin sees all users,
/// SchoolAdmin only sees users from their school.
/// 
/// # Authentication
/// Requires JWT Bearer token.
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(ListUsersQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of users", body = ListUsersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Json<ListUsersResponse>> {

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

/// Get user details
/// 
/// Returns detailed information about a specific user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = i32, Path, description = "User ID", example = 1)),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User details", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn get_user(
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

/// Get current user profile
/// 
/// Returns complete profile information of the currently authenticated user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn get_current_user_full(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<Json<UserResponse>> {

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Get user
    let user = user_service.get_user(auth_user.id).await?;

    Ok(Json(user.into()))
}

/// Update user
/// 
/// Updates user information. Only name, phone, and NIK can be updated.
/// 
/// # Authentication
/// Requires JWT Bearer token.
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = i32, Path, description = "User ID", example = 1)),
    request_body = UpdateUserRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Update user
    let user = user_service
        .update_user(id, payload.full_name, payload.phone, payload.nik)
        .await?;

    Ok(Json(user.into()))
}

/// Update current user profile
/// 
/// Updates the profile of the currently authenticated user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
#[utoipa::path(
    put,
    path = "/api/v1/users/me",
    tag = "Users",
    request_body = UpdateUserRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn update_current_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {

    // Create user service
    let user_repo = UserRepository::new(state.db.clone());
    let user_service = UserService::new(user_repo);

    // Update user
    let user = user_service
        .update_user(auth_user.id, payload.full_name, payload.phone, payload.nik)
        .await?;

    Ok(Json(user.into()))
}

/// Delete user
/// 
/// Deletes a user account. Only admins can delete users.
/// 
/// # Authentication
/// Requires JWT Bearer token with SchoolAdmin or SuperAdmin role.
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = i32, Path, description = "User ID", example = 1)),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User deleted successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn delete_user(
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

/// Change password
/// 
/// Changes the password of the currently authenticated user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
#[utoipa::path(
    post,
    path = "/api/v1/users/me/change-password",
    tag = "Users",
    request_body = ChangePasswordRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Password changed successfully", body = MessageResponse),
        (status = 400, description = "Old password is incorrect", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    // Validate password length
    if payload.new_password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".to_string()));
    }

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
