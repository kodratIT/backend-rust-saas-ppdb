mod test_helpers;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use test_helpers::TestContext;

// Helper function to get NaiveDate from DateTime
fn to_naive_date(dt: chrono::DateTime<Utc>) -> chrono::NaiveDate {
    dt.date_naive()
}

// Helper function to setup verified registrations for selection
async fn setup_verified_registrations(
    ctx: &TestContext,
    school_admin_token: &str,
    parent_token: &str,
    count: usize,
) -> (i32, i32) {
    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(120));

    // Create period with scoring config
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
                    "scoring_config": {
                        "distance_weight": 100
                    }
                }
            ]
        }),
        Some(school_admin_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Failed to create period");

    let period_id = period_response["period"]["id"].as_i64().unwrap() as i32;
    let path_id = period_response["paths"][0]["id"].as_i64().unwrap() as i32;

    // Activate period
    ctx.post(
        &format!("/api/v1/periods/{}/activate", period_id),
        json!({}),
        Some(school_admin_token)
    ).await;

    // Create, submit, and verify registrations
    for i in 0..count {
        let birth_date = Utc::now() - Duration::days(365 * 15);
        
        let (_, reg_response) = ctx.post(
            "/api/v1/registrations",
            json!({
                "period_id": period_id,
                "path_id": path_id,
                "student_nisn": format!("100000000{}", i),
                "student_name": format!("Student Select {}", i),
                "student_gender": "L",
                "student_birth_place": "Jakarta",
                "student_birth_date": birth_date,
                "student_religion": "Islam",
                "student_address": "Jl. Selection Test",
                "student_email": format!("select{}@test.com", i),
                "parent_name": "Parent Select",
                "parent_nik": format!("100000000000000{}", i),
                "parent_phone": format!("0810000000{:02}", i),
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

        // Submit registration
        ctx.post(
            &format!("/api/v1/registrations/{}/submit", registration_id),
            json!({}),
            Some(parent_token)
        ).await;

        // Verify registration
        ctx.post(
            &format!("/api/v1/verifications/{}/verify", registration_id),
            json!({}),
            Some(school_admin_token)
        ).await;
    }

    (period_id, path_id)
}

// ============================================================================
// Calculate Scores Tests
// ============================================================================

#[tokio::test]
async fn test_selection_calculate_scores() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST500").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_select@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_select@test.com").await;

    let (period_id, _) = setup_verified_registrations(&ctx, &school_admin.access_token, &parent.access_token, 3).await;

    let (status, response) = ctx.post(
        &format!("/api/v1/selection/periods/{}/calculate-scores", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["total_calculated"].as_i64().unwrap(), 3);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Update Rankings Tests
// ============================================================================

#[tokio::test]
async fn test_selection_update_rankings() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST501").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_select2@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_select2@test.com").await;

    let (period_id, _) = setup_verified_registrations(&ctx, &school_admin.access_token, &parent.access_token, 5).await;

    // Calculate scores first
    ctx.post(
        &format!("/api/v1/selection/periods/{}/calculate-scores", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Update rankings
    let (status, response) = ctx.post(
        &format!("/api/v1/selection/periods/{}/update-rankings", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["total_ranked"].as_i64().unwrap(), 5);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Get Rankings Tests
// ============================================================================

#[tokio::test]
async fn test_selection_get_rankings() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST502").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_select3@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_select3@test.com").await;

    let (period_id, path_id) = setup_verified_registrations(&ctx, &school_admin.access_token, &parent.access_token, 5).await;

    // Calculate scores and rankings
    ctx.post(
        &format!("/api/v1/selection/periods/{}/calculate-scores", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    ctx.post(
        &format!("/api/v1/selection/periods/{}/update-rankings", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Get rankings
    let (status, response) = ctx.get(
        &format!("/api/v1/selection/periods/{}/rankings?path_id={}", period_id, path_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let rankings = response["rankings"].as_array().unwrap();
    assert_eq!(rankings.len(), 5);
    
    // Check that rankings are ordered
    let mut prev_ranking = 0;
    for rank in rankings {
        let current_ranking = rank["ranking"].as_i64().unwrap();
        assert!(current_ranking > prev_ranking);
        prev_ranking = current_ranking;
    }

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Get Ranking Stats Tests
// ============================================================================

#[tokio::test]
async fn test_selection_get_ranking_stats() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST503").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_select4@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_select4@test.com").await;

    let (period_id, _) = setup_verified_registrations(&ctx, &school_admin.access_token, &parent.access_token, 3).await;

    // Calculate scores and rankings
    ctx.post(
        &format!("/api/v1/selection/periods/{}/calculate-scores", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    ctx.post(
        &format!("/api/v1/selection/periods/{}/update-rankings", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Get stats
    let (status, response) = ctx.get(
        &format!("/api/v1/selection/periods/{}/stats", period_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let stats = response.as_array().unwrap();
    assert!(stats.len() >= 1);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// RBAC Tests
// ============================================================================

#[tokio::test]
async fn test_selection_parent_cannot_calculate() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST504").await;
    let school_admin = ctx.create_school_admin(school_id, "admin_select5@test.com").await;
    let parent = ctx.create_parent(school_id, "parent_select5@test.com").await;

    let (period_id, _) = setup_verified_registrations(&ctx, &school_admin.access_token, &parent.access_token, 2).await;

    // Parent tries to calculate scores
    let (status, _response) = ctx.post(
        &format!("/api/v1/selection/periods/{}/calculate-scores", period_id),
        json!({}),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup_test_data().await;
}
