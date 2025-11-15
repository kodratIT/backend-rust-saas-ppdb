mod test_helpers;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use test_helpers::TestContext;

// Helper function to get NaiveDate from DateTime
fn to_naive_date(dt: chrono::DateTime<Utc>) -> chrono::NaiveDate {
    dt.date_naive()
}

// Helper function to setup registration for verification
async fn setup_registration_for_verification(
    ctx: &TestContext,
    school_admin_token: &str,
    parent_token: &str,
) -> (i32, i32) {
    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(120));

    // Create period
    let (status, period_response) = ctx.post(
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

    assert_eq!(status, StatusCode::CREATED, "Failed to create period: {:?}", period_response);

    let period_id = period_response["period"]["id"].as_i64().unwrap() as i32;
    let path_id = period_response["paths"][0]["id"].as_i64().unwrap() as i32;

    // Activate period
    let (status, _) = ctx.post(
        &format!("/api/v1/periods/{}/activate", period_id),
        json!({}),
        Some(school_admin_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Failed to activate period");

    // Create registration
    let birth_date = Utc::now() - Duration::days(365 * 15);
    let (status, reg_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id,
            "path_id": path_id,
            "student_nisn": "9999999999",
            "student_name": "Test Student Verify",
            "student_gender": "L",
            "student_birth_place": "Jakarta",
            "student_birth_date": birth_date,
            "student_religion": "Islam",
            "student_address": "Jl. Verify Test",
            "student_email": "verify@test.com",
            "parent_name": "Parent Verify",
            "parent_nik": "9999999999999999",
            "parent_phone": "089999999999",
            "path_data": {}
        }),
        Some(parent_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Failed to create registration: {:?}", reg_response);

    let registration_id = reg_response["id"].as_i64().unwrap() as i32;

    // Upload required document
    ctx.post(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        json!({
            "document_type": "birth_certificate",
            "file_url": "https://example.com/birth_cert.pdf",
            "file_name": "birth_certificate.pdf",
            "file_size": 102400,
            "mime_type": "application/pdf"
        }),
        Some(parent_token)
    ).await;

    // Submit registration
    let (status, _) = ctx.post(
        &format!("/api/v1/registrations/{}/submit", registration_id),
        json!({}),
        Some(parent_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Failed to submit registration");

    (period_id, registration_id)
}

// ============================================================================
// Get Pending Verifications Tests
// ============================================================================

#[tokio::test]
async fn test_verification_get_pending_verifications() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST400").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify@test.com").await;

    setup_registration_for_verification(&ctx, &school_admin.access_token, &parent.access_token).await;

    let (status, response) = ctx.get(
        "/api/v1/verifications/pending",
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["registrations"].as_array().unwrap().len() >= 1);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Get Verification Stats Tests
// ============================================================================

#[tokio::test]
async fn test_verification_get_stats() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST401").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify2@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify2@test.com").await;

    let (period_id, _) = setup_registration_for_verification(&ctx, &school_admin.access_token, &parent.access_token).await;

    let (status, response) = ctx.get(
        &format!("/api/v1/verifications/stats?period_id={}", period_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["submitted"].as_i64().unwrap() >= 1);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Verify Registration Tests
// ============================================================================

#[tokio::test]
async fn test_verification_verify_registration() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST402").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify3@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify3@test.com").await;

    let (_, registration_id) = setup_registration_for_verification(&ctx, &school_admin.access_token, &parent.access_token).await;

    let (status, response) = ctx.post(
        &format!("/api/v1/verifications/{}/verify", registration_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["status"], "verified");

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Reject Registration Tests
// ============================================================================

#[tokio::test]
async fn test_verification_reject_registration() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST403").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify4@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify4@test.com").await;

    let (_, registration_id) = setup_registration_for_verification(&ctx, &school_admin.access_token, &parent.access_token).await;

    let (status, response) = ctx.post(
        &format!("/api/v1/verifications/{}/reject", registration_id),
        json!({
            "reason": "Data tidak lengkap dan tidak sesuai dengan persyaratan"
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["status"], "rejected");
    assert!(response["rejection_reason"].as_str().unwrap().contains("tidak lengkap"));

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_verification_reject_with_short_reason() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST404").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify5@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify5@test.com").await;

    let (_, registration_id) = setup_registration_for_verification(&ctx, &school_admin.access_token, &parent.access_token).await;

    let (status, _response) = ctx.post(
        &format!("/api/v1/verifications/{}/reject", registration_id),
        json!({
            "reason": "Bad" // Too short
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Verify Document Tests
// ============================================================================

#[tokio::test]
async fn test_verification_verify_document() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST405").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify6@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify6@test.com").await;

    // Create registration but don't submit yet
    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(120));

    let (_, period_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": [{"path_type": "zonasi", "name": "Jalur Zonasi", "quota": 50, "scoring_config": {}}]
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = period_response["period"]["id"].as_i64().unwrap() as i32;
    let path_id = period_response["paths"][0]["id"].as_i64().unwrap() as i32;

    ctx.post(&format!("/api/v1/periods/{}/activate", period_id), json!({}), Some(&school_admin.access_token)).await;

    let birth_date = Utc::now() - Duration::days(365 * 15);
    let (_, reg_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id, "path_id": path_id, "student_nisn": "8888888888",
            "student_name": "Test Doc", "student_gender": "L", "student_birth_place": "Jakarta",
            "student_birth_date": birth_date, "student_religion": "Islam",
            "student_address": "Jl. Test", "student_email": "doc@test.com",
            "parent_name": "Parent Doc", "parent_nik": "8888888888888888",
            "parent_phone": "088888888888", "path_data": {}
        }),
        Some(&parent.access_token)
    ).await;

    let registration_id = reg_response["id"].as_i64().unwrap();

    // Upload document while still in draft
    let (_, doc_response) = ctx.post(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        json!({
            "document_type": "family_card",
            "file_url": "https://example.com/kk.pdf",
            "file_name": "kk.pdf",
            "file_size": 102400,
            "mime_type": "application/pdf"
        }),
        Some(&parent.access_token)
    ).await;

    let doc_id = doc_response["id"].as_i64().unwrap();

    // Verify document
    let (status, response) = ctx.post(
        &format!("/api/v1/verifications/documents/{}/verify", doc_id),
        json!({
            "verification_status": "approved",
            "verification_notes": "Document is valid and clear"
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["message"].as_str().unwrap().contains("successfully"));

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_verification_reject_document() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST406").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify7@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify7@test.com").await;

    // Create registration but don't submit yet
    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(120));

    let (_, period_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": [{"path_type": "zonasi", "name": "Jalur Zonasi", "quota": 50, "scoring_config": {}}]
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = period_response["period"]["id"].as_i64().unwrap() as i32;
    let path_id = period_response["paths"][0]["id"].as_i64().unwrap() as i32;

    ctx.post(&format!("/api/v1/periods/{}/activate", period_id), json!({}), Some(&school_admin.access_token)).await;

    let birth_date = Utc::now() - Duration::days(365 * 15);
    let (_, reg_response) = ctx.post(
        "/api/v1/registrations",
        json!({
            "period_id": period_id, "path_id": path_id, "student_nisn": "7777777777",
            "student_name": "Test Reject Doc", "student_gender": "L", "student_birth_place": "Jakarta",
            "student_birth_date": birth_date, "student_religion": "Islam",
            "student_address": "Jl. Test", "student_email": "rejectdoc@test.com",
            "parent_name": "Parent Reject", "parent_nik": "7777777777777777",
            "parent_phone": "087777777777", "path_data": {}
        }),
        Some(&parent.access_token)
    ).await;

    let registration_id = reg_response["id"].as_i64().unwrap();

    // Upload document while still in draft
    let (_, doc_response) = ctx.post(
        &format!("/api/v1/registrations/{}/documents", registration_id),
        json!({
            "document_type": "rapor",
            "file_url": "https://example.com/rapor.pdf",
            "file_name": "rapor.pdf",
            "file_size": 102400,
            "mime_type": "application/pdf"
        }),
        Some(&parent.access_token)
    ).await;

    let doc_id = doc_response["id"].as_i64().unwrap();

    // Reject document
    let (status, response) = ctx.post(
        &format!("/api/v1/verifications/documents/{}/verify", doc_id),
        json!({
            "verification_status": "rejected",
            "verification_notes": "Document is blurry and unreadable"
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["message"].as_str().unwrap().contains("successfully"));

    ctx.cleanup_test_data().await;
}

// ============================================================================
// RBAC Tests
// ============================================================================

#[tokio::test]
async fn test_verification_parent_cannot_verify() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST407").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_verify8@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_verify8@test.com").await;

    let (_, registration_id) = setup_registration_for_verification(&ctx, &school_admin.access_token, &parent.access_token).await;

    // Parent tries to verify
    let (status, _response) = ctx.post(
        &format!("/api/v1/verifications/{}/verify", registration_id),
        json!({}),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup_test_data().await;
}
