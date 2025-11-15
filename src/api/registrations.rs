use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::middleware::auth::{auth_middleware, AuthUser};
use crate::models::registration::{Document, Registration};
use crate::repositories::period_repo::PeriodRepository;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::services::registration_service::RegistrationService;
use crate::utils::error::{AppError, AppResult};
use crate::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_registrations).post(create_registration))
        .route("/:id", get(get_registration).put(update_registration))
        .route("/:id/submit", post(submit_registration))
        .route("/:id/documents", get(list_documents).post(upload_document))
        .route("/:id/documents/:doc_id", delete(delete_document))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

/// Request untuk membuat pendaftaran baru
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRegistrationRequest {
    /// ID periode PPDB
    #[schema(example = 1)]
    period_id: i32,
    
    /// ID jalur pendaftaran
    #[schema(example = 1)]
    path_id: i32,
    
    /// NISN siswa
    #[schema(example = "0012345678")]
    student_nisn: String,
    
    /// Nama lengkap siswa
    #[schema(example = "Ahmad Fauzi")]
    student_name: String,
    
    /// Jenis kelamin (L/P)
    #[schema(example = "L")]
    student_gender: String,
    
    /// Tempat lahir siswa
    #[schema(example = "Jakarta")]
    student_birth_place: String,
    
    /// Tanggal lahir siswa (format: YYYY-MM-DD)
    #[schema(value_type = String, example = "2010-01-15")]
    student_birth_date: NaiveDate,
    
    /// Agama siswa
    #[schema(example = "Islam")]
    student_religion: String,
    
    /// Alamat lengkap siswa
    #[schema(example = "Jl. Merdeka No. 123, Jakarta")]
    student_address: String,
    
    /// Nomor telepon siswa (opsional)
    #[schema(example = "081234567890")]
    student_phone: Option<String>,
    
    /// Email siswa (opsional)
    #[schema(example = "ahmad@example.com")]
    student_email: Option<String>,
    
    /// Nama orang tua/wali
    #[schema(example = "Budi Santoso")]
    parent_name: String,
    
    /// NIK orang tua/wali
    #[schema(example = "3201234567890123")]
    parent_nik: String,
    
    /// Nomor telepon orang tua/wali
    #[schema(example = "081234567890")]
    parent_phone: String,
    
    /// Pekerjaan orang tua/wali (opsional)
    #[schema(example = "Pegawai Swasta")]
    parent_occupation: Option<String>,
    
    /// Penghasilan orang tua/wali (opsional)
    #[schema(example = "5000000")]
    parent_income: Option<String>,
    
    /// Nama sekolah asal (opsional)
    #[schema(example = "SD Negeri 1 Jakarta")]
    previous_school_name: Option<String>,
    
    /// NPSN sekolah asal (opsional)
    #[schema(example = "12345678")]
    previous_school_npsn: Option<String>,
    
    /// Alamat sekolah asal (opsional)
    #[schema(example = "Jl. Pendidikan No. 1, Jakarta")]
    previous_school_address: Option<String>,
    
    /// Data khusus jalur pendaftaran (JSON)
    #[schema(value_type = Object)]
    path_data: serde_json::Value,
}

/// Request untuk update pendaftaran
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRegistrationRequest {
    /// Nama lengkap siswa (opsional)
    #[schema(example = "Ahmad Fauzi")]
    student_name: Option<String>,
    
    /// Jenis kelamin (opsional)
    #[schema(example = "L")]
    student_gender: Option<String>,
    
    /// Tempat lahir siswa (opsional)
    #[schema(example = "Jakarta")]
    student_birth_place: Option<String>,
    
    /// Tanggal lahir siswa (opsional, format: YYYY-MM-DD)
    #[schema(value_type = Option<String>, example = "2010-01-15")]
    student_birth_date: Option<NaiveDate>,
    
    /// Agama siswa (opsional)
    #[schema(example = "Islam")]
    student_religion: Option<String>,
    
    /// Alamat lengkap siswa (opsional)
    #[schema(example = "Jl. Merdeka No. 123, Jakarta")]
    student_address: Option<String>,
    
    /// Nomor telepon siswa (opsional)
    #[schema(example = "081234567890")]
    student_phone: Option<String>,
    
    /// Email siswa (opsional)
    #[schema(example = "ahmad@example.com")]
    student_email: Option<String>,
    
    /// Nama orang tua/wali (opsional)
    #[schema(example = "Budi Santoso")]
    parent_name: Option<String>,
    
    /// NIK orang tua/wali (opsional)
    #[schema(example = "3201234567890123")]
    parent_nik: Option<String>,
    
    /// Nomor telepon orang tua/wali (opsional)
    #[schema(example = "081234567890")]
    parent_phone: Option<String>,
    
    /// Pekerjaan orang tua/wali (opsional)
    #[schema(example = "Pegawai Swasta")]
    parent_occupation: Option<String>,
    
    /// Penghasilan orang tua/wali (opsional)
    #[schema(example = "5000000")]
    parent_income: Option<String>,
    
    /// Data khusus jalur pendaftaran (opsional)
    #[schema(value_type = Option<Object>)]
    path_data: Option<serde_json::Value>,
}

/// Request untuk upload dokumen
#[derive(Debug, Deserialize, ToSchema)]
pub struct UploadDocumentRequest {
    /// Tipe dokumen (akta_lahir/kartu_keluarga/ijazah/dll)
    #[schema(example = "akta_lahir")]
    document_type: String,
    
    /// URL file yang sudah diupload
    #[schema(example = "https://storage.example.com/documents/akta_123.pdf")]
    file_url: String,
    
    /// Nama file
    #[schema(example = "akta_lahir.pdf")]
    file_name: String,
    
    /// Ukuran file dalam bytes
    #[schema(example = 1024000)]
    file_size: i64,
    
    /// MIME type file
    #[schema(example = "application/pdf")]
    mime_type: String,
}

/// Query parameters untuk list pendaftaran
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListRegistrationsQuery {
    /// Nomor halaman
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    page: i64,
    
    /// Jumlah item per halaman
    #[serde(default = "default_page_size")]
    #[schema(example = 10)]
    page_size: i64,
    
    /// Filter berdasarkan status
    #[schema(example = "submitted")]
    status: Option<String>,
    
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

/// Response data pendaftaran
#[derive(Debug, Serialize, ToSchema)]
pub struct RegistrationResponse {
    /// ID pendaftaran
    #[schema(example = 1)]
    id: i32,
    
    /// ID sekolah
    #[schema(example = 1)]
    school_id: i32,
    
    /// ID user (orang tua)
    #[schema(example = 1)]
    user_id: i32,
    
    /// ID periode PPDB
    #[schema(example = 1)]
    period_id: i32,
    
    /// ID jalur pendaftaran
    #[schema(example = 1)]
    path_id: i32,
    
    /// Nomor pendaftaran
    #[schema(example = "PPDB-2024-001")]
    registration_number: Option<String>,
    
    /// NISN siswa
    #[schema(example = "0012345678")]
    student_nisn: String,
    
    /// Nama lengkap siswa
    #[schema(example = "Ahmad Fauzi")]
    student_name: String,
    
    /// Jenis kelamin
    #[schema(example = "L")]
    student_gender: String,
    
    /// Tempat lahir
    #[schema(example = "Jakarta")]
    student_birth_place: String,
    
    /// Tanggal lahir
    #[schema(value_type = String, example = "2010-01-15T00:00:00Z")]
    student_birth_date: DateTime<Utc>,
    
    /// Agama
    #[schema(example = "Islam")]
    student_religion: String,
    
    /// Alamat lengkap
    #[schema(example = "Jl. Merdeka No. 123, Jakarta")]
    student_address: String,
    
    /// Nomor telepon siswa
    #[schema(example = "081234567890")]
    student_phone: Option<String>,
    
    /// Email siswa
    #[schema(example = "ahmad@example.com")]
    student_email: Option<String>,
    
    /// Nama orang tua/wali
    #[schema(example = "Budi Santoso")]
    parent_name: String,
    
    /// NIK orang tua/wali
    #[schema(example = "3201234567890123")]
    parent_nik: String,
    
    /// Nomor telepon orang tua/wali
    #[schema(example = "081234567890")]
    parent_phone: String,
    
    /// Pekerjaan orang tua/wali
    #[schema(example = "Pegawai Swasta")]
    parent_occupation: Option<String>,
    
    /// Penghasilan orang tua/wali
    #[schema(example = "5000000")]
    parent_income: Option<String>,
    
    /// Nama sekolah asal
    #[schema(example = "SD Negeri 1 Jakarta")]
    previous_school_name: Option<String>,
    
    /// NPSN sekolah asal
    #[schema(example = "12345678")]
    previous_school_npsn: Option<String>,
    
    /// Alamat sekolah asal
    #[schema(example = "Jl. Pendidikan No. 1, Jakarta")]
    previous_school_address: Option<String>,
    
    /// Data khusus jalur pendaftaran
    #[schema(value_type = Object)]
    path_data: serde_json::Value,
    
    /// Skor seleksi
    #[schema(example = 85.5)]
    selection_score: Option<f64>,
    
    /// Peringkat
    #[schema(example = 10)]
    ranking: Option<i32>,
    
    /// Status pendaftaran
    #[schema(example = "submitted")]
    status: String,
    
    /// Alasan penolakan (jika ditolak)
    #[schema(example = "Dokumen tidak lengkap")]
    rejection_reason: Option<String>,
    
    /// Waktu pembuatan
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    created_at: DateTime<Utc>,
    
    /// Waktu update terakhir
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    updated_at: DateTime<Utc>,
}

impl From<Registration> for RegistrationResponse {
    fn from(reg: Registration) -> Self {
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
            student_birth_place: reg.student_birth_place,
            student_birth_date: reg.student_birth_date,
            student_religion: reg.student_religion,
            student_address: reg.student_address,
            student_phone: reg.student_phone,
            student_email: reg.student_email,
            parent_name: reg.parent_name,
            parent_nik: reg.parent_nik,
            parent_phone: reg.parent_phone,
            parent_occupation: reg.parent_occupation,
            parent_income: reg.parent_income,
            previous_school_name: reg.previous_school_name,
            previous_school_npsn: reg.previous_school_npsn,
            previous_school_address: reg.previous_school_address,
            path_data: reg.path_data,
            selection_score: reg.selection_score,
            ranking: reg.ranking,
            status: reg.status,
            rejection_reason: reg.rejection_reason,
            created_at: reg.created_at,
            updated_at: reg.updated_at,
        }
    }
}

/// Response data dokumen
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentResponse {
    /// ID dokumen
    #[schema(example = 1)]
    id: i32,
    
    /// ID pendaftaran
    #[schema(example = 1)]
    registration_id: i32,
    
    /// Tipe dokumen
    #[schema(example = "akta_lahir")]
    document_type: String,
    
    /// URL file
    #[schema(example = "https://storage.example.com/documents/akta_123.pdf")]
    file_url: String,
    
    /// Nama file
    #[schema(example = "akta_lahir.pdf")]
    file_name: String,
    
    /// Ukuran file dalam bytes
    #[schema(example = 1024000)]
    file_size: i64,
    
    /// MIME type file
    #[schema(example = "application/pdf")]
    mime_type: String,
    
    /// Status verifikasi
    #[schema(example = "pending")]
    verification_status: String,
    
    /// Catatan verifikasi
    #[schema(example = "Dokumen sudah sesuai")]
    verification_notes: Option<String>,
    
    /// Waktu pembuatan
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    created_at: DateTime<Utc>,
    
    /// Waktu update terakhir
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    updated_at: DateTime<Utc>,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            registration_id: doc.registration_id,
            document_type: doc.document_type,
            file_url: doc.file_url,
            file_name: doc.file_name,
            file_size: doc.file_size,
            mime_type: doc.mime_type,
            verification_status: doc.verification_status,
            verification_notes: doc.verification_notes,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

/// Response list pendaftaran dengan pagination
#[derive(Debug, Serialize, ToSchema)]
pub struct ListRegistrationsResponse {
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

/// Membuat pendaftaran baru
///
/// Endpoint ini digunakan untuk membuat pendaftaran PPDB baru.
/// Orang tua dapat mendaftarkan anak mereka ke periode dan jalur yang tersedia.
#[utoipa::path(
    post,
    path = "/api/registrations",
    tag = "Registrations",
    request_body = CreateRegistrationRequest,
    responses(
        (status = 201, description = "Pendaftaran berhasil dibuat", body = RegistrationResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 404, description = "Periode atau jalur tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_registration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateRegistrationRequest>,
) -> AppResult<(StatusCode, Json<RegistrationResponse>)> {
    // Validate gender
    if !["L", "P", "LAKI-LAKI", "PEREMPUAN"].contains(&payload.student_gender.to_uppercase().as_str()) {
        return Err(AppError::Validation("Invalid gender".to_string()));
    }

    let school_id = auth_user.school_id.ok_or_else(|| {
        AppError::Authentication("User must be associated with a school".to_string())
    })?;

    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // Create registration
    let registration = registration_service
        .create_registration(
            school_id,
            auth_user.id,
            payload.period_id,
            payload.path_id,
            payload.student_nisn,
            payload.student_name,
            payload.student_gender,
            payload.student_birth_place,
            payload.student_birth_date,
            payload.student_religion,
            payload.student_address,
            payload.student_phone,
            payload.student_email,
            payload.parent_name,
            payload.parent_nik,
            payload.parent_phone,
            payload.parent_occupation,
            payload.parent_income,
            payload.previous_school_name,
            payload.previous_school_npsn,
            payload.previous_school_address,
            payload.path_data,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(registration.into())))
}

/// Mendapatkan daftar pendaftaran
///
/// Endpoint ini mengembalikan daftar pendaftaran dengan pagination dan filter.
/// - Orang tua hanya dapat melihat pendaftaran mereka sendiri
/// - Admin sekolah dapat melihat semua pendaftaran di sekolahnya
#[utoipa::path(
    get,
    path = "/api/registrations",
    tag = "Registrations",
    params(ListRegistrationsQuery),
    responses(
        (status = 200, description = "Daftar pendaftaran berhasil diambil", body = ListRegistrationsResponse),
        (status = 401, description = "Tidak terautentikasi")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn list_registrations(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListRegistrationsQuery>,
) -> AppResult<Json<ListRegistrationsResponse>> {

    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // List registrations based on role
    let (registrations, total) = if auth_user.role == "parent" {
        // Parents can only see their own registrations
        let regs = registration_service
            .list_registrations_by_user(auth_user.id, query.page, query.page_size)
            .await?;
        let total = regs.len() as i64;
        (regs, total)
    } else {
        // Admins can see all registrations for their school
        let school_id = auth_user.school_id.ok_or_else(|| {
            AppError::Authentication("User must be associated with a school".to_string())
        })?;

        registration_service
            .list_registrations_by_school(
                school_id,
                query.page,
                query.page_size,
                query.status,
                query.period_id,
                query.path_id,
            )
            .await?
    };

    let total_pages = (total as f64 / query.page_size as f64).ceil() as i64;

    Ok(Json(ListRegistrationsResponse {
        registrations: registrations.into_iter().map(|r| r.into()).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    }))
}

/// Mendapatkan detail pendaftaran
///
/// Endpoint ini mengembalikan detail pendaftaran berdasarkan ID.
#[utoipa::path(
    get,
    path = "/api/registrations/{id}",
    tag = "Registrations",
    params(
        ("id" = i32, Path, description = "ID pendaftaran")
    ),
    responses(
        (status = 200, description = "Detail pendaftaran berhasil diambil", body = RegistrationResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Pendaftaran tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_registration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> AppResult<Json<RegistrationResponse>> {

    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // Get registration
    let registration = registration_service.get_registration(id).await?;

    // Check permission
    if auth_user.role == "parent" && registration.user_id != auth_user.id {
        return Err(AppError::Forbidden(
            "You don't have permission to view this registration".to_string(),
        ));
    }

    Ok(Json(registration.into()))
}

/// Update pendaftaran
///
/// Endpoint ini digunakan untuk mengupdate data pendaftaran.
/// Hanya dapat dilakukan oleh pemilik pendaftaran dan sebelum status submitted.
#[utoipa::path(
    put,
    path = "/api/registrations/{id}",
    tag = "Registrations",
    params(
        ("id" = i32, Path, description = "ID pendaftaran")
    ),
    request_body = UpdateRegistrationRequest,
    responses(
        (status = 200, description = "Pendaftaran berhasil diupdate", body = RegistrationResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Pendaftaran tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn update_registration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateRegistrationRequest>,
) -> AppResult<Json<RegistrationResponse>> {

    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // Check if registration belongs to user
    let registration = registration_service.get_registration(id).await?;
    if registration.user_id != auth_user.id {
        return Err(AppError::Forbidden(
            "You don't have permission to update this registration".to_string(),
        ));
    }

    // Update registration
    let updated_registration = registration_service
        .update_registration(
            id,
            payload.student_name,
            payload.student_gender,
            payload.student_birth_place,
            payload.student_birth_date,
            payload.student_religion,
            payload.student_address,
            payload.student_phone,
            payload.student_email,
            payload.parent_name,
            payload.parent_nik,
            payload.parent_phone,
            payload.parent_occupation,
            payload.parent_income,
            payload.path_data,
        )
        .await?;

    Ok(Json(updated_registration.into()))
}

/// Submit pendaftaran
///
/// Endpoint ini digunakan untuk submit pendaftaran setelah semua data lengkap.
/// Setelah disubmit, pendaftaran tidak dapat diubah lagi.
#[utoipa::path(
    post,
    path = "/api/registrations/{id}/submit",
    tag = "Registrations",
    params(
        ("id" = i32, Path, description = "ID pendaftaran")
    ),
    responses(
        (status = 200, description = "Pendaftaran berhasil disubmit", body = RegistrationResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Pendaftaran tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn submit_registration(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> AppResult<Json<RegistrationResponse>> {

    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // Check if registration belongs to user
    let registration = registration_service.get_registration(id).await?;
    if registration.user_id != auth_user.id {
        return Err(AppError::Forbidden(
            "You don't have permission to submit this registration".to_string(),
        ));
    }

    // Submit registration
    let submitted_registration = registration_service.submit_registration(id).await?;

    Ok(Json(submitted_registration.into()))
}

/// Mendapatkan daftar dokumen pendaftaran
///
/// Endpoint ini mengembalikan daftar dokumen yang sudah diupload untuk pendaftaran.
#[utoipa::path(
    get,
    path = "/api/registrations/{registration_id}/documents",
    tag = "Registrations",
    params(
        ("registration_id" = i32, Path, description = "ID pendaftaran")
    ),
    responses(
        (status = 200, description = "Daftar dokumen berhasil diambil", body = Vec<DocumentResponse>),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 404, description = "Pendaftaran tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn list_documents(
    State(state): State<AppState>,
    Path(registration_id): Path<i32>,
) -> AppResult<Json<Vec<DocumentResponse>>> {
    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // List documents
    let documents = registration_service.list_documents(registration_id).await?;

    Ok(Json(documents.into_iter().map(|d| d.into()).collect()))
}

/// Upload dokumen pendaftaran
///
/// Endpoint ini digunakan untuk menambahkan dokumen ke pendaftaran.
/// File harus sudah diupload ke storage terlebih dahulu, endpoint ini hanya menyimpan metadata.
#[utoipa::path(
    post,
    path = "/api/registrations/{registration_id}/documents",
    tag = "Registrations",
    params(
        ("registration_id" = i32, Path, description = "ID pendaftaran")
    ),
    request_body = UploadDocumentRequest,
    responses(
        (status = 201, description = "Dokumen berhasil diupload", body = DocumentResponse),
        (status = 400, description = "Request tidak valid"),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 404, description = "Pendaftaran tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn upload_document(
    State(state): State<AppState>,
    Path(registration_id): Path<i32>,
    Json(payload): Json<UploadDocumentRequest>,
) -> AppResult<(StatusCode, Json<DocumentResponse>)> {
    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // Upload document
    let document = registration_service
        .upload_document(
            registration_id,
            payload.document_type,
            payload.file_url,
            payload.file_name,
            payload.file_size,
            payload.mime_type,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(document.into())))
}

/// Hapus dokumen pendaftaran
///
/// Endpoint ini digunakan untuk menghapus dokumen dari pendaftaran.
#[utoipa::path(
    delete,
    path = "/api/registrations/{registration_id}/documents/{doc_id}",
    tag = "Registrations",
    params(
        ("registration_id" = i32, Path, description = "ID pendaftaran"),
        ("doc_id" = i32, Path, description = "ID dokumen")
    ),
    responses(
        (status = 200, description = "Dokumen berhasil dihapus", body = MessageResponse),
        (status = 401, description = "Tidak terautentikasi"),
        (status = 403, description = "Tidak memiliki akses"),
        (status = 404, description = "Dokumen tidak ditemukan")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn delete_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((_registration_id, doc_id)): Path<(i32, i32)>,
) -> AppResult<Json<MessageResponse>> {

    // Create registration service
    let registration_repo = RegistrationRepository::new(state.db.clone());
    let period_repo = PeriodRepository::new(state.db.clone());
    let registration_service = RegistrationService::new(registration_repo, period_repo);

    // Delete document
    registration_service.delete_document(doc_id, auth_user.id).await?;

    Ok(Json(MessageResponse {
        message: "Document deleted successfully".to_string(),
    }))
}
