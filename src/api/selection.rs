use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

/// Query parameters untuk mendapatkan ranking
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetRankingsQuery {
    /// ID jalur pendaftaran
    #[schema(example = 1)]
    path_id: i32,
    
    /// Nomor halaman
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    page: i64,
    
    /// Jumlah item per halaman
    #[serde(default = "default_page_size")]
    #[schema(example = 50)]
    page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    50
}

/// Response perhitungan skor
#[derive(Debug, Serialize, ToSchema)]
pub struct CalculateScoresResponse {
    /// Pesan hasil
    #[schema(example = "Successfully calculated scores for 100 registrations")]
    message: String,
    
    /// Total pendaftaran yang dihitung
    #[schema(example = 100)]
    total_calculated: usize,
}

/// Response update ranking
#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateRankingsResponse {
    /// Pesan hasil
    #[schema(example = "Successfully updated rankings for 100 registrations")]
    message: String,
    
    /// Total pendaftaran yang diranking
    #[schema(example = 100)]
    total_ranked: usize,
}

/// Response data ranking
#[derive(Debug, Serialize, ToSchema)]
pub struct RankingResponse {
    /// ID pendaftaran
    #[schema(example = 1)]
    id: i32,
    
    /// Nomor pendaftaran
    #[schema(example = "PPDB-2024-001")]
    registration_number: Option<String>,
    
    /// NISN siswa
    #[schema(example = "0012345678")]
    student_nisn: String,
    
    /// Nama siswa
    #[schema(example = "Ahmad Fauzi")]
    student_name: String,
    
    /// Skor seleksi
    #[schema(example = 85.5)]
    selection_score: Option<f64>,
    
    /// Peringkat
    #[schema(example = 10)]
    ranking: Option<i32>,
    
    /// Status pendaftaran
    #[schema(example = "accepted")]
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

/// Response daftar ranking
#[derive(Debug, Serialize, ToSchema)]
pub struct RankingsResponse {
    /// Daftar ranking
    rankings: Vec<RankingResponse>,
    
    /// Total data
    #[schema(example = 100)]
    total: usize,
    
    /// Halaman saat ini
    #[schema(example = 1)]
    page: i64,
    
    /// Jumlah item per halaman
    #[schema(example = 50)]
    page_size: i64,
}

/// Hitung skor seleksi
///
/// Endpoint ini digunakan untuk menghitung skor seleksi semua pendaftaran dalam periode.
/// Hanya dapat diakses oleh admin sekolah.
#[utoipa::path(
    post,
    path = "/api/selection/periods/{period_id}/calculate-scores",
    tag = "Selection",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Skor berhasil dihitung", body = CalculateScoresResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Update ranking
///
/// Endpoint ini digunakan untuk mengupdate ranking berdasarkan skor seleksi.
/// Hanya dapat diakses oleh admin sekolah.
#[utoipa::path(
    post,
    path = "/api/selection/periods/{period_id}/update-rankings",
    tag = "Selection",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Ranking berhasil diupdate", body = UpdateRankingsResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Mendapatkan daftar ranking
///
/// Endpoint ini mengembalikan daftar ranking untuk periode dan jalur tertentu.
#[utoipa::path(
    get,
    path = "/api/selection/periods/{period_id}/rankings",
    tag = "Selection",
    params(
        ("period_id" = i32, Path, description = "ID periode"),
        GetRankingsQuery
    ),
    responses(
        (status = 200, description = "Daftar ranking berhasil diambil", body = RankingsResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Mendapatkan statistik ranking
///
/// Endpoint ini mengembalikan statistik ranking per jalur untuk periode tertentu.
#[utoipa::path(
    get,
    path = "/api/selection/periods/{period_id}/stats",
    tag = "Selection",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Statistik ranking berhasil diambil", body = Vec<PathRankingStats>),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
