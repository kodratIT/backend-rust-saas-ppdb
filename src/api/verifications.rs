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

/// Query untuk pending verifications
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PendingVerificationsQuery {
    /// Nomor halaman
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    page: i64,
    
    /// Jumlah item per halaman
    #[serde(default = "default_page_size")]
    #[schema(example = 10)]
    page_size: i64,
    
    /// Filter berdasarkan ID periode
    #[schema(example = 1)]
    period_id: Option<i32>,
    
    /// Filter berdasarkan ID jalur
    #[schema(example = 1)]
    path_id: Option<i32>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

/// Query untuk statistik verifikasi
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct StatsQuery {
    /// Filter berdasarkan ID periode
    #[schema(example = 1)]
    period_id: Option<i32>,
}

/// Request untuk menolak pendaftaran
#[derive(Debug, Deserialize, ToSchema)]
pub struct RejectRegistrationRequest {
    /// Alasan penolakan (minimal 10 karakter)
    #[schema(example = "Dokumen tidak lengkap atau tidak sesuai")]
    reason: String,
}

/// Request untuk verifikasi dokumen
#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyDocumentRequest {
    /// Status verifikasi (approved/rejected)
    #[schema(example = "approved")]
    verification_status: String,
    
    /// Catatan verifikasi (opsional)
    #[schema(example = "Dokumen sudah sesuai")]
    verification_notes: Option<String>,
}

/// Response data pendaftaran (simplified)
#[derive(Debug, Serialize, ToSchema)]
pub struct RegistrationResponse {
    /// ID pendaftaran
    #[schema(example = 1)]
    id: i32,
    
    /// ID sekolah
    #[schema(example = 1)]
    school_id: i32,
    
    /// ID user
    #[schema(example = 1)]
    user_id: i32,
    
    /// ID periode
    #[schema(example = 1)]
    period_id: i32,
    
    /// ID jalur
    #[schema(example = 1)]
    path_id: i32,
    
    /// Nomor pendaftaran
    #[schema(example = "PPDB-2024-001")]
    registration_number: Option<String>,
    
    /// NISN siswa
    #[schema(example = "0012345678")]
    student_nisn: String,
    
    /// Nama siswa
    #[schema(example = "Ahmad Fauzi")]
    student_name: String,
    
    /// Jenis kelamin
    #[schema(example = "L")]
    student_gender: String,
    
    /// Nama orang tua
    #[schema(example = "Budi Santoso")]
    parent_name: String,
    
    /// Telepon orang tua
    #[schema(example = "081234567890")]
    parent_phone: String,
    
    /// Status
    #[schema(example = "submitted")]
    status: String,
    
    /// Alasan penolakan
    #[schema(example = "Dokumen tidak lengkap")]
    rejection_reason: Option<String>,
    
    /// Waktu pembuatan
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    created_at: chrono::DateTime<chrono::Utc>,
    
    /// Waktu update
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
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

/// Response pending verifications
#[derive(Debug, Serialize, ToSchema)]
pub struct PendingVerificationsResponse {
    /// Daftar pendaftaran
    registrations: Vec<RegistrationResponse>,
    
    /// Total data
    #[schema(example = 50)]
    total: i64,
    
    /// Halaman saat ini
    #[schema(example = 1)]
    page: i64,
    
    /// Jumlah item per halaman
    #[schema(example = 10)]
    page_size: i64,
    
    /// Total halaman
    #[schema(example = 5)]
    total_pages: i64,
}

/// Response pesan sukses
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    /// Pesan
    #[schema(example = "Operation successful")]
    message: String,
}

/// Mendapatkan daftar pendaftaran yang menunggu verifikasi
///
/// Endpoint ini mengembalikan daftar pendaftaran yang perlu diverifikasi oleh admin sekolah.
#[utoipa::path(
    get,
    path = "/api/verifications/pending",
    tag = "Verifications",
    params(PendingVerificationsQuery),
    responses(
        (status = 200, description = "Daftar pending verifications berhasil diambil", body = PendingVerificationsResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_pending_verifications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<PendingVerificationsQuery>,
) -> AppResult<Json<PendingVerificationsResponse>> {

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

/// Mendapatkan statistik verifikasi
///
/// Endpoint ini mengembalikan statistik verifikasi untuk sekolah.
#[utoipa::path(
    get,
    path = "/api/verifications/stats",
    tag = "Verifications",
    params(StatsQuery),
    responses(
        (status = 200, description = "Statistik berhasil diambil", body = VerificationStats),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_verification_stats(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<StatsQuery>,
) -> AppResult<Json<VerificationStats>> {

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

/// Verifikasi pendaftaran
///
/// Endpoint ini digunakan untuk memverifikasi pendaftaran yang sudah disubmit.
#[utoipa::path(
    post,
    path = "/api/verifications/{id}/verify",
    tag = "Verifications",
    params(
        ("id" = i32, Path, description = "ID pendaftaran")
    ),
    responses(
        (status = 200, description = "Pendaftaran berhasil diverifikasi", body = RegistrationResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Pendaftaran tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn verify_registration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> AppResult<Json<RegistrationResponse>> {

    // Create verification service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let verification_service = VerificationService::new(registration_repo);

    // Verify registration
    let verified_registration = verification_service
        .verify_registration(id, auth_user.id)
        .await?;

    Ok(Json(verified_registration.into()))
}

/// Tolak pendaftaran
///
/// Endpoint ini digunakan untuk menolak pendaftaran dengan memberikan alasan.
#[utoipa::path(
    post,
    path = "/api/verifications/{id}/reject",
    tag = "Verifications",
    params(
        ("id" = i32, Path, description = "ID pendaftaran")
    ),
    request_body = RejectRegistrationRequest,
    responses(
        (status = 200, description = "Pendaftaran berhasil ditolak", body = RegistrationResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Pendaftaran tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn reject_registration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(payload): Json<RejectRegistrationRequest>,
) -> AppResult<Json<RegistrationResponse>> {
    // Validate reason length
    if payload.reason.len() < 10 {
        return Err(AppError::Validation("Reason must be at least 10 characters".to_string()));
    }


    // Create verification service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let verification_service = VerificationService::new(registration_repo);

    // Reject registration
    let rejected_registration = verification_service
        .reject_registration(id, payload.reason, auth_user.id)
        .await?;

    Ok(Json(rejected_registration.into()))
}

/// Verifikasi dokumen
///
/// Endpoint ini digunakan untuk memverifikasi atau menolak dokumen pendaftaran.
#[utoipa::path(
    post,
    path = "/api/verifications/documents/{doc_id}/verify",
    tag = "Verifications",
    params(
        ("doc_id" = i32, Path, description = "ID dokumen")
    ),
    request_body = VerifyDocumentRequest,
    responses(
        (status = 200, description = "Dokumen berhasil diverifikasi", body = MessageResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Dokumen tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn verify_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(doc_id): Path<i32>,
    Json(payload): Json<VerifyDocumentRequest>,
) -> AppResult<Json<MessageResponse>> {
    // Validate verification status
    if !["approved", "rejected"].contains(&payload.verification_status.as_str()) {
        return Err(AppError::Validation("Invalid verification status".to_string()));
    }


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
