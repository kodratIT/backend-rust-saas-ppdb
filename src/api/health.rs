/// Health check endpoint with OpenAPI documentation
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::error::AppResult;
use crate::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "healthy",
    "version": "1.0.0",
    "database": "connected"
}))]
pub struct HealthResponse {
    /// Service status
    #[schema(example = "healthy")]
    pub status: String,
    
    /// API version
    #[schema(example = "1.0.0")]
    pub version: String,
    
    /// Database connection status
    #[schema(example = "connected")]
    pub database: String,
}

/// Health check endpoint
/// 
/// Returns the health status of the API and its dependencies.
/// This endpoint does not require authentication.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "System",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = ErrorResponse)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let response = HealthResponse {
        status: if db_status == "connected" {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status.to_string(),
    };

    Ok(Json(response))
}
