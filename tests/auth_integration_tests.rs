mod test_helpers;

use axum::http::StatusCode;
use serde_json::json;
use test_helpers::{parse_json, TestAuthResponse, TestContext};


#[tokio::test]
async fn test_auth_register_success() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, response) = ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "newuser@test.com",
            "password": "password123",
            "full_name": "New User",
            "phone": "081234567890",
            "nik": "1234567890123456"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["email"], "newuser@test.com");
    assert_eq!(response["full_name"], "New User");
    assert_eq!(response["role"], "parent");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_register_invalid_email() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, _response) = ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "invalid-email",
            "password": "password123",
            "full_name": "Test User"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    
    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_register_short_password() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, _response) = ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "test@test.com",
            "password": "short",
            "full_name": "Test User"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    
    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_register_duplicate_email() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    // First registration
    ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "duplicate@test.com",
            "password": "password123",
            "full_name": "First User"
        }),
        None
    ).await;

    // Duplicate registration
    let (status, _response) = ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "duplicate@test.com",
            "password": "password123",
            "full_name": "Second User"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::CONFLICT);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_login_success() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    // Register user first
    ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "logintest@test.com",
            "password": "password123",
            "full_name": "Login Test User"
        }),
        None
    ).await;

    // Mark email as verified
    sqlx::query!("UPDATE users SET email_verified = true WHERE email = $1", "logintest@test.com")
        .execute(&ctx.db)
        .await
        .expect("Failed to verify email");

    // Login
    let (status, response) = ctx.post(
        "/api/v1/auth/login",
        json!({
            "email": "logintest@test.com",
            "password": "password123"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let auth_response: TestAuthResponse = parse_json(response);
    assert!(!auth_response.access_token.is_empty());
    assert!(!auth_response.refresh_token.is_empty());
    assert_eq!(auth_response.token_type, "Bearer");
    assert_eq!(auth_response.user.email, "logintest@test.com");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_login_wrong_password() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    // Register user
    ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "wrongpass@test.com",
            "password": "password123",
            "full_name": "Wrong Pass User"
        }),
        None
    ).await;

    // Login with wrong password
    let (status, _response) = ctx.post(
        "/api/v1/auth/login",
        json!({
            "email": "wrongpass@test.com",
            "password": "wrongpassword"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_login_nonexistent_user() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, _response) = ctx.post(
        "/api/v1/auth/login",
        json!({
            "email": "nonexistent@test.com",
            "password": "password123"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    
    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_get_current_user() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, response) = ctx.get(
        "/api/v1/auth/me",
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["email"], super_admin.email);
    assert_eq!(response["role"], "super_admin");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_get_current_user_unauthorized() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, _response) = ctx.get(
        "/api/v1/auth/me",
        None
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    
    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_refresh_token() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, response) = ctx.post(
        "/api/v1/auth/refresh",
        json!({
            "refresh_token": super_admin.refresh_token
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(!response["access_token"].as_str().unwrap().is_empty());
    assert_eq!(response["token_type"], "Bearer");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_refresh_token_invalid() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, _response) = ctx.post(
        "/api/v1/auth/refresh",
        json!({
            "refresh_token": "invalid_token"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    
    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_logout() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, response) = ctx.post(
        "/api/v1/auth/logout",
        json!({}),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["message"].as_str().unwrap().contains("Logged out"));

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_auth_forgot_password() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    // Register user
    ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "forgot@test.com",
            "password": "password123",
            "full_name": "Forgot Password User"
        }),
        None
    ).await;

    let (status, _response) = ctx.post(
        "/api/v1/auth/forgot-password",
        json!({
            "email": "forgot@test.com"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::OK);

    ctx.cleanup_test_data().await;
}
