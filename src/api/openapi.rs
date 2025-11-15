use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;

/// Main OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "PPDB Sekolah API",
        version = "1.0.0",
        description = "API untuk Sistem Penerimaan Peserta Didik Baru (PPDB) - Multi-tenant SaaS Platform",
        contact(
            name = "PPDB Support",
            email = "support@ppdb.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8000", description = "Development server"),
        (url = "https://api.ppdb.com", description = "Production server")
    ),
    paths(
        // Authentication endpoints
        crate::api::auth::register,
        crate::api::auth::login,
        crate::api::auth::verify_email,
        crate::api::auth::logout,
        
        // School endpoints
        crate::api::schools::list_schools,
        crate::api::schools::create_school,
        crate::api::schools::get_school,
        crate::api::schools::update_school,
        crate::api::schools::delete_school,
        
        // User endpoints
        crate::api::users::list_users,
        crate::api::users::create_user,
        crate::api::users::get_user,
        crate::api::users::update_user,
        crate::api::users::delete_user,
        crate::api::users::get_current_user,
        crate::api::users::update_current_user,
        
        // Period endpoints
        crate::api::periods::list_periods,
        crate::api::periods::create_period,
        crate::api::periods::get_period,
        crate::api::periods::update_period,
        crate::api::periods::delete_period,
        crate::api::periods::activate_period,
        crate::api::periods::close_period,
        
        // Registration endpoints
        crate::api::registrations::list_registrations,
        crate::api::registrations::create_registration,
        crate::api::registrations::get_registration,
        crate::api::registrations::update_registration,
        crate::api::registrations::submit_registration,
        crate::api::registrations::verify_registration,
        crate::api::registrations::reject_registration,
        crate::api::registrations::get_pending_verifications,
        
        // Document endpoints
        crate::api::registrations::list_documents,
        crate::api::registrations::upload_document,
        crate::api::registrations::delete_document,
        
        // Selection endpoints
        crate::api::selection::calculate_scores,
        crate::api::selection::get_rankings,
        crate::api::selection::run_selection,
        crate::api::selection::announce_results,
        crate::api::selection::check_result,
    ),
    components(
        schemas(
            // Auth DTOs
            crate::dto::auth_dto::RegisterRequest,
            crate::dto::auth_dto::LoginRequest,
            crate::dto::auth_dto::LoginResponse,
            crate::dto::auth_dto::VerifyEmailRequest,
            
            // School DTOs
            crate::dto::school_dto::CreateSchoolDto,
            crate::dto::school_dto::UpdateSchoolDto,
            crate::dto::school_dto::SchoolResponse,
            
            // User DTOs
            crate::dto::user_dto::CreateUserDto,
            crate::dto::user_dto::UpdateUserDto,
            crate::dto::user_dto::UserResponse,
            
            // Period DTOs
            crate::dto::period_dto::CreatePeriodDto,
            crate::dto::period_dto::UpdatePeriodDto,
            crate::dto::period_dto::PeriodResponse,
            crate::dto::period_dto::CreatePathDto,
            crate::dto::period_dto::PathResponse,
            
            // Registration DTOs
            crate::dto::registration_dto::CreateRegistrationDto,
            crate::dto::registration_dto::UpdateRegistrationDto,
            crate::dto::registration_dto::RegistrationResponse,
            crate::dto::registration_dto::VerifyRegistrationDto,
            crate::dto::registration_dto::RejectRegistrationDto,
            
            // Document DTOs
            crate::dto::registration_dto::DocumentResponse,
            
            // Selection DTOs
            crate::dto::selection_dto::RankingResponse,
            crate::dto::selection_dto::SelectionSummaryResponse,
            crate::dto::selection_dto::CheckResultRequest,
            crate::dto::selection_dto::CheckResultResponse,
            
            // Common types
            crate::models::user::UserRole,
            crate::models::school::SchoolStatus,
            crate::models::period::PeriodStatus,
            crate::models::period::Level,
            crate::models::registration_path::PathType,
            crate::models::registration::RegistrationStatus,
            crate::models::document::DocumentType,
            crate::models::document::VerificationStatus,
            
            // Error response
            ErrorResponse,
            ValidationError,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and authorization endpoints"),
        (name = "Schools", description = "School management endpoints (Super Admin only)"),
        (name = "Users", description = "User management endpoints"),
        (name = "Periods", description = "PPDB period and registration path management"),
        (name = "Registrations", description = "Student registration and document management"),
        (name = "Selection", description = "Selection scoring, ranking, and announcement"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security scheme for JWT Bearer token
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
                        .description(Some("Enter your JWT token in the format: Bearer <token>"))
                        .build(),
                ),
            );
        }
    }
}

/// Error response schema
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct ErrorResponse {
    /// Error message
    #[schema(example = "Invalid credentials")]
    pub error: String,
    
    /// Error code
    #[schema(example = "AUTH_001")]
    pub code: Option<String>,
    
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

/// Validation error schema
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct ValidationError {
    /// Field name that failed validation
    #[schema(example = "email")]
    pub field: String,
    
    /// Validation error message
    #[schema(example = "Invalid email format")]
    pub message: String,
}
