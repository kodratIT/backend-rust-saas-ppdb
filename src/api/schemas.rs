//! OpenAPI Schema Definitions
//! 
//! This module contains all schema definitions for OpenAPI documentation.
//! These schemas are used across multiple endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Standard success message response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "Operation completed successfully"
}))]
pub struct MessageResponse {
    /// Success message
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

/// Email verification request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "token": "abc123def456"
}))]
pub struct VerifyEmailRequest {
    /// Email verification token
    #[schema(example = "abc123def456")]
    pub token: String,
}

/// Forgot password request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "user@example.com"
}))]
pub struct ForgotPasswordRequest {
    /// User email address
    #[schema(example = "user@example.com", format = "email")]
    pub email: String,
}

/// Reset password request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "token": "reset_token_123",
    "new_password": "newpassword123"
}))]
pub struct ResetPasswordRequest {
    /// Password reset token
    #[schema(example = "reset_token_123")]
    pub token: String,
    
    /// New password (minimum 8 characters)
    #[schema(example = "newpassword123", min_length = 8)]
    pub new_password: String,
}

/// Pagination query parameters
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PaginationParams {
    /// Page number (starts from 1)
    #[param(example = 1, minimum = 1)]
    pub page: Option<i64>,
    
    /// Items per page
    #[param(example = 20, minimum = 1, maximum = 100)]
    pub per_page: Option<i64>,
}

/// Search query parameters
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchParams {
    /// Search query string
    #[param(example = "Jakarta")]
    pub search: Option<String>,
    
    /// Page number
    #[param(example = 1, minimum = 1)]
    pub page: Option<i64>,
    
    /// Items per page
    #[param(example = 20, minimum = 1, maximum = 100)]
    pub per_page: Option<i64>,
}

/// Filter by status
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct StatusFilterParams {
    /// Filter by status
    #[param(example = "active")]
    pub status: Option<String>,
    
    /// Page number
    #[param(example = 1, minimum = 1)]
    pub page: Option<i64>,
    
    /// Items per page
    #[param(example = 20, minimum = 1, maximum = 100)]
    pub per_page: Option<i64>,
}
