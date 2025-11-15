use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use validator::Validate;

use crate::api::middleware::auth::auth_middleware;
use crate::api::middleware::rbac::require_super_admin;
use crate::models::school::School;
use crate::repositories::school_repo::SchoolRepository;
use crate::services::school_service::SchoolService;
use crate::utils::error::AppResult;
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_schools).post(create_school))
        .route("/:id", get(get_school).put(update_school).delete(deactivate_school))
        .route("/:id/activate", post(activate_school))
        .route_layer(middleware::from_fn(require_super_admin))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

/// Create school request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "name": "SMA Negeri 1 Jakarta",
    "npsn": "12345678",
    "code": "SMAN1JKT",
    "address": "Jl. Sudirman No. 1, Jakarta Pusat",
    "phone": "+62211234567",
    "email": "info@sman1jakarta.sch.id",
    "logo_url": "https://example.com/logo.png"
}))]
pub struct CreateSchoolRequest {
    /// School name (minimum 3 characters)
    #[validate(length(min = 3))]
    #[schema(example = "SMA Negeri 1 Jakarta", min_length = 3)]
    name: String,
    
    /// NPSN - Nomor Pokok Sekolah Nasional (exactly 8 digits)
    #[validate(length(min = 8, max = 8))]
    #[schema(example = "12345678", min_length = 8, max_length = 8)]
    npsn: String,
    
    /// Unique school code (minimum 3 characters)
    #[validate(length(min = 3))]
    #[schema(example = "SMAN1JKT", min_length = 3)]
    code: String,
    
    /// School address
    #[schema(example = "Jl. Sudirman No. 1, Jakarta Pusat")]
    address: Option<String>,
    
    /// Contact phone number
    #[schema(example = "+62211234567")]
    phone: Option<String>,
    
    /// Contact email
    #[validate(email)]
    #[schema(example = "info@sman1jakarta.sch.id", format = "email")]
    email: Option<String>,
    
    /// Logo URL
    #[schema(example = "https://example.com/logo.png", format = "uri")]
    logo_url: Option<String>,
}

/// Update school request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "name": "SMA Negeri 1 Jakarta Pusat",
    "address": "Jl. Sudirman No. 1, Jakarta Pusat 10110",
    "phone": "+62211234567",
    "email": "info@sman1jakarta.sch.id"
}))]
pub struct UpdateSchoolRequest {
    /// School name (minimum 3 characters)
    #[validate(length(min = 3))]
    #[schema(example = "SMA Negeri 1 Jakarta Pusat", min_length = 3)]
    name: Option<String>,
    
    /// School address
    #[schema(example = "Jl. Sudirman No. 1, Jakarta Pusat 10110")]
    address: Option<String>,
    
    /// Contact phone number
    #[schema(example = "+62211234567")]
    phone: Option<String>,
    
    /// Contact email
    #[validate(email)]
    #[schema(example = "info@sman1jakarta.sch.id", format = "email")]
    email: Option<String>,
    
    /// Logo URL
    #[schema(example = "https://example.com/logo.png", format = "uri")]
    logo_url: Option<String>,
}

/// List schools query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListSchoolsQuery {
    /// Page number (starts from 1)
    #[serde(default = "default_page")]
    #[param(example = 1, minimum = 1)]
    page: i64,
    
    /// Items per page
    #[serde(default = "default_page_size")]
    #[param(example = 10, minimum = 1, maximum = 100)]
    page_size: i64,
    
    /// Search query (searches in name, NPSN, code)
    #[param(example = "Jakarta")]
    search: Option<String>,
    
    /// Filter by status (active, inactive)
    #[param(example = "active")]
    status: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

/// School response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": 1,
    "name": "SMA Negeri 1 Jakarta",
    "npsn": "12345678",
    "code": "SMAN1JKT",
    "address": "Jl. Sudirman No. 1, Jakarta Pusat",
    "phone": "+62211234567",
    "email": "info@sman1jakarta.sch.id",
    "logo_url": "https://example.com/logo.png",
    "status": "active"
}))]
pub struct SchoolResponse {
    /// School ID
    #[schema(example = 1)]
    id: i32,
    
    /// School name
    #[schema(example = "SMA Negeri 1 Jakarta")]
    name: String,
    
    /// NPSN - Nomor Pokok Sekolah Nasional
    #[schema(example = "12345678")]
    npsn: String,
    
    /// Unique school code
    #[schema(example = "SMAN1JKT")]
    code: String,
    
    /// School address
    #[schema(example = "Jl. Sudirman No. 1, Jakarta Pusat")]
    address: Option<String>,
    
    /// Contact phone number
    #[schema(example = "+62211234567")]
    phone: Option<String>,
    
    /// Contact email
    #[schema(example = "info@sman1jakarta.sch.id")]
    email: Option<String>,
    
    /// Logo URL
    #[schema(example = "https://example.com/logo.png")]
    logo_url: Option<String>,
    
    /// School status (active, inactive)
    #[schema(example = "active")]
    status: String,
}

impl From<School> for SchoolResponse {
    fn from(school: School) -> Self {
        Self {
            id: school.id,
            name: school.name,
            npsn: school.npsn,
            code: school.code,
            address: school.address,
            phone: school.phone,
            email: school.email,
            logo_url: school.logo_url,
            status: school.status,
        }
    }
}

/// List schools response with pagination
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "schools": [{
        "id": 1,
        "name": "SMA Negeri 1 Jakarta",
        "npsn": "12345678",
        "code": "SMAN1JKT",
        "address": "Jl. Sudirman No. 1, Jakarta Pusat",
        "phone": "+62211234567",
        "email": "info@sman1jakarta.sch.id",
        "logo_url": "https://example.com/logo.png",
        "status": "active"
    }],
    "total": 100,
    "page": 1,
    "page_size": 10,
    "total_pages": 10
}))]
pub struct ListSchoolsResponse {
    /// List of schools
    pub schools: Vec<SchoolResponse>,
    
    /// Total number of schools
    #[schema(example = 100)]
    pub total: i64,
    
    /// Current page number
    #[schema(example = 1)]
    pub page: i64,
    
    /// Items per page
    #[schema(example = 10)]
    pub page_size: i64,
    
    /// Total number of pages
    #[schema(example = 10)]
    pub total_pages: i64,
}

/// Success message response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({"message": "Operation completed successfully"}))]
pub struct MessageResponse {
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

/// Create new school
/// 
/// Creates a new school in the system. Only SuperAdmin can create schools.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Business Rules
/// - NPSN must be exactly 8 digits
/// - School code must be unique
/// - NPSN must be unique
/// 
/// # Returns
/// - `201 Created`: School created successfully
/// - `400 Bad Request`: NPSN or code already exists
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    post,
    path = "/api/v1/schools",
    tag = "Schools",
    request_body = CreateSchoolRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "School created successfully", body = SchoolResponse),
        (status = 400, description = "NPSN or code already exists", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - SuperAdmin only", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
pub async fn create_school(
    State(state): State<AppState>,
    Json(payload): Json<CreateSchoolRequest>,
) -> AppResult<(StatusCode, Json<SchoolResponse>)> {
    // Validate request
    payload.validate().map_err(|e| {
        crate::utils::error::AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create school service
    let school_repo = SchoolRepository::new(state.db.clone());
    let school_service = SchoolService::new(school_repo);

    // Create school
    let school = school_service
        .create_school(
            payload.name,
            payload.npsn,
            payload.code,
            payload.address,
            payload.phone,
            payload.email,
            payload.logo_url,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(school.into())))
}

/// List all schools
/// 
/// Returns a paginated list of all schools. Only SuperAdmin can list all schools.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Query Parameters
/// - `page`: Page number (default: 1)
/// - `page_size`: Items per page (default: 10, max: 100)
/// - `search`: Search in name, NPSN, or code
/// - `status`: Filter by status (active, inactive)
/// 
/// # Returns
/// - `200 OK`: List of schools with pagination
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
#[utoipa::path(
    get,
    path = "/api/v1/schools",
    tag = "Schools",
    params(ListSchoolsQuery),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of schools", body = ListSchoolsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - SuperAdmin only", body = ErrorResponse)
    )
)]
pub async fn list_schools(
    State(state): State<AppState>,
    Query(query): Query<ListSchoolsQuery>,
) -> AppResult<Json<ListSchoolsResponse>> {
    // Create school service
    let school_repo = SchoolRepository::new(state.db.clone());
    let school_service = SchoolService::new(school_repo);

    // List schools
    let (schools, total) = school_service
        .list_schools(query.page, query.page_size, query.search, query.status)
        .await?;

    let total_pages = (total as f64 / query.page_size as f64).ceil() as i64;

    Ok(Json(ListSchoolsResponse {
        schools: schools.into_iter().map(|s| s.into()).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    }))
}

/// Get school details
/// 
/// Returns detailed information about a specific school.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Path Parameters
/// - `id`: School ID
/// 
/// # Returns
/// - `200 OK`: School details
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
/// - `404 Not Found`: School not found
#[utoipa::path(
    get,
    path = "/api/v1/schools/{id}",
    tag = "Schools",
    params(
        ("id" = i32, Path, description = "School ID", example = 1)
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "School details", body = SchoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - SuperAdmin only", body = ErrorResponse),
        (status = 404, description = "School not found", body = ErrorResponse)
    )
)]
pub async fn get_school(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<SchoolResponse>> {
    // Create school service
    let school_repo = SchoolRepository::new(state.db.clone());
    let school_service = SchoolService::new(school_repo);

    // Get school
    let school = school_service.get_school(id).await?;

    Ok(Json(school.into()))
}

/// Update school
/// 
/// Updates school information. Only SuperAdmin can update schools.
/// NPSN and code cannot be updated.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Path Parameters
/// - `id`: School ID
/// 
/// # Returns
/// - `200 OK`: School updated successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
/// - `404 Not Found`: School not found
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    put,
    path = "/api/v1/schools/{id}",
    tag = "Schools",
    params(
        ("id" = i32, Path, description = "School ID", example = 1)
    ),
    request_body = UpdateSchoolRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "School updated successfully", body = SchoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - SuperAdmin only", body = ErrorResponse),
        (status = 404, description = "School not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
pub async fn update_school(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateSchoolRequest>,
) -> AppResult<Json<SchoolResponse>> {
    // Validate request
    payload.validate().map_err(|e| {
        crate::utils::error::AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create school service
    let school_repo = SchoolRepository::new(state.db.clone());
    let school_service = SchoolService::new(school_repo);

    // Update school
    let school = school_service
        .update_school(
            id,
            payload.name,
            payload.address,
            payload.phone,
            payload.email,
            payload.logo_url,
        )
        .await?;

    Ok(Json(school.into()))
}

/// Deactivate school
/// 
/// Soft deletes a school by setting status to inactive.
/// Only SuperAdmin can deactivate schools.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Path Parameters
/// - `id`: School ID
/// 
/// # Returns
/// - `200 OK`: School deactivated successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
/// - `404 Not Found`: School not found
#[utoipa::path(
    delete,
    path = "/api/v1/schools/{id}",
    tag = "Schools",
    params(
        ("id" = i32, Path, description = "School ID", example = 1)
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "School deactivated successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - SuperAdmin only", body = ErrorResponse),
        (status = 404, description = "School not found", body = ErrorResponse)
    )
)]
pub async fn deactivate_school(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<MessageResponse>> {
    // Create school service
    let school_repo = SchoolRepository::new(state.db.clone());
    let school_service = SchoolService::new(school_repo);

    // Deactivate school
    school_service.deactivate_school(id).await?;

    Ok(Json(MessageResponse {
        message: "School deactivated successfully".to_string(),
    }))
}

/// Activate school
/// 
/// Activates a previously deactivated school.
/// Only SuperAdmin can activate schools.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Path Parameters
/// - `id`: School ID
/// 
/// # Returns
/// - `200 OK`: School activated successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
/// - `404 Not Found`: School not found
#[utoipa::path(
    post,
    path = "/api/v1/schools/{id}/activate",
    tag = "Schools",
    params(
        ("id" = i32, Path, description = "School ID", example = 1)
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "School activated successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - SuperAdmin only", body = ErrorResponse),
        (status = 404, description = "School not found", body = ErrorResponse)
    )
)]
pub async fn activate_school(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<MessageResponse>> {
    // Create school service
    let school_repo = SchoolRepository::new(state.db.clone());
    let school_service = SchoolService::new(school_repo);

    // Activate school
    school_service.activate_school(id).await?;

    Ok(Json(MessageResponse {
        message: "School activated successfully".to_string(),
    }))
}
