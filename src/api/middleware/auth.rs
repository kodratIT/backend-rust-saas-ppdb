use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::utils::{error::AppError, jwt};
use crate::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub role: String,
    pub school_id: Option<i32>,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract token
    let token = auth_header.trim_start_matches("Bearer ");

    // Verify token
    let claims = jwt::verify_token(token, &state.config.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Ensure it's an access token
    if claims.token_type != "access" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Create AuthUser from claims
    let auth_user = AuthUser {
        id: claims.sub,
        email: claims.email,
        role: claims.role,
        school_id: claims.school_id,
    };

    // Insert user into request extensions
    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}
