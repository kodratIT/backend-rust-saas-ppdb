use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::PgPool;
use tower::ServiceExt;

use ppdb_backend::{api, AppState, Config};

/// Test helper to create app state
pub async fn create_test_app_state() -> AppState {
    let config = Config {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/ppdb_test".to_string()),
        jwt_secret: "test-secret-key-for-testing-only".to_string(),
        jwt_expiration_hours: 24,
        port: 8080,
    };

    let pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    AppState {
        db: pool,
        config,
    }
}

/// Test helper to create router
pub fn create_test_router(state: AppState) -> Router {
    api::routes(state)
}

/// Test helper to make JSON request
pub async fn make_json_request<T: Serialize>(
    router: &mut Router,
    method: &str,
    uri: &str,
    body: Option<T>,
    token: Option<&str>,
) -> (StatusCode, String) {
    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    if let Some(token) = token {
        request_builder = request_builder.header("authorization", format!("Bearer {}", token));
    }

    let request = if let Some(body) = body {
        let json = serde_json::to_string(&body).unwrap();
        request_builder.body(Body::from(json)).unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    (status, body_string)
}

/// Test helper to parse JSON response
pub fn parse_json_response<T: DeserializeOwned>(body: &str) -> T {
    serde_json::from_str(body).expect("Failed to parse JSON response")
}

/// Test helper to register a test user
pub async fn register_test_user(
    router: &mut Router,
    email: &str,
    password: &str,
    full_name: &str,
) -> (StatusCode, String) {
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "full_name": full_name,
        "phone": "081234567890",
        "nik": "1234567890123456"
    });

    make_json_request(router, "POST", "/auth/register", Some(body), None).await
}

/// Test helper to login and get token
pub async fn login_and_get_token(
    router: &mut Router,
    email: &str,
    password: &str,
) -> String {
    let body = serde_json::json!({
        "email": email,
        "password": password
    });

    let (status, response_body) = make_json_request(router, "POST", "/auth/login", Some(body), None).await;
    
    assert_eq!(status, StatusCode::OK, "Login failed: {}", response_body);
    
    let response: serde_json::Value = parse_json_response(&response_body);
    response["access_token"].as_str().unwrap().to_string()
}

/// Test helper to clean up test data
pub async fn cleanup_test_data(pool: &PgPool) {
    // Clean up in reverse order of dependencies
    sqlx::query("DELETE FROM documents").execute(pool).await.ok();
    sqlx::query("DELETE FROM registrations").execute(pool).await.ok();
    sqlx::query("DELETE FROM registration_paths").execute(pool).await.ok();
    sqlx::query("DELETE FROM periods").execute(pool).await.ok();
    sqlx::query("DELETE FROM users WHERE email LIKE '%@test.com'").execute(pool).await.ok();
    sqlx::query("DELETE FROM schools WHERE code LIKE 'TEST%'").execute(pool).await.ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_app_state() {
        let state = create_test_app_state().await;
        assert!(!state.config.jwt_secret.is_empty());
    }
}
