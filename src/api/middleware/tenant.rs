use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

use super::auth::AuthUser;

pub async fn tenant_context(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Get authenticated user from extensions
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();

    // Store school_id in request extensions for easy access
    if let Some(school_id) = auth_user.school_id {
        req.extensions_mut().insert(school_id);
    }

    // Note: PostgreSQL session variable for RLS will be set in repository layer
    // when executing queries, using the school_id from extensions

    Ok(next.run(req).await)
}

// Helper to extract school_id from request
pub fn get_school_id(req: &Request) -> Option<i32> {
    req.extensions().get::<i32>().copied()
}
