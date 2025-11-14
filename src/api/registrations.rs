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

#[derive(Debug, Deserialize, Validate)]
struct CreateRegistrationRequest {
    period_id: i32,
    path_id: i32,
    
    #[validate(length(equal = 10))]
    student_nisn: String,
    
    #[validate(length(min = 3))]
    student_name: String,
    
    #[validate(custom = "validate_gender")]
    student_gender: String,
    
    student_birth_place: String,
    student_birth_date: DateTime<Utc>,
    student_religion: String,
    student_address: String,
    student_phone: Option<String>,
    student_email: Option<String>,
    
    #[validate(length(min = 3))]
    parent_name: String,
    
    #[validate(length(equal = 16))]
    parent_nik: String,
    
    parent_phone: String,
    parent_occupation: Option<String>,
    parent_income: Option<String>,
    
    previous_school_name: Option<String>,
    previous_school_npsn: Option<String>,
    previous_school_address: Option<String>,
    
    path_data: serde_json::Value,
}

fn validate_gender(gender: &str) -> Result<(), validator::ValidationError> {
    match gender.to_uppercase().as_str() {
        "L" | "P" | "LAKI-LAKI" | "PEREMPUAN" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_gender")),
    }
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateRegistrationRequest {
    student_name: Option<String>,
    student_gender: Option<String>,
    student_birth_place: Option<String>,
    student_birth_date: Option<DateTime<Utc>>,
    student_religion: Option<String>,
    student_address: Option<String>,
    student_phone: Option<String>,
    student_email: Option<String>,
    parent_name: Option<String>,
    parent_nik: Option<String>,
    parent_phone: Option<String>,
    parent_occupation: Option<String>,
    parent_income: Option<String>,
    path_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct UploadDocumentRequest {
    document_type: String,
    file_url: String,
    file_name: String,
    file_size: i64,
    mime_type: String,
}

#[derive(Debug, Deserialize)]
struct ListRegistrationsQuery {
    #[serde(default = "default_page")]
    page: i64,
    
    #[serde(default = "default_page_size")]
    page_size: i64,
    
    status: Option<String>,
    period_id: Option<i32>,
    path_id: Option<i32>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
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
    student_birth_place: String,
    student_birth_date: DateTime<Utc>,
    student_religion: String,
    student_address: String,
    student_phone: Option<String>,
    student_email: Option<String>,
    parent_name: String,
    parent_nik: String,
    parent_phone: String,
    parent_occupation: Option<String>,
    parent_income: Option<String>,
    previous_school_name: Option<String>,
    previous_school_npsn: Option<String>,
    previous_school_address: Option<String>,
    path_data: serde_json::Value,
    selection_score: Option<f64>,
    ranking: Option<i32>,
    status: String,
    rejection_reason: Option<String>,
    created_at: DateTime<Utc>,
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

#[derive(Debug, Serialize)]
struct DocumentResponse {
    id: i32,
    registration_id: i32,
    document_type: String,
    file_url: String,
    file_name: String,
    file_size: i64,
    mime_type: String,
    verification_status: String,
    verification_notes: Option<String>,
    created_at: DateTime<Utc>,
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

#[derive(Debug, Serialize)]
struct ListRegistrationsResponse {
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

async fn create_registration(
    State(state): State<AppState>,
    req: Request,
    Json(payload): Json<CreateRegistrationRequest>,
) -> AppResult<(StatusCode, Json<RegistrationResponse>)> {
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

async fn list_registrations(
    State(state): State<AppState>,
    req: Request,
    Query(query): Query<ListRegistrationsQuery>,
) -> AppResult<Json<ListRegistrationsResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

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

async fn get_registration(
    State(state): State<AppState>,
    req: Request,
    Path(id): Path<i32>,
) -> AppResult<Json<RegistrationResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

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

async fn update_registration(
    State(state): State<AppState>,
    req: Request,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateRegistrationRequest>,
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

async fn submit_registration(
    State(state): State<AppState>,
    req: Request,
    Path(id): Path<i32>,
) -> AppResult<Json<RegistrationResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

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

async fn delete_document(
    State(state): State<AppState>,
    req: Request,
    Path((registration_id, doc_id)): Path<(i32, i32)>,
) -> AppResult<Json<MessageResponse>> {
    // Get authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Authentication("Not authenticated".to_string()))?;

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
