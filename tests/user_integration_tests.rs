mod test_helpers;

use axum::http::StatusCode;
use serde_json::json;
use test_helpers::TestContext;

// ============================================================================
// List Users Tests
// ============================================================================

#[tokio::test]
async fn test_user_list_as_super_admin() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST001").await;
    
    // Create some test users
    ctx.create_school_admin(school_id, "admin1@test.com").await;
    ctx.create_parent(school_id, "parent1@test.com").await;

    let (status, response) = ctx.get(
        "/api/v1/users?page=1&page_size=10",
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["users"].as_array().unwrap().len() >= 3);
    assert!(response["total"].as_i64().unwrap() >= 3);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_list_as_school_admin() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST002").await;
    let school_admin = ctx.create_school_admin(school_id, "admin2@test.com").await;
    
    // Create parent in same school
    ctx.create_parent(school_id, "parent2@test.com").await;

    let (status, response) = ctx.get(
        "/api/v1/users?page=1&page_size=10",
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    // School admin should only see users from their school
    let users = response["users"].as_array().unwrap();
    for user in users {
        if user["school_id"].is_null() {
            continue; // Skip super_admin
        }
        assert_eq!(user["school_id"], school_id);
    }

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_list_filter_by_role() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST003").await;
    
    ctx.create_school_admin(school_id, "admin3@test.com").await;
    ctx.create_parent(school_id, "parent3@test.com").await;

    // Filter by parent role
    let (status, response) = ctx.get(
        "/api/v1/users?role=parent",
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let users = response["users"].as_array().unwrap();
    for user in users {
        assert_eq!(user["role"], "parent");
    }

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_list_search() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST004").await;
    
    // Create user with unique name
    let unique_email = format!("unique{}@test.com", chrono::Utc::now().timestamp());
    ctx.create_parent(school_id, &unique_email).await;

    let (status, response) = ctx.get(
        &format!("/api/v1/users?search={}", unique_email),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let users = response["users"].as_array().unwrap();
    assert!(users.iter().any(|u| u["email"] == unique_email));

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_list_unauthorized() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let (status, _response) = ctx.get(
        "/api/v1/users",
        None
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Get User Tests
// ============================================================================

#[tokio::test]
async fn test_user_get_by_id() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST005").await;
    let school_admin = ctx.create_school_admin(school_id, "admin5@test.com").await;

    let (status, response) = ctx.get(
        &format!("/api/v1/users/{}", school_admin.id),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["id"], school_admin.id);
    assert_eq!(response["email"], "admin5@test.com");
    assert_eq!(response["role"], "school_admin");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_get_current_user() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, response) = ctx.get(
        "/api/v1/users/me",
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["email"], super_admin.email);
    assert_eq!(response["role"], "super_admin");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_get_nonexistent() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, _response) = ctx.get(
        "/api/v1/users/99999",
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Update User Tests
// ============================================================================

#[tokio::test]
async fn test_user_update_profile() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST006").await;
    let school_admin = ctx.create_school_admin(school_id, "admin6@test.com").await;

    let (status, response) = ctx.put(
        &format!("/api/v1/users/{}", school_admin.id),
        json!({
            "full_name": "Updated Admin Name",
            "phone": "089999999999"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["full_name"], "Updated Admin Name");
    assert_eq!(response["phone"], "089999999999");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_update_current_user() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, response) = ctx.put(
        "/api/v1/users/me",
        json!({
            "full_name": "Updated Super Admin",
            "phone": "081111111111"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["full_name"], "Updated Super Admin");
    assert_eq!(response["phone"], "081111111111");

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Change Password Tests
// ============================================================================

#[tokio::test]
async fn test_user_change_password_success() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, response) = ctx.post(
        "/api/v1/users/me/change-password",
        json!({
            "old_password": "password123",
            "new_password": "newpassword123"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["message"].as_str().unwrap().contains("successfully"));

    // Verify can login with new password
    let (login_status, _login_response) = ctx.post(
        "/api/v1/auth/login",
        json!({
            "email": super_admin.email,
            "password": "newpassword123"
        }),
        None
    ).await;

    assert_eq!(login_status, StatusCode::OK);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_change_password_wrong_old_password() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, _response) = ctx.post(
        "/api/v1/users/me/change-password",
        json!({
            "old_password": "wrongpassword",
            "new_password": "newpassword123"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_change_password_too_short() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, _response) = ctx.post(
        "/api/v1/users/me/change-password",
        json!({
            "old_password": "password123",
            "new_password": "short"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Create User Tests (RBAC)
// ============================================================================

#[tokio::test]
async fn test_user_create_as_super_admin() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST007").await;

    let unique_email = format!("newuser{}@test.com", chrono::Utc::now().timestamp());
    
    let (status, response) = ctx.post(
        "/api/v1/users",
        json!({
            "email": unique_email,
            "password": "password123",
            "full_name": "New User",
            "phone": "081234567890",
            "role": "school_admin",
            "school_id": school_id
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["email"], unique_email);
    assert_eq!(response["role"], "school_admin");
    assert_eq!(response["school_id"], school_id);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_create_as_school_admin() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST008").await;
    let school_admin = ctx.create_school_admin(school_id, "admin8@test.com").await;

    let unique_email = format!("parent{}@test.com", chrono::Utc::now().timestamp());
    
    let (status, response) = ctx.post(
        "/api/v1/users",
        json!({
            "email": unique_email,
            "password": "password123",
            "full_name": "New Parent",
            "phone": "081234567890",
            "role": "parent"
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["email"], unique_email);
    assert_eq!(response["role"], "parent");
    // School admin creates users in their own school
    assert_eq!(response["school_id"], school_id);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_create_invalid_role() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;

    let (status, _response) = ctx.post(
        "/api/v1/users",
        json!({
            "email": "test@test.com",
            "password": "password123",
            "full_name": "Test User",
            "role": "invalid_role"
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_create_duplicate_email() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST009").await;

    let email = format!("duplicate{}@test.com", chrono::Utc::now().timestamp());
    
    // Create first user
    ctx.post(
        "/api/v1/users",
        json!({
            "email": email,
            "password": "password123",
            "full_name": "First User",
            "role": "parent",
            "school_id": school_id
        }),
        Some(&super_admin.access_token)
    ).await;

    // Try to create duplicate
    let (status, _response) = ctx.post(
        "/api/v1/users",
        json!({
            "email": email,
            "password": "password123",
            "full_name": "Second User",
            "role": "parent",
            "school_id": school_id
        }),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::CONFLICT);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Delete User Tests (RBAC)
// ============================================================================

#[tokio::test]
async fn test_user_delete_as_super_admin() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST010").await;
    let school_admin = ctx.create_school_admin(school_id, "admin10@test.com").await;

    let (status, response) = ctx.delete(
        &format!("/api/v1/users/{}", school_admin.id),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["message"].as_str().unwrap().contains("deleted"));

    // Verify user is deleted
    let (get_status, _) = ctx.get(
        &format!("/api/v1/users/{}", school_admin.id),
        Some(&super_admin.access_token)
    ).await;

    assert_eq!(get_status, StatusCode::NOT_FOUND);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_delete_as_school_admin() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST011").await;
    let school_admin = ctx.create_school_admin(school_id, "admin11@test.com").await;
    let parent = ctx.create_parent(school_id, "parent11@test.com").await;

    let (status, response) = ctx.delete(
        &format!("/api/v1/users/{}", parent.id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_user_delete_as_parent_forbidden() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST012").await;
    let parent1 = ctx.create_parent(school_id, "parent12a@test.com").await;
    let parent2 = ctx.create_parent(school_id, "parent12b@test.com").await;

    // Parent should not be able to delete other users
    let (status, _response) = ctx.delete(
        &format!("/api/v1/users/{}", parent2.id),
        Some(&parent1.access_token)
    ).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup_test_data().await;
}
