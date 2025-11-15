use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// User registration request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "email": "parent@example.com",
    "password": "password123",
    "full_name": "Ahmad Rizki",
    "phone": "+628123456789",
    "nik": "3201234567890123"
}))]
pub struct RegisterRequest {
    /// User email address
    #[validate(email)]
    #[schema(example = "parent@example.com", format = "email")]
    pub email: String,
    
    /// Password (minimum 8 characters)
    #[validate(length(min = 8))]
    #[schema(example = "password123", min_length = 8)]
    pub password: String,
    
    /// Full name (minimum 3 characters)
    #[validate(length(min = 3))]
    #[schema(example = "Ahmad Rizki", min_length = 3)]
    pub full_name: String,
    
    /// Phone number (optional)
    #[schema(example = "+628123456789")]
    pub phone: Option<String>,
    
    /// NIK - Nomor Induk Kependudukan (optional)
    #[schema(example = "3201234567890123", min_length = 16, max_length = 16)]
    pub nik: Option<String>,
}

/// User login request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "email": "admin@example.com",
    "password": "password123"
}))]
pub struct LoginRequest {
    /// User email address
    #[validate(email)]
    #[schema(example = "admin@example.com", format = "email")]
    pub email: String,
    
    /// User password
    #[schema(example = "password123")]
    pub password: String,
}

/// Authentication response with tokens and user info
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 86400,
    "user": {
        "id": 1,
        "email": "admin@example.com",
        "full_name": "Admin User",
        "role": "school_admin",
        "school_id": 1
    }
}))]
pub struct AuthResponse {
    /// JWT access token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    
    /// JWT refresh token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
    
    /// Token expiration time in seconds
    #[schema(example = 86400)]
    pub expires_in: i64,
    
    /// User information
    pub user: UserResponse,
}

/// Refresh token request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}))]
pub struct RefreshTokenRequest {
    /// Refresh token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

/// Refresh token response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 86400
}))]
pub struct RefreshTokenResponse {
    /// New JWT access token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
    
    /// Token expiration time in seconds
    #[schema(example = 86400)]
    pub expires_in: i64,
}

/// User information response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": 1,
    "email": "admin@example.com",
    "full_name": "Admin User",
    "role": "school_admin",
    "school_id": 1
}))]
pub struct UserResponse {
    /// User ID
    #[schema(example = 1)]
    pub id: i32,
    
    /// User email
    #[schema(example = "admin@example.com")]
    pub email: String,
    
    /// User full name
    #[schema(example = "Admin User")]
    pub full_name: String,
    
    /// User role (super_admin, school_admin, parent)
    #[schema(example = "school_admin")]
    pub role: String,
    
    /// School ID (null for super_admin)
    #[schema(example = 1)]
    pub school_id: Option<i32>,
}
