use axum::{
    extract::{Path, Query, Request, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::api::middleware::auth::{auth_middleware, AuthUser};
use crate::api::middleware::rbac::require_school_admin;
use crate::repositories::period_repo::PeriodRepository;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::services::announcement_service::{
    AnnouncementResult, AnnouncementService, ResultCheckResponse, SelectionResult,
    SelectionSummary,
};
use crate::utils::error::{AppError, AppResult};
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    // Admin routes (protected)
    let admin_routes = Router::new()
        .route("/periods/:period_id/run-selection", post(run_selection))
        .route("/periods/:period_id/announce", post(announce_results))
        .route("/periods/:period_id/summary", get(get_selection_summary))
        .route_layer(middleware::from_fn(require_school_admin))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Public routes
    let public_routes = Router::new().route("/check-result", get(check_result));

    admin_routes.merge(public_routes)
}

#[derive(Debug, Deserialize, Validate)]
struct CheckResultQuery {
    #[validate(length(min = 1))]
    registration_number: String,

    #[validate(length(equal = 10))]
    student_nisn: String,
}

#[derive(Debug, Serialize)]
struct RunSelectionResponse {
    message: String,
    result: SelectionResult,
}

#[derive(Debug, Serialize)]
struct AnnounceResultsResponse {
    message: String,
    result: AnnouncementResult,
}

async fn run_selection(
    State(state): State<AppState>,
    req: Request,
    Path(period_id): Path<i32>,
) -> AppResult<Json<RunSelectionResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Create announcement service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let announcement_service = AnnouncementService::new(registration_repo, period_repo);

    // Run selection
    let result = announcement_service
        .run_selection(period_id, auth_user.id)
        .await?;

    Ok(Json(RunSelectionResponse {
        message: format!(
            "Selection completed successfully. {} accepted, {} rejected",
            result.total_accepted, result.total_rejected
        ),
        result,
    }))
}

async fn announce_results(
    State(state): State<AppState>,
    req: Request,
    Path(period_id): Path<i32>,
) -> AppResult<Json<AnnounceResultsResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

    // Create announcement service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let announcement_service = AnnouncementService::new(registration_repo, period_repo);

    // Announce results
    let result = announcement_service
        .announce_results(period_id, auth_user.id)
        .await?;

    Ok(Json(AnnounceResultsResponse {
        message: format!(
            "Results announced successfully. {} notifications sent ({} accepted, {} rejected)",
            result.total_notified, result.accepted_notified, result.rejected_notified
        ),
        result,
    }))
}

async fn get_selection_summary(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
) -> AppResult<Json<SelectionSummary>> {
    // Create announcement service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let announcement_service = AnnouncementService::new(registration_repo, period_repo);

    // Get summary
    let summary = announcement_service
        .get_selection_summary(period_id)
        .await?;

    Ok(Json(summary))
}

async fn check_result(
    State(state): State<AppState>,
    Query(query): Query<CheckResultQuery>,
) -> AppResult<Json<ResultCheckResponse>> {
    // Validate query
    query.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Create announcement service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let announcement_service = AnnouncementService::new(registration_repo, period_repo);

    // Check result
    let result = announcement_service
        .check_result(query.registration_number, query.student_nisn)
        .await?;

    Ok(Json(result))
}
