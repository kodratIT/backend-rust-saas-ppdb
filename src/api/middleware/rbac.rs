use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

use super::auth::AuthUser;

pub async fn require_super_admin(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_user.role != "super_admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

pub async fn require_school_admin(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_user.role != "school_admin" && auth_user.role != "super_admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

pub async fn require_parent(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_user.role != "parent" {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

// Helper function to check multiple roles
pub fn has_role(auth_user: &AuthUser, allowed_roles: &[&str]) -> bool {
    allowed_roles.contains(&auth_user.role.as_str())
}
