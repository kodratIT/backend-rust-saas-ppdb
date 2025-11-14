use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::api::middleware::auth::{auth_middleware, AuthUser};
use crate::api::middleware::rbac::require_school_admin;
use crate::models::period::{Period, RegistrationPath};
use crate::repositories::period_repo::PeriodRepository;
use crate::services::period_service::PeriodService;
use crate::utils::error::{AppError, AppResult};
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_periods).post(create_period))
        .route("/:id", get(get_period).put(update_period).delete(delete_period))
        .route("/:id/activate", post(activate_period))
        .route("/:id/close", post(close_period))
        .route("/:id/paths", get(get_paths).post(create_path))
        .route("/paths/:path_id", put(update_path).delete(delete_path))
        .route_layer(middleware::from_fn(require_school_admin))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

#[derive(Debug, Deserialize, Validate)]
struct CreatePeriodRequest {
    #[validate(length(min = 9, max = 9))]
    academic_year: String, // e.g., "2024/2025"
    
    #[validate(custom = "validate_level")]
    level: String,
    
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    reenrollment_deadline: Option<DateTime<Utc>>,
    
    paths: Vec<CreatePathRequest>,
}

fn validate_level(level: &str) -> Result<(), validator::ValidationError> {
    match level.to_uppercase().as_str() {
        "SD" | "SMP" | "SMA" | "SMK" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_level")),
    }
}

#[derive(Debug, Deserialize, Validate)]
struct CreatePathRequest {
    #[validate(custom = "validate_path_type")]
    path_type: String,
    
    #[validate(length(min = 3))]
    name: String,
    
    #[validate(range(min = 1))]
    quota: i32,
    
    description: Option<String>,
    scoring_config: serde_json::Value,
}

fn validate_path_type(path_type: &str) -> Result<(), validator::ValidationError> {
    match path_type {
        "zonasi" | "prestasi" | "afirmasi" | "perpindahan_tugas" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_path_type")),
    }
}

#[derive(Debug, Deserialize, Validate)]
struct UpdatePeriodRequest {
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    announcement_date: Option<DateTime<Utc>>,
    reenrollment_deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdatePathRequest {
    #[validate(length(min = 3))]
    name: Option<String>,
    
    #[validate(range(min = 1))]
    quota: Option<i32>,
    
    description: Option<String>,
    scoring_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ListPeriodsQuery {
    #[serde(default = "default_page")]
    page: i64,
    
    #[serde(default = "default_page_size")]
    page_size: i64,
    
    status: Option<String>,
    academic_year: Option<String>,
    level: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

#[derive(Debug, Serialize)]
struct PeriodResponse {
    id: i32,
    school_id: i32,
    academic_year: String,
    level: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    announcement_date: Option<DateTime<Utc>>,
    reenrollment_deadline: Option<DateTime<Utc>>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Period> for PeriodResponse {
    fn from(period: Period) -> Self {
        Self {
            id: period.id,
            school_id: period.school_id,
            academic_year: period.academic_year,
            level: period.level,
            start_date: period.start_date,
            end_date: period.end_date,
            announcement_date: period.announcement_date,
            reenrollment_deadline: period.reenrollment_deadline,
            status: period.status,
            created_at: period.created_at,
            updated_at: period.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
struct PeriodWithPathsResponse {
    #[serde(flatten)]
    period: PeriodResponse,
    paths: Vec<PathResponse>,
}

#[derive(Debug, Serialize)]
struct PathResponse {
    id: i32,
    period_id: i32,
    path_type: String,
    name: String,
    quota: i32,
    description: Option<String>,
    scoring_config: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<RegistrationPath> for PathResponse {
    fn from(path: RegistrationPath) -> Self {
        Self {
            id: path.id,
            period_id: path.period_id,
            path_type: path.path_type,
            name: path.name,
            quota: path.quota,
            description: path.description,
            scoring_config: path.scoring_config,
            created_at: path.created_at,
            updated_at: path.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
struct ListPeriodsResponse {
    periods: Vec<PeriodResponse>,
    total: i64,
    page: i64,
    page_size: i64,
    total_pages: i64,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

async fn create_period(
    State(state): State<AppState>,
    req: Request,
    Json(payload): Json<CreatePeriodRequest>,
) -> AppResult<(StatusCode, Json<PeriodWithPathsResponse>)> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    let school_id = auth_user.school_id.ok_or_else(|| {
        AppError::Authentication("User must be associated with a school".to_string())
    })?;

    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Create period
    let period = period_service
        .create_period(
            school_id,
            payload.academic_year,
            payload.level,
            payload.start_date,
            payload.end_date,
            payload.reenrollment_deadline,
        )
        .await?;

    // Create paths
    let mut paths = Vec::new();
    for path_req in payload.paths {
        let path = period_service
            .create_path(
                period.id,
                path_req.path_type,
                path_req.name,
                path_req.quota,
                path_req.description,
                path_req.scoring_config,
            )
            .await?;
        paths.push(path.into());
    }

    Ok((
        StatusCode::CREATED,
        Json(PeriodWithPathsResponse {
            period: period.into(),
            paths,
        }),
    ))
}

async fn list_periods(
    State(state): State<AppState>,
    req: Request,
    Query(query): Query<ListPeriodsQuery>,
) -> AppResult<Json<ListPeriodsResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    let school_id = auth_user.school_id.ok_or_else(|| {
        AppError::Authentication("User must be associated with a school".to_string())
    })?;

    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // List periods
    let (periods, total) = period_service
        .list_periods(
            school_id,
            query.page,
            query.page_size,
            query.status,
            query.academic_year,
            query.level,
        )
        .await?;

    let total_pages = (total as f64 / query.page_size as f64).ceil() as i64;

    Ok(Json(ListPeriodsResponse {
        periods: periods.into_iter().map(|p| p.into()).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    }))
}

async fn get_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<PeriodWithPathsResponse>> {
    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Get period with paths
    let (period, paths) = period_service.get_period_with_paths(id).await?;

    Ok(Json(PeriodWithPathsResponse {
        period: period.into(),
        paths: paths.into_iter().map(|p| p.into()).collect(),
    }))
}

async fn update_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePeriodRequest>,
) -> AppResult<Json<PeriodResponse>> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Update period
    let period = period_service
        .update_period(
            id,
            payload.start_date,
            payload.end_date,
            payload.announcement_date,
            payload.reenrollment_deadline,
        )
        .await?;

    Ok(Json(period.into()))
}

async fn delete_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<MessageResponse>> {
    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Delete period
    period_service.delete_period(id).await?;

    Ok(Json(MessageResponse {
        message: "Period deleted successfully".to_string(),
    }))
}

async fn activate_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<PeriodResponse>> {
    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Activate period
    let period = period_service.activate_period(id).await?;

    Ok(Json(period.into()))
}

async fn close_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<PeriodResponse>> {
    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Close period
    let period = period_service.close_period(id).await?;

    Ok(Json(period.into()))
}

async fn get_paths(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
) -> AppResult<Json<Vec<PathResponse>>> {
    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Get paths
    let paths = period_service.get_paths_by_period(period_id).await?;

    Ok(Json(paths.into_iter().map(|p| p.into()).collect()))
}

async fn create_path(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
    Json(payload): Json<CreatePathRequest>,
) -> AppResult<(StatusCode, Json<PathResponse>)> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Create path
    let path = period_service
        .create_path(
            period_id,
            payload.path_type,
            payload.name,
            payload.quota,
            payload.description,
            payload.scoring_config,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(path.into())))
}

async fn update_path(
    State(state): State<AppState>,
    Path(path_id): Path<i32>,
    Json(payload): Json<UpdatePathRequest>,
) -> AppResult<Json<PathResponse>> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Update path
    let path = period_service
        .update_path(
            path_id,
            payload.name,
            payload.quota,
            payload.description,
            payload.scoring_config,
        )
        .await?;

    Ok(Json(path.into()))
}

async fn delete_path(
    State(state): State<AppState>,
    Path(path_id): Path<i32>,
) -> AppResult<Json<MessageResponse>> {
    // Create period service
    let period_repo = PeriodRepository::new(state.db.clone());
    let period_service = PeriodService::new(period_repo);

    // Delete path
    period_service.delete_path(path_id).await?;

    Ok(Json(MessageResponse {
        message: "Registration path deleted successfully".to_string(),
    }))
}
