use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

/// Request untuk membuat periode PPDB baru
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePeriodRequest {
    /// Tahun ajaran (format: YYYY/YYYY, contoh: "2024/2025")
    #[schema(example = "2024/2025")]
    academic_year: String,
    
    /// Jenjang pendidikan (SD/SMP/SMA/SMK)
    #[schema(example = "SMA")]
    level: String,
    
    /// Tanggal mulai periode
    #[schema(value_type = String, example = "2024-06-01")]
    start_date: NaiveDate,
    
    /// Tanggal akhir periode
    #[schema(value_type = String, example = "2024-07-31")]
    end_date: NaiveDate,
    
    /// Batas waktu daftar ulang (opsional)
    #[schema(value_type = Option<String>, example = "2024-08-15")]
    reenrollment_deadline: Option<NaiveDate>,
    
    /// Daftar jalur pendaftaran
    paths: Vec<CreatePathRequest>,
}

/// Request untuk membuat jalur pendaftaran
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePathRequest {
    /// Tipe jalur (zonasi/prestasi/afirmasi/perpindahan_tugas)
    #[schema(example = "zonasi")]
    path_type: String,
    
    /// Nama jalur
    #[schema(example = "Jalur Zonasi")]
    name: String,
    
    /// Kuota peserta
    #[schema(example = 100)]
    quota: i32,
    
    /// Deskripsi jalur (opsional)
    #[schema(example = "Jalur untuk siswa dalam zona sekolah")]
    description: Option<String>,
    
    /// Konfigurasi penilaian dalam format JSON
    #[schema(value_type = Object)]
    scoring_config: serde_json::Value,
}

/// Request untuk update periode PPDB
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePeriodRequest {
    /// Tanggal mulai periode (opsional)
    #[schema(value_type = Option<String>, example = "2024-06-01")]
    start_date: Option<NaiveDate>,
    
    /// Tanggal akhir periode (opsional)
    #[schema(value_type = Option<String>, example = "2024-07-31")]
    end_date: Option<NaiveDate>,
    
    /// Tanggal pengumuman (opsional)
    #[schema(value_type = Option<String>, example = "2024-08-01")]
    announcement_date: Option<NaiveDate>,
    
    /// Batas waktu daftar ulang (opsional)
    #[schema(value_type = Option<String>, example = "2024-08-15")]
    reenrollment_deadline: Option<NaiveDate>,
}

/// Request untuk update jalur pendaftaran
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePathRequest {
    /// Nama jalur (opsional)
    #[schema(example = "Jalur Zonasi")]
    name: Option<String>,
    
    /// Kuota peserta (opsional)
    #[schema(example = 120)]
    quota: Option<i32>,
    
    /// Deskripsi jalur (opsional)
    #[schema(example = "Jalur untuk siswa dalam zona sekolah")]
    description: Option<String>,
    
    /// Konfigurasi penilaian dalam format JSON (opsional)
    #[schema(value_type = Option<Object>)]
    scoring_config: Option<serde_json::Value>,
}

/// Query parameters untuk list periode
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListPeriodsQuery {
    /// Nomor halaman
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    page: i64,
    
    /// Jumlah item per halaman
    #[serde(default = "default_page_size")]
    #[schema(example = 10)]
    page_size: i64,
    
    /// Filter berdasarkan status (draft/active/closed)
    #[schema(example = "active")]
    status: Option<String>,
    
    /// Filter berdasarkan tahun ajaran
    #[schema(example = "2024/2025")]
    academic_year: Option<String>,
    
    /// Filter berdasarkan jenjang
    #[schema(example = "SMA")]
    level: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

/// Response data periode PPDB
#[derive(Debug, Serialize, ToSchema)]
pub struct PeriodResponse {
    /// ID periode
    #[schema(example = 1)]
    id: i32,
    
    /// ID sekolah
    #[schema(example = 1)]
    school_id: i32,
    
    /// Tahun ajaran
    #[schema(example = "2024/2025")]
    academic_year: String,
    
    /// Jenjang pendidikan
    #[schema(example = "SMA")]
    level: String,
    
    /// Tanggal mulai periode
    #[schema(value_type = String, example = "2024-06-01")]
    start_date: NaiveDate,
    
    /// Tanggal akhir periode
    #[schema(value_type = String, example = "2024-07-31")]
    end_date: NaiveDate,
    
    /// Tanggal mulai pendaftaran
    #[schema(value_type = String, example = "2024-06-01")]
    registration_start: NaiveDate,
    
    /// Tanggal akhir pendaftaran
    #[schema(value_type = String, example = "2024-06-30")]
    registration_end: NaiveDate,
    
    /// Tanggal pengumuman
    #[schema(value_type = Option<String>, example = "2024-08-01")]
    announcement_date: Option<NaiveDate>,
    
    /// Batas waktu daftar ulang
    #[schema(value_type = Option<String>, example = "2024-08-15")]
    reenrollment_deadline: Option<NaiveDate>,
    
    /// Status periode (draft/active/closed)
    #[schema(example = "active")]
    status: String,
    
    /// Waktu pembuatan
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    created_at: DateTime<Utc>,
    
    /// Waktu update terakhir
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
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
            registration_start: period.registration_start,
            registration_end: period.registration_end,
            announcement_date: period.announcement_date,
            reenrollment_deadline: period.reenrollment_deadline,
            status: period.status,
            created_at: period.created_at,
            updated_at: period.updated_at,
        }
    }
}

/// Response periode dengan jalur pendaftaran
#[derive(Debug, Serialize, ToSchema)]
pub struct PeriodWithPathsResponse {
    /// Data periode
    period: PeriodResponse,
    
    /// Daftar jalur pendaftaran
    paths: Vec<PathResponse>,
}

/// Response data jalur pendaftaran
#[derive(Debug, Serialize, ToSchema)]
pub struct PathResponse {
    /// ID jalur
    #[schema(example = 1)]
    id: i32,
    
    /// ID periode
    #[schema(example = 1)]
    period_id: i32,
    
    /// Tipe jalur
    #[schema(example = "zonasi")]
    path_type: String,
    
    /// Nama jalur
    #[schema(example = "Jalur Zonasi")]
    name: String,
    
    /// Kuota peserta
    #[schema(example = 100)]
    quota: i32,
    
    /// Deskripsi jalur
    #[schema(example = "Jalur untuk siswa dalam zona sekolah")]
    description: Option<String>,
    
    /// Konfigurasi penilaian
    #[schema(value_type = Object)]
    scoring_config: serde_json::Value,
    
    /// Waktu pembuatan
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    created_at: DateTime<Utc>,
    
    /// Waktu update terakhir
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
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

/// Response list periode dengan pagination
#[derive(Debug, Serialize, ToSchema)]
pub struct ListPeriodsResponse {
    /// Daftar periode
    periods: Vec<PeriodResponse>,
    
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

/// Membuat periode PPDB baru
///
/// Endpoint ini digunakan untuk membuat periode PPDB baru beserta jalur pendaftarannya.
/// Hanya dapat diakses oleh admin sekolah.
#[utoipa::path(
    post,
    path = "/api/periods",
    tag = "Periods",
    request_body = CreatePeriodRequest,
    responses(
        (status = 201, description = "Periode berhasil dibuat", body = PeriodWithPathsResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_period(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreatePeriodRequest>,
) -> AppResult<(StatusCode, Json<PeriodWithPathsResponse>)> {
    // Validate academic year format
    if payload.academic_year.len() != 9 {
        return Err(AppError::Validation("Academic year must be in format YYYY/YYYY".to_string()));
    }
    
    // Validate level
    if !["SD", "SMP", "SMA", "SMK"].contains(&payload.level.to_uppercase().as_str()) {
        return Err(AppError::Validation("Invalid level".to_string()));
    }
    
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

/// Mendapatkan daftar periode PPDB
///
/// Endpoint ini mengembalikan daftar periode PPDB dengan pagination dan filter.
#[utoipa::path(
    get,
    path = "/api/periods",
    tag = "Periods",
    params(ListPeriodsQuery),
    responses(
        (status = 200, description = "Daftar periode berhasil diambil", body = ListPeriodsResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn list_periods(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListPeriodsQuery>,
) -> AppResult<Json<ListPeriodsResponse>> {
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

/// Mendapatkan detail periode PPDB
///
/// Endpoint ini mengembalikan detail periode beserta jalur pendaftarannya.
#[utoipa::path(
    get,
    path = "/api/periods/{id}",
    tag = "Periods",
    params(
        ("id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Detail periode berhasil diambil", body = PeriodWithPathsResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Update periode PPDB
///
/// Endpoint ini digunakan untuk mengupdate informasi periode PPDB.
#[utoipa::path(
    put,
    path = "/api/periods/{id}",
    tag = "Periods",
    params(
        ("id" = i32, Path, description = "ID periode")
    ),
    request_body = UpdatePeriodRequest,
    responses(
        (status = 200, description = "Periode berhasil diupdate", body = PeriodResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn update_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePeriodRequest>,
) -> AppResult<Json<PeriodResponse>> {

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

/// Hapus periode PPDB
///
/// Endpoint ini digunakan untuk menghapus periode PPDB.
#[utoipa::path(
    delete,
    path = "/api/periods/{id}",
    tag = "Periods",
    params(
        ("id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Periode berhasil dihapus", body = MessageResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Aktifkan periode PPDB
///
/// Endpoint ini digunakan untuk mengaktifkan periode PPDB.
#[utoipa::path(
    post,
    path = "/api/periods/{id}/activate",
    tag = "Periods",
    params(
        ("id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Periode berhasil diaktifkan", body = PeriodResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Tutup periode PPDB
///
/// Endpoint ini digunakan untuk menutup periode PPDB.
#[utoipa::path(
    post,
    path = "/api/periods/{id}/close",
    tag = "Periods",
    params(
        ("id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Periode berhasil ditutup", body = PeriodResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Mendapatkan daftar jalur pendaftaran
///
/// Endpoint ini mengembalikan daftar jalur pendaftaran untuk periode tertentu.
#[utoipa::path(
    get,
    path = "/api/periods/{period_id}/paths",
    tag = "Periods",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    responses(
        (status = 200, description = "Daftar jalur berhasil diambil", body = Vec<PathResponse>),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Membuat jalur pendaftaran baru
///
/// Endpoint ini digunakan untuk menambahkan jalur pendaftaran ke periode PPDB.
#[utoipa::path(
    post,
    path = "/api/periods/{period_id}/paths",
    tag = "Periods",
    params(
        ("period_id" = i32, Path, description = "ID periode")
    ),
    request_body = CreatePathRequest,
    responses(
        (status = 201, description = "Jalur berhasil dibuat", body = PathResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Periode tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_path(
    State(state): State<AppState>,
    Path(period_id): Path<i32>,
    Json(payload): Json<CreatePathRequest>,
) -> AppResult<(StatusCode, Json<PathResponse>)> {
    // Validate path type
    if !["zonasi", "prestasi", "afirmasi", "perpindahan_tugas"].contains(&payload.path_type.as_str()) {
        return Err(AppError::Validation("Invalid path type".to_string()));
    }

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

/// Update jalur pendaftaran
///
/// Endpoint ini digunakan untuk mengupdate informasi jalur pendaftaran.
#[utoipa::path(
    put,
    path = "/api/periods/paths/{path_id}",
    tag = "Periods",
    params(
        ("path_id" = i32, Path, description = "ID jalur pendaftaran")
    ),
    request_body = UpdatePathRequest,
    responses(
        (status = 200, description = "Jalur berhasil diupdate", body = PathResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Jalur tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn update_path(
    State(state): State<AppState>,
    Path(path_id): Path<i32>,
    Json(payload): Json<UpdatePathRequest>,
) -> AppResult<Json<PathResponse>> {

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

/// Hapus jalur pendaftaran
///
/// Endpoint ini digunakan untuk menghapus jalur pendaftaran.
#[utoipa::path(
    delete,
    path = "/api/periods/paths/{path_id}",
    tag = "Periods",
    params(
        ("path_id" = i32, Path, description = "ID jalur pendaftaran")
    ),
    responses(
        (status = 200, description = "Jalur berhasil dihapus", body = MessageResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Jalur tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
