mod test_helpers;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use test_helpers::TestContext;

// Helper function to get NaiveDate from DateTime
fn to_naive_date(dt: chrono::DateTime<Utc>) -> chrono::NaiveDate {
    dt.date_naive()
}

// Helper function to setup complete selection flow for announcement
async fn setup_complete_selection_flow(
    ctx: &TestContext,
    school_admin_token: &str,
    parent_token: &str,
    count: usize,
) -> (i32, Vec<String>) {
    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(120));

    // Create period with small quota for testing acceptance/rejection
    let (_, period_response) = ctx.post(
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
                    "quota": 3,  // Small quota to test rejection
                    "scoring_config": {
                        "distance_weight": 100
                    }
                }
            ]
        }),
        Some(school_admin_token)
    ).await;

    let period_id = period_response["period"]["id"].as_i64().unwrap() as i32;
    let path_id = period_response["paths"][0]["id"].as_i64().unwrap() as i32;

    // Activate period
    ctx.post(
        &format!("/api/v1/periods/{}/activate", period_id),
        json!({}),
        Some(school_admin_token)
    ).await;

    let mut registration_numbers = Vec::new();

    // Create, submit, and verify registrations
    for i in 0..count {
        let birth_date = Utc::now() - Duration::days(365 * 15);
        let nisn = format!("200000000{}", i);
        
        let (_, reg_response) = ctx.post(
            "/api/v1/registrations",
            json!({
                "period_id": period_id,
                "path_id": path_id,
                "student_nisn": nisn,
                "student_name": format!("Student Announce {}", i),
                "student_gender": "L",
                "student_birth_place": "Jakarta",
                "student_birth_date": birth_date,
                "student_religion": "Islam",
                "student_address": "Jl. Announcement Test",
                "student_email": format!("announce{}@test.com", i),
                "parent_name": "Parent Announce",
                "parent_nik": format!("200000000000000{}", i),
                "parent_phone": format!("0820000000{:02}", i),
                "path_data": {
                    "distance_km": (i as f64 * 0.5) + 1.0
                }
            }),
            Some(parent_token)
        ).await;

        let registration_id = reg_response["id"].as_i64().unwrap();

        // Upload document
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

        // Submit
        let (_, submit_response) = ctx.post(
            &format!("/api/v1/registrations/{}/submit", registration_id),
            json!({}),
            Some(parent_token)
        ).await;
        
        registration_numbers.push(submit_response["registration_number"].as_str().unwrap().to_string());

        // Verify
        ctx.post(
            &format!("/api/v1/verifications/{}/verify", registration_id),
            json!({}),
            Some(school_admin_token)
        ).await;
    }

    // Calculate scores and rankings
    ctx.post(
        &format!("/api/v1/selection/periods/{}/calculate-scores", period_id),
        json!({}),
        Some(school_admin_token)
    ).await;

    ctx.post(
        &format!("/api/v1/selection/periods/{}/update-rankings", period_id),
        json!({}),
        Some(school_admin_token)
    ).await;

    (period_id, registration_numbers)
}

// ============================================================================
// Run Selection Tests
// ============================================================================

#[tokio::test]
async fn test_announcement_run_selection() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST600").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_announce@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_announce@test.com").await;

    let (period_id, _) = setup_complete_selection_flow(&ctx, &school_admin.access_token, &parent.access_token, 5).await;

    let (status, response) = ctx.post(
        &format!("/api/v1/announcements/periods/{}/run-selection", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["result"]["total_accepted"].as_i64().unwrap(), 3);
    assert_eq!(response["result"]["total_rejected"].as_i64().unwrap(), 2);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Get Selection Summary Tests
// ============================================================================

#[tokio::test]
async fn test_announcement_get_selection_summary() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST601").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_announce2@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_announce2@test.com").await;

    let (period_id, _) = setup_complete_selection_flow(&ctx, &school_admin.access_token, &parent.access_token, 4).await;

    // Run selection first
    ctx.post(
        &format!("/api/v1/announcements/periods/{}/run-selection", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Get summary
    let (status, response) = ctx.get(
        &format!("/api/v1/announcements/periods/{}/summary", period_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    // After run_selection, status changes from verified to accepted/rejected
    assert_eq!(response["verified"].as_i64().unwrap(), 0);  // No more verified after selection
    assert_eq!(response["accepted"].as_i64().unwrap(), 3);
    assert_eq!(response["rejected"].as_i64().unwrap(), 1);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Announce Results Tests
// ============================================================================

#[tokio::test]
async fn test_announcement_announce_results() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST602").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_announce3@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_announce3@test.com").await;

    let (period_id, _) = setup_complete_selection_flow(&ctx, &school_admin.access_token, &parent.access_token, 4).await;

    // Run selection first
    ctx.post(
        &format!("/api/v1/announcements/periods/{}/run-selection", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Announce results
    let (status, response) = ctx.post(
        &format!("/api/v1/announcements/periods/{}/announce", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["result"]["total_notified"].as_i64().unwrap(), 4);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Check Result (Public) Tests
// ============================================================================

#[tokio::test]
async fn test_announcement_check_result_public() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST603").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_announce4@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_announce4@test.com").await;

    let (period_id, registration_numbers) = setup_complete_selection_flow(&ctx, &school_admin.access_token, &parent.access_token, 2).await;

    // Run selection and announce
    ctx.post(
        &format!("/api/v1/announcements/periods/{}/run-selection", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    ctx.post(
        &format!("/api/v1/announcements/periods/{}/announce", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Check result without authentication (public endpoint)
    let (status, response) = ctx.get(
        &format!(
            "/api/v1/announcements/check-result?registration_number={}&student_nisn=2000000000",
            registration_numbers[0]
        ),
        None  // No authentication for public endpoint
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["registration_number"].as_str().is_some());
    assert!(response["status"].as_str().is_some());

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_announcement_check_result_invalid_nisn() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST604").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_announce5@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_announce5@test.com").await;

    let (_, registration_numbers) = setup_complete_selection_flow(&ctx, &school_admin.access_token, &parent.access_token, 1).await;

    // Check with invalid NISN length
    let (status, _response) = ctx.get(
        &format!(
            "/api/v1/announcements/check-result?registration_number={}&student_nisn=123",
            registration_numbers[0]
        ),
        None
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_announcement_check_result_not_found() {
    let ctx = TestContext::new().await;

    let (status, _response) = ctx.get(
        "/api/v1/announcements/check-result?registration_number=INVALID123&student_nisn=9999999999",
        None
    ).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================================================
// RBAC Tests
// ============================================================================

#[tokio::test]
async fn test_announcement_parent_cannot_run_selection() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST605").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_announce6@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_announce6@test.com").await;

    let (period_id, _) = setup_complete_selection_flow(&ctx, &school_admin.access_token, &parent.access_token, 2).await;

    // Parent tries to run selection
    let (status, _response) = ctx.post(
        &format!("/api/v1/announcements/periods/{}/run-selection", period_id),
        json!({}),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup_test_data().await;
}
