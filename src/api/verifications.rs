use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::api::middleware::auth::{auth_middleware, AuthUser};
use crate::api::middleware::rbac::require_school_admin;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::services::verification_service::{VerificationService, VerificationStats};
use crate::utils::error::{AppError, AppResult};
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/pending", get(get_pending_verifications))
        .route("/stats", get(get_verification_stats))
        .route("/:id/verify", post(verify_registration))
        .route("/:id/reject", post(reject_registration))
        .route("/documents/:doc_id/verify", post(verify_document))
        .route_layer(middleware::from_fn(require_school_admin))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

#[derive(Debug, Deserialize)]
struct PendingVerificationsQuery {
    #[serde(default = "default_page")]
    page: i64,
    
    #[serde(default = "default_page_size")]
    page_size: i64,
    
    period_id: Option<i32>,
    path_id: Option<i32>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

#[derive(Debug, Deserialize)]
struct StatsQuery {
    period_id: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
struct RejectRegistrationRequest {
    #[validate(length(min = 10))]
    reason: String,
}

#[derive(Debug, Deserialize, Validate)]
struct VerifyDocumentRequest {
    #[validate(custom = "validate_verification_status")]
    verification_status: String,
    
    verification_notes: Option<String>,
}

fn validate_verification_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "approved" | "rejected" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_verification_status")),
    }
}

#[derive(Debug, Serialize)]
struct RegistrationResponse {
    id: i32,
    school_id: i32,
    user_id: i32,
    period_id: i32,
    path_id: i32,
    registration_number: Option<String>,
    student_nisn: String,
    student_name: String,
    student_gender: String,
    parent_name: String,
    parent_phone: String,
    status: String,
    rejection_reason: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::models::registration::Registration> for RegistrationResponse {
    fn from(reg: crate::models::registration::Registration) -> Self {
        Self {
            id: reg.id,
            school_id: reg.school_id,
            user_id: reg.user_id,
            period_id: reg.period_id,
            path_id: reg.path_id,
            registration_number: reg.registration_number,
            student_nisn: reg.student_nisn,
            student_name: reg.student_name,
            student_gender: reg.student_gender,
            parent_name: reg.parent_name,
            parent_phone: reg.parent_phone,
            status: reg.status,
            rejection_reason: reg.rejection_reason,
            created_at: reg.created_at,
            updated_at: reg.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
struct PendingVerificationsResponse {
    registrations: Vec<RegistrationResponse>,
    total: i64,
    page: i64,
    page_size: i64,
    total_pages: i64,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

async fn get_pending_verifications(
    State(state): State<AppState>,
    req: Request,
    Query(query): Query<PendingVerificationsQuery>,
) -> AppResult<Json<PendingVerificationsResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    let school_id = auth_user.school_id.ok_or_else(|| {
        AppError::Authentication("User must be associated with a school".to_string())
    })?;

    // Create verification service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let verification_service = VerificationService::new(registration_repo);

    // Get pending verifications
    let (registrations, total) = verification_service
        .get_pending_verifications(
            school_id,
            query.page,
            query.page_size,
            query.period_id,
            query.path_id,
        )
        .await?;

    let total_pages = (total as f64 / query.page_size as f64).ceil() as i64;

    Ok(Json(PendingVerificationsResponse {
        registrations: registrations.into_iter().map(|r| r.into()).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    }))
}

async fn get_verification_stats(
    State(state): State<AppState>,
    req: Request,
    Query(query): Query<StatsQuery>,
) -> AppResult<Json<VerificationStats>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    let school_id = auth_user.school_id.ok_or_else(|| {
        AppError::Authentication("User must be associated with a school".to_string())
    })?;

    // Create verification service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let verification_service = VerificationService::new(registration_repo);

    // Get statistics
    let stats = verification_service
        .get_verification_statistics(school_id, query.period_id)
        .await?;

    Ok(Json(stats))
}

async fn verify_registration(
    State(state): State<AppState>,
    req: Request,
    Path(id): Path<i32>,
) -> AppResult<Json<RegistrationResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Create verification service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let verification_service = VerificationService::new(registration_repo);

    // Verify registration
    let verified_registration = verification_service
        .verify_registration(id, auth_user.id)
        .await?;

    Ok(Json(verified_registration.into()))
}

async fn reject_registration(
    State(state): State<AppState>,
    req: Request,
    Path(id): Path<i32>,
    Json(payload): Json<RejectRegistrationRequest>,
) -> AppResult<Json<RegistrationResponse>> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Create verification service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let verification_service = VerificationService::new(registration_repo);

    // Reject registration
    let rejected_registration = verification_service
        .reject_registration(id, payload.reason, auth_user.id)
        .await?;

    Ok(Json(rejected_registration.into()))
}

async fn verify_document(
    State(state): State<AppState>,
    req: Request,
    Path(doc_id): Path<i32>,
    Json(payload): Json<VerifyDocumentRequest>,
) -> AppResult<Json<MessageResponse>> {
    // Validate request
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Create verification service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let verification_service = VerificationService::new(registration_repo);

    // Verify document
    verification_service
        .verify_document(
            doc_id,
            payload.verification_status,
            payload.verification_notes,
            auth_user.id,
        )
        .await?;

    Ok(Json(MessageResponse {
        message: "Document verification status updated successfully".to_string(),
    }))
}
