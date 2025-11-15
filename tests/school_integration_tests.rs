mod test_helpers;

use axum::http::StatusCode;
use serde_json::json;
use test_helpers::TestContext;

#[tokio::test]
async fn test_school_create_success() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, response) = ctx.post(
        "/api/v1/schools",
        json!({
            "name": "Test School Create",
            "npsn": "12345678",
            "code": "TEST001",
            "address": "Test Address",
            "phone": "081234567890",
            "email": "school001@test.com"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["name"], "Test School Create");
    assert_eq!(response["npsn"], "12345678");
    assert_eq!(response["code"], "TEST001");
    assert_eq!(response["status"], "active");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_create_invalid_npsn() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, _response) = ctx.post(
        "/api/v1/schools",
        json!({
            "name": "Test School",
            "npsn": "123", // Invalid: too short
            "code": "TEST002"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_create_unauthorized() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, _response) = ctx.post(
        "/api/v1/schools",
        json!({
            "name": "Test School",
            "npsn": "12345678",
            "code": "TEST003"
        }),
        None
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    
    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_create_non_super_admin() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST004").await;
    let school_admin = ctx.create_school_admin(school_id, "schooladmin@test.com").await;

    let (status, _response) = ctx.post(
        "/api/v1/schools",
        json!({
            "name": "Another School",
            "npsn": "87654321",
            "code": "TEST005"
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_list() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    
    // Create multiple schools
    ctx.create_test_school(&super_admin.access_token, "TEST010").await;
    ctx.create_test_school(&super_admin.access_token, "TEST011").await;
    ctx.create_test_school(&super_admin.access_token, "TEST012").await;

    let (status, response) = ctx.get(
        "/api/v1/schools?page=1&page_size=10",
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["schools"].as_array().unwrap().len() >= 3);
    assert!(response["total"].as_i64().unwrap() >= 3);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_list_with_search() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    
    // Use unique code with timestamp to avoid conflicts
    let unique_code = format!("SEARCH{}", chrono::Utc::now().timestamp() % 100000);
    ctx.create_test_school(&super_admin.access_token, &unique_code).await;

    let (status, response) = ctx.get(
        &format!("/api/v1/schools?search={}", unique_code),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    let schools = response["schools"].as_array().unwrap();
    assert!(schools.iter().any(|s| s["code"] == unique_code));

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_get_by_id() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST020").await;

    let (status, response) = ctx.get(
        &format!("/api/v1/schools/{}", school_id),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["id"], school_id);
    assert_eq!(response["code"], "TEST020");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_get_nonexistent() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, _response) = ctx.get(
        "/api/v1/schools/99999",
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_update() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST030").await;

    let (status, response) = ctx.put(
        &format!("/api/v1/schools/{}", school_id),
        json!({
            "name": "Updated School Name",
            "address": "Updated Address",
            "phone": "089876543210"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["name"], "Updated School Name");
    assert_eq!(response["address"], "Updated Address");
    assert_eq!(response["phone"], "089876543210");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_deactivate() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST040").await;

    let (status, _response) = ctx.delete(
        &format!("/api/v1/schools/{}", school_id),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK);
    
    // Verify school is deactivated
    let (get_status, school) = ctx.get(
        &format!("/api/v1/schools/{}", school_id),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(school["status"], "inactive");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_school_activate() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST050").await;

    // Deactivate first
    ctx.delete(
        &format!("/api/v1/schools/{}", school_id),
        Some(&super_admin.access_token)
    ).await;

    // Activate
    let (status, _response) = ctx.post(
        &format!("/api/v1/schools/{}/activate", school_id),
        json!({}),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK);
    
    // Verify school is active
    let (get_status, school) = ctx.get(
        &format!("/api/v1/schools/{}", school_id),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(school["status"], "active");

    ctx.cleanup_test_data().await;
}
