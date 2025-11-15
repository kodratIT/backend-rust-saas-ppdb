use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

/// Query untuk cek hasil seleksi
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CheckResultQuery {
    /// Nomor pendaftaran
    #[schema(example = "PPDB-2024-001")]
    registration_number: String,
    
    /// NISN siswa
    #[schema(example = "0012345678")]
    student_nisn: String,
}

/// Response jalankan seleksi
#[derive(Debug, Serialize, ToSchema)]
pub struct RunSelectionResponse {
    /// Pesan hasil
    #[schema(example = "Selection completed successfully. 100 accepted, 50 rejected")]
    message: String,
    
    /// Detail hasil seleksi
    result: SelectionResult,
}

/// Response pengumuman hasil
#[derive(Debug, Serialize, ToSchema)]
pub struct AnnounceResultsResponse {
    /// Pesan hasil
    #[schema(example = "Results announced successfully. 150 notifications sent (100 accepted, 50 rejected)")]
    message: String,
    
    /// Detail hasil pengumuman
    result: AnnouncementResult,
}

/// Jalankan proses seleksi
///
/// Endpoint ini digunakan untuk menjalankan proses seleksi otomatis berdasarkan ranking dan kuota.
/// Hanya dapat diakses oleh admin sekolah.
#[utoipa::path(
    post,
    path = "/api/announcements/periods/{period_id}/run-selection",
    tag = "Selection",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Seleksi berhasil dijalankan", body = RunSelectionResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn run_selection(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(period_id): Path<i32>,
) -> AppResult<Json<RunSelectionResponse>> {

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

/// Umumkan hasil seleksi
///
/// Endpoint ini digunakan untuk mengumumkan hasil seleksi kepada peserta.
/// Hanya dapat diakses oleh admin sekolah.
#[utoipa::path(
    post,
    path = "/api/announcements/periods/{period_id}/announce",
    tag = "Selection",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Hasil berhasil diumumkan", body = AnnounceResultsResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn announce_results(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(period_id): Path<i32>,
) -> AppResult<Json<AnnounceResultsResponse>> {

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

/// Mendapatkan ringkasan seleksi
///
/// Endpoint ini mengembalikan ringkasan hasil seleksi untuk periode tertentu.
#[utoipa::path(
    get,
    path = "/api/announcements/periods/{period_id}/summary",
    tag = "Selection",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Ringkasan berhasil diambil", body = SelectionSummary),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Cek hasil seleksi (public)
///
/// Endpoint publik untuk mengecek hasil seleksi menggunakan nomor pendaftaran dan NISN.
#[utoipa::path(
    get,
    path = "/api/announcements/check-result",
    tag = "Selection",
    params(CheckResultQuery),
    responses(
        (status = 200, description = "Hasil berhasil ditemukan", body = ResultCheckResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 404, description = "Hasil tidak ditemukan")
    )
)]
async fn check_result(
    State(state): State<AppState>,
    Query(query): Query<CheckResultQuery>,
) -> AppResult<Json<ResultCheckResponse>> {
    // Validate NISN length
    if query.student_nisn.len() != 10 {
        return Err(AppError::Validation("NISN must be 10 characters".to_string()));
    }

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
