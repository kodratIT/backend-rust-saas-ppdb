use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
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
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .route_layer(middleware::from_fn(require_super_admin))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateSchoolRequest {
    #[validate(length(min = 3))]
    name: String,
    
    #[validate(length(min = 8, max = 8))]
    npsn: String,
    
    #[validate(length(min = 3))]
    code: String,
    
    address: Option<String>,
    phone: Option<String>,
    
    #[validate(email)]
    email: Option<String>,
    
    logo_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateSchoolRequest {
    #[validate(length(min = 3))]
    name: Option<String>,
    
    address: Option<String>,
    phone: Option<String>,
    
    #[validate(email)]
    email: Option<String>,
    
    logo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListSchoolsQuery {
    #[serde(default = "default_page")]
    page: i64,
    
    #[serde(default = "default_page_size")]
    page_size: i64,
    
    search: Option<String>,
    status: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

#[derive(Debug, Serialize)]
struct SchoolResponse {
    id: i32,
    name: String,
    npsn: String,
    code: String,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    logo_url: Option<String>,
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

#[derive(Debug, Serialize)]
struct ListSchoolsResponse {
    schools: Vec<SchoolResponse>,
    total: i64,
    page: i64,
    page_size: i64,
    total_pages: i64,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

async fn create_school(
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

async fn list_schools(
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

async fn get_school(
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

async fn update_school(
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

async fn deactivate_school(
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

async fn activate_school(
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
