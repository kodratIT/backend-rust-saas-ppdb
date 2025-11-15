mod test_helpers;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use test_helpers::TestContext;

// Helper function to get NaiveDate from DateTime
fn to_naive_date(dt: chrono::DateTime<Utc>) -> chrono::NaiveDate {
    dt.date_naive()
}

// Helper function to setup period and path
async fn setup_period_and_path(ctx: &TestContext, school_admin_token: &str) -> (i32, i32) {
    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(120));

    let (status, response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": [
                {
                    "path_type": "zonasi",
                    "name": "Jalur Zonasi",
                    "quota": 50,
                    "scoring_config": {}
                }
            ]
        }),
        Some(school_admin_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Failed to create period: {:?}", response);

    let period_id = response["period"]["id"].as_i64().unwrap() as i32;
    let path_id = response["paths"][0]["id"].as_i64().unwrap() as i32;

    // Activate period
    let (status, _) = ctx.post(
        &format!("/api/v1/periods/{}/activate", period_id),
        json!({}),
        Some(school_admin_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Failed to activate period");

    (period_id, path_id)
}

// ============================================================================
// Create Registration Tests
// ============================================================================

#[tokio::test]
async fn test_registration_create_success() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST300").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_reg@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_reg@test.com").await;

    let (period_id, path_id) = setup_period_and_path(&ctx, &school_admin.access_token).await;

    let birth_date = Utc::now() - Duration::days(365 * 15); // 15 years old

    let (status, response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "1234567890",
            "student_name": "Test Student",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Jl. Test No. 123",
            "student_email": "student@test.com",
            "parent_name": "Parent Test",
            "parent_nik": "1234567890123456",
            "parent_phone": "081234567890",
            "path_data": {
                "distance_km": 2.5
            }
        }),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["student_nisn"], "1234567890");
    assert_eq!(response["status"], "draft");

    ctx.cleanup_test_data().await;
}

// ============================================================================
// List Registration Tests
// ============================================================================

#[tokio::test]
async fn test_registration_list_parent_only_sees_own() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST301").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_reg2@test.com").await;
    let parent1 = ctx.create_parent(school_id, "parent_reg1@test.com").await;
    let parent2 = ctx.create_parent(school_id, "parent_reg2@test.com").await;

    let (period_id, path_id) = setup_period_and_path(&ctx, &school_admin.access_token).await;
    let birth_date = Utc::now() - Duration::days(365 * 15);

    // Parent 1 creates registration
    ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "1111111111",
            "student_name": "Student 1",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Address 1",
            "student_email": "student1@test.com",
            "parent_name": "Parent 1",
            "parent_nik": "1111111111111111",
            "parent_phone": "081111111111",
            "path_data": {}
        }),
        Some(&parent1.access_token)
    ).await;

    // Parent 2 creates registration
    ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "2222222222",
            "student_name": "Student 2",
            "student_gender": "P",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Address 2",
            "student_email": "student2@test.com",
            "parent_name": "Parent 2",
            "parent_nik": "2222222222222222",
            "parent_phone": "082222222222",
            "path_data": {}
        }),
        Some(&parent2.access_token)
    ).await;

    // Parent 1 should only see their own registration
    let (status, response) = ctx.get(
        "/api/v1/registrations",
        Some(&parent1.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let registrations = response["registrations"].as_array().unwrap();
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0]["student_nisn"], "1111111111");

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Update Registration Tests
// ============================================================================

#[tokio::test]
async fn test_registration_update() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST302").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_reg3@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_reg3@test.com").await;

    let (period_id, path_id) = setup_period_and_path(&ctx, &school_admin.access_token).await;
    let birth_date = Utc::now() - Duration::days(365 * 15);

    let (_, create_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "3333333333",
            "student_name": "Student 3",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Old Address",
            "student_email": "student3@test.com",
            "parent_name": "Parent 3",
            "parent_nik": "3333333333333333",
            "parent_phone": "083333333333",
            "path_data": {}
        }),
        Some(&parent.access_token)
    ).await;

    let registration_id = create_response["id"].as_i64().unwrap();

    let (status, response) = ctx.put(
        &format!("/api/v1/registrations/{}", registration_id),
        json!({
            "student_address": "New Address Updated"
        }),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["student_address"], "New Address Updated");

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Submit Registration Tests
// ============================================================================

#[tokio::test]
async fn test_registration_submit() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST303").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_reg4@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_reg4@test.com").await;

    let (period_id, path_id) = setup_period_and_path(&ctx, &school_admin.access_token).await;
    let birth_date = Utc::now() - Duration::days(365 * 15);

    let (_, create_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "4444444444",
            "student_name": "Student 4",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Address 4",
            "student_email": "student4@test.com",
            "parent_name": "Parent 4",
            "parent_nik": "4444444444444444",
            "parent_phone": "084444444444",
            "path_data": {}
        }),
        Some(&parent.access_token)
    ).await;

    let registration_id = create_response["id"].as_i64().unwrap();

    // Upload required document first
    ctx.post(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        json!({
            "document_type": "birth_certificate",
            "file_url": "https://example.com/birth_cert.pdf",
            "file_name": "birth_certificate.pdf",
            "file_size": 102400,
            "mime_type": "application/pdf"
        }),
        Some(&parent.access_token)
    ).await;

    let (status, response) = ctx.post(
        &format!("/api/v1/registrations/{}/submit", registration_id),
        json!({}),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["status"], "submitted");
    assert!(response["registration_number"].as_str().is_some());

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Document Upload Tests
// ============================================================================

#[tokio::test]
async fn test_registration_upload_document() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST304").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_reg5@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_reg5@test.com").await;

    let (period_id, path_id) = setup_period_and_path(&ctx, &school_admin.access_token).await;
    let birth_date = Utc::now() - Duration::days(365 * 15);

    let (_, create_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "5555555555",
            "student_name": "Student 5",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Address 5",
            "student_email": "student5@test.com",
            "parent_name": "Parent 5",
            "parent_nik": "5555555555555555",
            "parent_phone": "085555555555",
            "path_data": {}
        }),
        Some(&parent.access_token)
    ).await;

    let registration_id = create_response["id"].as_i64().unwrap();

    let (status, response) = ctx.post(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        json!({
            "document_type": "birth_certificate",
            "file_url": "https://example.com/birth_cert.pdf",
            "file_name": "birth_certificate.pdf",
            "file_size": 102400,
            "mime_type": "application/pdf"
        }),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["document_type"], "birth_certificate");
    assert_eq!(response["verification_status"], "pending");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_registration_list_documents() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST305").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_reg6@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_reg6@test.com").await;

    let (period_id, path_id) = setup_period_and_path(&ctx, &school_admin.access_token).await;
    let birth_date = Utc::now() - Duration::days(365 * 15);

    let (_, create_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "6666666666",
            "student_name": "Student 6",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Address 6",
            "student_email": "student6@test.com",
            "parent_name": "Parent 6",
            "parent_nik": "6666666666666666",
            "parent_phone": "086666666666",
            "path_data": {}
        }),
        Some(&parent.access_token)
    ).await;

    let registration_id = create_response["id"].as_i64().unwrap();

    // Upload multiple documents
    ctx.post(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        json!({
            "document_type": "birth_certificate",
            "file_url": "https://example.com/birth_cert.pdf",
            "file_name": "birth_certificate.pdf",
            "file_size": 102400,
            "mime_type": "application/pdf"
        }),
        Some(&parent.access_token)
    ).await;

    ctx.post(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        json!({
            "document_type": "family_card",
            "file_url": "https://example.com/family_card.pdf",
            "file_name": "family_card.pdf",
            "file_size": 204800,
            "mime_type": "application/pdf"
        }),
        Some(&parent.access_token)
    ).await;

    let (status, response) = ctx.get(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let documents = response.as_array().unwrap();
    assert_eq!(documents.len(), 2);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Parent Isolation Tests
// ============================================================================

#[tokio::test]
async fn test_registration_parent_cannot_see_others() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST306").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_reg7@test.com").await;
    let parent1 = ctx.create_parent(school_id, "parent_reg7@test.com").await;
    let parent2 = ctx.create_parent(school_id, "parent_reg8@test.com").await;

    let (period_id, path_id) = setup_period_and_path(&ctx, &school_admin.access_token).await;
    let birth_date = Utc::now() - Duration::days(365 * 15);

    let (_, create_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "7777777777",
            "student_name": "Student 7",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Address 7",
            "student_email": "student7@test.com",
            "parent_name": "Parent 7",
            "parent_nik": "7777777777777777",
            "parent_phone": "087777777777",
            "path_data": {}
        }),
        Some(&parent1.access_token)
    ).await;

    let registration_id = create_response["id"].as_i64().unwrap();

    // Parent 2 tries to access Parent 1's registration
    let (status, _response) = ctx.get(
        &format!("/api/v1/registrations/{}", registration_id),
        Some(&parent2.access_token)
    ).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup_test_data().await;
}
