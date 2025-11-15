/// API Documentation Module
/// 
/// This module provides OpenAPI/Swagger documentation for the PPDB API.
/// 
/// To add documentation to an endpoint:
/// 1. Add #[utoipa::path] macro above the handler function
/// 2. Add the path to ApiDoc in openapi.rs
/// 3. Add DTOs with #[derive(ToSchema)] to components
/// 
/// Example:
/// ```rust
/// #[utoipa::path(
///     post,
///     path = "/api/auth/login",
///     tag = "Authentication",
///     request_body = LoginRequest,
///     responses(
///         (status = 200, description = "Login successful", body = LoginResponse),
///         (status = 401, description = "Invalid credentials", body = ErrorResponse)
///     )
/// )]
/// async fn login(
///     State(state): State<AppState>,
///     Json(payload): Json<LoginRequest>,
/// ) -> AppResult<Json<LoginResponse>> {
///     // handler implementation
/// }
/// ```

use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;

/// Main OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "PPDB Sekolah API",
        version = "1.0.0",
        description = r#"
# Sistem PPDB (Penerimaan Peserta Didik Baru)

API untuk sistem penerimaan siswa baru berbasis multi-tenant SaaS.

## Features
- 🏫 Multi-tenant school management
- 👥 User authentication & RBAC
- 📝 Student registration flow
- 📄 Document upload & verification
- 🎯 Automatic scoring & ranking
- 📊 Selection process & announcement

## Authentication
Most endpoints require JWT Bearer token authentication.

1. Register or login to get access token
2. Add token to Authorization header: `Bearer <your_token>`
3. Token expires in 24 hours

## Roles & Permissions
- **SuperAdmin**: Full access to all schools
- **SchoolAdmin**: Access to own school data only
- **Parent**: Access to own registrations only

## Multi-tenant Isolation
All data is automatically filtered by school_id based on user's role.
        "#,
        contact(
            name = "PPDB Support",
            email = "support@ppdb.com"
        ),
        license(
            name = "MIT"
        )
    ),
    servers(
        (url = "http://localhost:8000", description = "Development server"),
        (url = "https://api.ppdb.com", description = "Production server")
    ),
    paths(
        // System endpoints
        crate::api::health::health_check,
        
        // Authentication endpoints
        crate::api::auth::register,
        crate::api::auth::login,
        crate::api::auth::verify_email,
        crate::api::auth::forgot_password,
        crate::api::auth::reset_password,
        crate::api::auth::refresh_token,
        crate::api::auth::logout,
        crate::api::auth::get_current_user,
        
        // School endpoints
        crate::api::schools::list_schools,
        crate::api::schools::create_school,
        crate::api::schools::get_school,
        crate::api::schools::update_school,
        crate::api::schools::deactivate_school,
        crate::api::schools::activate_school,
        
        // User endpoints
        crate::api::users::list_users,
        crate::api::users::create_user,
        crate::api::users::get_user,
        crate::api::users::get_current_user_full,
        crate::api::users::update_user,
        crate::api::users::update_current_user,
        crate::api::users::delete_user,
        crate::api::users::change_password,
        
        // Period endpoints
        crate::api::periods::list_periods,
        crate::api::periods::create_period,
        crate::api::periods::get_period,
        crate::api::periods::update_period,
        crate::api::periods::delete_period,
        crate::api::periods::activate_period,
        crate::api::periods::close_period,
        crate::api::periods::get_paths,
        crate::api::periods::create_path,
        crate::api::periods::update_path,
        crate::api::periods::delete_path,
        
        // Registration endpoints
        crate::api::registrations::list_registrations,
        crate::api::registrations::create_registration,
        crate::api::registrations::get_registration,
        crate::api::registrations::update_registration,
        crate::api::registrations::submit_registration,
        crate::api::registrations::list_documents,
        crate::api::registrations::upload_document,
        crate::api::registrations::delete_document,
        
        // Selection endpoints
        crate::api::selection::calculate_scores,
        crate::api::selection::update_rankings,
        crate::api::selection::get_rankings,
        crate::api::selection::get_ranking_stats,
        
        // Announcement endpoints
        crate::api::announcements::run_selection,
        crate::api::announcements::announce_results,
        crate::api::announcements::get_selection_summary,
        crate::api::announcements::check_result,
        
        // Verification endpoints
        crate::api::verifications::get_pending_verifications,
        crate::api::verifications::get_verification_stats,
        crate::api::verifications::verify_registration,
        crate::api::verifications::reject_registration,
        crate::api::verifications::verify_document,
    ),
    components(
        schemas(
            // Common schemas
            ErrorResponse,
            ValidationErrorResponse,
            FieldError,
            MessageResponse,
            PaginationMeta,
            
            // Auth DTOs
            crate::dto::auth_dto::RegisterRequest,
            crate::dto::auth_dto::LoginRequest,
            crate::dto::auth_dto::AuthResponse,
            crate::dto::auth_dto::RefreshTokenRequest,
            crate::dto::auth_dto::RefreshTokenResponse,
            crate::dto::auth_dto::UserResponse,
            
            // Auth helper DTOs
            crate::api::auth::VerifyEmailRequest,
            crate::api::auth::ForgotPasswordRequest,
            crate::api::auth::ResetPasswordRequest,
            crate::api::auth::MessageResponse,
            
            // School DTOs
            crate::api::schools::CreateSchoolRequest,
            crate::api::schools::UpdateSchoolRequest,
            crate::api::schools::SchoolResponse,
            crate::api::schools::ListSchoolsResponse,
            
            // User DTOs
            crate::api::users::CreateUserRequest,
            crate::api::users::UpdateUserRequest,
            crate::api::users::ChangePasswordRequest,
            crate::api::users::UserResponse,
            crate::api::users::ListUsersResponse,
            
            // Period DTOs
            crate::api::periods::CreatePeriodRequest,
            crate::api::periods::UpdatePeriodRequest,
            crate::api::periods::CreatePathRequest,
            crate::api::periods::UpdatePathRequest,
            crate::api::periods::PeriodResponse,
            crate::api::periods::PeriodWithPathsResponse,
            crate::api::periods::PathResponse,
            crate::api::periods::ListPeriodsResponse,
            crate::api::periods::ListPeriodsQuery,
            crate::api::periods::MessageResponse,
            
            // Registration DTOs
            crate::api::registrations::CreateRegistrationRequest,
            crate::api::registrations::UpdateRegistrationRequest,
            crate::api::registrations::UploadDocumentRequest,
            crate::api::registrations::RegistrationResponse,
            crate::api::registrations::DocumentResponse,
            crate::api::registrations::ListRegistrationsResponse,
            crate::api::registrations::ListRegistrationsQuery,
            crate::api::registrations::MessageResponse,
            
            // Selection DTOs
            crate::api::selection::CalculateScoresResponse,
            crate::api::selection::UpdateRankingsResponse,
            crate::api::selection::RankingResponse,
            crate::api::selection::RankingsResponse,
            crate::api::selection::GetRankingsQuery,
            crate::services::selection_service::PathRankingStats,
            
            // Announcement DTOs
            crate::api::announcements::RunSelectionResponse,
            crate::api::announcements::AnnounceResultsResponse,
            crate::api::announcements::CheckResultQuery,
            crate::services::announcement_service::SelectionResult,
            crate::services::announcement_service::AnnouncementResult,
            crate::services::announcement_service::ResultCheckResponse,
            crate::services::announcement_service::SelectionSummary,
            crate::services::announcement_service::PathSelectionSummary,
            
            // Verification DTOs
            crate::api::verifications::PendingVerificationsQuery,
            crate::api::verifications::StatsQuery,
            crate::api::verifications::RejectRegistrationRequest,
            crate::api::verifications::VerifyDocumentRequest,
            crate::api::verifications::RegistrationResponse,
            crate::api::verifications::PendingVerificationsResponse,
            crate::api::verifications::MessageResponse,
            crate::services::verification_service::VerificationStats,
            
            // Health check
            crate::api::health::HealthResponse,
            
            // Common Enums
            crate::models::enums_docs::UserRole,
            crate::models::enums_docs::SchoolStatus,
            crate::models::enums_docs::PeriodStatus,
            crate::models::enums_docs::Level,
            crate::models::enums_docs::PathType,
            crate::models::enums_docs::RegistrationStatus,
            crate::models::enums_docs::DocumentType,
            crate::models::enums_docs::VerificationStatus,
        )
    ),
    tags(
        (name = "System", description = "System health and status endpoints"),
        (name = "Authentication", description = "User authentication and authorization"),
        (name = "Schools", description = "School management (SuperAdmin only)"),
        (name = "Users", description = "User management"),
        (name = "Periods", description = "PPDB period and registration path management"),
        (name = "Registrations", description = "Student registration and document management"),
        (name = "Selection", description = "Selection scoring, ranking, and announcement"),
        (name = "Verifications", description = "Document and registration verification"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Add JWT Bearer authentication to OpenAPI spec
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "Enter your JWT token.\n\n\
                            Example: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
                        ))
                        .build(),
                ),
            );
        }
    }
}

/// Standard error response
#[derive(utoipa::ToSchema, serde::Serialize)]
#[schema(example = json!({
    "error": "Invalid credentials",
    "code": "AUTH_001"
}))]
pub struct ErrorResponse {
    /// Human-readable error message
    pub error: String,
    
    /// Optional error code for client handling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Validation error response
#[derive(utoipa::ToSchema, serde::Serialize)]
#[schema(example = json!({
    "error": "Validation failed",
    "fields": [
        {
            "field": "email",
            "message": "Invalid email format"
        }
    ]
}))]
pub struct ValidationErrorResponse {
    /// Error message
    pub error: String,
    
    /// List of field validation errors
    pub fields: Vec<FieldError>,
}

/// Field validation error
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct FieldError {
    /// Field name that failed validation
    pub field: String,
    
    /// Validation error message
    pub message: String,
}

/// Success message response
#[derive(utoipa::ToSchema, serde::Serialize)]
#[schema(example = json!({
    "message": "Operation completed successfully"
}))]
pub struct MessageResponse {
    /// Success message
    pub message: String,
}

/// Pagination metadata
#[derive(utoipa::ToSchema, serde::Serialize)]
#[schema(example = json!({
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
}))]
pub struct PaginationMeta {
    /// Current page number
    pub page: i64,
    
    /// Items per page
    pub per_page: i64,
    
    /// Total number of items
    pub total: i64,
    
    /// Total number of pages
    pub total_pages: i64,
}

/// Paginated response wrapper
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct PaginatedResponse<T> {
    /// List of items
    pub data: Vec<T>,
    
    /// Pagination metadata
    pub meta: PaginationMeta,
}
