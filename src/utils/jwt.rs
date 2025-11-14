use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,           // user_id
    pub email: String,
    pub role: String,
    pub school_id: Option<i32>,
    pub exp: i64,           // expiration timestamp
    pub iat: i64,           // issued at timestamp
    pub token_type: String, // "access" or "refresh"
}

pub fn generate_token(
    user_id: i32,
    email: String,
    role: String,
    school_id: Option<i32>,
    secret: &str,
    expiration_hours: i64,
) -> AppResult<String> {
    generate_token_with_type(
        user_id,
        email,
        role,
        school_id,
        secret,
        expiration_hours,
        "access",
    )
}

pub fn generate_token_with_type(
    user_id: i32,
    email: String,
    role: String,
    school_id: Option<i32>,
    secret: &str,
    expiration_hours: i64,
    token_type: &str,
) -> AppResult<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiration_hours);

    let claims = Claims {
        sub: user_id,
        email,
        role,
        school_id,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        token_type: token_type.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
}

pub fn generate_refresh_token(
    user_id: i32,
    email: String,
    role: String,
    school_id: Option<i32>,
    secret: &str,
) -> AppResult<String> {
    // Refresh token expires in 7 days
    generate_token_with_type(
        user_id,
        email,
        role,
        school_id,
        secret,
        24 * 7, // 7 days
        "refresh",
    )
}

pub fn verify_token(token: &str, secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))
}

pub fn verify_refresh_token(token: &str, secret: &str) -> AppResult<Claims> {
    let claims = verify_token(token, secret)?;
    
    if claims.token_type != "refresh" {
        return Err(AppError::Authentication("Invalid token type".to_string()));
    }
    
    Ok(claims)
}
