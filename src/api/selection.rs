use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::auth::auth_middleware;
use crate::api::middleware::rbac::require_school_admin;
use crate::repositories::period_repo::PeriodRepository;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::services::selection_service::{PathRankingStats, SelectionService};
use crate::utils::error::AppResult;
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/periods/:period_id/calculate-scores", post(calculate_scores))
        .route("/periods/:period_id/update-rankings", post(update_rankings))
        .route("/periods/:period_id/rankings", get(get_rankings))
        .route("/periods/:period_id/stats", get(get_ranking_stats))
        .route_layer(middleware::from_fn(require_school_admin))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

#[derive(Debug, Deserialize)]
struct GetRankingsQuery {
    path_id: i32,
    
    #[serde(default = "default_page")]
    page: i64,
    
    #[serde(default = "default_page_size")]
    page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    50
}

#[derive(Debug, Serialize)]
struct CalculateScoresResponse {
    message: String,
    total_calculated: usize,
}

#[derive(Debug, Serialize)]
struct UpdateRankingsResponse {
    message: String,
    total_ranked: usize,
}

#[derive(Debug, Serialize)]
struct RankingResponse {
    id: i32,
    registration_number: Option<String>,
    student_nisn: String,
    student_name: String,
    selection_score: Option<f64>,
    ranking: Option<i32>,
    status: String,
}

impl From<crate::models::registration::Registration> for RankingResponse {
    fn from(reg: crate::models::registration::Registration) -> Self {
        Self {
            id: reg.id,
            registration_number: reg.registration_number,
            student_nisn: reg.student_nisn,
            student_name: reg.student_name,
            selection_score: reg.selection_score,
            ranking: reg.ranking,
            status: reg.status,
        }
    }
}

#[derive(Debug, Serialize)]
struct RankingsResponse {
    rankings: Vec<RankingResponse>,
    total: usize,
    page: i64,
    page_size: i64,
}

async fn calculate_scores(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
) -> AppResult<Json<CalculateScoresResponse>> {
    // Create selection service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let selection_service = SelectionService::new(registration_repo, period_repo);

    // Calculate scores
    let total_calculated = selection_service
        .calculate_scores_for_period(period_id)
        .await?;

    Ok(Json(CalculateScoresResponse {
        message: format!(
            "Successfully calculated scores for {} registrations",
            total_calculated
        ),
        total_calculated,
    }))
}

async fn update_rankings(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
) -> AppResult<Json<UpdateRankingsResponse>> {
    // Create selection service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let selection_service = SelectionService::new(registration_repo, period_repo);

    // Update rankings
    let total_ranked = selection_service.update_rankings(period_id).await?;

    Ok(Json(UpdateRankingsResponse {
        message: format!(
            "Successfully updated rankings for {} registrations",
            total_ranked
        ),
        total_ranked,
    }))
}

async fn get_rankings(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
    Query(query): Query<GetRankingsQuery>,
) -> AppResult<Json<RankingsResponse>> {
    // Create selection service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let selection_service = SelectionService::new(registration_repo, period_repo);

    let offset = (query.page - 1) * query.page_size;

    // Get rankings
    let rankings = selection_service
        .get_rankings(period_id, query.path_id, query.page_size, offset)
        .await?;

    let total = rankings.len();

    Ok(Json(RankingsResponse {
        rankings: rankings.into_iter().map(|r| r.into()).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
    }))
}

async fn get_ranking_stats(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
) -> AppResult<Json<Vec<PathRankingStats>>> {
    // Create selection service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let selection_service = SelectionService::new(registration_repo, period_repo);

    // Get statistics
    let stats = selection_service
        .get_ranking_statistics(period_id)
        .await?;

    Ok(Json(stats))
}
