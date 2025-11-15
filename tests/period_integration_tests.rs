mod test_helpers;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use test_helpers::TestContext;

// Helper function to get NaiveDate from DateTime
fn to_naive_date(dt: chrono::DateTime<Utc>) -> chrono::NaiveDate {
    dt.date_naive()
}

// ============================================================================
// Create Period Tests
// ============================================================================

#[tokio::test]
async fn test_period_create_with_paths() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST001").await;
    let school_admin = ctx.create_school_admin(school_id, "admin1@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

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
                    "quota": 100,
                    "description": "Jalur zonasi untuk siswa dalam zona",
                    "scoring_config": {
                        "distance_weight": 60,
                        "age_weight": 40
                    }
                },
                {
                    "path_type": "prestasi",
                    "name": "Jalur Prestasi",
                    "quota": 50,
                    "scoring_config": {
                        "academic_weight": 70,
                        "achievement_weight": 30
                    }
                }
            ]
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["period"]["academic_year"], "2024/2025");
    assert_eq!(response["period"]["level"], "SMA");
    assert_eq!(response["period"]["status"], "draft");
    assert_eq!(response["paths"].as_array().unwrap().len(), 2);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_period_create_invalid_academic_year() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST002").await;
    let school_admin = ctx.create_school_admin(school_id, "admin2@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (status, _response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024", // Invalid format
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_period_create_invalid_level() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST003").await;
    let school_admin = ctx.create_school_admin(school_id, "admin3@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (status, _response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "INVALID",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_period_create_as_parent_forbidden() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST004").await;
    let parent = ctx.create_parent(school_id, "parent4@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (status, _response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&parent.access_token)
    ).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// List Periods Tests
// ============================================================================

#[tokio::test]
async fn test_period_list() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST005").await;
    let school_admin = ctx.create_school_admin(school_id, "admin5@test.com").await;

    // Create multiple periods
    for i in 0..3 {
        let start_date = to_naive_date(Utc::now() + Duration::days(30 + i * 100));
        let end_date = to_naive_date(Utc::now() + Duration::days(90 + i * 100));
        
        ctx.post(
            "/api/v1/periods",
            json!({
                "academic_year": "2024/2025",
                "level": "SMA",
                "start_date": start_date,
                "end_date": end_date,
                "paths": []
            }),
            Some(&school_admin.access_token)
        ).await;
    }

    let (status, response) = ctx.get(
        "/api/v1/periods?page=1&page_size=10",
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["periods"].as_array().unwrap().len() >= 3);
    assert!(response["total"].as_i64().unwrap() >= 3);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_period_list_filter_by_status() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST006").await;
    let school_admin = ctx.create_school_admin(school_id, "admin6@test.com").await;

    // Create period
    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));
    
    ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    let (status, response) = ctx.get(
        "/api/v1/periods?status=draft",
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let periods = response["periods"].as_array().unwrap();
    for period in periods {
        assert_eq!(period["status"], "draft");
    }

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Get Period Tests
// ============================================================================

#[tokio::test]
async fn test_period_get_with_paths() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST007").await;
    let school_admin = ctx.create_school_admin(school_id, "admin7@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
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
                    "quota": 100,
                    "scoring_config": {}
                }
            ]
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = create_response["period"]["id"].as_i64().unwrap();

    let (status, response) = ctx.get(
        &format!("/api/v1/periods/{}", period_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["period"]["id"], period_id);
    assert_eq!(response["paths"].as_array().unwrap().len(), 1);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Update Period Tests
// ============================================================================

#[tokio::test]
async fn test_period_update() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST008").await;
    let school_admin = ctx.create_school_admin(school_id, "admin8@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = create_response["period"]["id"].as_i64().unwrap();

    let new_end_date = to_naive_date(Utc::now() + Duration::days(120));
    let announcement_date = to_naive_date(Utc::now() + Duration::days(127));

    let (status, response) = ctx.put(
        &format!("/api/v1/periods/{}", period_id),
        json!({
            "end_date": new_end_date,
            "announcement_date": announcement_date
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["announcement_date"].is_string());

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Activate/Close Period Tests
// ============================================================================

#[tokio::test]
async fn test_period_activate() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST009").await;
    let school_admin = ctx.create_school_admin(school_id, "admin9@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = create_response["period"]["id"].as_i64().unwrap();

    let (status, response) = ctx.post(
        &format!("/api/v1/periods/{}/activate", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["status"], "active");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_period_close() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST010").await;
    let school_admin = ctx.create_school_admin(school_id, "admin10@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = create_response["period"]["id"].as_i64().unwrap();

    // Activate first
    ctx.post(
        &format!("/api/v1/periods/{}/activate", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Then close
    let (status, response) = ctx.post(
        &format!("/api/v1/periods/{}/close", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["status"], "closed");

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Delete Period Tests
// ============================================================================

#[tokio::test]
async fn test_period_delete_draft() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST011").await;
    let school_admin = ctx.create_school_admin(school_id, "admin11@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = create_response["period"]["id"].as_i64().unwrap();

    let (status, response) = ctx.delete(
        &format!("/api/v1/periods/{}", period_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["message"].as_str().unwrap().contains("deleted"));

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_period_delete_active_forbidden() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST012").await;
    let school_admin = ctx.create_school_admin(school_id, "admin12@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = create_response["period"]["id"].as_i64().unwrap();

    // Activate period
    ctx.post(
        &format!("/api/v1/periods/{}/activate", period_id),
        json!({}),
        Some(&school_admin.access_token)
    ).await;

    // Try to delete active period
    let (status, _response) = ctx.delete(
        &format!("/api/v1/periods/{}", period_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup_test_data().await;
}

// ============================================================================
// Registration Path Tests
// ============================================================================

#[tokio::test]
async fn test_path_create() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST013").await;
    let school_admin = ctx.create_school_admin(school_id, "admin13@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
        "/api/v1/periods",
        json!({
            "academic_year": "2024/2025",
            "level": "SMA",
            "start_date": start_date,
            "end_date": end_date,
            "paths": []
        }),
        Some(&school_admin.access_token)
    ).await;

    let period_id = create_response["period"]["id"].as_i64().unwrap();

    let (status, response) = ctx.post(
        &format!("/api/v1/periods/{}/paths", period_id),
        json!({
            "path_type": "afirmasi",
            "name": "Jalur Afirmasi",
            "quota": 20,
            "description": "Jalur untuk siswa kurang mampu",
            "scoring_config": {
                "income_weight": 100
            }
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {:?}", response);
    assert_eq!(response["path_type"], "afirmasi");
    assert_eq!(response["quota"], 20);

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_path_update() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST014").await;
    let school_admin = ctx.create_school_admin(school_id, "admin14@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
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
                    "quota": 100,
                    "scoring_config": {}
                }
            ]
        }),
        Some(&school_admin.access_token)
    ).await;

    let path_id = create_response["paths"][0]["id"].as_i64().unwrap();

    let (status, response) = ctx.put(
        &format!("/api/v1/periods/paths/{}", path_id),
        json!({
            "quota": 150,
            "description": "Updated description"
        }),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert_eq!(response["quota"], 150);
    assert_eq!(response["description"], "Updated description");

    ctx.cleanup_test_data().await;
}

#[tokio::test]
async fn test_path_delete() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    let super_admin = ctx.create_super_admin().await;
    let school_id = ctx.create_test_school(&super_admin.access_token, "TEST015").await;
    let school_admin = ctx.create_school_admin(school_id, "admin15@test.com").await;

    let start_date = to_naive_date(Utc::now() + Duration::days(30));
    let end_date = to_naive_date(Utc::now() + Duration::days(90));

    let (_, create_response) = ctx.post(
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
                    "quota": 100,
                    "scoring_config": {}
                }
            ]
        }),
        Some(&school_admin.access_token)
    ).await;

    let path_id = create_response["paths"][0]["id"].as_i64().unwrap();

    let (status, response) = ctx.delete(
        &format!("/api/v1/periods/paths/{}", path_id),
        Some(&school_admin.access_token)
    ).await;

    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    assert!(response["message"].as_str().unwrap().contains("deleted"));

    ctx.cleanup_test_data().await;
}
