mod common;

use axum::http::StatusCode;
use common::*;

#[tokio::test]
async fn test_register_user_success() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    // Clean up first
    cleanup_test_data(&state.db).await;

    let (status, body) = register_test_user(
        &mut router,
        "newuser@test.com",
        "password123",
        "Test User",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    
    let response: serde_json::Value = parse_json_response(&body);
    assert_eq!(response["email"], "newuser@test.com");
    assert_eq!(response["role"], "parent");
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Register first user
    register_test_user(&mut router, "duplicate@test.com", "password123", "User 1").await;

    // Try to register with same email
    let (status, body) = register_test_user(
        &mut router,
        "duplicate@test.com",
        "password456",
        "User 2",
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.contains("Email already registered"));
}

#[tokio::test]
async fn test_login_success() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Register user
    register_test_user(&mut router, "login@test.com", "password123", "Login User").await;

    // Verify email (in real scenario, this would be done via email link)
    sqlx::query("UPDATE users SET email_verified = true WHERE email = 'login@test.com'")
        .execute(&state.db)
        .await
        .unwrap();

    // Login
    let body = serde_json::json!({
        "email": "login@test.com",
        "password": "password123"
    });

    let (status, response_body) = make_json_request(&mut router, "POST", "/auth/login", Some(body), None).await;

    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = parse_json_response(&response_body);
    assert!(response["access_token"].is_string());
    assert!(response["refresh_token"].is_string());
    assert_eq!(response["token_type"], "Bearer");
    assert_eq!(response["user"]["email"], "login@test.com");
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    let body = serde_json::json!({
        "email": "nonexistent@test.com",
        "password": "wrongpassword"
    });

    let (status, body) = make_json_request(&mut router, "POST", "/auth/login", Some(body), None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.contains("Invalid credentials"));
}

#[tokio::test]
async fn test_get_current_user() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Register and verify user
    register_test_user(&mut router, "current@test.com", "password123", "Current User").await;
    sqlx::query("UPDATE users SET email_verified = true WHERE email = 'current@test.com'")
        .execute(&state.db)
        .await
        .unwrap();

    // Login to get token
    let token = login_and_get_token(&mut router, "current@test.com", "password123").await;

    // Get current user
    let (status, body) = make_json_request::<()>(&mut router, "GET", "/auth/me", None, Some(&token)).await;

    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = parse_json_response(&body);
    assert_eq!(response["email"], "current@test.com");
}

#[tokio::test]
async fn test_refresh_token() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Register and verify user
    register_test_user(&mut router, "refresh@test.com", "password123", "Refresh User").await;
    sqlx::query("UPDATE users SET email_verified = true WHERE email = 'refresh@test.com'")
        .execute(&state.db)
        .await
        .unwrap();

    // Login to get tokens
    let body = serde_json::json!({
        "email": "refresh@test.com",
        "password": "password123"
    });

    let (_, response_body) = make_json_request(&mut router, "POST", "/auth/login", Some(body), None).await;
    let login_response: serde_json::Value = parse_json_response(&response_body);
    let refresh_token = login_response["refresh_token"].as_str().unwrap();

    // Use refresh token
    let body = serde_json::json!({
        "refresh_token": refresh_token
    });

    let (status, response_body) = make_json_request(&mut router, "POST", "/auth/refresh", Some(body), None).await;

    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = parse_json_response(&response_body);
    assert!(response["access_token"].is_string());
}

#[tokio::test]
async fn test_unauthorized_access() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state);

    // Try to access protected endpoint without token
    let (status, _) = make_json_request::<()>(&mut router, "GET", "/auth/me", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_token() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state);

    // Try to access with invalid token
    let (status, _) = make_json_request::<()>(
        &mut router,
        "GET",
        "/auth/me",
        None,
        Some("invalid-token"),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
